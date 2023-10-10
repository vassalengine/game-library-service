use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct User {
    pub username: String
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Users {
    pub users: Vec<User>
}

pub struct Owner(pub String);
