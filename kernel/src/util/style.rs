use core::fmt;

#[derive(Copy, Clone, Debug)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Default,
}

impl From<&Color> for u8 {
    fn from(color: &Color) -> u8 {
        match color {
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::White => 37,
            Color::Default => 38,
        }
    }
}
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let byte: u8 = self.into();
        write!(f, "{}", byte)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Weight {
    Bold,
    Normal,
    Light,
}

impl From<&Weight> for u8 {
    fn from(weight: &Weight) -> u8 {
        match weight {
            Weight::Bold => 1,
            Weight::Light => 2,
            Weight::Normal => 22,
        }
    }
}

impl fmt::Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let byte: u8 = self.into();
        write!(f, "{}", byte)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Style {
    weight: Option<Weight>,
    color: Option<Color>,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[")?;
        match (self.weight, self.color) {
            (Some(weight), Some(color)) => write!(f, "{};{}m", weight, color),
            (Some(weight), None) => write!(f, "{}m", weight),
            (None, Some(color)) => write!(f, "{}m", color),
            (None, None) => write!(f, "0m"),
        }
    }
}

impl Style {
    pub fn weight(mut self, weight: Weight) -> Style {
        self.weight = Some(weight);
        self
    }

    pub fn color(mut self, color: Color) -> Style {
        self.color = Some(color);
        self
    }
}
