use tokio::sync::oneshot;

use crate::matcher::domain::{
    book_info::BookInfo, execution_result::ExecutionResult, order::Order,
};

pub enum Cmd {
    Info {
        resp: Option<oneshot::Sender<anyhow::Result<BookInfo>>>,
    },
    Place {
        order: Order,
        resp: Option<oneshot::Sender<anyhow::Result<ExecutionResult>>>,
    },
    Cancel {
        id: u64,
        resp: Option<oneshot::Sender<anyhow::Result<bool>>>,
    },
}
