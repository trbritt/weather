use std::str::from_utf8;

use axum::{async_trait, extract::FromRequestParts};
use axum::http::request::Parts;
use base64::{Engine as _, engine::general_purpose};
use reqwest::StatusCode;

pub struct User;

#[async_trait]
impl<S> FromRequestParts<S> for User 
where
 S: Send + Sync
{
    type Rejection = axum::http::Response<axum::body::Body>;
    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection>{
        let auth_header = parts 
            .headers
            .get("Authorization")
            .and_then(|header| header.to_str().ok());
        if let Some(auth_header) = auth_header {
            if auth_header.starts_with("Basic ") {
                let credentials = auth_header.trim_start_matches("Basic ");
                let decoded = general_purpose::STANDARD.decode(credentials).unwrap_or_default();
                let credential_str = from_utf8(&decoded).unwrap_or("");
                //our username and password here
                if credential_str == "forecast:forecast" {
                    return Ok(User);
                }
            }
        }
        let reject_response = axum::http::Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(
                "WWW-Authenticate",
                "Basic realm=\"Please enter your credentials\"",
            )
            .body(axum::body::Body::from("Unauthorized"))
            .unwrap();
        Err(reject_response)
        
    }
}