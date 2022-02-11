use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use hyper::StatusCode;

use serenity::client::Context;
use serenity::model::channel::Message;

use serenity::prelude::SerenityError;

/// Error type that occurs when an API request fails for some reason.
#[derive(Debug)]
pub enum BotError {
    /// Occurs when a listing has run out of results. Only used internally - the `Listing` class
    /// will not raise this when iterating.
    ExhaustedListing,
    /// Occurs when the API has returned a non-success error code. Important status codes include:
    /// - 401 Unauthorized - this usually occurs if your tokens are incorrect or invalid
    /// - 403 Forbidden - you are not allowed to access this, but your request was valid.
    HTTPError(StatusCode),
    HyperHTTPError(hyper::http::Error),
    /// Occurs if the HTTP response from Reddit was corrupt and Hyper could not parse it.
    HyperError(hyper::Error),
    /// Occurs if JSON deserialization fails. This will always be a bug, so please report it
    /// if it does occur, but the error type is provided so you can fail gracefully.
    JSONError(serde_json::Error),
    DBError(diesel::result::Error),
    SerenityError(serenity::Error),
    Other(String),
}

impl BotError {
    pub async fn discord_message(&self, message: &Message, error: &str, context: &Context) {
        let _msg = message
            .channel_id
            .send_message(&context.http, |m| {
                m.reference_message(message);
                m.embed(|e| {
                    e.title("An Error has occurred");
                    e.description(error.clone());
                    e.footer(|f| {
                        f.text("Robotic Monarch");
                        f
                    });

                    e
                });
                m
            })
            .await;
    }
}

impl Display for BotError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            BotError::HTTPError(_) => f.write_str("The API returned a non-success error code"),
            BotError::HyperError(h) => {
                f.write_str("An error occurred while processing the HTTP response\n");
                f.write_str(h.to_string().as_str())
            }
            BotError::HyperHTTPError(h) => {
                f.write_str("An error occurred while processing the HTTP response\n");
                f.write_str(h.to_string().as_str())
            }
            BotError::JSONError(j) => {
                f.write_str("Unable to parse Json\n");
                f.write_str(j.to_string().as_str())
            }
            BotError::Other(s) => f.write_str(s.clone().as_str()),
            BotError::DBError(error) => {
                f.write_str("Unable to execute query.\n");
                f.write_str(error.to_string().as_str())
            }
            BotError::SerenityError(error) => {
                f.write_str("Failed to use Discord API\n");
                f.write_str(error.to_string().as_str())
            }

            _ => f.write_str("This error should not have occurred. Please file a bug"),
        }
    }
}

impl Error for BotError {}

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
