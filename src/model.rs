use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct User(pub String);

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Users {
    pub users: Vec<User>
}

#[derive(Debug, Serialize)]
pub struct Project {
}

#[derive(Debug, Serialize)]
pub struct Projects {
}

pub struct Owner(pub String);
