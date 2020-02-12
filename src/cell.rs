use std::fmt;
use termion::color::{
    Blue, Cyan, Fg, Green, LightCyan, LightGreen, LightMagenta, LightRed, LightYellow, Magenta,
    Red, Reset, Yellow,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell {
    pub value: u32,
}

impl Cell {
    /// Creates a new empty cell
    pub fn new() -> Self {
        Cell { value: 0 }
    }

    /// Determines the cell's value and returns the escape sequence for the appropriate value.
    pub fn color(self) -> String {
        match self.value {
            8 => format!("{}", Fg(Blue)),
            16 => format!("{}", Fg(Yellow)),
            32 => format!("{}", Fg(Magenta)),
            64 => format!("{}", Fg(Red)),
            128 => format!("{}", Fg(Green)),
            256 => format!("{}", Fg(LightMagenta)),
            512 => format!("{}", Fg(LightYellow)),
            1024 => format!("{}", Fg(Cyan)),
            2048 => format!("{}", Fg(LightRed)),
            4096 => format!("{}", Fg(LightGreen)),
            8192 => format!("{}", Fg(LightCyan)),
            _ => String::from(""),
        }
    }

    /// Adds a value to the cell
    pub fn move_from(&mut self, other: Cell) {
        self.value += other.value;
    }

    /// Empties the cell
    pub fn clear(&mut self) {
        self.value = 0;
    }

    /// Checks whether the cell is empty
    pub fn is_empty(self) -> bool {
        self.value == 0
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value == 0 {
            write!(f, "│            │")
        } else {
            write!(
                f,
                "│{color}{:^12}{reset}│",
                self.value,
                color = self.color(),
                reset = Fg(Reset)
            )
        }
    }
}

impl PartialEq<u32> for Cell {
    fn eq(&self, other: &u32) -> bool {
        self.value == *other
    }
}
