use std::env;

#[derive(Clone, PartialOrd, PartialEq)]
pub enum AppEnvs {
    DEV,
    STAGING,
    PROD,
}

impl AppEnvs {
    pub fn get() -> AppEnvs {
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "dev".into());
        match app_env.as_str() {
            "prod" => AppEnvs::PROD,
            "staging" => AppEnvs::STAGING,
            _ => AppEnvs::DEV, // dev
        }
    }
}
