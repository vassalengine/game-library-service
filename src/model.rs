use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct User {
    pub username: String
}

#[derive(Debug, Serialize)]
pub struct Users {
    pub users: Vec<User>
}

pub struct Owner(pub String);
