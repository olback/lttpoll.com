mod database;
mod frontend;
mod voter_id;

use {database::DbCon, rocket::fs::FileServer, rocket_dyn_templates::Template};

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/assets", FileServer::from("assets"))
        .mount("/", frontend::routes())
        .attach(DbCon::fairing())
        .attach(Template::fairing())
}
