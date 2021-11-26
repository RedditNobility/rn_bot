use hyper::{Body, Method, Request};

use crate::boterror::BotError;
use hyper::header::{HeaderName, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};

use crate::site::api_response::APIResponse;
use crate::site::model::AuthToken;
use crate::site::site_client::SiteClient;
use serde::Serialize;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::option::Option::Some;
pub mod api_response;
pub mod model;
pub mod site_client;

#[derive(Serialize)]
pub struct Authenticator {
    #[serde(skip_serializing)]
    pub token: Option<String>,
    pub username: String,
    pub password: String,
}

impl Authenticator {
    async fn login(&mut self, site: &SiteClient) -> Result<(), BotError> {
        let url = format!("{}/api/login/password", site.site_url);
        let request = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header(CONTENT_TYPE, "application/json")
            .header(USER_AGENT, "RoboticMonarch Discord by KingTux :)")
            .body(Body::from(serde_json::to_string(self)?));
        if request.is_err() {
            println!("{}", request.err().unwrap().to_string());
            return Err(BotError::Other("Good Question".to_string()));
        }
        let request = request.unwrap();

        let result = site.http.borrow().request(request).await;
        if result.is_err() {
            let option = result.err().unwrap();
            println!("{}", &option.to_string());
            return Err(BotError::HyperError(option));
        }
        let result = result.unwrap();
        if result.status() != hyper::StatusCode::OK {
            let value = hyper::body::to_bytes(result.into_body()).await;

            let value = String::from_utf8(value.unwrap().to_vec());
            println!("{}", value.unwrap());
            return Err(BotError::HTTPError(hyper::StatusCode::BAD_REQUEST));
        } else {
            let value = hyper::body::to_bytes(result.into_body()).await;

            let value = String::from_utf8(value.unwrap().to_vec());
            let string = value.unwrap();
            println!("{}", string.clone());
            let result1: Result<APIResponse<AuthToken>, serde_json::Error> =
                serde_json::from_str(&string);
            if let Ok(response) = result1 {
                if let Some(token) = response.data {
                    self.token = Some(token.token);
                    return Ok(());
                }
            }
            return Err(BotError::Other("Unable to login".to_string()));
        }
    }

    fn scopes(&self) -> Vec<String> {
        vec![String::from("*")]
    }

    fn headers(&self) -> HashMap<HeaderName, String> {
        let mut map = HashMap::new();
        map.insert(
            AUTHORIZATION,
            format!("Bearer {}", self.token.as_ref().unwrap().clone()),
        );
        map
    }

    fn oauth(&self) -> bool {
        true
    }
}
