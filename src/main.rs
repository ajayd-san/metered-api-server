use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use tokio::sync;

use std::io;

use metered_api_server::mpsc_bridge;
use metered_api_server::reset_quota;
use metered_api_server::services::{get_data, hola, register_client};

#[actix_web::main]
async fn main() -> io::Result<()> {
    let (sender, receiver) = sync::mpsc::channel(256);
    tokio::spawn(async move { mpsc_bridge::bridge(receiver).await });
    tokio::spawn(reset_quota(sender.clone()));
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(sender.clone()))
            .service(hola)
            .service(register_client)
            .service(get_data)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
