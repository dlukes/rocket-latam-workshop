use rocket::outcome::{IntoOutcome, Outcome};
use rocket::request::{self, FromRequest, Request};

#[derive(Debug)]
pub struct User {
    pub id: usize,
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, !> {
        request
            .cookies()
            .get_private("user_id")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| User { id })
            .or_forward(())
    }
}

#[derive(Debug)]
pub struct Admin(pub User);

// FIXME: Implement `FromRequest` for `Admin`, which authenticates a user as an
// administrator if and only if the user's ID is 0. Otherwise, it forwards.

impl<'a, 'r> FromRequest<'a, 'r> for Admin {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Admin, !> {
        User::from_request(request).and_then(|user| match user.id {
            0 => Outcome::Success(Admin(user)),
            _ => Outcome::Forward(()),
        })
    }
}
