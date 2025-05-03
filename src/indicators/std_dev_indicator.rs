use std::collections::VecDeque;

use super::indicator::Indicator;

pub struct StdDevIndicator {
    window: VecDeque<f64>,
    period: usize,
}
impl StdDevIndicator {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            window: VecDeque::with_capacity(period),
        }
    }
}
impl Indicator for StdDevIndicator {
    fn update(&mut self, price: f64) {
        self.window.push_back(price);
        if self.window.len() > self.period {
            self.window.pop_front();
        }
    }
    fn value(&self) -> Option<f64> {
        if self.window.len() == self.period {
            let mean = self.window.iter().sum::<f64>() / self.period as f64;
            let var =
                self.window.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / self.period as f64;
            Some(var.sqrt())
        } else {
            None
        }
    }
    fn name(&self) -> &'static str {
        "stddev"
    }
}
