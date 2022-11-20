use iced::{Color, Element, Length};
use iced_native::widget::{column, scrollable, text};

use hex_colors::color_from_hex;

#[derive(Clone)]
enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

macro_rules! hex {
    ($($tt:tt)*) => {{
        let [r, g, b]: [u8; 3] = color_from_hex!($($tt)*);
        Color::from_rgb(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
        )
    }};
}

const PURPLE: Color = hex!(0x800080);
const BLUE: Color = hex!(0x0000ff);
const GREEN: Color = hex!(0x008000);
const YELLOW: Color = hex!(0xffff00);
const RED: Color = hex!(0xff0000);

impl Level {
    const fn as_color(&self) -> Color {
        match self {
            Self::Trace => PURPLE,
            Self::Debug => BLUE,
            Self::Info => GREEN,
            Self::Warn => YELLOW,
            Self::Error => RED,
        }
    }
}

#[derive(Default)]
pub struct Log(Vec<(Level, String)>);

macro_rules! log_fn {
    ($(($name:ident, $ty:expr)),* $(,)?) => {
        $(pub fn $name(&mut self, message: impl ToString) {
            self.0.push(($ty, message.to_string()))
        })*
    };
}

impl Log {
    log_fn!(
        (trace, Level::Trace),
        (debug, Level::Debug),
        (info, Level::Info),
        (warn, Level::Warn),
        (error, Level::Error),
    );

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn view(&self) -> Element<'_, !> {
        fn error_text<'a>((level, message): (Level, String)) -> Element<'a, !> {
            println!("{:?}", level.as_color());
            text(message).style(level.as_color()).into()
        }

        scrollable(
            column(self.0.iter().rev().cloned().map(error_text).collect()).width(Length::Fill),
        )
        .into()
    }
}
