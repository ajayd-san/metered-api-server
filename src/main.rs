use std::io;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
struct ResponseData {
    id: u32,
    msg: String,
}

#[derive(Serialize)]
struct KeyRegistarationData {
    api_key: String,
    quota_per_min: u32,
}

impl KeyRegistarationData {
    fn new() -> Self {
        let api_key = Uuid::new_v4().simple().to_string();
        KeyRegistarationData {
            api_key,
            quota_per_min: 10,
        }
    }
}

#[get("/")]
async fn hola(res_data: web::Data<ResponseData>) -> impl Responder {
    web::Json(res_data)
}

#[get("/register")]
async fn register_client() -> impl Responder {
    let res = KeyRegistarationData::new();
    web::Json(res)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(ResponseData {
                id: 2,
                msg: "idk".to_string(),
            }))
            .service(hola)
            .service(register_client)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
