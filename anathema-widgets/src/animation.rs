use std::ops::{Add, Mul, Sub};
use std::time::Duration;

use super::Pos;
use crate::values::Easing;

/// The animation context holds the animation for the [`crate::WidgetContainer`].
#[derive(Debug, Clone)]
pub struct AnimationCtx {
    transitions: Vec<(String, Animation<f32>)>,
    position: Option<Animation<Pos>>,
}

impl AnimationCtx {
    /// Add a new transition to the list of transitions
    /// If the key already exists it's replaced
    pub fn push(&mut self, key: impl Into<String>, animation: Animation<f32>) {
        let key = key.into();
        // Remove old animation if one exists
        self.remove(&key);
        self.transitions.push((key, animation));
    }

    /// Remove a transition with a given key if it exists
    pub fn remove(&mut self, key: impl AsRef<str>) {
        self.transitions.retain(|(k, _)| key.as_ref().ne(k));
    }

    /// Update all transitions with the new time delta.
    /// Without this function call none of the transitions would progress.
    pub fn update(&mut self, elapsed: Duration) {
        self.transitions.iter_mut().for_each(|(_, a)| {
            let _ = a.update(elapsed);
        });
        self.position.as_mut().map(|a| {
            a.update(elapsed);
        });
    }

    /// Set the position for the position transition (which is a unique transition that operates on
    /// [`Pos`] rather than `f32`).
    pub fn set_position(&mut self, duration: Duration, easing: Easing) {
        match &mut self.position {
            Some(pos) => pos.duration = duration,
            None => self.position = Some(Animation::new(duration, easing)),
        }
    }

    pub(super) fn new() -> Self {
        Self {
            transitions: vec![],
            position: None,
        }
    }

    pub(super) fn update_dst(&mut self, key: &str, val: f32) -> bool {
        if let Some((_, anim)) = self.transitions.iter_mut().find(|(k, _)| k.eq(key)) {
            anim.set_dst(val);
            true
        } else {
            false
        }
    }

    // pub(super) fn attributes(&self) -> Attributes {
    //     let mut attributes = Attributes::empty();

    //     for (k, val) in self.transitions.iter().filter_map(|(k, a)| a.get_value().map(|v| (k, v))) {
    //         attributes.set(k, val as i64);
    //     }

    //     attributes
    // }

    pub(super) fn update_pos(&mut self, current: Pos, pos: Pos) -> Option<()> {
        let position = self.position.as_mut()?;
        if position.set_dst(pos) {
            position.set_src(current);
        }
        Some(())
    }

    pub(super) fn get_pos(&self) -> Option<Pos> {
        self.position.as_ref().and_then(|a| a.get_value())
    }

    pub(super) fn get_value(&self, key: &str) -> Option<f32> {
        self.transitions
            .iter()
            .find(|(k, _)| k.eq(key))
            .and_then(|(_, a)| a.get_value())
    }
}

/// An animation / transition consists of a source (starting value) and a destination (end value),
/// an easing function
#[derive(Debug, Clone)]
pub struct Animation<T> {
    src: Option<T>,
    dst: Option<T>,
    easing: Easing,
    duration: Duration,
    elapsed: Duration,
}

impl<T: PartialEq> Animation<T> {
    /// Create a new animation given a [`Duration`] and [`Easing`].
    /// A new animation will not progress until a source and destination has been set.
    pub fn new(duration: Duration, easing: Easing) -> Self {
        Self {
            src: None,
            dst: None,
            duration,
            elapsed: Duration::new(0, 0),
            easing,
        }
    }

    /// Set the destination value.
    /// Returns `true` if the value was changed
    pub fn set_dst(&mut self, dst: T) -> bool {
        if self.dst.as_ref().map(|curr| dst.eq(curr)).unwrap_or(false) {
            return false;
        }

        if self.dst.is_some() {
            self.src = self.dst.take();
        }

        self.dst = dst.into();
        self.elapsed = Duration::new(0, 0);
        true
    }

    /// Set the source of the transition.
    /// This is where it will transition from
    pub fn set_src(&mut self, src: T) {
        self.src = src.into();
    }

    // The animation is considered inactive once the elapsed time exceed the duration.
    fn active(&self) -> bool {
        self.elapsed < self.duration
    }

    fn update(&mut self, delta: Duration) -> Option<()> {
        self.active().then_some(())?;
        self.elapsed += delta;
        Some(())
    }
}

impl<T> Animation<T>
where
    T: std::fmt::Debug
        + Copy
        + PartialEq
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<f32, Output = T>,
{
    fn get_value(&self) -> Option<T> {
        if !self.active() {
            return self.dst;
        }

        let time = self.elapsed.as_millis() as f32 / self.duration.as_millis() as f32;
        let time = self.easing.apply(time);
        let src = self.src?;
        let dst = self.dst?;

        Some(src + (dst - src) * time)
    }
}
