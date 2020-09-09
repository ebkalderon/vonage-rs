#![deny(missing_debug_implementations)]
#![forbid(unsafe_code)]

pub use self::sig::{Signature, SignatureMethod};

use std::fmt::{self, Debug, Formatter};

use hyper::body::Body;
use hyper::client::HttpConnector;
use hyper::service::Service;
use hyper::{Request, Response};
use hyper_tls::HttpsConnector;
use phonenumber::PhoneNumber;

use self::auth::{Auth, AuthBuilder, AuthError};
use self::verify::Verify;

pub mod verify;

mod auth;
mod sig;

const VONAGE_URL_BASE: &str = "https://api.nexmo.com";

pub type Result<T> = std::result::Result<T, Error>;

type HyperClient = hyper::Client<HttpsConnector<HttpConnector>>;

pub struct Client<C = HyperClient> {
    http_client: C,
    authentication: Auth,
    sms_signature: Option<Signature>,
}

impl Client {
    pub fn new(api_key: impl Into<String>, secret: impl Into<String>) -> Self {
        Client::builder()
            .auth_api_key(api_key, secret)
            .build()
            .unwrap()
    }

    pub fn builder() -> ClientBuilder {
        let client = hyper::Client::builder().build(HttpsConnector::new());
        Client::with_client(client)
    }
}

impl<C> Client<C>
where
    C: Service<Request<Body>, Response = Response<Body>, Error = hyper::Error> + Clone,
{
    #[inline]
    pub fn with_client(http_client: C) -> ClientBuilder<C> {
        ClientBuilder::new(http_client)
    }

    pub fn verify(&self, phone: PhoneNumber, brand: impl Into<String>) -> Result<Verify<C>> {
        Verify::new(
            self.http_client.clone(),
            &self.authentication,
            phone,
            brand.into(),
        )
    }
}

impl<C> Debug for Client<C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct(stringify!(Client))
            .field("authentication", &self.authentication)
            .field("sms_signature", &self.sms_signature)
            .finish()
    }
}

pub struct ClientBuilder<C = HyperClient> {
    http_client: C,
    auth_builder: AuthBuilder,
    sms_signature: Option<Signature>,
}

impl<C> ClientBuilder<C> {
    fn new(http_client: C) -> Self {
        ClientBuilder {
            http_client,
            auth_builder: Auth::builder(),
            sms_signature: None,
        }
    }

    pub fn auth_api_key<T, U>(mut self, api_key: T, secret: U) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        self.auth_builder.api_key(api_key, secret);
        self
    }

    pub fn auth_jwt<T, U>(mut self, application_id: T, private_key: U) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        self.auth_builder.jwt(application_id, private_key);
        self
    }

    pub fn sms_signature(mut self, sig: Signature) -> Self {
        self.sms_signature = Some(sig);
        self
    }

    pub fn build(self) -> Result<Client<C>> {
        Ok(Client {
            http_client: self.http_client,
            authentication: self.auth_builder.build()?,
            sms_signature: self.sms_signature,
        })
    }
}

impl<C> Debug for ClientBuilder<C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct(stringify!(ClientBuilder))
            .field("auth_builder", &self.auth_builder)
            .field("sms_signature", &self.sms_signature)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("authentication error: {0}")]
    Auth(#[from] AuthError),
    #[error("HTTP error: {0}")]
    Http(#[from] hyper::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("www-urlencode error: {0}")]
    UrlEncode(#[from] serde_urlencoded::ser::Error),
    #[error("received unexpected status code: {0}")]
    Status(hyper::StatusCode),
    #[error("{0}")]
    Custom(String),
}

impl From<hyper::StatusCode> for Error {
    fn from(code: hyper::StatusCode) -> Self {
        Error::Status(code)
    }
}

impl From<String> for Error {
    fn from(text: String) -> Self {
        Error::Custom(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_client() {
        // client with api key and secret by default.
        let client = Client::new("api key", "private key");

        // Different methods of creating signatures.
        let signature = Signature::new("secret");
        let signature = Signature::with_method(SignatureMethod::Md5Hash, "secret");

        let client = Client::builder()
            .auth_api_key("api key", "private key")
            .sms_signature(signature.clone())
            .build();

        let client = Client::builder()
            .auth_jwt("app id", "private key")
            .sms_signature(signature)
            .build();
    }
}
