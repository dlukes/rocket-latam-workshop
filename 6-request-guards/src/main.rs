#![feature(proc_macro_hygiene, decl_macro, never_type)]

#[macro_use]
extern crate rocket;

use rocket::http::{Cookie, Cookies};
use rocket::request::Form;
use rocket::response::NamedFile;
use rocket::response::Redirect;

mod roles;
#[cfg(test)]
mod tests;

use roles::{Admin, User};

// FIXME: Implement the following routes:
//
//   * (admin_index) GET /
//
//     Responds only if an `Admin` user is logged in. Returns the text:
//
//        "Hello, admin <id>!"
//
//     where <id> is the admin's user ID.

#[get("/")]
fn admin_index(admin: Admin) -> String {
    format!("Hello, admin {}!", admin.0.id)
}

//   * (user_index) GET /
//
//     Responds only if a regular `User` user is logged in. Returns the text:
//
//        "Hello, user <id>!"
//
//     where <id> is the user's user ID. NOTE: The `admin_index` route takes
//     precedence over this route.

#[get("/", rank = 2)]
fn user_index(user: User) -> String {
    format!("Hello, user {}!", user.id)
}

//   * (index) GET /
//
//     Redirects to `login_page`. NOTE: The `admin_index` and `user_index`
//     routes take precedence over this route.

#[get("/", rank = 3)]
fn index() -> Redirect {
    Redirect::to(uri!(login_page))
}

//   * (login_page) GET /login
//
//     Responds with the contents of `static/login.html`.

#[get("/login")]
fn login_page() -> Option<NamedFile> {
    NamedFile::open("static/login.html").ok()
}

//   * (login_submit) POST /login
//
//     Accepts a form with the following fields:
//
//       * `username` - an arbitrary string
//       * `password` - an arbitrary string
//
//     If the username is "admin" and the password is "password", logs in a user
//     with an id of `0` and redirects to `index`. If the username is "bob" and
//     the password is "123456", logs in a user with an id of `1` and redirects
//     to `index`. If the username and password are anything else, redirects to
//     `login_page`.

#[derive(FromForm)]
struct LoginCredentials {
    username: String,
    password: String,
}

#[post("/login", data = "<credentials>")]
fn login_submit(credentials: Form<LoginCredentials>, mut cookies: Cookies) -> Redirect {
    let user_id = match (credentials.username.as_ref(), credentials.password.as_ref()) {
        ("admin", "password") => "0",
        ("bob", "123456") => "1",
        _ => return Redirect::to(uri!(login_page)),
    };
    cookies.add_private(Cookie::new("user_id", user_id));
    Redirect::to(uri!(index))
}

//   * (logout) GET /logout
//
//     Logs the current user out, if any, then redirects to `index`.
//
//     NOTE: This should really be a `POST`; we use `GET` purely for
//     convenience. In reality, we'd use Rocket's _method rewriting_ mechanism
//     to issue a `POST`.

#[get("/logout")]
fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("user_id"));
    Redirect::to(uri!(index))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![admin_index, user_index, index])
        .mount("/", routes![login_page, login_submit, logout])
        .launch();
}
