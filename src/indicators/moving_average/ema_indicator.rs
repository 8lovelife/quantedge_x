use crate::indicators::indicator::Indicator;

pub struct EmaIndicator {
    multiplier: f64,
    prev: Option<f64>,
}
impl EmaIndicator {
    pub fn new(period: usize) -> Self {
        Self {
            multiplier: 2.0 / (period as f64 + 1.0),
            prev: None,
        }
    }
}
impl Indicator for EmaIndicator {
    fn update(&mut self, price: f64) {
        self.prev = Some(match self.prev {
            Some(p) => (price - p) * self.multiplier + p,
            None => price,
        });
    }
    fn value(&self) -> Option<f64> {
        self.prev
    }
    fn name(&self) -> &'static str {
        "ema"
    }
}
