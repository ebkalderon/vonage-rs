use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Request {
    pub payee: String,
    pub amount: f64,
    #[serde(rename = "lg")]
    pub language: Option<Language>,
}

impl super::Kind for Request {
    const PATH: &'static str = "/psd2";
}

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
