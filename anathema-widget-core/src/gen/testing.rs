use anathema_render::Size;
use unicode_width::UnicodeWidthStr;

use super::generator::Direction;
use super::scope::Scope;
use super::store::Values;
use crate::contexts::{DataCtx, LayoutCtx, PositionCtx};
use crate::error::Result;
use crate::template::Template;
use crate::{
    AnyWidget, Attributes, Factory, TextPath, Value, ValuesAttributes, Widget, WidgetContainer,
    WidgetFactory,
};

pub struct TestWidget(pub String);

impl Widget for TestWidget {
    fn layout<'widget, 'parent>(
        &mut self,
        _: LayoutCtx<'widget, 'parent>,
        _: &mut Vec<WidgetContainer>,
    ) -> Result<Size> {
        Ok(Size::new(self.0.width(), 1))
    }

    fn position<'tpl>(&mut self, _: PositionCtx, _: &mut [WidgetContainer]) {}
}

struct TestWidgetFactory;

impl WidgetFactory for TestWidgetFactory {
    fn make(
        &self,
        store: ValuesAttributes<'_, '_>,
        text: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let text = text
            .map(|path| store.text_to_string(path).to_string())
            .unwrap_or_else(String::new);
        Ok(Box::new(TestWidget(text)))
    }
}

pub fn test_template(text: impl Into<TextPath>) -> Template {
    Template::Node {
        ident: "testwidget".into(),
        attributes: Attributes::empty(),
        text: Some(text.into()),
        children: std::sync::Arc::new([]),
    }
}

pub struct TestSetup {
    templates: Vec<Template>,
    root: DataCtx,
}

impl TestSetup {
    pub fn new() -> Self {
        let _ = Factory::register("testwidget", TestWidgetFactory);
        Self {
            templates: vec![],
            root: DataCtx::default(),
        }
    }

    pub fn with_templates(templates: impl Into<Vec<Template>>) -> Self {
        Self {
            templates: templates.into(),
            root: DataCtx::default(),
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
        let mut store = Values::new(&self.root);
        let inner = Scope::new(&self.templates, &mut store, Direction::Forward);

        TestScope {
            values: store,
            inner,
        }
    }
}

pub struct TestScope<'a> {
    values: Values<'a>,
    pub inner: Scope<'a>,
}

impl TestScope<'_> {
    pub fn next_unchecked(&mut self) -> WidgetContainer {
        self.inner.next(&mut self.values).unwrap().unwrap()
    }

    pub fn next_assume_text(&mut self) -> String {
        let wc = self.next_unchecked();
        wc.to_ref::<TestWidget>().0.clone()
    }
}

impl Iterator for TestScope<'_> {
    type Item = WidgetContainer;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next(&mut self.values).transpose().unwrap()
    }
}
