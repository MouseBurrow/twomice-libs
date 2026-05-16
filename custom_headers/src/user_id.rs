use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo};
use sqlx::{Encode, Postgres, Type};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserId(pub Uuid);

#[axum::async_trait]
impl<S: Send + Sync> FromRequestParts<S> for UserId {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("X-User-Id")
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "missing X-User-Id".into()))?;

        let s = header
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid X-User-Id".into()))?;

        let uuid = Uuid::parse_str(s)
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid X-User-Id".into()))?;

        Ok(UserId(uuid))
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
