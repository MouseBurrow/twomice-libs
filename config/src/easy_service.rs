#[macro_export]
macro_rules! launch_service {
    (
        service: $service:expr,
        routes: [ $( $route:expr ),* $(,)? ]
    ) => {{
        use actix_web::{App, HttpServer, web};
        use $crate::{config::Config, logger::Logger, app_data::AppData};

        let config = Config::load($service);
        let logger = Logger::load();
        let app_data = AppData::new(config.clone(), logger).await?;
        let global_addr = format!("0.0.0.0:{}", config.port);

        let local_addr = format!("http://{}-service:{}/", $service, config.port);
        HttpServer::new(move || {
            let mut app = App::new().app_data(web::Data::new(app_data.clone()));
            $( app = app.service($route); )*
            app
        })
        .bind(&global_addr)?
        .run()
        .await?;
    }};
}
