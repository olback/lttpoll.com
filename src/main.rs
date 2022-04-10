#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", rocket::routes![])
        .mount("/assets", rocket::routes![])
}
