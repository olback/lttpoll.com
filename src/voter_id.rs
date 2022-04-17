use {
    rand::Rng,
    rocket::{
        http::{Cookie, CookieJar, Status},
        outcome::IntoOutcome,
    },
};

const VOTER_ID_COOKIE: &str = "voterid";

#[derive(Debug)]
pub struct VoterId(String);

impl VoterId {
    fn set_random(cookies: &CookieJar<'_>) -> Self {
        let vid = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(16)
            .map(char::from)
            .collect::<String>();
        cookies.add_private(Cookie::new(VOTER_ID_COOKIE, vid.clone()));
        Self(vid)
    }

    pub fn get_or_set(cookies: &CookieJar<'_>) -> Self {
        match Self::get(cookies) {
            Some(vid) => vid,
            None => Self::set_random(cookies),
        }
    }

    pub fn get(cookies: &CookieJar<'_>) -> Option<Self> {
        cookies
            .get_private(VOTER_ID_COOKIE)
            .map(|c| Self(String::from(c.value())))
    }
}

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for VoterId {
    type Error = ();

    async fn from_request(
        request: &'r rocket::request::Request<'_>,
    ) -> rocket::request::Outcome<VoterId, Self::Error> {
        VoterId::get(request.cookies()).into_outcome((Status::BadRequest, ()))
    }
}

impl Into<String> for VoterId {
    fn into(self) -> String {
        self.0
    }
}

impl std::ops::Deref for VoterId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
