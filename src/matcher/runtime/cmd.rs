use tokio::sync::oneshot;

use crate::matcher::domain::{execution_result::ExecutionResult, order::Order};

pub enum Cmd {
    Place {
        order: Order,
        resp: Option<oneshot::Sender<anyhow::Result<ExecutionResult>>>,
    },

    Cancel {
        id: u64,
        resp: Option<oneshot::Sender<anyhow::Result<bool>>>,
    },
}
