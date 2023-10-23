use axum::extract::FromRef;
use std::sync::Arc;

use crate::{
    core::Core,
    jwt::DecodingKey
};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub key: DecodingKey,
    pub core: Arc<dyn Core + Send + Sync>
}
