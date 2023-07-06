use uuid::Uuid;
use serde::Serialize;

#[derive(Serialize)]
pub struct ResponseData {
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
    pub kind: InstructionKind,
    pub key_data: KeyRegistarationData,
}

#[derive(Serialize, Clone, Debug)]
pub struct KeyRegistarationData {
    pub api_key: String,
    pub quota_per_min: u32,
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
}
