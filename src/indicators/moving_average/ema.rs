use super::MovingAverage;
use std::collections::VecDeque;
/// Exponential Moving Average implementation
pub struct ExponentialMovingAverage {
    period: usize,
    alpha: f64,
    current_value: Option<f64>,
    initialization_values: VecDeque<f64>,
}

impl ExponentialMovingAverage {
    /// Create a new Exponential Moving Average with the specified period
    pub fn new(period: usize) -> Self {
        if period == 0 {
            panic!("Period must be greater than 0");
        }

        ExponentialMovingAverage {
            period,
            alpha: 2.0 / (period as f64 + 1.0),
            current_value: None,
            initialization_values: VecDeque::with_capacity(period),
        }
    }

    /// Initialize EMA with a specific starting value instead of calculating initial SMA
    pub fn with_initial_value(period: usize, initial_value: f64) -> Self {
        if period == 0 {
            panic!("Period must be greater than 0");
        }

        ExponentialMovingAverage {
            period,
            alpha: 2.0 / (period as f64 + 1.0),
            current_value: Some(initial_value),
            initialization_values: VecDeque::new(),
        }
    }
}

impl MovingAverage for ExponentialMovingAverage {
    fn update(&mut self, value: f64) {
        match self.current_value {
            None => {
                // Initialize with SMA
                self.initialization_values.push_back(value);

                // Once we have enough values, calculate the initial SMA
                if self.initialization_values.len() == self.period {
                    let sma: f64 =
                        self.initialization_values.iter().sum::<f64>() / self.period as f64;
                    self.current_value = Some(sma);

                    // Clear initialization values as they're no longer needed
                    self.initialization_values.clear();
                }
            }
            Some(prev_ema) => {
                // EMA = Price * alpha + Previous EMA * (1 - alpha)
                let new_ema = value * self.alpha + prev_ema * (1.0 - self.alpha);
                self.current_value = Some(new_ema);
            }
        }
    }

    fn value(&self) -> Option<f64> {
        self.current_value
    }

    fn period(&self) -> usize {
        self.period
    }

    fn reset(&mut self) {
        self.current_value = None;
        self.initialization_values.clear();
    }

    fn name(&self) -> &str {
        "EMA"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_moving_average() {
        let mut ema = ExponentialMovingAverage::new(3);

        // Feed initial values to get SMA first
        ema.update(10.0);
        ema.update(20.0);
        ema.update(30.0);

        // First EMA is same as SMA
        assert_eq!(ema.value(), Some(20.0));

        // Calculate expected EMA manually
        // alpha = 2/(3+1) = 0.5
        // EMA = 40 * 0.5 + 20 * 0.5 = 30
        ema.update(40.0);
        assert_eq!(ema.value().unwrap().round(), 30.0);
    }

    #[test]
    fn test_with_initial_value() {
        let mut ema = ExponentialMovingAverage::with_initial_value(3, 20.0);

        // First value is initialized
        assert_eq!(ema.value(), Some(20.0));

        // Calculate expected EMA manually
        // alpha = 2/(3+1) = 0.5
        // EMA = 40 * 0.5 + 20 * 0.5 = 30
        ema.update(40.0);
        assert_eq!(ema.value().unwrap().round(), 30.0);
    }

    #[test]
    fn test_reset() {
        let mut ema = ExponentialMovingAverage::new(3);
        ema.update(10.0);
        ema.update(20.0);
        ema.update(30.0);

        assert_eq!(ema.value(), Some(20.0));

        ema.reset();
        assert_eq!(ema.value(), None);

        ema.update(40.0);
        assert_eq!(ema.value(), None);
    }

    #[test]
    #[should_panic(expected = "Period must be greater than 0")]
    fn test_zero_period() {
        ExponentialMovingAverage::new(0);
    }
}
