use super::generator::Direction;
use super::scope::Scope;
use super::store::Store;
use crate::template::Template;
use crate::{DataCtx, Lookup, Value, WidgetContainer};

pub struct TestSetup {
    templates: Vec<Template>,
    root: DataCtx,
    factory: Lookup,
}

impl TestSetup {
    pub fn new() -> Self {
        Self {
            templates: vec![],
            root: DataCtx::default(),
            factory: Lookup::default(),
        }
    }

    pub fn with_templates(templates: impl Into<Vec<Template>>) -> Self {
        Self {
            templates: templates.into(),
            root: DataCtx::default(),
            factory: Lookup::default(),
        }
    }

    pub fn set(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.root.insert(key, value);
        self
    }

    pub fn template(mut self, template: Template) -> Self {
        self.templates.push(template);
        self
    }

    pub fn scope<'a>(&'a mut self) -> TestScope<'a> {
        let mut store = Store::new(&self.root);
        let inner = Scope::new(
            &self.templates,
            &self.factory,
            &mut store,
            Direction::Forward,
        );

        TestScope {
            values: store,
            inner,
        }
    }
}

pub struct TestScope<'a> {
    values: Store<'a>,
    pub inner: Scope<'a, 'a>,
}

impl<'a> TestScope<'a> {
    pub fn next_unchecked(&mut self) -> WidgetContainer<'a> {
        self.inner.next(&mut self.values).unwrap().unwrap()
    }

    pub fn next_assume_text(&mut self) -> String {
        let wc = self.next_unchecked();
        wc.to_ref::<crate::Text>().text.to_string()
    }
}

impl<'a> Iterator for TestScope<'a> {
    type Item = WidgetContainer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next(&mut self.values).transpose().unwrap()
    }
}
