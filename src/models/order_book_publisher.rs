use crate::models::{
    delta_builder::DeltaBuilder, depth_aggregator::DepthAggregator, level_update::LevelChange,
    order_book_message::OrderBookMessage,
};

pub struct OrderBookPublisher {
    depth: DepthAggregator,
    delta: DeltaBuilder,

    last_sent_update_id: Option<u64>,
    force_snapshot: bool,
}

impl OrderBookPublisher {
    pub fn new(top_n: usize, delta_threshold: usize) -> Self {
        Self {
            depth: DepthAggregator::new(top_n),
            delta: DeltaBuilder::new(delta_threshold),
            last_sent_update_id: None,
            force_snapshot: false,
        }
    }

    pub fn on_level_change(&mut self, change: LevelChange) -> Option<OrderBookMessage> {
        let id = change.update_id;

        if let Some(last) = self.last_sent_update_id {
            if id != last + 1 {
                self.force_snapshot = true;
            }
        }

        self.depth.ingest(change.clone().level_updates, id);

        if self.force_snapshot {
            return self.emit_snapshot();
        }

        let msg = self.delta.on_level_updates(change);

        if let Some(ref m) = msg {
            self.last_sent_update_id = Some(m.end_id());
        }

        msg
    }

    fn emit_snapshot(&mut self) -> Option<OrderBookMessage> {
        println!("emit snapshot");
        let snapshot = self.depth.snapshot();

        if let Some(ref msg) = snapshot {
            self.last_sent_update_id = Some(msg.end_id());
        }

        self.delta.reset();
        self.force_snapshot = false;

        snapshot
    }
}
