pub use self::pending::*;
pub use self::request::*;
pub use self::search::*;

use std::fmt::{self, Debug, Display, Formatter};

use hyper::{http, Body, Method, Response, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{Result, VONAGE_URL_BASE};

mod pending;
mod request;
mod search;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct RequestId(String);

impl Display for RequestId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

fn encode_request<T>(method: Method, path: &str, body: T) -> Result<http::Request<Body>>
where
    T: Serialize,
{
    use hyper::header::CONTENT_TYPE;

    let encoded = serde_urlencoded::to_string(body)?;
    let request = http::Request::builder()
        .method(method)
        .uri(format!("{}/verify{}/json", VONAGE_URL_BASE, path))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(encoded.into())
        .expect("http::RequestBuilder cannot fail");

    Ok(request)
}

async fn decode_response<T, E>(response: Response<Body>) -> Result<T>
where
    T: DeserializeOwned,
    E: DeserializeOwned + Display,
{
    #[derive(Deserialize)]
    enum SuccessCode {
        #[serde(rename = "0")]
        Success,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(untagged)]
    enum Response<T, E> {
        Success {
            #[allow(dead_code)]
            status: SuccessCode,
            #[serde(flatten)]
            inner: T,
        },
        Error {
            #[allow(dead_code)]
            request_id: Option<RequestId>,
            status: E,
            error_text: String,
        },
    }

    match response.status() {
        StatusCode::OK => {}
        other => return Err(other.into()),
    }

    let bytes = hyper::body::to_bytes(response.into_body()).await?;
    match serde_json::from_slice::<Response<_, E>>(&bytes)? {
        Response::Success { inner, .. } => Ok(inner),
        Response::Error {
            status, error_text, ..
        } => Err(format!("{}: {}", status, error_text).into()),
    }
}

#[derive(Deserialize)]
enum ErrorCode {
    #[serde(rename = "1")]
    Throttled,
    #[serde(rename = "2")]
    MissingParam,
    #[serde(rename = "3")]
    InvalidParam,
    #[serde(rename = "4")]
    InvalidCredentials,
    #[serde(rename = "5")]
    InternalError,
    #[serde(rename = "6")]
    RouteError,
    #[serde(rename = "7")]
    BlacklistedPhone,
    #[serde(rename = "8")]
    BarredApiKey,
    #[serde(rename = "9")]
    ExceededPartnerQuota,
    #[serde(rename = "10")]
    Concurrent,
    #[serde(rename = "15")]
    UnsupportedNetwork,
    #[serde(rename = "16")]
    CodeMismatch,
    #[serde(rename = "17")]
    TooManyAttempts,
    #[serde(rename = "19")]
    CancelOrTriggerNextFailed,
    #[serde(rename = "20")]
    PinCodeNotSupported,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ErrorCode::Throttled => f.write_str("error 1"),
            ErrorCode::MissingParam => f.write_str("error 2"),
            ErrorCode::InvalidParam => f.write_str("error 3"),
            ErrorCode::InvalidCredentials => f.write_str("error 4"),
            ErrorCode::InternalError => f.write_str("error 5"),
            ErrorCode::RouteError => f.write_str("error 6"),
            ErrorCode::BlacklistedPhone => f.write_str("error 7"),
            ErrorCode::BarredApiKey => f.write_str("error 8"),
            ErrorCode::ExceededPartnerQuota => f.write_str("error 9"),
            ErrorCode::Concurrent => f.write_str("error 10"),
            ErrorCode::UnsupportedNetwork => f.write_str("error 15"),
            ErrorCode::CodeMismatch => f.write_str("error 16"),
            ErrorCode::TooManyAttempts => f.write_str("error 17"),
            ErrorCode::CancelOrTriggerNextFailed => f.write_str("error 19"),
            ErrorCode::PinCodeNotSupported => f.write_str("error 20"),
        }
    }
}
