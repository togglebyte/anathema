pub use self::map::Map;
pub use self::list::List;

mod map;
mod list;

pub enum ValueV2<T> {
    Single(T),
    Map(Map<T>),
    List(List<T>),
}
