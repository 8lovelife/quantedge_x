use tokio::task::JoinHandle;

use crate::matcher::engine::engine::Engine;

pub struct EngineRuntime {
    pub engine: Engine,
    pub trade_task: JoinHandle<()>,
}
