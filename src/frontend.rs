use {
    crate::{
        database::{Answer, DbCon, Question},
        voter_id::VoterId,
    },
    rocket::{
        form::Form,
        http::CookieJar,
        request::FlashMessage,
        response::{Flash, Redirect},
        FromForm, Route,
    },
    rocket_dyn_templates::Template,
    serde::Serialize,
};

#[derive(Serialize)]
struct Message {
    msg: String,
    kind: &'static str,
}

impl Message {
    fn new<S: Into<String>>(msg: S, kind: &'static str) -> Self {
        Self {
            msg: msg.into(),
            kind,
        }
    }
    pub fn ok<S: Into<String>>(msg: S) -> Self {
        Self::new(msg, "ok")
    }
    pub fn warning<S: Into<String>>(msg: S) -> Self {
        Self::new(msg, "warning")
    }
    pub fn error<S: Into<String>>(msg: S) -> Self {
        Self::new(msg, "error")
    }
}

impl From<FlashMessage<'_>> for Message {
    fn from(flash: FlashMessage) -> Self {
        match flash.kind() {
            "success" => Self::ok(flash.message()),
            "warning" => Self::warning(flash.message()),
            "error" => Self::error(flash.message()),
            _ => unreachable!(),
        }
    }
}

#[derive(Serialize)]
struct TemplateContext<T> {
    message: Option<Message>,
    data: Option<T>,
}

impl Default for TemplateContext<()> {
    fn default() -> Self {
        Self {
            message: None,
            data: None,
        }
    }
}

impl<T: Serialize> TemplateContext<T> {
    pub fn new<M: Into<Message>>(message: M, data: T) -> Self {
        Self {
            message: Some(message.into()),
            data: Some(data),
        }
    }

    pub fn data(data: T) -> Self {
        Self {
            message: None,
            data: Some(data),
        }
    }
}

impl TemplateContext<()> {
    pub fn message<M: Into<Message>>(message: M) -> Self {
        Self {
            message: Some(message.into()),
            data: None,
        }
    }
}

#[rocket::get("/")]
async fn index(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render(
        "index",
        match flash {
            Some(flash) => TemplateContext::message(flash),
            None => TemplateContext::default(),
        },
    )
}

#[derive(Debug, FromForm)]
struct NewPoll {
    question: String,
    answer: Vec<String>,
}

#[rocket::post("/", data = "<form>")]
async fn new_poll(form: Form<NewPoll>, db: DbCon) -> Result<Flash<Redirect>, Flash<Redirect>> {
    if form.question.trim().is_empty() {
        return Err(Flash::error(
            Redirect::to("/"),
            "Question may not be empty.",
        ));
    }

    let answers = form
        .answer
        .iter()
        .filter_map(|a| {
            let ans = a.trim();
            if ans.is_empty() {
                None
            } else {
                Some(String::from(ans))
            }
        })
        .collect::<Vec<String>>();

    if answers.len() < 2 || answers.len() > 10 {
        return Err(Flash::error(
            Redirect::to("/"),
            "Poll must contain between 2 and 10 answers.",
        ));
    }

    let question_text = form.question.clone();
    let mut insert_question_res = db
        .run(move |con| {
            con.query(
                "insert into questions (text) values ($1) returning *",
                &[&question_text.trim()],
            )
        })
        .await
        .map_err(|err| Flash::error(Redirect::to("/"), err.to_string()))?
        .iter()
        .filter_map(|r| Question::try_from(r).ok())
        .collect::<Vec<_>>();

    let question = match insert_question_res.len() {
        1 => Ok(insert_question_res.remove(0)),
        _ => Err(Flash::error(Redirect::to("/"), "Insert failed")),
    }?;

    let qid = question.id;
    db.run(
        move |con| -> Result<(), rocket_sync_db_pools::postgres::Error> {
            for ans in &answers {
                con.execute(
                    "insert into answers (question, text) values ($1, $2)",
                    &[&qid, &ans],
                )?;
            }
            Ok(())
        },
    )
    .await
    .map_err(|err| Flash::error(Redirect::to("/"), err.to_string()))?;

    Ok(Flash::success(
        Redirect::to(format!("/{}", question.slug)),
        "Poll created!",
    ))
}

#[rocket::get("/<slug>")]
async fn view_poll(
    slug: String,
    voter: Option<VoterId>,
    cookies: &CookieJar<'_>,
    flash: Option<FlashMessage<'_>>,
    db: DbCon,
) -> Option<Template> {
    let voter = match voter {
        Some(v) => v,
        None => VoterId::get_or_set(cookies),
    };

    let mut questions = db
        .run(move |c| c.query("select * from questions where slug = $1", &[&slug]))
        .await
        .ok()?
        .iter()
        .filter_map(|r| Question::try_from(r).ok())
        .collect::<Vec<_>>();

    let question = match questions.len() {
        1 => questions.remove(0),
        _ => return None,
    };

    const QUERY: &str = r#"
        select A.*, count(V.id) as votes from answers A
        left join votes V on V.answer = A.id
        where A.question = $1
        group by A.id
    "#;

    let qid = question.id;
    let answers = db
        .run(move |c| c.query(QUERY, &[&qid]))
        .await
        .ok()?
        .iter()
        .filter_map(|r| Answer::try_from(r).ok())
        .collect::<Vec<_>>();

    let can_vote = db
        .run(move |c| {
            c.query(
                "select id from votes where question = $1 and voter = $2",
                &[&qid, &&*voter],
            )
        })
        .await
        .ok()?
        .len()
        == 0;

    #[derive(Serialize)]
    struct Ctx {
        question: Question,
        answers: Vec<Answer>,
        total_votes: i64,
        can_vote: bool,
    }

    let ctx = Ctx {
        question,
        total_votes: answers.iter().map(|a| a.votes).sum(),
        answers,
        can_vote,
    };

    Some(Template::render(
        "question",
        match flash {
            Some(flash) => TemplateContext::new(flash, ctx),
            None => TemplateContext::data(ctx),
        },
    ))
}

#[rocket::post("/<slug>/<ans_id>")]
async fn vote(
    slug: String,
    ans_id: i64,
    voter: VoterId,
    db: DbCon,
) -> Option<Result<Flash<Redirect>, Flash<Redirect>>> {
    let slug_clone = slug.clone();
    let mut questions = db
        .run(move |c| c.query("select * from questions where slug = $1", &[&slug_clone]))
        .await
        .ok()?
        .iter()
        .filter_map(|r| Question::try_from(r).ok())
        .collect::<Vec<_>>();

    let question = match questions.len() {
        1 => questions.remove(0),
        _ => return None,
    };

    Some(
        match db
            .run(move |c| {
                c.execute(
                    "insert into votes (question, answer, voter) values ($1, $2, $3)",
                    &[&question.id, &ans_id, &&*voter],
                )
            })
            .await
        {
            Ok(1) => Ok(Flash::success(
                Redirect::to(format!("/{}", slug)),
                "Vote registered!",
            )),
            Ok(_) => Err(Flash::error(
                Redirect::to(format!("/{}", slug)),
                "Vote not registered. Unknown error.",
            )),
            Err(_) => Err(Flash::error(
                Redirect::to(format!("/{}", slug)),
                "Vote not registered. Already voted.",
            )),
        },
    )
}

pub fn routes() -> impl Into<Vec<Route>> {
    rocket::routes![index, new_poll, view_poll, vote]
}
