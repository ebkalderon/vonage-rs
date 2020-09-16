//! Types specific to the `/verify` endpoint.

use serde::Serialize;

use super::Verification;

/// Request fields specific to the `/verify` endpoint.
#[derive(Debug, Default, Serialize)]
pub struct Normal {
    pub brand: String,
    pub sender_id: Option<String>,
    #[serde(rename = "lg")]
    pub language: Option<Language>,
}

impl Verification for Normal {
    const PATH: &'static str = "/verify";
}

/// A list of supported languages for verify SMS or TTS messages.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum Language {
    #[serde(rename = "ar-xa")]
    Arabic,
    #[serde(rename = "cs-cz")]
    Czech,
    #[serde(rename = "cy-cy")]
    Welsh,
    #[serde(rename = "cy-gb")]
    WelshUk,
    #[serde(rename = "da-dk")]
    Danish,
    #[serde(rename = "de-de")]
    German,
    #[serde(rename = "el-gr")]
    Greek,
    #[serde(rename = "en-au")]
    EnglishAu,
    #[serde(rename = "en-gb")]
    EnglishUk,
    #[serde(rename = "en-in")]
    EnglishIndia,
    #[serde(rename = "en-us")]
    EnglishUs,
    #[serde(rename = "es-es")]
    Spanish,
    #[serde(rename = "es-mx")]
    SpanishMexico,
    #[serde(rename = "es-us")]
    SpanishUs,
    #[serde(rename = "fi-fi")]
    Finnish,
    #[serde(rename = "fil-ph")]
    Filipino,
    #[serde(rename = "fr-ca")]
    FrenchCanada,
    #[serde(rename = "fr-fr")]
    French,
    #[serde(rename = "hi-in")]
    Hindi,
    #[serde(rename = "hu-hu")]
    Hungarian,
    #[serde(rename = "id-id")]
    Indonesian,
    #[serde(rename = "is-is")]
    Icelandic,
    #[serde(rename = "it-it")]
    Italian,
    #[serde(rename = "ja-jp")]
    Japanese,
    #[serde(rename = "ko-kr")]
    Korean,
    #[serde(rename = "nb-no")]
    Norwegian,
    #[serde(rename = "nl-nl")]
    Dutch,
    #[serde(rename = "pl-pl")]
    Polish,
    #[serde(rename = "pt-br")]
    PortugueseBrazil,
    #[serde(rename = "pt-pt")]
    Portuguese,
    #[serde(rename = "ro-ro")]
    Romanian,
    #[serde(rename = "sv-se")]
    Swedish,
    #[serde(rename = "th-th")]
    Thai,
    #[serde(rename = "vi-vn")]
    Vietnamese,
    #[serde(rename = "yue-cn")]
    Cantonese,
    #[serde(rename = "zh-cn")]
    ChineseMainland,
    #[serde(rename = "zh-tw")]
    ChineseTaiwan,
}
