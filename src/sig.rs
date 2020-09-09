use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Formatter};

use hmac::{digest::Digest, Hmac, Mac, NewMac};

pub(crate) struct Hash(pub String);

#[derive(Clone)]
pub struct Signature {
    secret: Cow<'static, str>,
    method: SignatureMethod,
}

impl Signature {
    #[inline]
    pub fn new(secret: impl Into<Cow<'static, str>>) -> Self {
        Signature::with_method(SignatureMethod::default(), secret)
    }

    #[inline]
    pub fn with_method<T>(method: SignatureMethod, secret: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Signature {
            secret: secret.into(),
            method,
        }
    }

    pub(crate) fn generate_from<'a, I>(&self, query_params: I) -> Hash
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let sanitized = to_sanitized_str(query_params);
        let hash = match &self.method {
            SignatureMethod::Md5Hash => {
                let concat = format!("{}{}", sanitized, self.secret);
                format!("{:x}", md5::Md5::new().chain(concat).finalize())
            }
            SignatureMethod::Md5Hmac => {
                let mut hmac = Hmac::<md5::Md5>::new_varkey(self.secret.as_bytes()).unwrap();
                hmac.update(sanitized.as_bytes());
                format!("{:x}", hmac.finalize().into_bytes())
            }
            SignatureMethod::Sha1Hmac => {
                let mut hmac = Hmac::<sha1::Sha1>::new_varkey(self.secret.as_bytes()).unwrap();
                hmac.update(sanitized.as_bytes());
                format!("{:x}", hmac.finalize().into_bytes())
            }
            SignatureMethod::Sha256Hmac => {
                let mut hmac = Hmac::<sha2::Sha256>::new_varkey(self.secret.as_bytes()).unwrap();
                hmac.update(sanitized.as_bytes());
                format!("{:x}", hmac.finalize().into_bytes())
            }
            SignatureMethod::Sha512Hmac => {
                let mut hmac = Hmac::<sha2::Sha512>::new_varkey(self.secret.as_bytes()).unwrap();
                hmac.update(sanitized.as_bytes());
                format!("{:x}", hmac.finalize().into_bytes())
            }
        };

        Hash(hash)
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct(stringify!(Signature))
            .field("secret", &"secret")
            .field("method", &self.method)
            .finish()
    }
}

fn to_sanitized_str<'a>(query_params: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let sorted: BTreeMap<_, _> = query_params.into_iter().collect();
    sorted
        .into_iter()
        .map(|(k, v)| format!("&{}={}", k, v.replace('&', "_").replace('=', "_")))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SignatureMethod {
    Md5Hash,
    Md5Hmac,
    Sha1Hmac,
    Sha256Hmac,
    Sha512Hmac,
}

impl Default for SignatureMethod {
    #[inline]
    fn default() -> Self {
        SignatureMethod::Md5Hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sig_method_is_md5_hash() {
        assert_eq!(SignatureMethod::default(), SignatureMethod::Md5Hash);
        assert_eq!(Signature::new("hello").method, SignatureMethod::Md5Hash);
    }
}
