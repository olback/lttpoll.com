use rocket_sync_db_pools::{database, postgres};

#[database("lttpoll")]
pub struct DbCon(postgres::Client);

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
