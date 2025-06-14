use anathema::prelude::*;
use anathema::component::*;

#[derive(Debug, Default, State)]
pub struct Thing {
    value: Value<u32>
}
