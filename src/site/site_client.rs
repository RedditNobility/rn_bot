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
            site_url: std::env::var("SITE").expect("Missing Site Value"),
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
            .body(Body::empty())?;

        let re = self.http.borrow().request(request).await?;
        if !re.status().is_success(){
            return Err(BotError::HTTPError(re.status()))
        }
        let value = hyper::body::to_bytes(re.into_body()).await?;
        let string = String::from_utf8(value.to_vec()).unwrap();
        Ok(string)
    }
    pub async fn get_user(&self, username: &str) -> Result<Option<User>, BotError> {
        let value = self.get_json(format!("moderator/user/{}", username)).await?;
        let result: APIResponse<User> = serde_json::from_str(&*value)?;
        Ok(result.data)
    }
}
