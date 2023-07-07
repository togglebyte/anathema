use anathema_render::Size;
use unicode_width::UnicodeWidthStr;

use super::generator::Direction;
use super::scope::Scope;
use super::store::Store;
use crate::contexts::{DataCtx, LayoutCtx, PositionCtx};
use crate::error::Result;
use crate::template::Template;
use crate::{
    AnyWidget, Attributes, Factory, TextPath, Value, ValuesAttributes, Widget, WidgetContainer,
    WidgetFactory,
};

pub struct TestWidget(pub String);

impl Widget for TestWidget {
    fn layout<'widget, 'tpl, 'parent>(
        &mut self,
        _: LayoutCtx<'widget, 'tpl, 'parent>,
        _: &mut Vec<WidgetContainer<'tpl>>,
    ) -> Result<Size> {
        Ok(Size::new(self.0.width(), 1))
    }

    fn position<'tpl>(&mut self, _: PositionCtx, _: &mut [WidgetContainer<'tpl>]) {}
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
        children: vec![],
    }
}

pub struct TestSetup {
    templates: Vec<Template>,
    root: DataCtx,
}

impl TestSetup {
    pub fn new() -> Self {
        Factory::register("testwidget", TestWidgetFactory);
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
        let mut store = Store::new(&self.root);
        let inner = Scope::new(&self.templates, &mut store, Direction::Forward);

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
        wc.to_ref::<TestWidget>().0.clone()
    }
}

impl<'a> Iterator for TestScope<'a> {
    type Item = WidgetContainer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next(&mut self.values).transpose().unwrap()
    }
}
