pub trait Indicator {
    fn update(&mut self, price: f64);
    fn value(&self) -> Option<f64>;
    fn name(&self) -> &'static str;
}
