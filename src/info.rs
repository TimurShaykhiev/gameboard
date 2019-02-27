//! Information area.

use termion::{cursor};

use crate::chars;
use crate::game::Position;

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
    position: Position,
    width: usize,
    height: usize,
    size: usize,
    layout: InfoLayout,
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
    /// # Examples
    ///
    /// Information area is above the board. It has height 15 and width the same as a board.
    /// ```no_run
    /// let board = Board::new(5, 5, 10, 5, true, None);
    /// let info = Info::new(15, InfoLayout::Top);
    /// ```
    pub fn new(size: usize, layout: InfoLayout) -> Self {
        Info {
            position: Position(1, 1),
            width: 1,
            height: 1,
            size: size + 2, // add borders
            layout,
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

    pub(crate) fn get_updates(&self) -> Option<String> {
        None
    }
}
