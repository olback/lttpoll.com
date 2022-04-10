use {
    crate::database::{DbCon, Question},
    rocket::{form::Form, http::Status, response::Redirect, FromForm, Route},
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

impl From<Message> for TemplateContext<()> {
    fn from(msg: Message) -> Self {
        Self {
            message: Some(msg),
            data: None,
        }
    }
}

#[rocket::get("/")]
async fn index() -> Template {
    Template::render("index", TemplateContext::default())
}

#[derive(Debug, FromForm)]
struct NewPoll {
    question: String,
    answer: Vec<String>,
}

#[rocket::post("/", data = "<form>")]
async fn new_poll(form: Form<NewPoll>, db: DbCon) -> Result<Redirect, (Status, Template)> {
    if form.question.trim().is_empty() {
        return Err((
            Status::BadRequest,
            Template::render(
                "index",
                TemplateContext::from(Message::error("Question may not be empty.")),
            ),
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
        return Err((
            Status::BadRequest,
            Template::render(
                "index",
                TemplateContext::from(Message::error(
                    "Poll must contain between 2 and 10 answers.",
                )),
            ),
        ));
    }

    let question_text = form.question.clone();
    let mut insert_question_res = db
        .run(move |con| {
            con.query(
                "insert into questions (text) values ($1) returning *",
                &[&question_text],
            )
        })
        .await
        .map_err(|err| {
            (
                Status::InternalServerError,
                Template::render(
                    "index",
                    TemplateContext::from(Message::error(err.to_string())),
                ),
            )
        })?
        .iter()
        .filter_map(|r| Question::try_from(r).ok())
        .collect::<Vec<_>>();

    let question = match insert_question_res.len() {
        1 => Ok(insert_question_res.remove(0)),
        _ => Err((
            Status::InternalServerError,
            Template::render(
                "index",
                TemplateContext::from(Message::error("Insert failed")),
            ),
        )),
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
    .map_err(|err| {
        (
            Status::InternalServerError,
            Template::render(
                "index",
                TemplateContext::from(Message::error(err.to_string())),
            ),
        )
    })?;

    Ok(Redirect::to(format!("/{}", question.slug)))
}

#[rocket::get("/<poll_id>")]
async fn view_poll(poll_id: String, mut db: DbCon) -> Option<Template> {
    println!("Poll: {poll_id}");
    None
}

pub fn routes() -> impl Into<Vec<Route>> {
    rocket::routes![index, new_poll, view_poll]
}
