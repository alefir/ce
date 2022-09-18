use std::fmt;

enum Instructions {
    LoopOpen(),
    LoopClose(u8),
    Increment(),
    ShiftRight(),
}

impl fmt::Display for Instructions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            LoopOpen -> '[',
            LoopClose -> ']',
            Increment -> '+',
            ShiftRight -> '>',
        })
    }
}
