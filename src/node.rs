use colored::*;
use std::fmt;

#[rustfmt::skip]
const COLORS: &'static [Color] = &[
    Color::TrueColor { r: 76,  g: 175, b: 80  }, // #4caf50
    Color::TrueColor { r: 0,   g: 150, b: 136 }, // #009688
    Color::TrueColor { r: 33,  g: 150, b: 243 }, // #2196f3
    Color::TrueColor { r: 63,  g: 81,  b: 181 }, // #3f51b5
    Color::TrueColor { r: 103, g: 58,  b: 183 }, // #673ab7
    Color::TrueColor { r: 156, g: 39,  b: 176 }, // #9c27b0
    Color::TrueColor { r: 233, g: 30,  b: 99  }, // #e91e63
    Color::TrueColor { r: 244, g: 67,  b: 54  }, // #f44336
    Color::TrueColor { r: 255, g: 152, b: 0   }, // #ff9800
];

#[derive(Clone)]
pub struct Node {
    pub layer: u8,
    pub has_key: bool,
}

impl Default for Node {
    fn default() -> Node {
        Node {
            layer: 0,
            has_key: false,
        }
    }
}

impl Node {
    pub fn with_layer(layer: u8) -> Node {
        Node {
            layer,
            ..Node::default()
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = self.layer.to_string().color(COLORS[self.layer as usize]);

        if self.has_key {
            f.write_fmt(format_args!("{}", string.reversed()))
        } else {
            f.write_fmt(format_args!("{}", string))
        }
    }
}
