mod error;
mod scope;
mod vm;

use anathema_compiler::{ViewId, ViewIds};
use anathema_values::hashmap::HashMap;
use anathema_widget_core::expressions::{root_view, Expression};
use anathema_widget_core::views::{AnyView, RegisteredViews, View};
pub use vm::VirtualMachine;

use self::error::{Error, Result};

struct ViewTemplates {
    view_ids: ViewIds,
    inner: HashMap<ViewId, Template>,
    dep_list: Vec<ViewId>,
}

impl ViewTemplates {
    fn new() -> Self {
        Self {
            view_ids: ViewIds::new(),
            inner: HashMap::new(),
            dep_list: vec![],
        }
    }

    fn get(&mut self, view: ViewId) -> Result<Vec<Expression>> {
        if self.dep_list.iter().any(|v| view.eq(v)) {
            panic!("circular dependencies");
        }

        self.dep_list.push(view);

        let ret = match self.inner.remove(&view) {
            None => return Err(Error::TemplateMissing),
            Some(Template::Pending(src)) => {
                let expressions = templates(&src, self)?;
                self.inner
                    .insert(view, Template::Evaluated(expressions.clone()));
                Ok(expressions)
            }
            Some(Template::Evaluated(expressions)) => {
                let e = expressions.clone();
                self.inner.insert(view, Template::Evaluated(expressions));
                Ok(e)
            }
        };

        self.dep_list.pop();

        ret
    }

    fn insert(&mut self, view: String, template: String) -> ViewId {
        let view = self.view_ids.push(view);
        self.inner.insert(view, Template::Pending(template));
        view
    }
}

pub struct Templates {
    root: String,
    view_templates: ViewTemplates,
}

impl Templates {
    pub fn new(root: String, view: impl View + Send + 'static) -> Self {
        let view_templates = ViewTemplates::new();
        RegisteredViews::add_view(view_templates.view_ids.root_id(), view);
        Self {
            root,
            view_templates,
        }
    }

    pub fn compile(&mut self) -> Result<CompiledTemplates> {
        let expressions = templates(&self.root, &mut self.view_templates)?;
        let root = root_view(expressions, self.view_templates.view_ids.root_id());
        Ok(CompiledTemplates { root: vec![root] })
    }

    pub fn add_view(
        &mut self,
        ident: impl Into<String>,
        template: String,
        view: impl AnyView + 'static,
    ) {
        let ident = ident.into();
        let view_id = self.view_templates.insert(ident.clone(), template);
        RegisteredViews::add_view(view_id.0, view)
    }

    pub fn add_prototype<F, T>(&mut self, ident: impl Into<String>, template: String, f: F)
    where
        F: Send + 'static + Fn() -> T,
        T: 'static + View + Send,
    {
        let ident = ident.into();
        let view_id = self.view_templates.insert(ident.clone(), template);
        RegisteredViews::add_prototype(view_id.0, f)
    }
}

pub struct CompiledTemplates {
    root: Vec<Expression>,
}

impl CompiledTemplates {
    pub fn expressions(&self) -> &[Expression] {
        &self.root
    }
}

enum Template {
    Pending(String),
    Evaluated(Vec<Expression>),
}

fn templates(root: &str, views: &mut ViewTemplates) -> Result<Vec<Expression>> {
    let (instructions, constants) = anathema_compiler::compile(root, &mut views.view_ids)?;
    let vm = VirtualMachine::new(instructions, constants);
    vm.exec(views)
}

#[cfg(test)]
mod test {
    use super::*;

    struct AView;
    impl View for AView {}

    #[test]
    #[should_panic(expected = "circular dependencies")]
    fn circular_deps() {
        let mut t = Templates::new("@a".into(), ());
        t.add_view("a", "@b".to_string(), AView);
        t.add_view("b", "@a".to_string(), AView);
        t.compile().unwrap();
    }
}
