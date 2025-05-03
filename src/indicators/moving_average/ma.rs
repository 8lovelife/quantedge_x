/// Common trait for all moving average implementations
pub trait MovingAverage {
    /// Add a new price data point to the moving average
    fn update(&mut self, value: f64);

    /// Get the current moving average value, if available
    fn value(&self) -> Option<f64>;

    /// Check if the moving average has enough data to return a valid value
    fn is_ready(&self) -> bool {
        self.value().is_some()
    }

    /// Get the period of the moving average
    fn period(&self) -> usize;

    /// Reset the moving average to its initial state
    fn reset(&mut self);

    /// Get the name of the moving average type
    fn name(&self) -> &str;
}
