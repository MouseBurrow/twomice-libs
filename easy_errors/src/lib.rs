pub use actix_web;
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
            pub fn http_status(&self) -> $crate::actix_web::http::StatusCode {
                match self {
                    $(
                        Self::$variant => $crate::actix_web::http::StatusCode::$status,
                    )*
                    Self::Unexpected(_) => $crate::actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
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

        impl $crate::actix_web::ResponseError for $name {
            fn status_code(&self) -> $crate::actix_web::http::StatusCode {
                self.http_status()
            }

            fn error_response(&self) -> $crate::actix_web::HttpResponse {
                if <_ as easy_errors::DbErrorTrait>::is_unexpected(self) {
                    return $crate::actix_web::HttpResponse::InternalServerError().finish();
                }

                $crate::actix_web::HttpResponse::build(self.http_status()).json(
                    $crate::serde_json::json!({
                        "error": self.code(),
                        "message": self.to_string()
                    })
                )
            }
        }
    };
}
