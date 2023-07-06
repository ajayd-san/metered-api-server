use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder, http::header};

use metered_api_server::{DbInstruction, ResponseData, InstructionKind, KeyRegistarationData};
use tokio::sync::{self, oneshot};

#[get("/")]
async fn hola(res_data: web::Data<ResponseData>) -> impl Responder {
    web::Json(res_data)
}

#[get("/register")]
async fn register_client(
    mpsc_sender: web::Data<sync::mpsc::Sender<(DbInstruction, oneshot::Sender<sqlx::Result<()>>)>>,
) -> impl Responder {
    let res = KeyRegistarationData::new();
    let db_instruction = DbInstruction::new(InstructionKind::Register, res.clone());
    let (oneshot_sender, oneshot_receiver) = oneshot::channel::<sqlx::Result<()>>();
    mpsc_sender.send((db_instruction, oneshot_sender)).await.unwrap();
    let res = oneshot_receiver.await;

    web::Json()
}

#[get("/get")]
async fn get_data(mpsc_sender: web::Data<sync::mpsc::Sender<DbInstruction>>, request: HttpRequest) -> impl Responder {
    let headers = request.headers();
    if let Some(api_key) = headers.get(header::AUTHORIZATION) {
        api_key.set_sensitive(true);
        let api_key = api_key.to_str().unwrap();

    }
    web::Json()
}
