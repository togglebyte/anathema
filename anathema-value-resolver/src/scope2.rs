use crate::Value;

struct ScopeId(Box<[u16]>);

pub struct Scope<'bp> {
    children: Vec<ScopeId>,
    values: Vec<Value<'bp>>
}
