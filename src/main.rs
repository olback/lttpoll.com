mod database;
mod frontend;

use {database::DbCon, rocket::fs::FileServer, rocket_dyn_templates::Template};

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/assets", FileServer::from("frontend/assets"))
        .mount("/", frontend::routes())
        .attach(DbCon::fairing())
        .attach(Template::fairing())
}
