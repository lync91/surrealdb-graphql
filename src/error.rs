use async_graphql::ErrorExtensions;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use uuid::Uuid;

use crate::ctx::Ctx;

#[derive(Debug, PartialEq, Eq)]
pub struct ApiError {
    pub error: Error,
    pub req_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
    Generic { description: String },
    LoginFail,
    TicketDeleteFailIdNotFound { id: u64 },
    AuthFailNoJwtCookie,
    AuthFailJwtInvalid { source: String },
    AuthFailCtxNotInRequestExt,
    Serde { source: String },
    SurrealDb { source: String },
    SurrealDbNoResult { source: String, id: String },
    SurrealDbParse { source: String, id: String },
}

/// ApiError has to have the req_id to report to the client and implements IntoResponse.
pub type ApiResult<T> = core::result::Result<T, ApiError>;
/// Any error for storing before composing a response.
/// For errors that either don't affect the response, or are build before attaching the req_id.
pub type Result<T> = core::result::Result<T, Error>;

impl std::error::Error for Error {}
// We don't implement Error for ApiError, because it doesn't implement Display.
// Implementing Display for it triggers a generic impl From ApiError for gql-Error on async-graphql - and we want to implement it ourselves, to always include extensions on Errors. It would create conflicting implementations.

// for slightly less verbose error mappings
pub trait IntoApiError {
    fn into_api_error(self, ctx: &Ctx) -> ApiError;
}
impl<E: Into<Error>> IntoApiError for E {
    fn into_api_error(self, ctx: &Ctx) -> ApiError {
        ApiError {
            req_id: ctx.req_id(),
            error: self.into(),
        }
    }
}
impl ApiError {
    pub fn from<T: Into<Error>>(ctx: &Ctx) -> impl FnOnce(T) -> ApiError + '_ {
        |e| e.into_api_error(ctx)
    }
}

const INTERNAL: &str = "Internal error";
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic { description } => write!(f, "{description}"),
            Self::LoginFail => write!(f, "Login fail"),
            Self::TicketDeleteFailIdNotFound { id } => write!(f, "Ticket id {id} not found"),
            Self::AuthFailNoJwtCookie => write!(f, "You are not logged in"),
            Self::AuthFailJwtInvalid { .. } => {
                write!(f, "The provided JWT token is not valid")
            }
            Self::Serde { source } => write!(f, "Serde error - {source}"),
            Self::AuthFailCtxNotInRequestExt => write!(f, "{INTERNAL}"),
            Self::SurrealDb { .. } => write!(f, "{INTERNAL}"),
            Self::SurrealDbNoResult { id, .. } => write!(f, "No result for id {id}"),
            Self::SurrealDbParse { id, .. } => write!(f, "Couldn't parse id {id}"),
        }
    }
}

// REST error response
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        println!("->> {:<12} - into_response - {self:?}", "ERROR");
        let status_code = match self.error {
            Error::TicketDeleteFailIdNotFound { .. }
            | Error::Serde { .. }
            | Error::SurrealDbNoResult { .. }
            | Error::SurrealDbParse { .. } => StatusCode::BAD_REQUEST,
            Error::Generic { .. }
            | Error::LoginFail
            | Error::AuthFailNoJwtCookie
            | Error::AuthFailJwtInvalid { .. }
            | Error::AuthFailCtxNotInRequestExt
            | Error::SurrealDb { .. } => StatusCode::FORBIDDEN,
        };
        let body = Json(json!({
            "error": {
                "error": self.error.to_string(),
                "req_id": self.req_id.to_string()
            }
        }));
        let mut response = (status_code, body).into_response();
        // Insert the real Error into the response - for the logger
        response.extensions_mut().insert(self.error);
        response
    }
}

// for sending serialized keys through gql extensions
pub const ERROR_SER_KEY: &str = "error_ser";

// GQL error response
impl From<ApiError> for async_graphql::Error {
    fn from(value: ApiError) -> Self {
        Self::new(value.error.to_string())
            .extend_with(|_, e| e.set("req_id", value.req_id.to_string()))
            // storing the original as json in the error extension - for the logger
            .extend_with(|_, e| e.set(ERROR_SER_KEY, serde_json::to_string(&value.error).unwrap()))
    }
}

// External Errors
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde {
            source: value.to_string(),
        }
    }
}

impl From<surrealdb::Error> for Error {
    fn from(value: surrealdb::Error) -> Self {
        Self::SurrealDb {
            source: value.to_string(),
        }
    }
}

impl From<surrealdb::error::Db> for Error {
    fn from(value: surrealdb::error::Db) -> Self {
        Self::SurrealDb {
            source: value.to_string(),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(value: jsonwebtoken::errors::Error) -> Self {
        Self::AuthFailJwtInvalid {
            source: value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn display_description() {
        let err = super::Error::Generic {
            description: "super description".to_owned(),
        };
        assert_eq!(format!("{err}"), "super description");
        assert_eq!(err.to_string(), "super description");
    }
}
