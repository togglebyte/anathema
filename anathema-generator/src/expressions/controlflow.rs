use anathema_values::ScopeValue;

pub enum Cond {
    If(ScopeValue),
    Else(Option<ScopeValue>),
}

#[derive(Debug)]
pub struct FlowState {
}
