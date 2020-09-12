use std::fmt::{self, Debug, Formatter};
use std::time::SystemTime;

use anyhow::anyhow;
use hyper::header::{HeaderName, AUTHORIZATION};
use serde::Serialize;

use crate::{Error, Result};

static CLOCK_SEQUENCE: uuid::v1::Context = uuid::v1::Context::new(0);

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize)]
pub struct ApiKey(String);

#[derive(Clone, Default, Eq, Hash, PartialEq, Serialize)]
pub struct ApiSecret(String);

impl Debug for ApiSecret {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_tuple(stringify!(ApiSecret))
            .field(&"<secret>")
            .finish()
    }
}

#[derive(Default)]
pub struct Auth {
    api_key: Option<(ApiKey, ApiSecret)>,
    jwt: Option<(String, String)>,
}

impl Auth {
    pub fn builder() -> AuthBuilder {
        AuthBuilder {
            inner: Auth::default(),
        }
    }

    pub fn api_key_pair(&self) -> Result<&(ApiKey, ApiSecret)> {
        self.api_key
            .as_ref()
            .ok_or_else(|| Error::new_auth(anyhow!("product requires an API key to authenticate")))
    }

    pub fn to_auth_header(&self) -> Result<(HeaderName, String)> {
        let (ApiKey(key), ApiSecret(secret)) = self.api_key_pair()?;
        let header_value = format!("Bearer {}", base64::encode(format!("{}:{}", key, secret)));
        Ok((AUTHORIZATION, header_value))
    }

    #[rustfmt::skip]
    pub fn generate_jwt<T: Serialize>(&self, claims: T) -> Result<String> {
        use chrono::Utc;
        use jsonwebtoken::{EncodingKey, Header};
        use serde_json::json;

        if let Some((application_id, private_key)) = self.jwt.as_ref() {
            let mut claims = serde_json::to_value(claims)
                .map_err(|e| anyhow!("could not deserialize claims: {}", e))
                .map_err(Error::new_auth)?;

            if let Some(ref mut map) = claims.as_object_mut() {
                map.insert("application_id".into(), json!(application_id));
                map.entry("iat").or_insert_with(|| json!(Utc::now().timestamp()));
                map.entry("jti").or_insert_with(|| json!(gen_uuid_v1_str()));
            }

            let private_key = EncodingKey::from_secret(private_key.as_bytes());
            let token = jsonwebtoken::encode(&Header::default(), &claims, &private_key)?;
            Ok(token)
        } else {
            Err(Error::new_auth(anyhow!(
                "product requires an application ID and private key to generate JWTs"
            )))
        }
    }
}

impl Debug for Auth {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct(stringify!(Auth))
            .field("api_key", &self.api_key)
            .field("jwt", &self.jwt.as_ref().map(|(k, _)| (k, "<private-key>")))
            .finish()
    }
}

fn gen_uuid_v1_str() -> String {
    use uuid::{v1::Timestamp, Uuid};

    // Source: https://github.com/uuidjs/uuid/blob/0e6c10ba1bf9517796ff23c052fc0468eedfd5f4/src/v1.js#L32-L40
    let mut node_id: [u8; 6] = rand::random();
    node_id[0] = node_id[0] | 0x01;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime is before the Unix epoch");

    let time = Timestamp::from_unix(&CLOCK_SEQUENCE, now.as_secs(), now.subsec_nanos());
    Uuid::new_v1(time, &node_id)
        .expect("node_id must be of length 6")
        .to_string()
}

#[derive(Debug)]
pub struct AuthBuilder {
    inner: Auth,
}

impl AuthBuilder {
    pub fn api_key(&mut self, api_key: impl Into<String>, secret: impl Into<String>) -> &mut Self {
        self.inner.api_key = Some((ApiKey(api_key.into()), ApiSecret(secret.into())));
        self
    }

    pub fn jwt(&mut self, app_id: impl Into<String>, private_key: impl Into<String>) -> &mut Self {
        self.inner.jwt = Some((app_id.into(), private_key.into()));
        self
    }

    pub fn build(self) -> Result<Auth> {
        if self.inner.api_key.is_some() || self.inner.jwt.is_some() {
            Ok(self.inner)
        } else {
            Err(Error::new_auth(anyhow!("no credentials specified")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_auth() {
        let mut builder = Auth::builder();
        builder.api_key("hi", "there").jwt("hi", "there");
        let _ = builder.build().unwrap();
    }
}
