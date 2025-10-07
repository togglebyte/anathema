pub trait Queryable {
    fn by_tag(self) -> impl Queryable;
}

pub struct QueryBuilder<T> {
    inner: T
}

impl<T> Queryable for QueryBuilder<T> {
    fn by_tag(self) -> impl Queryable {
        self
    }
}
