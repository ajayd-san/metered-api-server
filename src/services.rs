use actix_web::{
    error::{self, ErrorInternalServerError},
    get,
    http::{
        header::{self, ContentType},
        StatusCode,
    },
    web::{self, Json},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};

use metered_api_server::{
    DbInstruction, InstructionKind, KeyRegistarationData, ResponseData, BAD_REQUEST_MSG,
    TOO_MANY_REQUESTS_MSG,
};

use thiserror::Error;
use tokio::sync::{self, oneshot};

use crate::database::DbResult;

#[derive(Debug, Error)]
enum CustomError {
    #[error("sqlx error hogaya bhai")]
    SqlxErr(sqlx::Error),
    #[error("Quoata exhausted. Try again later.")]
    QuotaExhausted,
    #[error("error i didn't care about bhai")]
    OtherErr,
}

impl error::ResponseError for CustomError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            CustomError::SqlxErr(e) => match e {
                sqlx::Error::RowNotFound => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            CustomError::QuotaExhausted => StatusCode::TOO_MANY_REQUESTS,
            CustomError::OtherErr => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let resp_body = match self.status_code() {
            StatusCode::BAD_REQUEST => BAD_REQUEST_MSG,
            StatusCode::TOO_MANY_REQUESTS => TOO_MANY_REQUESTS_MSG,
            _ => panic!(),
        };
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(resp_body)
    }
}

#[get("/")]
async fn hola(res_data: web::Data<ResponseData>) -> impl Responder {
    web::Json(res_data)
}

#[get("/register")]
async fn register_client(
    mpsc_sender: web::Data<
        sync::mpsc::Sender<(DbInstruction, oneshot::Sender<sqlx::Result<DbResult>>)>,
    >,
) -> impl Responder {
    let key_reg = KeyRegistarationData::new();
    let db_instruction = DbInstruction::new(InstructionKind::Register, key_reg.clone());
    let (oneshot_sender, oneshot_receiver) = oneshot::channel();
    mpsc_sender
        .send((db_instruction, oneshot_sender))
        .await
        .unwrap();
    let res = oneshot_receiver.await;

    match res {
        Ok(_) => HttpResponse::Ok().json(key_reg),
        Err(e) => panic!(),
    }
}

#[get("/get")]
async fn get_data(
    mpsc_sender: web::Data<
        sync::mpsc::Sender<(DbInstruction, oneshot::Sender<sqlx::Result<DbResult>>)>,
    >,
    request: HttpRequest,
) -> actix_web::Result<impl Responder> {
    let headers = request.headers();
    if let Some(api_key) = headers.get(header::AUTHORIZATION) {
        let api_key = api_key.to_str().unwrap();
        let key_data = KeyRegistarationData::get_with_exisiting(api_key);
        let db_instruction = DbInstruction::new(InstructionKind::Query, key_data);

        let (oneshot_sender, oneshot_receiver) = oneshot::channel();
        mpsc_sender
            .send((db_instruction, oneshot_sender))
            .await
            .unwrap();
        let db_response = oneshot_receiver.await.unwrap();
        let db_response = db_response.map_err(|e| CustomError::SqlxErr(e))?;

        if let DbResult::QueryRes(queries_left) = db_response {
            if queries_left > 0 {
                let (oneshot_sender, oneshot_receiver) = oneshot::channel();
                let key_data = KeyRegistarationData::get_with_exisiting(api_key);
                let db_instruction = DbInstruction::new(InstructionKind::Update, key_data);
                mpsc_sender
                    .send((db_instruction, oneshot_sender))
                    .await
                    .map_err(ErrorInternalServerError)?;
                oneshot_receiver.await.map_err(ErrorInternalServerError)?;

                return Ok(web::Json(ResponseData {
                    id: 10,
                    msg: String::from("data"),
                }));
            } else {
                return Err(CustomError::QuotaExhausted)?;
            }
        }
    }
    Ok(web::Json(ResponseData {
        id: 10000,
        msg: String::from("default"),
    }))
}
