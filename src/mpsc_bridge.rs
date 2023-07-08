use crate::Db;
use tokio::sync::{mpsc, oneshot};

use crate::DbInstruction;
use crate::InstructionKind::*;

use crate::database::{DatabaseMgr, DbResult};

pub async fn bridge(
    mut receiver: mpsc::Receiver<(DbInstruction, oneshot::Sender<sqlx::Result<DbResult>>)>,
) {
    let dbm = DatabaseMgr::new().await;

    while let Some((db_instruction, oneshot_sender)) = receiver.recv().await {
        dbg!(&db_instruction);
        let db_res = match db_instruction.key_data.db_name {
            Db::API_KEY => {
                let db_res = match db_instruction.kind {
                    Register => dbm.add_api_key(&db_instruction.key_data).await,
                    Update => dbm.update_quota_api_key(&db_instruction.key_data).await,
                    Query => dbm.check_quota_api_key(&db_instruction.key_data).await,
                    Reset => dbm.reset_quota_api_key().await,
                };
                db_res
            }
            Db::IP_BOOK => {
                let db_res = match db_instruction.kind {
                    Register => dbm.add_ip(&db_instruction.key_data).await,
                    Update => dbm.update_quota_ip(&db_instruction.key_data).await,
                    Query => dbm.check_quota_ip(&db_instruction.key_data).await,
                    Reset => dbm.reset_quota_ip().await,
                };
                db_res
            }
        };

        oneshot_sender.send(db_res).unwrap();
    }
}
