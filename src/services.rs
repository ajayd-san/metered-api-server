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
use serde_json::json;

use crate::{
    get_random_quote, send_to_mpsc, Db, DbInstruction, InstructionKind, KeyRegistarationData,
    ResponseData, BAD_REQUEST_MSG, TOO_MANY_REQUESTS_MSG,
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
    let res = send_to_mpsc(
        InstructionKind::Register,
        key_reg.clone(),
        mpsc_sender.as_ref().clone(),
    )
    .await;

    match res {
        Ok(_) => HttpResponse::Ok().json(json!({
            "api_key": key_reg.key,
            "quota": key_reg.quota
        })),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

//TODO: lots of repeated code, refactor
#[get("/get")]
async fn get_data(
    mpsc_sender: web::Data<mpsc::Sender<(DbInstruction, oneshot::Sender<sqlx::Result<DbResult>>)>>,
    request: HttpRequest,
) -> actix_web::Result<impl Responder> {
    let headers = request.headers();
    let (key_data, db_response) = {
        
        // if headers have api-key
        if let Some(api_key) = headers.get(header::AUTHORIZATION) {
            let api_key = api_key.to_str().unwrap();

            let key_data = KeyRegistarationData::get_with_exisiting(api_key, Db::API_KEY);
            let db_response = send_to_mpsc(
                InstructionKind::Query,
                key_data.clone(),
                mpsc_sender.get_ref().clone(),
            )
            .await;

            (key_data, db_response)
        } else {
            // ip tracking done here
            let conn_info = request.connection_info();
            let client_ip = conn_info.realip_remote_addr().unwrap();

            let key_data = KeyRegistarationData::get_with_exisiting(client_ip, Db::IP_BOOK);
            let db_response = send_to_mpsc(
                InstructionKind::Query,
                key_data.clone(),
                mpsc_sender.get_ref().clone(),
            )
            .await;

            (key_data, db_response)
        }
    };

    let db_response = db_response.map_err(|e| CustomError::SqlxErr(e))?;

    let queries_left = match db_response {
        DbResult::QueryRes(val) => val,
        _ => unreachable!(),
    };

    if queries_left > 0 {
        let db_response = send_to_mpsc(
            InstructionKind::Update,
            key_data,
            mpsc_sender.get_ref().clone(),
        )
        .await;
        db_response.map_err(CustomError::SqlxErr)?;
        return Ok(web::Json(json!({
                "queries_left": queries_left - 1,
                "msg": get_random_quote()
        })));
    } else {
        return Err(CustomError::QuotaExhausted)?;
    }
}
