use crate::models::{
    delta_builder::DeltaBuilder, depth_aggregator::DepthAggregator, level_update::LevelChange,
    order_book_message::OrderBookMessage,
};

pub struct OrderBookPublisher {
    depth: DepthAggregator,
    delta: DeltaBuilder,
    last_ingested_id: Option<u64>,
    last_sent_update_id: Option<u64>,
    force_snapshot: bool,
}

impl OrderBookPublisher {
    pub fn new(top_n: usize) -> Self {
        Self {
            depth: DepthAggregator::new(top_n),
            delta: DeltaBuilder::new(),
            last_sent_update_id: None,
            last_ingested_id: None,
            force_snapshot: false,
        }
    }

    pub fn on_level_change(&mut self, change: LevelChange) {
        let id = change.update_id;

        self.depth.ingest(change.level_updates.clone(), id);

        if self.force_snapshot {
            return;
        }

        let reference_id = self.last_ingested_id.or(self.last_sent_update_id);

        if let Some(ref_id) = reference_id {
            if id != ref_id + 1 {
                self.force_snapshot = true;
                self.delta.reset();
                self.last_ingested_id = None;
                return;
            }
        } else {
            self.force_snapshot = true;
            return;
        }

        self.delta.on_level_updates(change);
        self.last_ingested_id = Some(id);
    }

    fn emit_snapshot(&mut self) -> Option<OrderBookMessage> {
        let snapshot = self.depth.snapshot();

        if let Some(ref msg) = snapshot {
            self.last_sent_update_id = Some(msg.end_id());
            self.force_snapshot = false;
            self.delta.reset();
        }

        snapshot
    }

    pub fn publish_tick(&mut self) -> Option<OrderBookMessage> {
        let msg = if self.force_snapshot {
            self.emit_snapshot()
        } else {
            self.delta.flush()
        };

        if let Some(ref m) = msg {
            self.last_sent_update_id = Some(m.end_id());
            self.last_ingested_id = None;
            self.force_snapshot = false;
        }

        msg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::order::Side;
    use crate::matcher::domain::price_ticks::PriceTicks;
    use crate::matcher::domain::qty_lots::QtyLots;
    use crate::models::level_update::LevelUpdate;

    fn mock_change(id: u64, price: PriceTicks, qty: Option<QtyLots>) -> LevelChange {
        LevelChange {
            update_id: id,
            level_updates: vec![LevelUpdate {
                price: price,
                new_qty: qty,
                side: Side::Bid,
            }],
        }
    }

    #[test]
    fn test_normal_incremental_flow() {
        let mut publisher = OrderBookPublisher::new(10);

        publisher.on_level_change(mock_change(100, PriceTicks(60000), Some(QtyLots(10))));
        let msg = publisher.publish_tick().expect("Should emit snapshot");

        match msg {
            OrderBookMessage::Snapshot { last_update_id, .. } => assert_eq!(last_update_id, 100),
            _ => panic!("Expected snapshot for first message"),
        }

        publisher.on_level_change(mock_change(101, PriceTicks(60001), Some(QtyLots(10))));
        publisher.on_level_change(mock_change(102, PriceTicks(60000), Some(QtyLots(50))));

        let msg = publisher.publish_tick().expect("Should emit delta");
        if let OrderBookMessage::Delta {
            bids,
            start_id,
            end_id,
            ..
        } = msg
        {
            assert_eq!(start_id, 101);
            assert_eq!(end_id, 102);
            assert_eq!(bids.len(), 2);
            let price_60000 = bids
                .iter()
                .find(|(p, _)| p.to_f64(0.1) == PriceTicks(60000).to_f64(0.1))
                .unwrap();
            assert_eq!(price_60000.1.unwrap().to_f64(0.1), QtyLots(50).to_f64(0.1));
        } else {
            panic!("Expected delta");
        }
    }

    #[test]
    fn test_gap_triggers_snapshot() {
        let mut publisher = OrderBookPublisher::new(10);

        publisher.on_level_change(mock_change(100, PriceTicks(60000), Some(QtyLots(1))));
        publisher.publish_tick();

        publisher.on_level_change(mock_change(102, PriceTicks(60002), Some(QtyLots(3))));

        let msg = publisher
            .publish_tick()
            .expect("Should emit snapshot due to gap");
        match msg {
            OrderBookMessage::Snapshot { last_update_id, .. } => assert_eq!(last_update_id, 102),
            _ => panic!("Expected snapshot after gap"),
        }
    }

    #[test]
    fn test_id_rollback_triggers_snapshot() {
        let mut publisher = OrderBookPublisher::new(10);

        publisher.on_level_change(mock_change(200, PriceTicks(60000), Some(QtyLots(1))));
        publisher.publish_tick();

        publisher.on_level_change(mock_change(50, PriceTicks(60000), Some(QtyLots(2))));

        let msg = publisher
            .publish_tick()
            .expect("Should emit snapshot due to rollback");
        match msg {
            OrderBookMessage::Snapshot { last_update_id, .. } => assert_eq!(last_update_id, 50),
            _ => panic!("Expected snapshot after rollback"),
        }
    }

    #[test]
    fn test_no_changes_no_message() {
        let mut publisher = OrderBookPublisher::new(10);

        publisher.on_level_change(mock_change(100, PriceTicks(60000), Some(QtyLots(10))));
        publisher.publish_tick();

        let msg = publisher.publish_tick();
        assert!(
            msg.is_none(),
            "Should not emit anything if no changes occurred"
        );
    }
}
