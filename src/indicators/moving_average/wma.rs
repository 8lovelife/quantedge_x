use super::MovingAverage;
use std::collections::VecDeque;

/// Weighted Moving Average implementation
pub struct WeightedMovingAverage {
    period: usize,
    values: VecDeque<f64>,
    weight_sum: usize,
    current_value: Option<f64>,
}

impl WeightedMovingAverage {
    /// Create a new Weighted Moving Average with the specified period
    pub fn new(period: usize) -> Self {
        if period == 0 {
            panic!("Period must be greater than 0");
        }

        // Calculate the sum of weights: 1 + 2 + 3 + ... + period
        let weight_sum = (period * (period + 1)) / 2;

        WeightedMovingAverage {
            period,
            values: VecDeque::with_capacity(period),
            weight_sum,
            current_value: None,
        }
    }
}

impl MovingAverage for WeightedMovingAverage {
    fn update(&mut self, value: f64) {
        // Add new value
        self.values.push_back(value);

        // Remove oldest value if we're at capacity
        if self.values.len() > self.period {
            self.values.pop_front();
        }

        // Calculate WMA if we have enough data
        if self.values.len() == self.period {
            let mut weighted_sum = 0.0;
            for (i, &price) in self.values.iter().enumerate() {
                let weight = i + 1; // 1, 2, 3, etc.
                weighted_sum += price * weight as f64;
            }
            self.current_value = Some(weighted_sum / self.weight_sum as f64);
        } else {
            self.current_value = None;
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
        self.current_value = None;
    }

    fn name(&self) -> &str {
        "WMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_moving_average() {
        let mut wma = WeightedMovingAverage::new(3);

        wma.update(10.0);
        assert_eq!(wma.value(), None);

        wma.update(20.0);
        assert_eq!(wma.value(), None);

        // WMA = (10*1 + 20*2 + 30*3) / (1+2+3) = (10 + 40 + 90) / 6 = 140/6 = 23.33
        wma.update(30.0);
        assert!((wma.value().unwrap() - 23.33).abs() < 0.01);

        // WMA = (20*1 + 30*2 + 40*3) / 6 = (20 + 60 + 120) / 6 = 200/6 = 33.33
        wma.update(40.0);
        assert!((wma.value().unwrap() - 33.33).abs() < 0.01);
    }

    #[test]
    fn test_reset() {
        let mut wma = WeightedMovingAverage::new(3);
        wma.update(10.0);
        wma.update(20.0);
        wma.update(30.0);

        assert!(wma.value().is_some());

        wma.reset();
        assert_eq!(wma.value(), None);

        wma.update(40.0);
        assert_eq!(wma.value(), None);
    }

    #[test]
    #[should_panic(expected = "Period must be greater than 0")]
    fn test_zero_period() {
        WeightedMovingAverage::new(0);
    }
}
