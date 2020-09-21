use std::convert::Infallible;

use serde::Serialize;
use thiserror::Error;
use warp::{http::StatusCode, Rejection, Reply};

pub type Result<T> = std::result::Result<T, Rejection>;

#[derive(Debug)]
pub struct SqlxError {
    pub error: sqlx::error::Error,
}

impl warp::reject::Reject for Error {}
impl warp::reject::Reject for SqlxError {}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("error executing DB query: {0}")]
    DBQueryError(#[from] sqlx::Error),
    #[error("error creating table: {0}")]
    DBInitError(sqlx::Error),
    #[error("error reading file: {0}")]
    ReadFileError(#[from] std::io::Error),
}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if err
        .find::<warp::filters::body::BodyDeserializeError>()
        .is_some()
    {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid Body";
    } else if let Some(e) = err.find::<Error>() {
        match e {
            Error::DBQueryError(_) => {
                code = StatusCode::BAD_REQUEST;
                message = "Could not Execute request";
            }
            _ => {
                eprintln!("unhandled application error: {:?}", err);
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = "Internal Server Error";
            }
        }
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method Not Allowed";
    } else {
        eprintln!("unhandled error: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }

    let json = warp::reply::json(&ErrorResponse {
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}
