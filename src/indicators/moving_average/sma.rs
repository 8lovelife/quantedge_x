use super::MovingAverage;
use std::collections::VecDeque;

/// Simple Moving Average implementation
pub struct SimpleMovingAverage {
    period: usize,
    values: VecDeque<f64>,
    sum: f64,
    current_value: Option<f64>,
}

impl SimpleMovingAverage {
    /// Create a new Simple Moving Average with the specified period
    pub fn new(period: usize) -> Self {
        if period == 0 {
            panic!("Period must be greater than 0");
        }

        SimpleMovingAverage {
            period,
            values: VecDeque::with_capacity(period),
            sum: 0.0,
            current_value: None,
        }
    }
}

impl MovingAverage for SimpleMovingAverage {
    fn update(&mut self, value: f64) {
        // Add new value
        self.sum += value;
        self.values.push_back(value);

        // Remove oldest value if we're at capacity
        if self.values.len() > self.period {
            self.sum -= self.values.pop_front().unwrap();
        }

        // Calculate current SMA
        if self.values.len() == self.period {
            self.current_value = Some(self.sum / self.period as f64);
        } else {
            self.current_value = None; // Not enough data yet
        }
    }

    fn value(&self) -> Option<f64> {
        self.current_value
    }

    fn period(&self) -> usize {
        self.period
    }

    fn reset(&mut self) {
        self.values.clear();
        self.sum = 0.0;
        self.current_value = None;
    }

    fn name(&self) -> &str {
        "SMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_moving_average() {
        let mut sma = SimpleMovingAverage::new(3);

        // Not enough data
        assert_eq!(sma.value(), None);

        sma.update(10.0);
        assert_eq!(sma.value(), None);

        sma.update(20.0);
        assert_eq!(sma.value(), None);

        sma.update(30.0);
        assert_eq!(sma.value(), Some(20.0)); // (10+20+30)/3 = 20

        sma.update(40.0);
        assert_eq!(sma.value(), Some(30.0)); // (20+30+40)/3 = 30
    }

    #[test]
    fn test_reset() {
        let mut sma = SimpleMovingAverage::new(3);
        sma.update(10.0);
        sma.update(20.0);
        sma.update(30.0);

        assert_eq!(sma.value(), Some(20.0));

        sma.reset();
        assert_eq!(sma.value(), None);

        sma.update(40.0);
        assert_eq!(sma.value(), None);
    }

    #[test]
    #[should_panic(expected = "Period must be greater than 0")]
    fn test_zero_period() {
        SimpleMovingAverage::new(0);
    }
}
