use axum::Router;
use crate::app_data::AppData;
use crate::config::Config;
use crate::logger;

pub async fn serve(service: &str, router: Router<AppData>) -> anyhow::Result<()> {
    logger::init();
    let config = Config::load(service);
    let app_data = AppData::new(config).await?;
    let addr = format!("0.0.0.0:{}", app_data.config.port);
    let app = router.with_state(app_data);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
