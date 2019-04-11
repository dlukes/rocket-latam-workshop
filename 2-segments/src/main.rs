#![feature(proc_macro_hygiene, decl_macro)]

use std::path::PathBuf;

#[macro_use]
extern crate rocket;
use rocket::http::uri::Segments;
use rocket::request::FromSegments;

#[cfg(test)]
mod tests;

struct CustomPath<'a>(&'a str);

// FIXME: Implement `FromSegments` for `CustomPath`. Don't modify `CustomPath`.
// The `Error` associated type should likely be `i32`. You should store the 2nd
// path component in the `CustomPath` value.

impl<'a> FromSegments<'a> for CustomPath<'a> {
    type Error = usize;

    fn from_segments(segments: Segments<'a>) -> Result<Self, Self::Error> {
        let mut n = 0;
        for seg in segments {
            n += 1;
            if n == 2 {
                return Ok(CustomPath(seg));
            }
        }
        Err(n)
    }
}

// FIXME: Implement the following routes:
//
//   * (outer) GET /outer/<path..>
//
//     If `path` has at least two segments, simply responds with the raw text in
//     the second segment. Otherwise, returns the following message, where <n>
//     is the actual number of segments in `<path..>`:
//
//     Expected >= 2 segments, found <n>.
//
//   * (inner) GET /inner/<path..>
//
//     If `path` has at least two segments, simply responds with the raw text in
//     the second segment. Otherwise, this route should not be called.
//
//   * (echo) GET /<path..>
//
//     Echos the user's `<path..>`.
//
// The `outer` and `inner` routes should take precedence over the `echo` route.
// That is, if the request's path starts with `/outer`, `outer` should response.
// If the request's path starts with `/inner`, `inner` should be tried before
// `echo`. If all else fails, `echo` should respond.

#[get("/outer/<path..>")]
fn outer(path: Result<CustomPath, usize>) -> String {
    match path {
        Ok(path) => path.0.to_owned(),
        Err(n) => format!("Expected >= 2 segments, found {}.", n),
    }
}

#[get("/inner/<path..>")]
fn inner(path: CustomPath) -> String {
    path.0.to_owned()
}

#[get("/<path..>", rank = 2)]
fn echo(path: PathBuf) -> String {
    path.into_os_string().into_string().unwrap()
}

fn main() {
    rocket::ignite()
        .mount("/", routes![inner, outer, echo])
        .launch();
}
