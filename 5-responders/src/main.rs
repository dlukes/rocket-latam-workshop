#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate serde;

#[cfg(test)]
mod tests;

use rocket::http::RawStr;
use rocket::request::{FromFormValue, FromParam};
use rocket::response::Redirect;
use rocket_contrib::json::JsonValue;
use rocket_contrib::templates::Template;

#[derive(FromFormValue)]
enum Kind {
    Html,
    Json,
}

impl<'a> FromParam<'a> for Kind {
    type Error = &'a RawStr;

    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        Kind::from_form_value(param)
    }
}

#[derive(Serialize, Deserialize)]
struct Task {
    id: usize,
    name: &'static str,
    severity: u8,
}

static TASKS: &[Task] = &[
    Task {
        id: 0,
        name: "Hi! This is the 0th task.",
        severity: 100,
    },
    Task {
        id: 1,
        name: "Beep boop. Robot here!",
        severity: 0,
    },
    Task {
        id: 2,
        name: "RustConf Rocket Workshop",
        severity: 127,
    },
    Task {
        id: 3,
        name: "Make `TASKS` Slice",
        severity: 50,
    },
];

// FIXME: Implement the following routes:
//
//   * (index) GET /
//
//     Redirects to the `html_tasks` route.

#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!(html_tasks))
}

//   * (json_tasks) GET /tasks (Accept = application/json)
//
//     Renders the list of tasks in a JSON dictionary with a single key:
//     `tasks`. Example:
//
//     {
//         "tasks": [
//             { "id": 0, name: "Hi! This is...", "severity", 100 }
//         ]
//     }

#[get("/tasks", format = "json", rank = 2)]
fn json_tasks() -> JsonValue {
    json!({ "tasks": TASKS })
}

//   * (html_tasks) GET /tasks (Accept = text/html)
//
//     Renders the list of tasks using the `tasks.html.tera` template.

#[get("/tasks", format = "html")]
fn html_tasks() -> Template {
    Template::render("tasks", json!({ "tasks": TASKS }))
}

//   * (one_task) GET /tasks/<id>/<kind>
//
//     Where:
//
//      * <id> - a unsigned integer in range [0, 256) (`u8`)
//      * <kind> - one of `html` or `json`
//
//     If `kind `is `html`, renders the `task.html.tera` template for the task
//     with id `id`. If `kind` is `json`, returns the JSON representation of the
//     task with id `id`. In either case, if `id` doesn't correspond to a known
//     task, returns a 404 page with the text "Unknown task: <id>". If `kind`
//     is neither `html` nor `json`, this route does not get called.

#[derive(Responder)]
enum OneTaskResponder {
    Html(Template),
    Json(JsonValue),
    #[response(status = 404)]
    Error(String),
}

struct OneTaskKind(String);

impl<'r> FromParam<'r> for OneTaskKind {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        let decoded = param.percent_decode_lossy();
        match decoded.as_ref() {
            "html" | "json" => Ok(Self(decoded.into_owned())),
            _ => Err(param),
        }
    }
}

#[get("/tasks/<id>/<kind>")]
fn one_task(id: u8, kind: OneTaskKind) -> OneTaskResponder {
    let task = match TASKS.get(id as usize) {
        Some(task) => task,
        None => return OneTaskResponder::Error(format!("Unknown task: {}", id)),
    };
    match kind.0.as_ref() {
        "json" => OneTaskResponder::Json(json!(task)),
        "html" => OneTaskResponder::Html(Template::render("task", task)),
        _ => unreachable!(),
    }
}

//  NOTE: If no 'Accept' is provided to `GET /tasks`, `html_tasks` should run.

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index, html_tasks, json_tasks, one_task])
        .launch();
}
