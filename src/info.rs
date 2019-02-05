use termion::{cursor};

use crate::chars;

#[derive(Copy, Clone)]
pub enum InfoLayout {
    Left = 0,
    Right,
    Top,
    Bottom,
}

pub struct Info {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    size: u16,
    layout: InfoLayout,
}

impl Info {
    pub fn new(size: u16, layout: InfoLayout) -> Info {
        Info {
            x: 1,
            y: 1,
            width: 1,
            height: 1,
            size: size + 2, // add borders
            layout,
        }
    }

    pub(crate) fn get_size(&self) -> u16 {
        self.size
    }

    pub(crate) fn get_layout(&self) -> InfoLayout {
        self.layout
    }

    pub(crate) fn set_position_and_size(&mut self, x: u16, y: u16, w: u16, h: u16) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
    }

    pub(crate) fn get_content(&self) -> String {
        let mut y = self.y;
        // Add 16 chars to row width for 'goto' sequences
        let mut s = String::with_capacity((self.width as usize + 16) * self.height as usize);

        s.push_str(&format!(
            "{}{}{}{}{}",
            cursor::Goto(self.x, y),
            chars::DOUBLE_BORDER_TOP_LEFT,
            chars::DOUBLE_BORDER_HOR_LINE.repeat(self.width as usize - 2),
            chars::DOUBLE_BORDER_TOP_RIGHT,
            cursor::Goto(self.x, y + 1)
        ));
        y += 1;

        for _ in 1..self.height - 1 {
            s.push_str(&format!(
                "{}{}{}{}",
                chars::DOUBLE_BORDER_VERT_LINE,
                cursor::Goto(self.x + self.width - 1, y),
                chars::DOUBLE_BORDER_VERT_LINE,
                cursor::Goto(self.x, y + 1),
            ));
            y += 1;
        }

        s.push_str(&format!(
            "{}{}{}",
            chars::DOUBLE_BORDER_BOTTOM_LEFT,
            chars::DOUBLE_BORDER_HOR_LINE.repeat(self.width as usize - 2),
            chars::DOUBLE_BORDER_BOTTOM_RIGHT
        ));
        s
    }
}
