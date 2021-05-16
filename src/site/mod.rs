use hyper::{Client, Method, Request, Body};
use hyper_tls::HttpsConnector;
use hyper::client::HttpConnector;
use crate::boterror::BotError;
use new_rawr::responses::auth::TokenResponseData;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT, HeaderName};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use crate::site::site_client::SiteClient;
use std::borrow::Borrow;
use crate::site::api_response::APIResponse;
use crate::site::model::AuthToken;
use std::option::Option::Some;

pub mod site_client;
mod api_response;
mod model;


pub struct Authenticator {
    pub token: Option<String>,
    pub username: String,
    pub password: String,
    pub client_key: String,
    pub client_id: i64,
}

impl Authenticator {
    async fn login(&mut self, site: &SiteClient) -> Result<(), BotError> {
        let url = format!("{}/api/login", site.site_url);
        let body = format!("username={}&password={}", &self.username, &self.password);
        let request = Request::builder().method(Method::POST).uri(url)
            .header(AUTHORIZATION, format!("Basic {}", format!("{}:{}", self.client_id.to_owned(), self.client_key.to_owned())))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(USER_AGENT, "RoboticMonarch Discord by KingTux :)")
            .body(Body::from(body));
        if request.is_err() {
            println!("{}", request.err().unwrap().to_string());
            return Err(BotError::Other("Good Question".to_string()));
        }
        let request = request.unwrap();

        let mut result = site.http.borrow().request(request).await;
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
            let result1: Result<APIResponse::<AuthToken>, serde_json::Error> = serde_json::from_str(&string);
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
        map.insert(AUTHORIZATION, format!("Bearer {}", self.token.as_ref().unwrap().clone()));
        map
    }

    fn oauth(&self) -> bool {
        true
    }
}