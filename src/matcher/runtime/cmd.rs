use tokio::sync::oneshot;

use crate::matcher::domain::{order::Order, tif_result::TifResult};

pub enum Cmd {
    Place {
        order: Order,
        resp: Option<oneshot::Sender<anyhow::Result<TifResult>>>,
    },

    Cancel {
        id: u64,
        resp: Option<oneshot::Sender<anyhow::Result<bool>>>,
    },
}
