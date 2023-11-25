use axum::extract::FromRef;

use crate::{
    core::CoreArc,
    jwt::DecodingKey
};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub key: DecodingKey,
    pub core: CoreArc
}
