use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo};
use sqlx::{Encode, Postgres, Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalUserId(pub Option<i64>);

#[axum::async_trait]
impl<S: Send + Sync> FromRequestParts<S> for OptionalUserId {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Some(header) = parts.headers.get("X-User-Id") else {
            return Ok(OptionalUserId(None));
        };

        let s = header
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid X-User-Id".into()))?;

        let id: i64 = s
            .parse()
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid X-User-Id".into()))?;

        Ok(OptionalUserId(Some(id)))
    }
}

impl From<OptionalUserId> for Option<i64> {
    fn from(value: OptionalUserId) -> Self {
        value.0
    }
}

impl Type<Postgres> for OptionalUserId {
    fn type_info() -> PgTypeInfo {
        <i64 as Type<Postgres>>::type_info()
    }
}

impl<'q> Encode<'q, Postgres> for OptionalUserId {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        self.0.encode_by_ref(buf)
    }
}
