mod database;
pub mod mpsc_bridge;
pub mod services;

use database::DbResult;
use serde::Serialize;
use tokio::{sync::{mpsc, oneshot}, time::{self, Duration}};
use uuid::Uuid;

pub const TOO_MANY_REQUESTS_MSG: ErrorResponse = ErrorResponse {
    message: "Quota exhausted try again later",
};

pub const BAD_REQUEST_MSG: ErrorResponse = ErrorResponse {
    message: "bad credentials",
};

async fn send_to_mpsc(
    instr_kind: InstructionKind,
    key_data: KeyRegistarationData,
    mpsc_sender: mpsc::Sender<(DbInstruction, oneshot::Sender<sqlx::Result<DbResult>>)>,
) -> sqlx::Result<DbResult> {
    let db_instruction = DbInstruction::new(instr_kind, key_data.clone());
    let (oneshot_sender, oneshot_receiver) = oneshot::channel();
    mpsc_sender
        .send((db_instruction, oneshot_sender))
        .await
        .unwrap();
    oneshot_receiver
        .await
        .map_err(actix_web::error::ErrorInternalServerError)
        .unwrap()
}

pub async fn reset_quota(mpsc_sender: mpsc::Sender<(DbInstruction, oneshot::Sender<sqlx::Result<DbResult>>)>) {
    loop {
        time::sleep(Duration::from_secs(30*60)).await;
        let key_data = KeyRegistarationData::get_with_exisiting("dummy", Db::API_KEY);
        send_to_mpsc(InstructionKind::Reset, key_data, mpsc_sender.clone()).await.unwrap();
        let key_data = KeyRegistarationData::get_with_exisiting("dummy", Db::IP_BOOK);
        send_to_mpsc(InstructionKind::Reset, key_data, mpsc_sender.clone()).await.unwrap();
    }
}

#[derive(Serialize)]
pub struct ResponseData {
    pub id: u32,
    pub msg: String,
}

#[derive(Debug, Serialize, Clone)]
pub enum Db {
    API_KEY,
    IP_BOOK
}

#[derive(Debug)]
pub enum InstructionKind {
    Register,
    Update,
    Query,
    Reset
}

#[derive(Debug)]
pub struct DbInstruction {
    pub kind: InstructionKind,
    pub key_data: KeyRegistarationData,
}

#[derive(Serialize,Clone, Debug)]
pub struct KeyRegistarationData {
    pub key: String,
    pub quota_per_min: u32,
    pub db_name: Db,
}

#[derive(Serialize)]
pub struct ErrorResponse<'a> {
    message: &'a str,
}

impl DbInstruction {
    pub fn new(kind: InstructionKind, key_data: KeyRegistarationData) -> Self {
        DbInstruction { kind, key_data }
    }
}

impl KeyRegistarationData {
    pub fn new(db_name: Db) -> Self {
        let api_key = Uuid::new_v4().simple().to_string();
        KeyRegistarationData {
            key: api_key,
            quota_per_min: 10,
            db_name,
        }
    }

    pub fn get_with_exisiting(key: &str, db_name: Db) -> Self {
        KeyRegistarationData {
            key: key.to_string(),
            quota_per_min: 10,
            db_name
        }
    }
}
