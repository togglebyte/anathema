#[derive(Debug)]
pub struct ViewCollection {
    inner: Vec<(String, Box<dyn View>)>,
}

impl Default for ViewCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewCollection {
    const fn new() -> Self {
        Self { inner: vec![] }
    }

    pub fn register(&mut self, key: String, view: impl View + 'static) {
        self.inner.push((key, Box::new(view)))
    }

    pub(crate) fn get(&self, key: &str) -> Option<&dyn View> {
        self.inner
            .iter()
            .filter_map(|(k, v)| k.eq(key).then_some(v.as_ref()))
            .next()
    }
}

pub trait View: std::fmt::Debug + Send + Sync {
    fn templates(&self) -> ();
}

impl View for Box<dyn View> {
    fn templates(&self) -> () {
        self.as_ref().templates()
    }
}
