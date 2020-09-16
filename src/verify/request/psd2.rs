//! Types specific to the `/verify/psd2` endpoint.

use serde::Serialize;

use super::Verification;

/// Request fields specific to the `/verify/psd2` endpoint.
#[derive(Debug, Default, Serialize)]
pub struct Psd2 {
    pub payee: String,
    pub amount: f64,
    #[serde(rename = "lg")]
    pub language: Option<Language>,
}

impl Verification for Psd2 {
    const PATH: &'static str = "/verify/psd2";
}

/// A list of supported languages for PSD2 SMS or TTS messages.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum Language {
    #[serde(rename = "bg-bg")]
    Bulgarian,
    #[serde(rename = "cs-cz")]
    Czech,
    #[serde(rename = "da-dk")]
    Danish,
    #[serde(rename = "de-de")]
    German,
    #[serde(rename = "en-gb")]
    EnglishUk,
    #[serde(rename = "ee-et")]
    Estonian,
    #[serde(rename = "el-gr")]
    Greek,
    #[serde(rename = "es-es")]
    Spanish,
    #[serde(rename = "fi-fi")]
    Finnish,
    #[serde(rename = "fr-fr")]
    French,
    #[serde(rename = "ga-ie")]
    Gaelic,
    #[serde(rename = "hu-hu")]
    Hungarian,
    #[serde(rename = "it-it")]
    Italian,
    #[serde(rename = "lv-lv")]
    Latvian,
    #[serde(rename = "lt-lt")]
    Lithuanian,
    #[serde(rename = "mt-mt")]
    Maltese,
    #[serde(rename = "nl-nl")]
    Dutch,
    #[serde(rename = "pl-pl")]
    Polish,
    #[serde(rename = "sk-sk")]
    Slovak,
    #[serde(rename = "sl-si")]
    Slovenian,
    #[serde(rename = "sv-se")]
    Swedish,
}
