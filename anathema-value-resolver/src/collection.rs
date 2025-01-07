use anathema_strings::HStrings;
use anathema_templates::Expression;

use crate::Resolver;

pub struct CollectionResolver;

impl<'bp> Resolver<'bp> for CollectionResolver {
    type Output = ();

    // Collection

    fn resolve(&self, expr: &'bp Expression) -> Self::Output {
        todo!()
    }
}
