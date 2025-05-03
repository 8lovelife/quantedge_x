use std::collections::VecDeque;

use crate::indicators::indicator::Indicator;

pub struct WmaIndicator {
    period: usize,
    window: VecDeque<f64>,
}
impl WmaIndicator {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            window: VecDeque::with_capacity(period),
        }
    }
}
impl Indicator for WmaIndicator {
    fn update(&mut self, price: f64) {
        self.window.push_back(price);
        if self.window.len() > self.period {
            self.window.pop_front();
        }
    }
    fn value(&self) -> Option<f64> {
        if self.window.len() == self.period {
            let mut num = 0.0;
            let mut den = 0.0;
            for (i, &p) in self.window.iter().enumerate() {
                let w = (i + 1) as f64;
                num += p * w;
                den += w;
            }
            Some(num / den)
        } else {
            None
        }
    }
    fn name(&self) -> &'static str {
        "wma"
    }
}
