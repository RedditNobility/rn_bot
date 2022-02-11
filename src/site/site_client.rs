use crate::bot_error::BotError;
use crate::site::api_response::APIResponse;
use crate::site::Authenticator;
use hyper::client::HttpConnector;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::site::model::User;
use std::borrow::Borrow;

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
        this
    }
    pub fn get_authenticator(&self) -> MutexGuard<Authenticator> {
        self.auth.lock().unwrap()
    }
    pub async fn get_json(&self, url: String) -> Result<String, BotError> {
        let url = format!("{}/{}", self.site_url, url);
        println!("{}", url);
        let request = Request::builder()
            .method(Method::GET)
            .uri(url)
            .header(
                AUTHORIZATION,
                format!(
                    "Bearer {}",
                    self.get_authenticator().token.as_ref().unwrap().clone()
                ),
            )
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(USER_AGENT, "RoboticMonarch Discord by KingTux :)")
            .body(Body::empty());
        if request.is_err() {
            println!("{}", request.err().unwrap());
            return Err(BotError::Other("Good Question".to_string()));
        }
        let request = request.unwrap();

        let result = self.http.borrow().request(request).await;
        if let Ok(re) = result {
            let value = hyper::body::to_bytes(re.into_body()).await;
            let string = String::from_utf8(value.unwrap().to_vec()).unwrap();
            println!("Data {}", &string);
            return Ok(string);
        } else if let Err(error) = result {
            return Err(BotError::HyperError(error));
        }
        Err(BotError::Other(
            "I am extremely curious how we got here".to_string(),
        ))
    }
    pub async fn get_user(&self, username: String) -> Result<Option<User>, BotError> {
        println!("HEY");
        let x = self.get_json(format!("moderator/user/{}", username)).await;
        if let Ok(value) = x {
            let result: Result<APIResponse<User>, serde_json::Error> =
                serde_json::from_str(&*value);
            if let Ok(response) = result {
                return Ok(response.data);
            } else if let Err(error) = result {
                return Err(BotError::JSONError(error));
            }
        } else if let Err(error) = x {
            return Err(error);
        }
        Err(BotError::Other(
            "I am extremely curious how we got here".to_string(),
        ))
    }
}
