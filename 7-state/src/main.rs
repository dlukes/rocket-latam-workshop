#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::State;

#[cfg(test)]
mod tests;

struct Token(i64);

// FIXME: Implement the following routes:
//
//   * (token) GET /token
//
//     Returns the value of the `token` configuration parameter. The value
//     should be stored in managed state on launch. If there is no such
//     configuration value or the value isn't a valid `i64`, launch should be
//     aborted.

#[get("/token")]
fn token(token: State<Token>) -> String {
    format!("{}", token.0)
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(AdHoc::on_attach("Token Config", |rocket| {
            let token_val = match rocket.config().get_int("token") {
                Ok(val) => val,
                Err(_) => return Err(rocket),
            };
            Ok(rocket.manage(Token(token_val)))
        }))
        .mount("/", routes![token])
}

fn main() {
    rocket().launch();
}
