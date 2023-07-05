mod database;
mod mpsc_bridge;

use std::{io, thread};

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use tokio::sync;
use uuid::Uuid;

#[derive(Serialize)]
struct ResponseData {
    id: u32,
    msg: String,
}

#[derive(Debug)]
pub enum InstructionKind {
    Register,
    Update,
    Query,
}

#[derive(Debug)]
pub struct DbInstruction {
    kind: InstructionKind,
    key_data: KeyRegistarationData,
}

#[derive(Serialize, Clone, Debug)]
struct KeyRegistarationData {
    api_key: String,
    quota_per_min: u32,
}

impl DbInstruction {
    fn new(kind: InstructionKind, key_data: KeyRegistarationData) -> Self {
        DbInstruction { kind, key_data }
    }
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
async fn register_client(mpsc_sender: web::Data<sync::mpsc::Sender<DbInstruction>>) -> impl Responder {
    let res = KeyRegistarationData::new();
    let db_instruction = DbInstruction::new(InstructionKind::Register, res.clone());
    mpsc_sender.send(db_instruction).await.unwrap();
    web::Json(res)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let (sender, receiver) = sync::mpsc::channel(256);
    tokio::spawn(async move { mpsc_bridge::bridge(receiver).await });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(ResponseData {
                id: 2,
                msg: "idk".to_string(),
            }))
            .app_data(web::Data::new(sender.clone()))
            .service(hola)
            .service(register_client)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
