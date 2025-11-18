pub trait Brush {
    fn bool(&self, key: &str) -> Option<bool>;
    fn i64(&self, key: &str) -> Option<i64>;
    fn f64(&self, key: &str) -> Option<f64>;
}
