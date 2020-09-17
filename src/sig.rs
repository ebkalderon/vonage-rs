//! Contains types for SMS message signing.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Formatter, Write};

use hmac::{digest::Digest, Hmac, Mac, NewMac};
use serde::Serialize;

#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct SignatureHash(pub String);

#[cfg(test)]
impl<T: AsRef<str>> PartialEq<T> for SignatureHash {
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other.as_ref())
    }
}

/// A cryptographic signature used for signing SMS message requests.
#[derive(Clone)]
pub struct Signature {
    secret: Cow<'static, str>,
    method: SignatureMethod,
}

impl Signature {
    /// Creates a new `Signature` from the given API signature secret, as defined in the
    /// [Vonage API dashboard](https://dashboard.nexmo.com/).
    ///
    /// This constructor employs [`SignatureMethod::Md5Hash`] by default. If this does not match
    /// the method set in the Vonage dashboard, use [`Signature::with_method()`] to select the
    /// correct value instead.
    ///
    /// [`SignatureMethod::Md5Hash`]: ./enum.SignatureMethod.html#variant.Md5Hash
    /// [`Signature::with_method()`]: #method.with_method
    #[inline]
    pub fn new(secret: impl Into<Cow<'static, str>>) -> Self {
        Signature::with_method(SignatureMethod::default(), secret)
    }

    /// Creates a new `Signature` from the given API signature secret and [`SignatureMethod`].
    ///
    /// [`SignatureMethod`]: ./enum.SignatureMethod.html
    ///
    /// Both the secret and signature method _must_ match the value set in the
    /// [Vonage API dashboard](https://dashboard.nexmo.com/).
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

    pub(crate) fn sign<T: Serialize>(&self, query_params: T) -> SignatureHash {
        let payload = to_payload_str(query_params);
        let hash = match &self.method {
            SignatureMethod::Md5Hash => {
                let hasher = md5::Md5::new().chain(payload).chain(self.secret.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            SignatureMethod::Md5Hmac => {
                let mut hmac = Hmac::<md5::Md5>::new_varkey(self.secret.as_bytes()).unwrap();
                hmac.update(payload.as_bytes());
                format!("{:x}", hmac.finalize().into_bytes())
            }
            SignatureMethod::Sha1Hmac => {
                let mut hmac = Hmac::<sha1::Sha1>::new_varkey(self.secret.as_bytes()).unwrap();
                hmac.update(payload.as_bytes());
                format!("{:x}", hmac.finalize().into_bytes())
            }
            SignatureMethod::Sha256Hmac => {
                let mut hmac = Hmac::<sha2::Sha256>::new_varkey(self.secret.as_bytes()).unwrap();
                hmac.update(payload.as_bytes());
                format!("{:x}", hmac.finalize().into_bytes())
            }
            SignatureMethod::Sha512Hmac => {
                let mut hmac = Hmac::<sha2::Sha512>::new_varkey(self.secret.as_bytes()).unwrap();
                hmac.update(payload.as_bytes());
                format!("{:x}", hmac.finalize().into_bytes())
            }
        };

        SignatureHash(hash)
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

#[inline]
fn to_payload_str<T: Serialize>(query_params: T) -> String {
    let encoded = serde_urlencoded::to_string(query_params).expect("query_params must be map-like");
    let mut sorted: BTreeMap<&str, &str> = serde_urlencoded::from_str(&encoded).unwrap();
    sorted.remove("sig");
    let buf = String::with_capacity(encoded.len()); // Reasonable heuristic, given likely similar lengths.
    sorted.into_iter().fold(buf, |mut acc, (k, v)| {
        write!(acc, "&{}={}", k, v.replace(&['&', '='][..], "_")).unwrap();
        acc
    })
}

/// A list of supported SMS signature methods.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SignatureMethod {
    /// Concatenates the query string together with the signature secret and hashes the resulting
    /// string with MD5.
    ///
    /// This is the default signature method.
    Md5Hash,
    /// Signs the query string with an MD5 HMAC using the signature secret as a key.
    Md5Hmac,
    /// Signs the query string with a SHA-1 HMAC using the signature secret as a key.
    Sha1Hmac,
    /// Signs the query string with a SHA-256 HMAC using the signature secret as a key.
    Sha256Hmac,
    /// Signs the query string with a SHA-512 HMAC using the signature secret as a key.
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
        assert_eq!(Signature::new("secret").method, SignatureMethod::Md5Hash);
    }

    #[test]
    fn strips_sig_field_if_present() {
        #[derive(Serialize)]
        struct Params {
            sig: &'static str,
        }

        let hash = Signature::new("secret").sign(Params { sig: "hello" });
        assert_eq!(hash, "5ebe2294ecd0e0f08eab7690d2a6ee69");
    }

    #[test]
    fn generates_md5_signature() {
        #[derive(Serialize)]
        struct Params {
            from: &'static str,
        }

        let hash = Signature::new("secret").sign(());
        assert_eq!(hash, "5ebe2294ecd0e0f08eab7690d2a6ee69");

        let hash = Signature::new("secret").sign(Params { from: "VONAGE" });
        assert_eq!(hash, "129d3e7ca8b1acf36cb5ccb92dfec55c");
    }

    #[test]
    fn generates_sha1_signature() {
        let hash = Signature::with_method(SignatureMethod::Sha1Hmac, "secret").sign(());
        assert_eq!(hash, "25af6174a0fcecc4d346680a72b7ce644b9a88e8");
    }

    #[test]
    fn generates_sha256_signature() {
        let hash = Signature::with_method(SignatureMethod::Sha256Hmac, "secret").sign(());
        assert_eq!(
            hash,
            "f9e66e179b6747ae54108f82f8ade8b3c25d76fd30afde6c395822c530196169"
        );
    }

    #[test]
    fn generates_sha512_signature() {
        let hash = Signature::with_method(SignatureMethod::Sha512Hmac, "secret").sign(());
        assert_eq!(hash, "b0e9650c5faf9cd8ae02276671545424104589b3656731ec193b25d01b07561c27637c2d4d68389d6cf5007a8632c26ec89ba80a01c77a6cdd389ec28db43901");
    }
}
