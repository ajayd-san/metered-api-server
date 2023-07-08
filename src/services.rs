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

use crate::{
    Db, DbInstruction, InstructionKind, KeyRegistarationData, ResponseData, BAD_REQUEST_MSG,
    TOO_MANY_REQUESTS_MSG, send_to_mpsc,
};

use thiserror::Error;
use tokio::sync::{self, mpsc, oneshot};

use crate::database::DbResult;

#[derive(Debug, Error)]
enum CustomError {
    #[error("Something wrong with sqlx")]
    SqlxErr(sqlx::Error),
    #[error("Quoata exhausted. Try again later.")]
    QuotaExhausted,
    #[error("Unexpected error occured.")]
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
    let key_reg = KeyRegistarationData::new(Db::API_KEY);
    let res = send_to_mpsc(InstructionKind::Register, key_reg.clone(), mpsc_sender.as_ref().clone()).await;

    match res {
        Ok(_) => HttpResponse::Ok().json(key_reg),
        Err(e) => panic!(),
    }
}

#[get("/get")]
async fn get_data(
    mpsc_sender: web::Data<mpsc::Sender<(DbInstruction, oneshot::Sender<sqlx::Result<DbResult>>)>>,
    request: HttpRequest,
) -> actix_web::Result<impl Responder> {
    let headers = request.headers();
    if let Some(api_key) = headers.get(header::AUTHORIZATION) {
        let api_key = api_key.to_str().unwrap();

        let key_data = KeyRegistarationData::get_with_exisiting(api_key, Db::API_KEY);
        let db_response = send_to_mpsc(
            InstructionKind::Query,
            key_data.clone(),
            mpsc_sender.get_ref().clone(),
        )
        .await;
        let db_response = db_response.map_err(|e| CustomError::SqlxErr(e))?;

        if let DbResult::QueryRes(queries_left) = db_response {
            if queries_left > 0 {
                let db_response = send_to_mpsc(
                    InstructionKind::Update,
                    key_data,
                    mpsc_sender.get_ref().clone(),
                )
                .await;
                db_response.map_err(CustomError::SqlxErr)?;
                return Ok(web::Json(ResponseData {
                    id: 10,
                    msg: String::from("data"),
                }));
            } else {
                return Err(CustomError::QuotaExhausted.into());
            }
        }
    }

    //INFO: REQUESTS WITHOUT API KEY HANDLED HERE

    let conn_info = request.connection_info();
    let client_ip = conn_info.realip_remote_addr().unwrap();

    let key_data = KeyRegistarationData::get_with_exisiting(client_ip, Db::IP_BOOK);
    let db_response = send_to_mpsc(
        InstructionKind::Query,
        key_data.clone(),
        mpsc_sender.get_ref().clone(),
    )
    .await;
    if db_response.is_err() {
        // add ip to db and give result
        let db_response = send_to_mpsc(
            InstructionKind::Register,
            key_data.clone(),
            mpsc_sender.get_ref().clone(),
        )
        .await;
        db_response.map_err(CustomError::SqlxErr)?;
        return Ok(web::Json(ResponseData {
            id: 10,
            msg: String::from("data from ip"),
        }));
    }

    if let DbResult::QueryRes(queries_left) = db_response.unwrap() {
        if queries_left > 0 {
        let db_response = send_to_mpsc(
            InstructionKind::Update,
            key_data,
            mpsc_sender.get_ref().clone(),
        )
        .await;
        db_response.map_err(CustomError::SqlxErr)?;
            return Ok(web::Json(ResponseData {
                id: 10,
                msg: String::from("data from ip"),
            }));
        } else {
            return Err(CustomError::QuotaExhausted)?;
        }
    }

    Ok(web::Json(ResponseData {
        id: 10000,
        msg: String::from("default"),
    }))
}
