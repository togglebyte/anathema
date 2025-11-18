use anathema_compiler::Color;

pub trait Brush {
    fn bool(&self, key: &str) -> Option<bool>;
    fn i64(&self, key: &str) -> Option<i64>;
    fn f64(&self, key: &str) -> Option<f64>;
    fn color(&self, key: &str) -> Option<Color>;
}
