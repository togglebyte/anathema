bitflags::bitflags! {
    /// Style attributes
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Attributes: u16 {
        // Turn style on
        /// Make the characters bold
        const BOLD =            0b0000_0000_0000_0001;
        /// Make the characters dim
        const DIM =             0b0000_0000_0000_0010;
        /// Make the characters italic
        const ITALIC =          0b0000_0000_0000_0100;
        /// Make the characters underlined
        const UNDERLINED =      0b0000_0000_0000_1000;
        /// Make the characters crossed out
        const CROSSED_OUT =     0b0000_0000_0001_0000;
        /// Make the characters overlined
        const OVERLINED =       0b0000_0000_0010_0000;
        /// Make the characters inverse
        const REVERSED =        0b0000_0000_0100_0000;

        // Turn style off
        /// Make the characters not bold
        const NORMAL =          0b0000_0000_1000_0000;
        /// Make the characters not dim
        const NOT_DIM =         0b0000_0001_0000_0000;
        /// Make the characters not italic
        const NOT_ITALIC =      0b0000_0010_0000_0000;
        /// Make the characters not underlined
        const NOT_UNDERLINED =  0b0000_0100_0000_0000;
        /// Make the characters not crossed out
        const NOT_CROSSED_OUT = 0b0000_1000_0000_0000;
        /// Make the characters not overlined
        const NOT_OVERLINED =   0b0001_0000_0000_0000;
        /// Make the characters not inverse
        const NOT_REVERSED =    0b0010_0000_0000_0000;
    }
}
