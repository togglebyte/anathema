use anathema_values::ScopeId;

pub enum Node {
    Single,
    Collection(ScopeId),
}
