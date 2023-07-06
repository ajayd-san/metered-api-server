use uuid::Uuid;
use serde::Serialize;

pub const TOO_MANY_REQUESTS_MSG: ErrorResponse = ErrorResponse {
    message: "Quota exhausted try again later"
};

pub const BAD_REQUEST_MSG: ErrorResponse = ErrorResponse {
    message: "bad credentials"
};

#[derive(Serialize)]
pub struct ResponseData {
    pub id: u32,
    pub msg: String,
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

#[derive(Serialize, Clone, Debug)]
pub struct KeyRegistarationData {
    pub api_key: String,
    pub quota_per_min: u32,
}

#[derive(Serialize)]
pub struct ErrorResponse<'a> {
    message: &'a str
}

impl DbInstruction {
    pub fn new(kind: InstructionKind, key_data: KeyRegistarationData) -> Self {
        DbInstruction { kind, key_data }
    }
}

impl KeyRegistarationData {
    pub fn new() -> Self {
        let api_key = Uuid::new_v4().simple().to_string();
        KeyRegistarationData {
            api_key,
            quota_per_min: 10,
        }
    }

    pub fn get_with_exisiting(key: &str) -> Self {
        KeyRegistarationData { api_key: key.to_string(), quota_per_min: 10 }
    }
}
