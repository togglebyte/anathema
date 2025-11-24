use std::marker::PhantomData;

/// Screen coordinates refers to the coordinate space of the active monitor.
/// `0,0` is the top left of the monitor.
#[derive(Debug, Copy, Clone)]
pub struct Screen;

/// The coordinate space of the window.
/// `0,0` is the top left of the window.
#[derive(Debug, Copy, Clone)]
pub struct Window;

/// The coordinate space of the "world".
/// `0,0` is the centre of the world.
#[derive(Debug, Copy, Clone)]
pub struct World;

/// The coordinate space of the UI layer.
/// `0,0` is the same as 
#[derive(Debug, Copy, Clone)]
pub struct Ui;

/// The coordinate space of the UI layer.
/// `0,0` is the same as 
#[derive(Debug, Copy, Clone)]
pub struct Character;

#[derive(Debug, Copy, Clone)]
pub struct Pos<C> {
    coord_sys: C,
    inner: anathema_geometry::Pos,
}
