use axum::extract::FromRef;

use crate::{
    core::CoreArc,
    jwt::DecodingKey,
    model::User
};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub key: DecodingKey,
    pub core: CoreArc,
    pub admins: Vec<User>
}
