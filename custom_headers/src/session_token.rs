use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo};
use sqlx::{Encode, Postgres, Type};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken(pub String);

#[axum::async_trait]
impl<S: Send + Sync> FromRequestParts<S> for SessionToken {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("X-Session-Token")
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing X-Session-Token".into()))?;

        let token = header
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid X-Session-Token".into()))?
            .to_string();

        Ok(SessionToken(token))
    }
}

impl fmt::Display for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<SessionToken> for String {
    fn from(value: SessionToken) -> Self {
        value.0
    }
}

impl Type<Postgres> for SessionToken {
    fn type_info() -> PgTypeInfo {
        <String as Type<Postgres>>::type_info()
    }
}

impl<'q> Encode<'q, Postgres> for SessionToken {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as Encode<Postgres>>::encode_by_ref(&self.0, buf)
    }
}

impl SessionToken {
    pub fn cookie_value(secure: bool, token: String) -> String {
        let mut header = format!(
            "session_token={}; HttpOnly; Path=/; Max-Age={}",
            token,
            30 * 24 * 3600
        );
        if secure {
            header.push_str("; Secure");
        }
        header.push_str("; SameSite=Lax");
        header
    }

    pub fn clear_cookie_value(secure: bool) -> String {
        let mut header = "session_token=; HttpOnly; Path=/; Max-Age=0".to_string();
        if secure {
            header.push_str("; Secure");
        }
        header.push_str("; SameSite=Lax");
        header
    }

    pub fn set_cookie_response<T: serde::Serialize>(
        cookie_value: String,
        body: T,
    ) -> (axum::http::HeaderMap, axum::Json<T>) {
        use axum::http::header::{HeaderMap, HeaderValue, SET_COOKIE};
        let mut headers = HeaderMap::new();
        headers.insert(SET_COOKIE, HeaderValue::from_str(&cookie_value).unwrap());
        (headers, axum::Json(body))
    }
}

pub fn parse_session_token(cookie_header: &http::HeaderValue) -> Option<String> {
    let cookie_str = cookie_header.to_str().ok()?;
    for pair in cookie_str.split("; ") {
        if let Some(value) = pair.strip_prefix("session_token=") {
            return Some(value.to_string());
        }
    }
    None
}

pub fn extract_session_token_from_headers(headers: &http::HeaderMap) -> Option<String> {
    headers
        .get(http::header::COOKIE)
        .and_then(parse_session_token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_value_includes_token() {
        let val = SessionToken::cookie_value(false, "mytoken".into());
        assert!(val.contains("session_token=mytoken"));
    }

    #[test]
    fn cookie_value_has_flags() {
        let val = SessionToken::cookie_value(false, "t".into());
        assert!(val.contains("HttpOnly"));
        assert!(val.contains("Path=/"));
        assert!(val.contains("Max-Age="));
    }

    #[test]
    fn cookie_value_secure_when_enabled() {
        assert!(SessionToken::cookie_value(true, "t".into()).contains("Secure"));
        assert!(!SessionToken::cookie_value(false, "t".into()).contains("Secure"));
    }

    #[test]
    fn clear_cookie_value_is_zero_max_age() {
        let val = SessionToken::clear_cookie_value(false);
        assert!(val.contains("Max-Age=0"));
    }

    #[test]
    fn clear_cookie_value_secure() {
        assert!(SessionToken::clear_cookie_value(true).contains("Secure"));
        assert!(!SessionToken::clear_cookie_value(false).contains("Secure"));
    }

    #[test]
    fn parse_session_token_found() {
        let val = http::HeaderValue::from_static("session_token=abc123; Other=val");
        assert_eq!(parse_session_token(&val), Some("abc123".into()));
    }

    #[test]
    fn parse_session_token_not_found() {
        let val = http::HeaderValue::from_static("other=value");
        assert_eq!(parse_session_token(&val), None);
    }

    #[test]
    fn parse_session_token_empty_cookie() {
        let val = http::HeaderValue::from_static("");
        assert_eq!(parse_session_token(&val), None);
    }

    #[test]
    fn extract_session_token_from_headers_with_cookie() {
        let mut headers = http::HeaderMap::new();
        headers.insert(
            http::header::COOKIE,
            http::HeaderValue::from_static("session_token=xyz789"),
        );
        assert_eq!(
            extract_session_token_from_headers(&headers),
            Some("xyz789".into())
        );
    }

    #[test]
    fn extract_session_token_from_headers_missing() {
        let headers = http::HeaderMap::new();
        assert_eq!(extract_session_token_from_headers(&headers), None);
    }
}
