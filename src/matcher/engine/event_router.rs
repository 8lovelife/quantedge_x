use std::{collections::HashMap, sync::Arc};

use tokio::{sync::mpsc, task::JoinHandle};

use crate::matcher::engine::{engine_event::EngineEvent, event_kind::EventKind};

type RouteFn = Arc<dyn Fn(EngineEvent) + Send + Sync>;

pub struct EventRouter {
    rx: mpsc::Receiver<EngineEvent>,
    tx: mpsc::Sender<EngineEvent>,
    handlers: HashMap<EventKind, mpsc::Sender<EngineEvent>>,
}

impl EventRouter {
    pub fn spawn(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(event) = self.rx.recv().await {
                let kind = event.kind();
                if let Some(tx) = self.handlers.get(&kind) {
                    let _ = tx.send(event).await;
                }
            }
        })
    }
}

pub fn create_router_with_workers(
    levelchange_handler: RouteFn,
    trade_tick_handler: RouteFn,
) -> EventRouter {
    let mut handlers = HashMap::new();
    let (tx, mut rx) = mpsc::channel(1024);
    let handler = Arc::clone(&levelchange_handler);
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            handler(event);
        }
    });
    handlers.insert(EventKind::LevelChange, tx);

    let (tx, mut rx) = mpsc::channel(1024);
    let handler = Arc::clone(&trade_tick_handler);
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            handler(event);
        }
    });
    handlers.insert(EventKind::TradeEventResult, tx);

    let (tx_event, rx_event) = mpsc::channel(4096);
    EventRouter {
        rx: rx_event,
        tx: tx_event,
        handlers,
    }
}
