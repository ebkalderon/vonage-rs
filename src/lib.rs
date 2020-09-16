//! [Vonage](https://www.vonage.com/communications-apis/) (formerly Nexmo) API bindings for Rust.
//!
//! This library (`vonage-rs`) is intended to be an idiomatic Rust equivalent of
//! [`vonage-node-sdk`]. It enables you to quickly add communications functionality to your
//! application, including sending SMS messages, making voice calls, text-to-speech, gathering
//! phone number insights, two-factor authentication, and more.
//!
//! [`vonage-node-sdk`]: https://github.com/Vonage/vonage-node-sdk
//!
//! To use this library, a Vonage account is required. If you don't have an account available, you
//! can always [sign up for free][sign-up].
//!
//! [sign-up]: https://dashboard.nexmo.com/sign-up?utm_source=DEV_REL&utm_medium=github
//!
//! See [developer.nexmo.com](https://developer.nexmo.com/) for upstream documentation.

#![deny(missing_debug_implementations)]
#![forbid(unsafe_code)]

pub use self::error::{Error, ErrorKind};
pub use self::sig::{Signature, SignatureMethod};

use std::fmt::{self, Debug, Formatter};

use hyper::body::Body;
use hyper::client::HttpConnector;
use hyper::service::Service;
use hyper::{Request, Response};
use hyper_tls::HttpsConnector;
use phonenumber::PhoneNumber;
use serde::Serialize;

use self::auth::{Auth, AuthBuilder};
use self::verify::Verify;

pub mod verify;

mod auth;
mod error;
mod sig;

const VONAGE_URL_BASE: &str = "https://api.nexmo.com";

/// A specialized [`Result`] error type for convenience.
///
/// [`Result`]: enum@std::result::Result
pub type Result<T> = std::result::Result<T, Error>;

type HyperClient = hyper::Client<HttpsConnector<HttpConnector>>;

/// A client to interface with the Vonage APIs.
pub struct Client<C = HyperClient> {
    http_client: C,
    authentication: Auth,
    sms_signature: Option<Signature>,
}

impl Client {
    /// Creates a new `Client` using the given API key and API secret pair. These values are
    /// defined in the [Vonage API dashboard](https://dashboard.nexmo.com/).
    ///
    /// # Authentication
    ///
    /// Note that not all Vonage products support API keys and secrets for authentication. See the
    /// support matrix in the official [authentication guide] for details. For alternative
    /// authentication methods, use [`Client::builder()`](#method.builder) instead.
    ///
    /// [authentication guide]: https://developer.nexmo.com/concepts/guides/authentication
    pub fn new(api_key: impl Into<String>, secret: impl Into<String>) -> Self {
        Client::builder().api_key(api_key, secret).build().unwrap()
    }

    /// Creates a builder to configure a new `Client`.
    ///
    /// This option allows for configuration of all available API authentication options.
    pub fn builder() -> ClientBuilder {
        let client = hyper::Client::builder().build(HttpsConnector::new());
        Client::from_service(client)
    }
}

impl<C> Client<C>
where
    C: Service<Request<Body>, Response = Response<Body>, Error = hyper::Error> + Clone,
{
    /// Creates a builder to configure a new `Client` built on the given `http_client`.
    ///
    /// Similar to [`Client::builder()`](#method.builder) except it allows for specifying a custom
    /// HTTP client instead of the default [`hyper::Client`]. This option allows for configuration
    /// of all available API authentication options.
    ///
    /// [`hyper::Client`]: https://docs.rs/hyper/0.13/hyper/client/struct.Client.html
    #[inline]
    pub fn from_service(http_client: C) -> ClientBuilder<C> {
        ClientBuilder::new(http_client)
    }

    /// Initiates a new [verify (2FA) request][verify] for the given phone number.
    ///
    /// [verify]: https://developer.nexmo.com/api/verify
    ///
    /// Returns `Err` if this client was not configured with an API key and API secret, and returns
    /// `Ok` otherwise.
    pub fn verify(&self, phone: PhoneNumber, brand: impl Into<String>) -> Result<Verify<C>> {
        // FIXME: While "brand" is a required field in regular verify requests, it is not present
        // at all in PSD2 verify requests. Currently, we discard the `brand` parameter if `.psd2()`
        // is called anywhere in the method chain. There might be a better way to do this.
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

/// A builder to configure a new [`Client`](./struct.Client.html).
///
/// This is returned from [`Client::builder()`](./struct.Client.html#method.builder). It is the
/// more flexible alternative to [`Client::new()`](./struct.Client.html#method.new), offering
/// more advanced configuration options, particularly for authentication.
///
/// According to the authentication method support matrix in the official [authentication guide],
/// each Vonage product supports exactly _one_ of the following authentication methods:
///
/// 1. API key and API secret
/// 2. [JSON Web Token (JWT)][jwt]
///
/// [authentication guide]: https://developer.nexmo.com/concepts/guides/authentication
/// [jwt]: https://jwt.io/
///
/// This builder lets you to specify one or both authentication methods when constructing a new
/// `Client`, depending on which Vonage products are to be used at runtime.
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

    /// Configures the API key and API secret pair for products that require this form of
    /// authentication.
    ///
    /// # Product support
    ///
    /// This authentication method is required for the [SMS](https://developer.nexmo.com/api/sms)
    /// and [Verify (2FA)](https://developer.nexmo.com/api/verify) products.
    pub fn api_key(mut self, api_key: impl Into<String>, secret: impl Into<String>) -> Self {
        self.auth_builder.api_key(api_key, secret);
        self
    }

    /// Configures the application ID and private [JWT] signing key for products that require this
    /// form of authentication.
    ///
    /// [JWT]: https://jwt.io/
    ///
    /// # Product support
    ///
    /// This authentication method is required for the
    /// [Voice](https://developer.nexmo.com/api/voice) product.
    pub fn jwt(mut self, app_id: impl Into<String>, private_key: impl Into<String>) -> Self {
        self.auth_builder.jwt(app_id, private_key);
        self
    }

    /// Configures the optional SMS signature to be used when sending messages and responding to
    /// webhooks.
    ///
    /// # Product support
    ///
    /// This feature is only supported by the [SMS](https://developer.nexmo.com/api/sms) product.
    pub fn sms_signature(mut self, sig: Signature) -> Self {
        self.sms_signature = Some(sig);
        self
    }

    /// Constructs the configured `Client`.
    ///
    /// Returns `Ok` if at least one authentication method has been specified, and returns `Err`
    /// otherwise.
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

fn encode_request_post<T>(path: &str, form: T) -> Result<Request<Body>>
where
    T: Serialize,
{
    use hyper::header::{ACCEPT, CONTENT_TYPE};

    let encoded = serde_urlencoded::to_string(form)?;
    let request = Request::builder()
        .method(hyper::Method::POST)
        .uri(format!("{}{}/json", VONAGE_URL_BASE, path))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(ACCEPT, "application/json")
        .body(encoded.into())
        .expect("http::RequestBuilder cannot fail");

    Ok(request)
}

fn encode_request_get<T>(path: &str, query_params: T) -> Result<Request<Body>>
where
    T: Serialize,
{
    use hyper::header::{ACCEPT, CONTENT_TYPE};

    let encoded = serde_urlencoded::to_string(query_params)?;
    let request = Request::builder()
        .method(hyper::Method::GET)
        .uri(format!("{}{}/json?{}", VONAGE_URL_BASE, path, encoded))
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .body(Body::empty())
        .expect("http::RequestBuilder cannot fail");

    Ok(request)
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
            .api_key("api key", "private key")
            .sms_signature(signature.clone())
            .build();

        let client = Client::builder()
            .jwt("app id", "private key")
            .sms_signature(signature)
            .build();
    }
}
