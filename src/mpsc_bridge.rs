use tokio::sync::{mpsc, oneshot};

use metered_api_server::DbInstruction;
use metered_api_server::InstructionKind::*;

use crate::database::DatabaseMgr;

pub async fn bridge(mut receiver: mpsc::Receiver<(DbInstruction, oneshot::Sender<sqlx::Result<()>>)>) {
    let dbm = DatabaseMgr::new().await;

    while let Some((db_instruction, oneshot_sender)) = receiver.recv().await {
        dbg!("got instruction!!", &db_instruction);
        let _ = match db_instruction.kind {
            Register => {
                let db_res = dbm.add_key(&db_instruction.key_data).await;
                oneshot_sender.send(db_res);
            },
            Update => {
                let db_res = dbm.update_quota(&db_instruction.key_data).await;
                oneshot_sender.send(db_res);

            },
            Query => {
                let db_res = dbm.check_quota(&db_instruction.key_data).await;
                oneshot_sender.send(db_res);

            }
        };
    }
}
