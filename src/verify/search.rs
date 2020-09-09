use std::borrow::Cow;

use chrono::NaiveDateTime;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response, StatusCode};
use phonenumber::PhoneNumber;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

use super::{PendingVerify, RequestId, Result};
use crate::auth::{ApiKey, ApiSecret};

pub async fn search<'a, I, C>(iter: I) -> Result<Vec<Option<VerifyInfo>>>
where
    I: IntoIterator<Item = &'a PendingVerify<C>>,
    C: Service<Request<Body>, Response = Response<Body>, Error = hyper::Error> + Clone + 'static,
{
    #[derive(Serialize)]
    struct RequestBody<'a> {
        api_key: &'a ApiKey,
        api_secret: &'a ApiSecret,
        request_ids: Vec<&'a RequestId>,
    }

    #[derive(Deserialize)]
    enum ErrorCode {
        #[serde(rename = "101")]
        DoesNotExist,
    }

    #[allow(dead_code)]
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Response {
        Success(VerifyInfo),
        Error { status: ErrorCode },
    }

    let queries: Vec<_> = iter
        .into_iter()
        .map(|v| (&v.http_client, &v.api_key, &v.api_secret, &v.request_id))
        .collect();

    if queries.is_empty() {
        Ok(Vec::new())
    } else {
        let (client, api_key, api_secret, _) = queries[0];
        let mut http_client = client.clone();
        let request = super::encode_request(
            Method::GET,
            "/search",
            RequestBody {
                api_key,
                api_secret,
                request_ids: queries.into_iter().map(|(_, _, _, id)| id).collect(),
            },
        )?;

        let response = http_client.call(request).await?;
        match response.status() {
            StatusCode::OK => {}
            other => return Err(other.into()),
        }

        let bytes = hyper::body::to_bytes(response.into_body()).await?;
        let list: Vec<Response> = serde_json::from_slice(&bytes)?;
        let results = list
            .into_iter()
            .map(|res| match res {
                Response::Success(info) => Some(info),
                Response::Error { .. } => None,
            })
            .collect();

        Ok(results)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyInfo {
    pub request_id: RequestId,
    pub account_id: String,
    pub status: VerifyStatus,
    pub number: PhoneNumber,
    pub price: String,
    pub currency: String,
    pub sender_id: String,
    #[serde(serialize_with = "deserialize_date")]
    pub date_submitted: NaiveDateTime,
    #[serde(serialize_with = "deserialize_date")]
    pub date_finalized: NaiveDateTime,
    #[serde(serialize_with = "deserialize_date")]
    pub first_event_date: NaiveDateTime,
    #[serde(serialize_with = "deserialize_date")]
    pub last_event_date: NaiveDateTime,
    pub checks: Vec<Check>,
    pub events: Vec<(EventType, String)>,
    pub estimated_price_messages_sent: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub enum VerifyStatus {
    #[serde(rename = "IN PROGRESS")]
    InProgress,
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "EXPIRED")]
    Expired,
    #[serde(rename = "CANCELLED")]
    Cancelled,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    #[serde(serialize_with = "deserialize_date")]
    pub date_received: NaiveDateTime,
    pub code: String,
    pub status: CheckStatus,
    pub ip_address: Option<std::net::IpAddr>,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckStatus {
    Valid,
    Invalid,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Tts,
    Sms,
}

fn deserialize_date<'de, D>(deserializer: D) -> std::result::Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Cow::<'de, str>::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(|e| de::Error::custom(e))
}
