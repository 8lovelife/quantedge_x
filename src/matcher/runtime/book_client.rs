use tokio::sync::{mpsc, oneshot};

use crate::matcher::{
    domain::{order::Order, tif_policy_result::TifPolicyResult},
    runtime::cmd::Cmd,
};

pub struct BookClient {
    tx: mpsc::Sender<Cmd>,
}

impl BookClient {
    pub fn new(tx: mpsc::Sender<Cmd>) -> Self {
        Self { tx }
    }

    pub async fn place_order(&self, order: Order) -> anyhow::Result<TifPolicyResult> {
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
