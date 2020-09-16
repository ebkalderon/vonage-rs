//! Contains types for the `/verify/check` and `/verify/control` requests.

use std::hash::{Hash, Hasher};

use hyper::service::Service;
use hyper::{Body, Request, Response};
use serde::{Deserialize, Serialize};

use super::{RequestId, Result};
use std::fmt::{self, Debug, Formatter};

use crate::auth::{ApiKey, ApiSecret};
use crate::HyperClient;

/// A handle to a pending verify request.
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
    /// Attempts to cancel this pending verify request.
    #[inline]
    pub async fn cancel(&mut self) -> Result<()> {
        self.control_command(ControlCommand::Cancel).await
    }

    /// Attempts to trigger the next phase of the request [`Workflow`](./enum.Workflow.html).
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

        let request = crate::encode_request_post(
            "/verify/control",
            RequestBody {
                api_key: &self.api_key,
                api_secret: &self.api_secret,
                request_id: &self.request_id,
                cmd,
            },
        )?;

        let response = self.http_client.call(request).await?;
        super::decode_response(response).await
    }

    /// Checks whether the user-provided PIN code matches the expected value.
    ///
    /// Returns `Ok(Code::Match(_))` if the given PIN code is correct. Returns
    /// `Ok(Code::Mismatch(_))` if the given PIN code is incorrect, allowing up to 3 attempts.
    /// Returns `Err` if the code expired, the request was canceled, or some other error occurred.
    pub async fn check(mut self, code: &str) -> Result<Code<C>> {
        #[derive(Serialize)]
        struct RequestBody<'a> {
            api_key: &'a ApiKey,
            api_secret: &'a ApiSecret,
            request_id: &'a RequestId,
            code: &'a str,
        }

        let request = crate::encode_request_post(
            "/verify/check",
            RequestBody {
                api_key: &self.api_key,
                api_secret: &self.api_secret,
                request_id: &self.request_id,
                code,
            },
        )?;

        let response = self.http_client.call(request).await?;
        match super::decode_response(response).await {
            Ok(verified) => Ok(Code::Match(verified)),
            Err(e) if e.kind().is_code_mismatch() && self.attempts_remaining > 0 => {
                self.attempts_remaining -= 1;
                Ok(Code::Mismatch(self))
            }
            Err(e) => Err(e),
        }
    }

    /// Returns the number of check attempts remaining (maximum 3).
    #[inline]
    pub fn attempts_remaining(&self) -> usize {
        self.attempts_remaining
    }

    /// Returns the unique request ID.
    #[inline]
    pub fn request_id(&self) -> &RequestId {
        &self.request_id
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

/// The result of a PIN code check.
#[derive(Debug)]
pub enum Code<C> {
    /// The user-provided code matched the expected value.
    Match(Verified),
    /// The user-provided code didn't match the expected value.
    Mismatch(PendingVerify<C>),
}

/// Details returned when a verify request succeeded.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verified {
    /// The originating verify request ID.
    pub request_id: RequestId,
    /// The ID of the verification event, such as an SMS or TTS call.
    pub event_id: String,
    /// The cost incurred for this request.
    pub price: String,
    /// The currency code.
    pub currency: String,
    /// The value indicates the cost (in EUR) of the calls made and messages sent for the
    /// verification process. This field may not be present, depending on your pricing model.
    ///
    /// This value may be updated during and shortly after the request completes because user input
    /// events can overlap with message/call events. When this field is present, the total cost of
    /// the verification is the sum of this field and the price field.
    pub estimated_price_messages_sent: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ControlCommand {
    Cancel,
    TriggerNextEvent,
}
