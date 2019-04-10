#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

// FIXME: Declare a `GET /` route named `index` that returns `Hello, world!`.
#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![index])
}

fn main() {
    // FIXME: Ignite a rocket, mount some routes, and launch it.
    rocket().launch();
}
