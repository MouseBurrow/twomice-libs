use crate::app_envs::AppEnvs;
use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub app_env: AppEnvs,
}

impl Config {
    pub fn load(service: &str) -> Self {
        let db_var = format!("{}_DATABASE_URL", service.to_uppercase());

        let database_url =
            env::var(&db_var).unwrap_or_else(|_| panic!("Missing env variable: {db_var}"));

        let port = env::var("PORT")
            .unwrap_or_else(|_| panic!("Missing env variable: PORT"))
            .parse::<u16>()
            .unwrap_or_else(|_| panic!("PORT must be a u16"));

        let app_env = AppEnvs::get();

        Self {
            database_url,
            port,
            app_env,
        }
    }
}
