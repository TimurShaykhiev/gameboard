use termion::{clear, cursor, style};

use crate::chars;

pub struct Board {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    cell_size: u16,
    cell_borders: bool,
}

impl Board {
    pub fn new(width: u16, height: u16, cell_size: u16, cell_borders: bool) -> Board {
        let mut w_borders = 2;
        let mut h_borders = 2;
        if cell_borders {
            w_borders += width - 1;
            h_borders += height - 1;
        }
        let w = width * cell_size + w_borders;
        let h = height * cell_size + h_borders;

        Board {
            x: 1,
            y: 1,
            width: w,
            height: h,
            cell_size,
            cell_borders,
        }
    }

    pub(crate) fn get_width(&self) -> u16 {
        self.width
    }

    pub(crate) fn get_height(&self) -> u16 {
        self.height
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub(crate) fn get_content(&self) -> String {
        let mut y = self.y;
        // Add 16 chars to row width for 'goto' sequences
        let mut s = String::with_capacity((self.width as usize + 16) * self.height as usize);

        s.push_str(&format!(
            "{}{}",
            cursor::Goto(self.x, y),
            chars::DOUBLE_BORDER_TOP_LEFT
        ));
        for _ in 1..self.width - 1 {
            s.push_str(&format!("{}", chars::DOUBLE_BORDER_HOR_LINE));
        }
        y += 1;
        s.push_str(&format!(
            "{}{}",
            chars::DOUBLE_BORDER_TOP_RIGHT,
            cursor::Goto(self.x, y)
        ));

        for _ in 1..self.height - 1 {
            s.push_str(&format!("{}", chars::DOUBLE_BORDER_VERT_LINE));
            for _ in 1..self.width - 1 {
                s.push_str("x");
            }
            y += 1;
            s.push_str(&format!(
                "{}{}",
                chars::DOUBLE_BORDER_VERT_LINE,
                cursor::Goto(self.x, y)
            ));
        }

        s.push_str(&format!("{}", chars::DOUBLE_BORDER_BOTTOM_LEFT));
        for _ in 1..self.width - 1 {
            s.push_str(&format!("{}", chars::DOUBLE_BORDER_HOR_LINE));
        }
        s.push_str(&format!("{}", chars::DOUBLE_BORDER_BOTTOM_RIGHT));
        s
    }
}
