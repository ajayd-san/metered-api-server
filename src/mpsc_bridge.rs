use tokio::sync;

use crate::DbInstruction;
use crate::InstructionKind::*;

pub async fn bridge(mut receiver: sync::mpsc::Receiver<DbInstruction>) {
    while let Some(db_instruction) = receiver.recv().await {
        dbg!("got instruction!!", db_instruction);
        // match db_instruction.kind {
        //     Register => dbg!("got instruction"),
        //     Update => todo!(), 
        //     Query => todo!()
        // };
    }
}
