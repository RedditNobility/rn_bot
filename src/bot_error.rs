
use hyper::StatusCode;


use serenity::prelude::SerenityError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("HTTP Error {0}")]
    HTTPError(StatusCode),
    #[error("HTTP Error {0}")]
    HyperHTTPError(hyper::http::Error),
    #[error("HTTP Error {0}")]
    HyperError(hyper::Error),
    #[error("Serde_JSON Error {0}")]
    JSONError(serde_json::Error),
    #[error("Diesel Error {0}")]
    DBError(diesel::result::Error),
    #[error("Serenity Error {0}")]
    SerenityError(serenity::Error),
    #[error(" Error {0}")]
    Other(String),
}



impl From<diesel::result::Error> for BotError {
    fn from(err: diesel::result::Error) -> BotError {
        BotError::DBError(err)
    }
}
impl From<SerenityError> for BotError {
    fn from(err: SerenityError) -> BotError {
        BotError::SerenityError(err)
    }
}

impl From<hyper::Error> for BotError {
    fn from(err: hyper::Error) -> BotError {
        BotError::HyperError(err)
    }
}

impl From<serde_json::Error> for BotError {
    fn from(err: serde_json::Error) -> BotError {
        BotError::JSONError(err)
    }
}
