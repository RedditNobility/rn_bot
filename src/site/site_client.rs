use crate::site::Authenticator;
use std::sync::{Arc, Mutex, MutexGuard};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use hyper::Client;

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
        if result.is_err() {
            panic!("Unable to login")
        }
        return this;
    }
    pub fn get_authenticator(&self) -> MutexGuard<Authenticator> {
        self.auth.lock().unwrap()
    }
}
