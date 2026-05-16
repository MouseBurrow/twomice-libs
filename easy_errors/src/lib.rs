pub use axum;
pub use http;
pub use log;
pub use serde_json;
pub use sqlx;
pub use strum;
pub use strum_macros;

pub trait DbErrorTrait: Sized {
    fn from_code(code: &str) -> Self;
    fn unexpected(err: sqlx::Error) -> Self;
    fn is_unexpected(&self) -> bool;
}

pub fn map_sqlx_error<E: DbErrorTrait>(err: sqlx::Error) -> E {
    if let sqlx::Error::Database(db_err) = &err {
        if let Some(code) = db_err.code() {
            let mapped = E::from_code(&code);
            if E::is_unexpected(&mapped) {
                log::error!("UNEXPECTED SQLx ERROR (code {code}): {err:?}");
            } else {
                return mapped;
            }
        }
    }
    log::error!("UNEXPECTED SQLx ERROR: {err:?}");
    E::unexpected(err)
}

#[macro_export]
macro_rules! define_errors {
    (
        $name:ident {
            $(
                $variant:ident => {
                    code: $code:literal,
                    status: $status:ident,
                    message: $message:literal
                }
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, $crate::strum_macros::AsRefStr)]
        #[strum(serialize_all = "snake_case")]
        pub enum $name {
            $(
                $variant,
            )*
            Unexpected(String),
        }

        impl $crate::DbErrorTrait for $name {
            fn from_code(code: &str) -> Self {
                match code {
                    $(
                        $code => Self::$variant,
                    )*
                    other => Self::Unexpected(other.to_string()),
                }
            }

            fn unexpected(err: sqlx::Error) -> Self {
                Self::Unexpected(err.to_string())
            }

            fn is_unexpected(&self) -> bool {
                matches!(self, Self::Unexpected(_))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant => f.write_str($message),
                    )*
                    Self::Unexpected(_) => f.write_str("Internal server error"),
                }
            }
        }

        impl $name {
            pub fn http_status(&self) -> $crate::http::StatusCode {
                match self {
                    $(
                        Self::$variant => $crate::http::StatusCode::$status,
                    )*
                    Self::Unexpected(_) => $crate::http::StatusCode::INTERNAL_SERVER_ERROR,
                }
            }

            pub fn code(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => stringify!($variant),
                    )*
                    Self::Unexpected(_) => "internal_error",
                }
            }
        }

        impl $crate::axum::response::IntoResponse for $name {
            fn into_response(self) -> $crate::axum::response::Response {
                use $crate::axum::response::IntoResponse as _;

                if <_ as $crate::DbErrorTrait>::is_unexpected(&self) {
                    return $crate::http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }

                let body = $crate::serde_json::json!({
                    "error": self.code(),
                    "message": self.to_string()
                });

                (self.http_status(), $crate::axum::Json(body)).into_response()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::StatusCode;

    define_errors!( TestError {
        NotFound => { code: "P0000", status: NOT_FOUND, message: "Not found" },
        Conflict => { code: "23505", status: CONFLICT, message: "Already exists" },
    });

    #[test]
    fn error_display_message() {
        assert_eq!(TestError::NotFound.to_string(), "Not found");
        assert_eq!(TestError::Conflict.to_string(), "Already exists");
        assert_eq!(
            TestError::Unexpected("x".into()).to_string(),
            "Internal server error"
        );
    }

    #[test]
    fn error_http_status() {
        assert_eq!(TestError::NotFound.http_status(), StatusCode::NOT_FOUND);
        assert_eq!(TestError::Conflict.http_status(), StatusCode::CONFLICT);
        assert_eq!(
            TestError::Unexpected("x".into()).http_status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn from_code_maps_known_codes() {
        assert_eq!(TestError::from_code("P0000"), TestError::NotFound);
        assert_eq!(TestError::from_code("23505"), TestError::Conflict);
    }

    #[test]
    fn from_code_returns_unexpected_for_unknown() {
        let err = TestError::from_code("99999");
        assert!(err.is_unexpected());
    }

    #[test]
    fn is_unexpected_distinguishes() {
        assert!(!TestError::NotFound.is_unexpected());
        assert!(TestError::Unexpected("db crash".into()).is_unexpected());
    }

    #[test]
    fn http_status_on_unexpected_is_500() {
        let err = TestError::Unexpected("oops".into());
        assert_eq!(err.http_status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
