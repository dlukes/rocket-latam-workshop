#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::http::RawStr;
use rocket::request::FromParam;

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

struct GoodAge(usize);

impl<'r> FromParam<'r> for GoodAge {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        match usize::from_param(param) {
            Ok(age @ 1..=122) => Ok(Self(age)),
            _ => Err(param),
        }
    }
}

#[get("/<name>")]
fn simple_hello(name: String) -> String {
    format!("Hello, {}!", name)
}

#[get("/<name>/<age>")]
fn good_aged_hello(name: String, age: GoodAge) -> String {
    format!("Hello, {} year old {}.", age.0, name)
}

#[get("/<name>/<age>", rank = 2)]
fn bad_aged_hello(name: String, age: usize) -> String {
    format!("'{}' is a funky age, {}.", age, name)
}

fn main() {
    rocket::ignite()
        .mount("/", routes![simple_hello, good_aged_hello, bad_aged_hello])
        .launch();
}
