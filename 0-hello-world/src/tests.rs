use super::*;
use rocket::{http::Status, local::Client};

#[test]
fn hello_world() {
    let rocket = rocket();
    let client = Client::new(rocket).unwrap();

    let mut response = client.get("/").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("Hello, world!".into()));
}

// FIXME: Write a test that ensures the following:
//  * A HEAD request to `/` responds successfully.
//  * The response has an empty body.

#[test]
fn head_test() {
    let rocket = rocket();
    let client = Client::new(rocket).unwrap();

    let mut response = client.head("/").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert!(response.body_bytes().unwrap().is_empty());
}
