use std::{collections::HashMap, sync::Arc};

use tokio::{sync::mpsc, task::JoinHandle};

use crate::matcher::{
    domain::match_output::MatchOutput,
    engine::{engine_event::EngineEvent, event_kind::EventKind},
};

type RouteFn = Arc<dyn Fn(EngineEvent) + Send + Sync>;

pub struct EventRouter {
    rx: Option<mpsc::Receiver<EngineEvent>>,
    pub tx: mpsc::Sender<EngineEvent>,
    handlers: HashMap<EventKind, mpsc::Sender<EngineEvent>>,
}

impl EventRouter {
    pub fn spawn(&mut self) -> JoinHandle<()> {
        let mut rx = self.rx.take().expect("spawn can only be called once");
        let handlers = std::mem::take(&mut self.handlers);

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Some(tx) = handlers.get(&event.kind()) {
                    let _ = tx.send(event).await;
                }
            }
        })
    }

    pub async fn send_event(&self, out: MatchOutput) {
        let _ = self.tx.send(EngineEvent::LevelChange(out.deltas)).await;
        let _ = self
            .tx
            .send(EngineEvent::TradeEventResult(out.trade_event))
            .await;
    }
}

pub fn create_router_with_workers(
    levelchange_handler: RouteFn,
    trade_tick_handler: RouteFn,
) -> EventRouter {
    let mut handlers = HashMap::new();

    let (tx, mut rx) = mpsc::channel(1024);
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            levelchange_handler(event);
        }
    });
    handlers.insert(EventKind::LevelChange, tx);

    let (tx, mut rx) = mpsc::channel(1024);
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            trade_tick_handler(event);
        }
    });
    handlers.insert(EventKind::TradeEventResult, tx);

    let (tx_event, rx_event) = mpsc::channel(4096);

    EventRouter {
        rx: Some(rx_event),
        tx: tx_event,
        handlers,
    }
}
