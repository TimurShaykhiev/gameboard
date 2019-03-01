//! Information area.

use termion::{cursor};

use crate::chars;
use crate::game::Position;
use crate::str_utils;

/// Information area layout.
#[derive(Copy, Clone)]
pub enum InfoLayout {
    /// Info area at the left from the board.
    Left = 0,
    /// Info area at the right from the board.
    Right,
    /// Info area above the board.
    Top,
    /// Info area below the board.
    Bottom,
}

/// Information area structure.
pub struct Info {
    /// Info top left position.
    position: Position,
    /// Total info width in characters (with borders).
    width: usize,
    /// Total info height in characters (with borders).
    height: usize,
    size: usize,
    layout: InfoLayout,
    lines: Vec<String>,
}

impl Info {
    /// Creates new information area.
    ///
    /// # Arguments
    ///
    /// `size` - information area size in characters. You need to set one dimension size only.
    /// Another one will be taken from the board. For example, if layout is `InfoLayout::Left` then
    /// info area will be at the left of the board and have the same height. `size` will be a
    /// width of the info area.
    ///
    /// `layout` - information area layout
    ///
    /// `lines` - information area content. A list of strings to display. If line number is more
    /// than information area height, last lines will be ignored. Too long lines will be truncated.
    /// If you want space between lines, add empty string to list.
    ///
    /// # Implementation note
    ///
    /// This crate iterates Unicode strings as a set of [grapheme clusters] to handle characters
    /// like *g̈* correctly. When we slice strings to put them inside cells or dialogs, we expect
    /// characters to have the same width. This is not always true for some Unicode symbols. Such
    /// symbols will break layout.
    ///
    /// [grapheme clusters]: http://www.unicode.org/reports/tr29/
    ///
    /// # Examples
    ///
    /// Information area is above the board. It has height 15 and width the same as a board.
    /// ```no_run
    /// let board = Board::new(5, 5, 10, 5, true, None);
    /// let info = Info::new(15, InfoLayout::Top, &[
    ///     "This is line 1.",
    ///     "",
    ///     "This is line 3.",
    ///     "This is line 4.",
    /// ]);
    /// ```
    pub fn new(size: usize, layout: InfoLayout, lines: &[&str]) -> Self {
        let mut v = Vec::with_capacity(lines.len());
        for &l in lines {
            v.push(String::from(l));
        }

        Info {
            position: Position(1, 1),
            width: 1,
            height: 1,
            size: size + 2, // add borders
            layout,
            lines: v
        }
    }

    pub(crate) fn get_size(&self) -> usize {
        self.size
    }

    pub(crate) fn get_layout(&self) -> InfoLayout {
        self.layout
    }

    pub(crate) fn set_position_and_size(&mut self, pos: Position, w: usize, h: usize) {
        self.position = pos;
        self.width = w;
        self.height = h;
    }

    pub(crate) fn get_border(&self) -> String {
        let x = self.position.0 as u16;
        let mut y = self.position.1 as u16;
        // Add 16 chars to row width for Goto sequences
        let mut res = String::with_capacity((self.width + 16) * self.height);

        res.push_str(&format!(
            "{}{}{}{}{}",
            cursor::Goto(x, y),
            chars::DOUBLE_BORDER_TOP_LEFT,
            chars::DOUBLE_BORDER_HOR_LINE.to_string().repeat(self.width - 2),
            chars::DOUBLE_BORDER_TOP_RIGHT,
            cursor::Goto(x, y + 1)
        ));
        y += 1;

        for _ in 1..self.height - 1 {
            res.push_str(&format!(
                "{}{}{}{}",
                chars::DOUBLE_BORDER_VERT_LINE,
                cursor::Goto(x + self.width as u16 - 1, y),
                chars::DOUBLE_BORDER_VERT_LINE,
                cursor::Goto(x, y + 1),
            ));
            y += 1;
        }

        res.push_str(&format!(
            "{}{}{}",
            chars::DOUBLE_BORDER_BOTTOM_LEFT,
            chars::DOUBLE_BORDER_HOR_LINE.to_string().repeat(self.width - 2),
            chars::DOUBLE_BORDER_BOTTOM_RIGHT
        ));
        res
    }

    pub(crate) fn update(&mut self, lines: &[&str]) {
        self.lines = Vec::with_capacity(lines.len());
        for &l in lines {
            self.lines.push(String::from(l));
        }
    }

    pub(crate) fn get_updates(&self) -> Option<String> {
        let line_num = self.lines.len();
        if line_num == 0 {
            return None
        }

        let x = self.position.0 as u16 + 1;
        let mut y = self.position.1 as u16 + 1;
        let text_width = self.width - 2;

        let mut res =
            String::with_capacity((self.width + str_utils::GOTO_SEQUENCE_WIDTH) * self.height);
        for i in 0..self.height - 2 {
            if i < line_num {
                let line = &self.lines[i];
                let s = if str_utils::get_str_len(line) < text_width {
                    format!("{:width$}", &line, width = text_width)
                } else {
                    str_utils::get_str_range(line, 0, text_width).to_string()
                };
                res.push_str(&format!("{}{}", cursor::Goto(x, y), s));
            } else {
                res.push_str(&format!("{}{}", cursor::Goto(x, y), " ".repeat(text_width)));
            }
            y += 1;
        }
        Some(res)
    }
}
