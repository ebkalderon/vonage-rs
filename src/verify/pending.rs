use std::hash::{Hash, Hasher};

use hyper::service::Service;
use hyper::{Body, Method, Request, Response};
use serde::{Deserialize, Serialize};

use super::{RequestId, Result};
use std::fmt::{self, Debug, Formatter};

use crate::auth::{ApiKey, ApiSecret};
use crate::{Error, HyperClient};

const VERIFY_CONTROL_PATH: &str = "/control";

pub struct PendingVerify<C = HyperClient> {
    pub(super) http_client: C,
    pub(super) api_key: ApiKey,
    pub(super) api_secret: ApiSecret,
    pub(super) request_id: RequestId,
    pub(super) attempts_remaining: usize,
}

impl<C> PendingVerify<C>
where
    C: Service<Request<Body>, Response = Response<Body>, Error = hyper::Error>,
{
    #[inline]
    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    #[inline]
    pub async fn cancel(&mut self) -> Result<()> {
        self.control_command(ControlCommand::Cancel).await
    }

    #[inline]
    pub async fn trigger_next_event(&mut self) -> Result<()> {
        self.control_command(ControlCommand::TriggerNextEvent).await
    }

    async fn control_command(&mut self, cmd: ControlCommand) -> Result<()> {
        #[derive(Serialize)]
        struct RequestBody<'a> {
            api_key: &'a ApiKey,
            api_secret: &'a ApiSecret,
            request_id: &'a RequestId,
            cmd: ControlCommand,
        }

        let request = super::encode_request(
            Method::POST,
            VERIFY_CONTROL_PATH,
            RequestBody {
                api_key: &self.api_key,
                api_secret: &self.api_secret,
                request_id: &self.request_id,
                cmd,
            },
        )?;

        let response = self
            .http_client
            .call(request)
            .await
            .map_err(Error::new_verify)?;

        super::decode_response(response).await
    }

    pub async fn check(mut self, code: &str) -> Result<Code<C>> {
        #[derive(Serialize)]
        struct RequestBody<'a> {
            api_key: &'a ApiKey,
            api_secret: &'a ApiSecret,
            request_id: &'a RequestId,
            code: &'a str,
        }

        let request = super::encode_request(
            Method::POST,
            "/check",
            RequestBody {
                api_key: &self.api_key,
                api_secret: &self.api_secret,
                request_id: &self.request_id,
                code,
            },
        )?;

        let response = self
            .http_client
            .call(request)
            .await
            .map_err(Error::new_verify)?;

        match super::decode_response(response).await {
            Ok(verified) => Ok(Code::Match(verified)),
            Err(e) if e.kind().is_code_mismatch() && self.attempts_remaining > 0 => {
                self.attempts_remaining -= 1;
                Ok(Code::Mismatch(self))
            }
            Err(e) => Err(e),
        }
    }

    #[inline]
    pub fn attempts_remaining(&self) -> usize {
        self.attempts_remaining
    }
}

impl<C> Debug for PendingVerify<C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct(stringify!(PendingVerify))
            .field("api_key", &self.api_key)
            .field("api_secret", &self.api_secret)
            .field("request_id", &self.request_id)
            .field("attempts_remaining", &self.attempts_remaining)
            .finish()
    }
}

impl<C> Eq for PendingVerify<C> {}

impl<C> PartialEq for PendingVerify<C> {
    fn eq(&self, other: &Self) -> bool {
        self.request_id == other.request_id
    }
}

impl<C> Hash for PendingVerify<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.request_id.hash(state);
    }
}

#[derive(Debug)]
pub enum Code<C> {
    Match(Verified),
    Mismatch(PendingVerify<C>),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verified {
    pub request_id: RequestId,
    pub event_id: String,
    pub price: String,
    pub currency: String,
    pub estimated_price_messages_sent: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ControlCommand {
    Cancel,
    TriggerNextEvent,
}
