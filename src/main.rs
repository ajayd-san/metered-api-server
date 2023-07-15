use actix_cors::Cors;
use actix_web::http;
use actix_web::{web, App, HttpServer};
use tokio::sync;

use std::io;

use metered_api_server::mpsc_bridge;
use metered_api_server::reset_quota;
use metered_api_server::services::{get_data, register_client};

#[actix_web::main]
async fn main() -> io::Result<()> {
    let (sender, receiver) = sync::mpsc::channel(256);
    tokio::spawn(async move { mpsc_bridge::bridge(receiver).await });
    tokio::spawn(reset_quota(sender.clone()));
    HttpServer::new(move || {
        let cors = Cors::default().allowed_origin("http://localhost:3000");
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(sender.clone()))
            .service(register_client)
            .service(get_data)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
