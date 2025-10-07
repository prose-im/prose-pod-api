// prose-pod-api
//
// Copyright: 2024, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use mime::Mime;
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE},
    Request, Response, StatusCode, Url,
};
pub use sea_orm::DbErr;
use serdev::{de::DeserializeOwned, Serialize, Serializer};

use crate::app_config::CONFIG_FILE_NAME;

#[derive(Debug, thiserror::Error)]
#[repr(transparent)]
#[error(
    "Missing key `{0}` in the app configuration. Add it to `{CONFIG_FILE_NAME}` or use environment variables."
)]
pub struct MissingConfiguration(pub &'static str);

#[derive(Debug, thiserror::Error)]
#[repr(transparent)]
#[error("Not implemented: {0}")]
pub struct NotImplemented(pub &'static str);

#[derive(Debug, thiserror::Error)]
#[repr(transparent)]
#[error("Unauthorized: {0}")]
pub struct Unauthorized(pub String);

#[derive(Debug, thiserror::Error)]
#[repr(transparent)]
#[error("Forbidden: {0}")]
pub struct Forbidden(pub String);

#[derive(Debug, thiserror::Error, Serialize)]
#[error("{message} (response: {response:#?})")]
pub struct UnexpectedHttpResponse {
    pub message: String,
    pub request: Option<RequestData>,
    pub response: ResponseData,
}

#[derive(Debug, Serialize)]
pub struct RequestData {
    pub url: Url,
    #[serde(serialize_with = "ser_headermap")]
    pub headers: HeaderMap,
    pub body: Option<String>,
}

impl RequestData {
    pub async fn from(request: Request) -> Self {
        Self {
            url: request.url().clone(),
            headers: request.headers().clone(),
            body: request
                .body()
                .and_then(|body| body.as_bytes())
                .map(|b| std::str::from_utf8(b).unwrap_or("<error>").to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ResponseData {
    #[serde(serialize_with = "ser_status_code")]
    pub status: StatusCode,
    #[serde(
        serialize_with = "ser_content_type",
        skip_serializing_if = "Option::is_none"
    )]
    content_type: Option<Mime>,
    #[serde(serialize_with = "ser_headermap")]
    pub headers: HeaderMap,
    #[serde(serialize_with = "ser_body")]
    pub body: Result<serde_json::Value, String>,
}

impl ResponseData {
    pub async fn from(response: Response) -> Self {
        let status = response.status();
        let content_type: Option<Mime> = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse().ok());
        let headers = response.headers().clone();
        let text: Option<String> = response.text().await.ok();
        let json: Option<serde_json::Value> = text
            .as_ref()
            .map(|s| serde_json::from_str(s).ok())
            .flatten();
        Self {
            status,
            content_type,
            headers,
            body: json.ok_or(text.unwrap_or("<none>".to_string())),
        }
    }
    pub fn text(&self) -> String {
        match self.body.as_ref() {
            Ok(json) => json.to_string(),
            Err(text) => text.clone(),
        }
    }
}
impl ResponseData {
    pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        match self.body {
            Ok(json) => serde_json::from_value(json),
            Err(text) => serde_json::from_str(&text),
        }
    }
}

fn ser_status_code<S: Serializer>(sc: &StatusCode, serializer: S) -> Result<S::Ok, S::Error> {
    sc.to_string().serialize(serializer)
}
fn ser_content_type<S: Serializer>(ct: &Option<Mime>, serializer: S) -> Result<S::Ok, S::Error> {
    ct.as_ref().map(ToString::to_string).serialize(serializer)
}
fn headermap_to_vec(hm: &HeaderMap) -> Vec<String> {
    hm.iter()
        .map(|(k, v)| format!("{k}: {}", v.to_str().unwrap_or("<error>")))
        .collect()
}
fn ser_headermap<S: Serializer>(hm: &HeaderMap, serializer: S) -> Result<S::Ok, S::Error> {
    headermap_to_vec(hm).serialize(serializer)
}
fn ser_body<S: Serializer>(
    body: &Result<serde_json::Value, String>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match body {
        Ok(json) => json.serialize(serializer),
        Err(text) => text.serialize(serializer),
    }
}

impl UnexpectedHttpResponse {
    pub fn new(
        request: Option<RequestData>,
        response: ResponseData,
        error_description: impl FnOnce(
            Option<Mime>,
            Option<serde_json::Value>,
            Option<String>,
        ) -> String,
    ) -> Self {
        let response_text: Option<String> = match response.body.as_ref() {
            Ok(json) => serde_json::to_string(&json).ok(),
            Err(str) => Some(str.clone()),
        };
        let response_json: Option<serde_json::Value> = response.body.as_ref().ok().cloned();

        Self {
            message: error_description(
                response.content_type.clone(),
                response_json.clone(),
                response_text.clone(),
            ),
            request,
            response,
        }
    }
}

// TODO: Move somewhere else.
#[derive(Debug, thiserror::Error)]
#[error("Group ‘{group_id}’ not found.")]
pub struct GroupNotFound {
    pub group_id: String,
}

// TODO: Move somewhere else.
#[derive(Debug, thiserror::Error)]
#[error("Group ‘{group_id}’ already exists.")]
pub struct GroupAlreadyExists {
    pub group_id: String,
}
