use std::collections::VecDeque;

use crate::indicators::indicator::Indicator;

pub struct SmaIndicator {
    period: usize,
    window: VecDeque<f64>,
    sum: f64,
}
impl SmaIndicator {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            window: VecDeque::with_capacity(period),
            sum: 0.0,
        }
    }
}
impl Indicator for SmaIndicator {
    fn update(&mut self, price: f64) {
        self.window.push_back(price);
        self.sum += price;
        if self.window.len() > self.period {
            self.sum -= self.window.pop_front().unwrap();
        }
    }
    fn value(&self) -> Option<f64> {
        if self.window.len() == self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }
    fn name(&self) -> &'static str {
        "sma"
    }
}
