use crate::site::Authenticator;
use std::sync::{Arc, Mutex, MutexGuard};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use hyper::{Client, Method, Request, Body};
use hyper::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use crate::site::api_response::APIResponse;
use crate::boterror::BotError;
use serde::Serialize;
use std::borrow::Borrow;
use crate::site::model::User;

pub struct SiteClient {
    pub site_url: String,
    pub auth: Arc<Mutex<Authenticator>>,
    pub http: Client<HttpsConnector<HttpConnector>>,
}

impl SiteClient {
    pub async fn new(auth: Authenticator) -> SiteClient {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let this = SiteClient {
            site_url: std::env::var("SITE").unwrap_or("https://redditnobility.org".to_string()),
            auth: Arc::new(Mutex::new(auth)),
            http: client,
        };
        let result = this.get_authenticator().login(&this).await;
        if let Err(error) = result {
            panic!("Unable to login {}", error);
        }
        return this;
    }
    pub fn get_authenticator(&self) -> MutexGuard<Authenticator> {
        self.auth.lock().unwrap()
    }
    pub async fn get_json(&self, url: String) -> Result<String, BotError> {
        let url = format!("{}/{}", self.site_url, url);
        println!("{}", url);
        let request = Request::builder().method(Method::GET).uri(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.get_authenticator().token.as_ref().unwrap().clone()))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(USER_AGENT, "RoboticMonarch Discord by KingTux :)")
            .body(Body::empty());
        if request.is_err() {
            println!("{}", request.err().unwrap().to_string());
            return Err(BotError::Other("Good Question".to_string()));
        }
        let request = request.unwrap();

        let mut result = self.http.borrow().request(request).await;
        if let Ok(re) = result {
            let value = hyper::body::to_bytes(re.into_body()).await;
            let string = String::from_utf8(value.unwrap().to_vec()).unwrap();
            println!("Data {}", &string);
            return Ok(string);
        } else if let Err(error) = result {
            return Err(BotError::HyperError(error));
        }
        return Err(BotError::Other("I am extremely curious how we got here".to_string()));
    }
    pub async fn get_user(&self, username: String) -> Result<Option<User>, BotError> {
        let x = self.get_json(format!("api/user/{}", username)).await;
        if let Ok(value) = x {
            let result: Result<APIResponse<User>, serde_json::Error> = serde_json::from_str(&*value);
            if let Ok(response) = result {
                return Ok(response.data);
            } else if let Err(error) = result {
                return Err(BotError::JSONError(error));
            }
        } else if let Err(error) = x {
            return Err(error);
        }
        return Err(BotError::Other("I am extremely curious how we got here".to_string()));
    }
}

