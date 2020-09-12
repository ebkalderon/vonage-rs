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

#[doc(hidden)]
pub trait Verification: Default + Serialize {
    const PATH: &'static str;
}

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
                brand,
                ..Default::default()
            },
        })
    }

    pub fn psd2(self, payee: impl Into<String>, amount_eur: f64) -> Verify<C, psd2::Psd2> {
        Verify {
            http_client: self.http_client,
            request_body: RequestBody {
                api_key: self.request_body.api_key,
                api_secret: self.request_body.api_secret,
                number: self.request_body.number,
                country: self.request_body.country,
                brand: self.request_body.brand,
                sender_id: self.request_body.sender_id,
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

    pub fn language(mut self, lang: Language) -> Self {
        self.request_body.req_specific.language = Some(lang);
        self
    }
}

impl<C> Verify<C, psd2::Psd2> {
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
    pub fn country(mut self, country: Id) -> Self {
        self.request_body.country = Some(country);
        self
    }

    pub fn sender_id(mut self, id: impl Into<String>) -> Self {
        self.request_body.sender_id = Some(id.into());
        self
    }

    pub fn code_length(mut self, len: CodeLength) -> Self {
        self.request_body.code_length = Some(len);
        self
    }

    pub fn pin_expiry(mut self, valid_for: Duration) -> Self {
        self.request_body.pin_expiry = Some(valid_for.as_secs());
        self
    }

    pub fn next_event_wait(mut self, wait: Duration) -> Self {
        self.request_body.next_event_wait = Some(wait.as_secs());
        self
    }

    pub fn workflow(mut self, w: Workflow) -> Self {
        self.request_body.workflow_id = Some(w);
        self
    }

    pub async fn send(mut self) -> Result<PendingVerify<C>> {
        #[derive(Deserialize)]
        struct ResponseBody {
            request_id: RequestId,
        }

        let req = super::encode_request(Method::POST, V::PATH, &self.request_body)?;
        let res = self.http_client.call(req).await?;
        let ResponseBody { request_id } = super::decode_response(res).await?;

        Ok(PendingVerify {
            http_client: self.http_client,
            api_key: self.request_body.api_key,
            api_secret: self.request_body.api_secret,
            request_id,
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
    brand: String,
    sender_id: Option<String>,
    code_length: Option<CodeLength>,
    pin_expiry: Option<u64>,
    next_event_wait: Option<u64>,
    workflow_id: Option<Workflow>,
    #[serde(flatten)]
    req_specific: V,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(into = "u8")]
pub enum Workflow {
    SmsTtsTts,
    SmsSmsTts,
    TtsTts,
    SmsSms,
    SmsTts,
    Sms,
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
