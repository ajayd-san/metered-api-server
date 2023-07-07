use serde::Serialize;
use uuid::Uuid;

pub const TOO_MANY_REQUESTS_MSG: ErrorResponse = ErrorResponse {
    message: "Quota exhausted try again later",
};

pub const BAD_REQUEST_MSG: ErrorResponse = ErrorResponse {
    message: "bad credentials",
};

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
