use actix_web::cookie::{time, Cookie, SameSite};
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo};
use sqlx::{Encode, Postgres, Type};
use std::future::{ready, Ready};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken(pub String);

impl FromRequest for SessionToken {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, actix_web::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let header = match req.headers().get("X-Session-Token") {
            Some(v) => v,
            None => {
                return ready(Err(actix_web::error::ErrorBadRequest(
                    "missing X-Session-Token",
                )));
            }
        };

        let token = match header.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return ready(Err(actix_web::error::ErrorBadRequest(
                    "invalid X-Session-Token",
                )));
            }
        };

        ready(Ok(SessionToken(token)))
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
    pub fn create_cookie<'a>(secure: bool, token: String) -> Cookie<'a> {
        Cookie::build("session_token", token)
            .http_only(true)
            .secure(secure)
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(time::Duration::days(30))
            .finish()
    }
}
