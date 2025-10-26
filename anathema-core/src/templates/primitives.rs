use std::fmt::Display;

use anathema_state::Hex;

/// Primitive values that can appear in template expressions.
///
/// Primitives represent the basic literal values that can be written directly in
/// template source code. They are compile-time constants that are resolved during
/// template parsing and stored in the blueprint.
///
/// # Primitive Types
///
/// - **Bool**: Boolean values (`true` or `false`)
/// - **Char**: Single Unicode characters (`'a'`, `'🎉'`)
/// - **Int**: 64-bit signed integers (`123`, `-42`)
/// - **Float**: 64-bit floating point numbers (`3.14`, `-0.5`)
/// - **Hex**: RGB color values (`#ff0000`, `#00ff00`)
///
/// # Usage in Templates
///
/// Primitives are written directly in template expressions:
///
/// ```text
/// text [bold: true, size: 14, color: #ff0000] 'A'
/// ```
///
/// In this example:
/// - `true` is a boolean primitive
/// - `14` is an integer primitive
/// - `#ff0000` is a hex color primitive
/// - `'A'` is a character primitive
///
/// # Evaluation
///
/// Primitives are evaluated at compile time during template parsing. They are
/// stored directly in the blueprint and don't require runtime evaluation, making
/// them very efficient.
///
/// # Display
///
/// Primitives implement `Display` for debugging and error messages. The display
/// format matches their template syntax where possible.
///
/// # Conversions
///
/// Primitives can be created from standard Rust types using `From` implementations:
///
/// ```rust
/// use anathema_core::templates::Primitive;
///
/// let bool_prim: Primitive = true.into();
/// let int_prim: Primitive = 42i64.into();
/// let float_prim: Primitive = 3.14f64.into();
/// let char_prim: Primitive = 'x'.into();
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Primitive {
    /// A boolean value: `true` or `false`
    Bool(bool),
    /// A single Unicode character: `'a'`, `'🎉'`
    Char(char),
    /// A 64-bit signed integer: `123`, `-42`
    Int(i64),
    /// A 64-bit floating point number: `3.14`, `-0.5`
    Float(f64),
    /// An RGB color value: `#ff0000`, `#00ff00`
    Hex(Hex),
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(v) => write!(f, "{v}"),
            Self::Char(v) => write!(f, "{v}"),
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Hex(Hex { r, g, b }) => write!(f, "r: {r}, g: {g}, b: {b}"),
        }
    }
}

macro_rules! from_value {
    ($from_type:tt, $variant:ident) => {
        impl From<$from_type> for Primitive {
            fn from(value: $from_type) -> Self {
                Self::$variant(value)
            }
        }
    };
}

from_value!(f64, Float);
from_value!(i64, Int);
from_value!(bool, Bool);
from_value!(char, Char);
from_value!(Hex, Hex);

impl From<(u8, u8, u8)> for Primitive {
    fn from(value: (u8, u8, u8)) -> Self {
        let (r, g, b) = value;
        Self::Hex(Hex { r, g, b })
    }
}
