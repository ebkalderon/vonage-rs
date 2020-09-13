//! Contains types for the `/verify` and `/verify/psd2` requests.

pub use self::normal::Language;
pub use self::psd2::Language as Psd2Language;

use std::fmt::{self, Debug, Formatter};
use std::time::Duration;

use hyper::body::Body;
use hyper::service::Service;
use hyper::{Method, Request, Response};
use phonenumber::{country::Id, PhoneNumber};
use serde::{Deserialize, Serialize};

use super::{PendingVerify, RequestId, Result};
use crate::auth::{ApiKey, ApiSecret, Auth};

mod normal;
mod psd2;

const MAX_CHECK_ATTEMPTS: usize = 3;

#[doc(hidden)]
pub trait Verification: Default + Serialize {
    const PATH: &'static str;
}

/// A builder to configure a new verify request.
///
/// It corresponds to either a [`/verify`] or [`/verify/psd2`] request, depending on the settings
/// used.
///
/// [`/verify`]: https://developer.nexmo.com/api/verify#verifyRequest
/// [`/verify/psd2`]: https://developer.nexmo.com/api/verify#verifyRequestWithPSD2
pub struct Verify<C, V: Verification = normal::Normal> {
    http_client: C,
    request_body: RequestBody<V>,
}

impl<C> Verify<C> {
    pub(crate) fn new(
        http_client: C,
        auth: &Auth,
        phone: PhoneNumber,
        brand: String,
    ) -> Result<Self> {
        let (api_key, api_secret) = auth.api_key_pair()?;
        Ok(Verify {
            http_client,
            request_body: RequestBody {
                api_key: api_key.clone(),
                api_secret: api_secret.clone(),
                number: phone.to_string(),
                req_specific: normal::Normal {
                    brand,
                    ..Default::default()
                },
                ..Default::default()
            },
        })
    }

    /// Sets the 11-character alphanumeric string that represents the identity of the sender of the
    /// request.
    ///
    /// Depending on the destination of the phone number you are sending the verification SMS to,
    /// restrictions might apply.
    pub fn sender_id(mut self, id: impl Into<String>) -> Self {
        self.request_body.req_specific.sender_id = Some(id.into());
        self
    }

    /// Sets the language used for the SMS or TTS message to be sent.
    ///
    /// By default, the SMS or text-to-speech (TTS) message is generated in the locale that matches
    /// the number. For example, the text message or TTS message for a `33*` number is sent in
    /// French.
    ///
    /// Use this parameter to explicitly control the language used for the Verify request.
    pub fn language(mut self, lang: Language) -> Self {
        self.request_body.req_specific.language = Some(lang);
        self
    }

    /// Changes this builder to construct a [Payment Services Directive 2 (PSD2)] request.
    ///
    /// [Payment Services Directive 2 (PSD2)]: https://developer.nexmo.com/api/verify#verifyRequestWithPSD2
    pub fn psd2(self, payee: impl Into<String>, amount_eur: f64) -> Verify<C, psd2::Psd2> {
        Verify {
            http_client: self.http_client,
            request_body: RequestBody {
                api_key: self.request_body.api_key,
                api_secret: self.request_body.api_secret,
                number: self.request_body.number,
                country: self.request_body.country,
                code_length: self.request_body.code_length,
                pin_expiry: self.request_body.pin_expiry,
                next_event_wait: self.request_body.next_event_wait,
                workflow_id: self.request_body.workflow_id,
                req_specific: psd2::Psd2 {
                    payee: payee.into(),
                    amount: amount_eur,
                    language: None,
                },
            },
        }
    }
}

impl<C> Verify<C, psd2::Psd2> {
    /// Sets the language used for the SMS or TTS message to be sent.
    ///
    /// By default, the SMS or text-to-speech (TTS) message is generated in the locale that matches
    /// the number. For example, the text message or TTS message for a `33*` number is sent in
    /// French.
    ///
    /// Use this parameter to explicitly control the language used for the PSD2 Verify request.
    pub fn language(mut self, lang: Psd2Language) -> Self {
        self.request_body.req_specific.language = Some(lang);
        self
    }
}

impl<C, V> Verify<C, V>
where
    C: Service<Request<Body>, Response = Response<Body>, Error = hyper::Error>,
    V: Verification,
{
    /// Overrides the country code of the phone number.
    pub fn country(mut self, country: Id) -> Self {
        self.request_body.country = Some(country);
        self
    }

    /// Sets the verification code length.
    pub fn code_length(mut self, len: CodeLength) -> Self {
        self.request_body.code_length = Some(len);
        self
    }

    /// Sets how long the generated verification code is valid for.
    ///
    /// When you specify both `pin_expiry` and `next_event_wait`, then `pin_expiry` (in seconds)
    /// must be an integer multiple of `next_event_wait`, otherwise `pin_expiry` defaults to equal
    /// `next_event_wait`.
    ///
    /// This duration can be set to 60 seconds, at minimum, and 3600 seconds, at maximum.
    pub fn pin_expiry(mut self, valid_for: Duration) -> Self {
        self.request_body.pin_expiry = Some(valid_for.as_secs());
        self
    }

    /// Sets the wait time in between attempts to deliver the verification code.
    ///
    /// This setting affects the behavior of
    /// [`PendingVerify::trigger_next_event()`](./struct.PendingVerify.html#method.trigger_next_event).
    ///
    /// This duration can be set to 60 seconds, at minimum, and 900 seconds, at maximum.
    pub fn next_event_wait(mut self, wait: Duration) -> Self {
        self.request_body.next_event_wait = Some(wait.as_secs());
        self
    }

    /// Sets the predefined sequence of SMS and TTS (text-to-speech) actions to use in order to
    /// convey the PIN to your user.
    ///
    /// If unspecified, this value defaults to `Workflow::SmsTtsTts`.
    pub fn workflow(mut self, w: Workflow) -> Self {
        self.request_body.workflow_id = Some(w);
        self
    }

    /// Submits the verify request and returns a `PendingVerify` to control its state.
    pub async fn send(mut self) -> Result<PendingVerify<C>> {
        #[derive(Deserialize)]
        struct ResponseBody {
            request_id: RequestId,
        }

        let request = super::encode_request(Method::POST, V::PATH, &self.request_body)?;
        let response = self.http_client.call(request).await?;
        let ResponseBody { request_id } = super::decode_response(response).await?;

        Ok(PendingVerify {
            http_client: self.http_client,
            api_key: self.request_body.api_key,
            api_secret: self.request_body.api_secret,
            request_id,
            attempts_remaining: MAX_CHECK_ATTEMPTS,
        })
    }
}

impl<C, V: Debug + Verification> Debug for Verify<C, V> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct(stringify!(Verify))
            .field("request_body", &self.request_body)
            .finish()
    }
}

#[derive(Debug, Default, Serialize)]
#[serde(deny_unknown_fields)]
struct RequestBody<V: Verification> {
    api_key: ApiKey,
    api_secret: ApiSecret,
    number: String,
    country: Option<Id>,
    code_length: Option<CodeLength>,
    pin_expiry: Option<u64>,
    next_event_wait: Option<u64>,
    workflow_id: Option<Workflow>,
    #[serde(flatten)]
    req_specific: V,
}

/// The number of digits in a verification code.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(into = "u8")]
pub enum CodeLength {
    Four,
    Six,
}

impl Into<u8> for CodeLength {
    fn into(self) -> u8 {
        match self {
            CodeLength::Four => 4,
            CodeLength::Six => 6,
        }
    }
}

/// A list of possible verify workflows supported by the Verify API ([source]).
///
/// [source]: https://developer.nexmo.com/verify/guides/workflows-and-events
///
/// The Verify API gives the best chance of reaching your users by combining SMS and TTS
/// (text-to-speech) calls in sequence. The basic model is that when you create a Verify request,
/// it is assigned a `request_id` and Vonage will begin the sequence of actions to reach the user
/// with a PIN code.
///
/// When the user sends you the code, you send the code along with the `request_id` through to
/// Vonage to check the code is correct.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(into = "u8")]
pub enum Workflow {
    /// Send a PIN code by text message, follow up with two subsequent voice calls if the request
    /// has not been verified.
    ///
    /// This is the default workflow.
    SmsTtsTts,
    /// Send a PIN code by text message, follow up with a second text message and finally a voice
    /// call if the request has not been verified.
    SmsSmsTts,
    /// Call the user and speak a PIN code, follow up with a second call if the request has not
    /// been verified.
    TtsTts,
    /// Send a PIN code by text message, follow up with a second text message if the code has not
    /// been verified.
    SmsSms,
    /// Send a PIN code by text message, and follow up with a voice call if the code has not been
    /// verified.
    SmsTts,
    /// Send a PIN code by text message once only.
    Sms,
    /// Call the user and speak a PIN code once only.
    Tts,
}

impl Into<u8> for Workflow {
    fn into(self) -> u8 {
        match self {
            Workflow::SmsTtsTts => 1,
            Workflow::SmsSmsTts => 2,
            Workflow::TtsTts => 3,
            Workflow::SmsSms => 4,
            Workflow::SmsTts => 5,
            Workflow::Sms => 6,
            Workflow::Tts => 7,
        }
    }
}
