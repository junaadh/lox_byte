use core::fmt;

pub enum TermColor {
    NoColor,
    Black,
    Gray,
    Red,
    LightRed,
    Green,
    LightGreen,
    Brown,
    Yellow,
    Blue,
    LightBlue,
    Purple,
    LightPurple,
    Cyan,
    LightCyan,
    LightGray,
    White,
}

impl fmt::Display for TermColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoColor => write!(f, "\x1b[0m"),
            Self::Black => write!(f, "\x1b[0;30m"),
            Self::Gray => write!(f, "\x1b[1;30m"),
            Self::Red => write!(f, "\x1b[0;31m"),
            Self::LightRed => write!(f, "\x1b[1;31m"),
            Self::Green => write!(f, "\x1b[0;32m"),
            Self::LightGreen => write!(f, "\x1b[1;32m"),
            Self::Brown => write!(f, "\x1b[0;33m"),
            Self::Yellow => write!(f, "\x1b[1;33m"),
            Self::Blue => write!(f, "\x1b[0;34m"),
            Self::LightBlue => write!(f, "\x1b[1;34m"),
            Self::Purple => write!(f, "\x1b[0;35m"),
            Self::LightPurple => write!(f, "\x1b[1;35m"),
            Self::Cyan => write!(f, "\x1b[0;36m"),
            Self::LightCyan => write!(f, "\x1b[1;36m"),
            Self::LightGray => write!(f, "\x1b[0;37m"),
            Self::White => write!(f, "\x1b[1;37m"),
        }
    }
}

#[macro_export]
macro_rules! cprintln {
    ($color: ident, $($args:tt)*) => {{
        use $crate::macros::TermColor;
        println!("{}{}{}", TermColor::$color, format!($($args)*), TermColor::NoColor);
    }};
}

#[macro_export]
macro_rules! cprint {
    ($color: ident, $($args:tt)*) => {{
        use $crate::macros::TermColor;
        print!("{}{}{}", TermColor::$color, format!($($args)*), TermColor::NoColor);
    }};
}
