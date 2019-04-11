#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::http::RawStr;
use rocket::request::{Form, FromFormValue, LenientForm};

#[cfg(test)]
mod tests;

// FIXME: Implement the following routes:
//
//   * (post_task) POST /task
//
//     Accepts a web form in the body with _exactly_ two fields:
//
//         `name` - any arbitrary string
//         `category` - any arbitrary string
//
//      Responds with a `Debug` string of the corresponding Rust structure.
//      Requests with unknown or missing form fields are rejected with a 422
//      error.

#[post("/task", format = "form", data = "<task>")]
fn post_task(task: Form<Task>) -> String {
    format!("{:?}", task.into_inner())
}

//   * (post_lenient_task) POST /lenient_task
//
//     Accepts a web form in the body with at least two fields:
//
//         `name` - any arbitrary string
//         `category` - any arbitrary string
//
//     The route is _lenient_; any additional fields in the form submission are
//     simply discarded. Requests with missing form fields are rejected with a
//     422 error. Responds with a `Debug` string of the corresponding Rust
//     structure.

#[post("/lenient_task", format = "form", data = "<task>")]
fn post_lenient_task(task: LenientForm<Task>) -> String {
    format!("{:?}", task.into_inner())
}

//   * (post_strict_task) POST /strict_task
//
//     Accepts a web form in the body with exactly two fields:
//
//         `name` - a non-empty string of length <= 128 bytes
//         `type` - one of "leisure", "business", or "critical"
//
//     Requests with invalid form values are rejected with a 422 error. Responds
//     with a `Debug` string of the corresponding Rust structure.

#[post("/strict_task", format = "form", data = "<strict_task>")]
fn post_strict_task(strict_task: Form<StrictTask>) -> String {
    format!("{:?}", strict_task.into_inner())
}

#[derive(Debug, FromForm)]
struct Task {
    name: String,
    category: String,
}

#[derive(Debug)]
struct Name(String);

impl<'v> FromFormValue<'v> for Name {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
        match form_value.url_decode() {
            Ok(string) => {
                let length = string.len();
                if length > 0 && length <= 128 {
                    Ok(Name(string))
                } else {
                    Err(form_value)
                }
            }
            _ => Err(form_value),
        }
    }
}

#[derive(Debug, FromFormValue)]
enum Kind {
    Leisure,
    Business,
    Critical,
}

#[derive(Debug, FromForm)]
struct StrictTask {
    name: Name,
    #[form(field = "type")]
    kind: Kind,
}

fn main() {
    rocket::ignite()
        .mount("/", routes![post_task, post_lenient_task, post_strict_task])
        .launch();
}
