mod error;
mod scope;
mod vm;

use anathema_values::hashmap::HashMap;
use anathema_widget_core::expressions::Expression;
use anathema_widget_core::views::{RegisteredViews, View, AnyView};
pub use vm::VirtualMachine;

use self::error::Result;

pub struct ViewTemplates {
    inner: HashMap<String, Template>,
    dep_list: Vec<String>,
}

impl ViewTemplates {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            dep_list: vec![],
        }
    }

    pub fn get(&mut self, key: &str) -> Result<Vec<Expression>> {
        if self.dep_list.iter().any(|s| s == key) {
            panic!("cyclic dependency");
        }

        self.dep_list.push(key.into());

        let ret = match self.inner.remove(key) {
            None => panic!("no template, make this an error instead: {key}"),
            Some(Template::Pending(src)) => {
                let expressions = templates(&src, self)?;
                self.inner
                    .insert(key.into(), Template::Evaluated(expressions.clone()));
                Ok(expressions)
            }
            Some(Template::Evaluated(expressions)) => {
                let e = expressions.clone();
                self.inner
                    .insert(key.into(), Template::Evaluated(expressions));
                Ok(e)
            }
        };

        self.dep_list.pop();

        ret
    }

    fn insert(&mut self, ident: String, template: String) {
        self.inner.insert(ident, Template::Pending(template));
    }
}

pub struct Templates {
    root: String,
    root_expressons: Vec<Expression>,
    view_templates: ViewTemplates,
}

impl Templates {
    pub fn new(root: String) -> Self {
        Self {
            root,
            root_expressons: vec![],
            view_templates: ViewTemplates::new(),
        }
    }

    pub fn compile(&mut self) -> Result<()> {
        let expressions = templates(&self.root, &mut self.view_templates);
        self.root_expressons = expressions?;
        Ok(())
    }

    pub fn add_view(&mut self, ident: String, template: String, view: impl AnyView + 'static) {
        self.view_templates.insert(ident.clone(), template);
        RegisteredViews::add_view(ident, view)
    }

    pub fn add_prototype<F, T>(&mut self, ident: String, template: String, f: F)
    where
        F: Send + 'static + Fn() -> T,
        T: 'static + View + std::fmt::Debug + Send,
    {
        self.view_templates.insert(ident.clone(), template);
        RegisteredViews::add_prototype(ident, f)
    }

    pub fn expressions(&self) -> &[Expression] {
        &self.root_expressons
    }
}

enum Template {
    Pending(String),
    Evaluated(Vec<Expression>),
}

fn templates(root: &str, views: &mut ViewTemplates) -> Result<Vec<Expression>> {
    let (instructions, constants) = anathema_compiler::compile(root)?;
    let vm = VirtualMachine::new(instructions, constants);
    vm.exec(views)
}
