use crate::app_envs::AppEnvs;
use env_logger::Env;

#[derive(Clone)]
pub struct Logger {}

impl Logger {
    pub fn load() -> Self {
        let app_env = AppEnvs::get();

        let filter = match app_env {
            AppEnvs::PROD => "warn,axum=info,tower_http=info",
            AppEnvs::STAGING => "info,axum=info,tower_http=debug",
            AppEnvs::DEV => "debug",
        };
        env_logger::init_from_env(Env::default().default_filter_or(filter));

        Self {}
    }
}
