use {
    rocket_sync_db_pools::{database, postgres},
    serde::Serialize,
};

#[database("lttpoll")]
pub struct DbCon(postgres::Client);

#[derive(Debug, Serialize)]
pub struct Question {
    pub id: i64,
    pub slug: String,
    pub text: String,
    // expires: Option<DateTime>
}

impl TryFrom<&postgres::Row> for Question {
    type Error = postgres::Error;

    fn try_from(row: &postgres::Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            slug: row.try_get("slug")?,
            text: row.try_get("text")?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Answer {
    pub id: i64,
    pub question: i64,
    pub text: String,
    pub votes: i64,
}

impl TryFrom<&postgres::Row> for Answer {
    type Error = postgres::Error;

    fn try_from(row: &postgres::Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            question: row.try_get("question")?,
            text: row.try_get("text")?,
            votes: row.try_get("votes")?,
        })
    }
}
