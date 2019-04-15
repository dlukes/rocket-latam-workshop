#![feature(proc_macro_hygiene, decl_macro)]
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
extern crate rand;
extern crate reqwest;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use rocket::fairing::AdHoc;
use rocket::http::{Cookie, Cookies, Status};
use rocket::outcome::IntoOutcome;
use rocket::request::{Form, FromRequest, Request};
use rocket::response::Redirect;
use rocket_contrib::{database, serve::StaticFiles, templates::Template};
use std::collections::HashMap;

mod auth;
mod schema;
mod talk;
mod user;

use talk::{Talk, TalkStatus};
use user::User;

type Result<T> = std::result::Result<T, Status>;

#[database("cfp")]
pub struct DbConn(diesel::SqliteConnection);

/// Retrieves the logged-in user based on the session cookie.
impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(req: &'a Request<'r>) -> rocket::request::Outcome<Self, Self::Error> {
        // FIXME: Implement a request guard that provides a "User" based on the
        // authentication cookie.
        let conn = req.guard::<DbConn>()?;
        req.cookies()
            .get_private("user")
            .and_then(|cookie| {
                let id = cookie.value().parse::<i32>().ok()?;
                User::get(&conn, id).ok()
            })
            .or_forward(())
    }
}

/// A User that is guaranteed to also be an administrator.
#[derive(Debug)]
pub struct Admin(pub User);

/// Retrieves the logged-in administrator based on the session cookie.
/// Forwards if no user is logged in, or if a non-administrator is logged in.
impl<'a, 'r> FromRequest<'a, 'r> for Admin {
    type Error = ();

    fn from_request(req: &'a Request<'r>) -> rocket::request::Outcome<Self, Self::Error> {
        // FIXME: Implement a request guard that verifies that a user is logged in and is also an
        // administrator.
        req.guard::<User>().and_then(|user| {
            if user.is_admin {
                Some(Admin(user))
            } else {
                None
            }
            .or_forward(())
        })
    }
}

// FIXME: Implement the following routes:
//
//  * (index) GET /
//
//    When a user is logged in, render the current user's talks using a
//    Template. On the same page, provide a form that will post data to the
//    `new_talk` route.

#[get("/")]
fn index(conn: DbConn, user: User) -> Option<Template> {
    let talks = Talk::get_for_user(&conn, &user).ok()?;
    Some(Template::render(
        "index",
        json!({ "user": user, "talks": talks }),
    ))
}

//  * (index_anonymous) GET /
//
//    Render the "welcome" template, which suggests that the user should log in.

#[get("/", rank = 2)]
fn index_anonymous() -> Template {
    let ctx: HashMap<&str, &str> = HashMap::new();
    Template::render("welcome", &ctx)
}

//  * (logout) GET /logout
//
//    Delete the "user" cookie, clearing the session.

#[get("/logout")]
fn logout(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("user"));
    Redirect::to(uri!(index_anonymous))
}

//  * (admin) GET /admin
//
//    When an administrator is logged in, render a Template containing all talks
//    in the database.

#[get("/admin")]
fn admin(conn: DbConn, admin: Admin) -> Option<Template> {
    Talk::get_all(&conn, &admin)
        .ok()
        .map(|talks| Template::render("admin", json!({ "user": admin.0, "talks": talks })))
}

//  * (admin_notice) GET /admin
//
//    Render a Template indicating that only logged in administrators can use the page.

#[get("/admin", rank = 2)]
fn admin_notice() -> Template {
    let ctx: HashMap<&str, &str> = HashMap::new();
    Template::render("admin_notice", &ctx)
}

//  * (get_talk) GET /talks/<id>
//
//    Render a Template containing all information on the given talk.

#[get("/talks/<id>")]
fn get_talk(id: i32, conn: DbConn, user: User) -> Option<Template> {
    Talk::get_one(&conn, &user, id)
        .ok()?
        .map(|talk| Template::render("talk", json!({ "talk": talk, "user": user })))
}

//  * (new_talk) POST /talks/
//
//    Accepts a web form in the body with two fields:
//
//        `title` - The title of the talk
//        `description` - A description of the talk
//
//    Creates a new talk with the given title and description, presented by
//    the current user. Responds with a Redirect to the `index` route.

#[derive(FromForm)]
struct NewTalk {
    title: String,
    description: String,
}

#[post("/talks", data = "<talk>")]
fn new_talk(talk: Form<NewTalk>, conn: DbConn, user: User) -> Result<Redirect> {
    Talk::new(&conn, &user, &talk.title, &talk.description)
        .map_err(|_| Status::InternalServerError)?;
    Ok(Redirect::to(uri!(index)))
}

//  * (set_status) PUT /talks/<id>/status
//
//    Accepts a web form in the body with one field:
//
//        `new_status` - a `TalkStatus`.
//
//    `TalkStatus` already implements `FromFormValue` so that it can be
//    conveniently used in a form.
//
//    Updates the talk's status. Responds with a Redirect to the `admin` route.

#[derive(FromForm)]
struct NewStatus {
    new_status: TalkStatus,
}

#[put("/talks/<id>/status", data = "<status>")]
fn set_status(id: i32, status: Form<NewStatus>, conn: DbConn, admin: Admin) -> Result<Redirect> {
    Talk::set_status(&conn, &admin, id, status.new_status)
        .map_err(|_| Status::InternalServerError)?;
    Ok(Redirect::to(uri!(admin)))
}

// Embed the SQL database schema (in the `migrations/` directory) directly into
// the application so that they can be run automatically when the server is
// launched.
embed_migrations!();

/// Initialize and launch the Rocket application.
fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![index, index_anonymous, logout, admin, admin_notice],
        )
        .mount("/", routes![get_talk, set_status, new_talk])
        .mount("/static/css", StaticFiles::from("templates/css"))
        .attach(Template::fairing())
        .attach(DbConn::fairing())
        .attach(AdHoc::on_attach("Database Migrations", |rocket| {
            let conn = DbConn::get_one(&rocket).expect("database connection");
            match embedded_migrations::run(&*conn) {
                Ok(()) => Ok(rocket),
                Err(e) => {
                    error!("Failed to run database migrations: {:?}", e);
                    Err(rocket)
                }
            }
        }))
        .attach(auth::fairing())
        .launch();
}
