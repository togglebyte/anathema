// #![deny(missing_docs)]
#[allow(unused_extern_crates)]
extern crate anathema_state as anathema;

// Has docs
pub mod attributes;
pub mod frontend;
pub mod layout;

// Dosn't like has docs so is a bit mid
pub mod runtime;
pub mod templates;

// Decide what to do here
// pub mod ui;

pub(crate) mod testing;
