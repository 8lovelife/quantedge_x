use tokio::sync::oneshot;

use crate::matcher::domain::{order::Order, tif_policy_result::TifPolicyResult};

pub enum Cmd {
    Place {
        order: Order,
        resp: Option<oneshot::Sender<anyhow::Result<TifPolicyResult>>>,
    },

    Cancel {
        id: u64,
        resp: Option<oneshot::Sender<anyhow::Result<bool>>>,
    },
}
