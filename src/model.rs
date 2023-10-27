use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct User(pub String);

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Users {
    pub users: Vec<User>
}

#[derive(Debug, PartialEq)]
pub struct UserID(pub i64);

#[derive(Debug, Serialize)]
pub struct Project(pub String);

#[derive(Debug, PartialEq)]
pub struct ProjectID(pub i64);

#[derive(Debug, Serialize)]
pub struct Projects {
}

#[derive(Debug, PartialEq)]
pub struct Owner(pub String);
