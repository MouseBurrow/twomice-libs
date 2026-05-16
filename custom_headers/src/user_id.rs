use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo};
use sqlx::{Encode, Postgres, Type};
use std::future::{ready, Ready};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserId(pub Uuid);

impl FromRequest for UserId {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, actix_web::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let header = match req.headers().get("X-User-Id") {
            Some(v) => v,
            None => {
                return ready(Err(actix_web::error::ErrorUnauthorized(
                    "missing X-User-Id",
                )));
            }
        };

        let s = match header.to_str() {
            Ok(s) => s,
            Err(_) => return ready(Err(actix_web::error::ErrorBadRequest("invalid X-User-Id"))),
        };

        let uuid = match Uuid::parse_str(s) {
            Ok(u) => u,
            Err(_) => return ready(Err(actix_web::error::ErrorBadRequest("invalid X-User-Id"))),
        };

        ready(Ok(UserId(uuid)))
    }
}

impl From<UserId> for Uuid {
    fn from(value: UserId) -> Self {
        value.0
    }
}

impl Type<Postgres> for UserId {
    fn type_info() -> PgTypeInfo {
        <Uuid as Type<Postgres>>::type_info()
    }
}

impl<'q> Encode<'q, Postgres> for UserId {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        self.0.encode_by_ref(buf)
    }
}
