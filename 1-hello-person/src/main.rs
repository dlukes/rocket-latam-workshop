#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;

#[cfg(test)]
mod tests;

// FIXME: Declare the following routes:
//
//   * `simple_hello`
//
//      GET '/<name>' => "Hello, <name>!"
//
//   * `good_aged_hello`
//
//      GET '/<name>/<age>' => "Hello, <age> year old <name>."
//
//      where 0 < age <= 122
//
//   * `bad_aged_hello`
//
//      GET '/<name>/<age>' => "'<age>' is a funky age, <name>."
//
//      where 0 < age <= usize::max_value()
//

struct GoodAge;

impl<'a, 'r> FromRequest<'a, 'r> for GoodAge {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let age = request.get_param(1).unwrap();
        match age {
            Ok(0..=122) => Outcome::Success(Self),
            _ => Outcome::Forward(()),
        }
    }
}

#[get("/<name>")]
fn simple_hello(name: String) -> String {
    format!("Hello, {}!", name)
}

#[get("/<name>/<age>")]
fn good_aged_hello(name: String, age: usize, _good_age: GoodAge) -> String {
    format!("Hello, {} year old {}.", age, name)
}

#[get("/<name>/<age>", rank = 2)]
fn bad_aged_hello(name: String, age: usize) -> String {
    format!("'{}' is a funky age, {}.", age, name)
}

fn main() {
    println!("foo");
    rocket::ignite()
        .mount("/", routes![simple_hello, good_aged_hello, bad_aged_hello])
        .launch();
}
