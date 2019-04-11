#![feature(proc_macro_hygiene, decl_macro)]

extern crate rocket_contrib;
use rocket_contrib::json::Json;
use rocket_contrib::msgpack::MsgPack;
extern crate serde;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
mod tests;

#[derive(Debug, Deserialize)]
struct Task {
    name: String,
    category: String,
}

// FIXME: Implement the following routes:
//
//   * (add_json_task) POST /task
//
//     Accepts a JSON-serialized `Task` in the body when the `Content-Type` of
//     the incoming request is `application/json`. Returns a `Debug`
//     representation of the submitted structure.
//
//   * (add_msgpack_task) POST /task
//
//     Accepts a MessagePack-serialized `Task` in the body when the
//     `Content-Type` of the incoming request is `application/msgpack`. Returns
//     a `Debug` representation of the submitted structure.

#[post("/task", format = "json", data = "<task>")]
fn add_json_task(task: Json<Task>) -> String {
    format!("{:?}", task.into_inner())
}

#[post("/task", format = "msgpack", data = "<task>")]
fn add_msgpack_task(task: MsgPack<Task>) -> String {
    format!("{:?}", task.into_inner())
}

fn main() {
    rocket::ignite()
        .mount("/", routes![add_json_task, add_msgpack_task])
        .launch();
}
