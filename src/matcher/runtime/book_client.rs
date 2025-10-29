use tokio::sync::{mpsc, oneshot};

use crate::matcher::{
    domain::{book_info::BookInfo, execution_result::ExecutionResult, order::Order},
    runtime::cmd::Cmd,
};

pub struct BookClient {
    tx: mpsc::Sender<Cmd>,
}

impl BookClient {
    pub fn new(tx: mpsc::Sender<Cmd>) -> Self {
        Self { tx }
    }

    pub async fn info_book(&self) -> anyhow::Result<BookInfo> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Cmd::Info { resp: Some(tx) }).await?;
        rx.await?
    }

    pub async fn place_order(&self, order: Order) -> anyhow::Result<ExecutionResult> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Cmd::Place {
                order: order,
                resp: Some(tx),
            })
            .await?;
        rx.await?
    }

    pub async fn cancel_order(&self, id: u64) -> anyhow::Result<bool> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Cmd::Cancel { id, resp: Some(tx) }).await?;
        rx.await?
    }
}
