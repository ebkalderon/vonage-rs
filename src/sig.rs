//! Contains types for SMS message signing.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Formatter};

use hmac::{digest::Digest, Hmac, Mac, NewMac};

pub(crate) struct Hash(pub String);

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
        assert_eq!(Signature::new("hello").method, SignatureMethod::Md5Hash);
    }
}
