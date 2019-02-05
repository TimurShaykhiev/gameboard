use termion::{cursor};

use crate::chars;

pub struct Board {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    cell_width: u16,
    cell_height: u16,
    cell_borders: bool,
}

impl Board {
    pub fn new(width: u16, height: u16,
               cell_width: u16, cell_height: u16, cell_borders: bool) -> Board {
        let mut w_borders = 2;
        let mut h_borders = 2;
        if cell_borders {
            w_borders += width - 1;
            h_borders += height - 1;
        }
        let w = width * cell_width + w_borders;
        let h = height * cell_height + h_borders;

        Board {
            x: 1,
            y: 1,
            width: w,
            height: h,
            cell_width,
            cell_height,
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

        for h in 0..self.height {
            let mut w = 0;
            s.push_str(&format!("{}", cursor::Goto(self.x, y)));
            while w < self.width {
                w += match self.get_border_char(w, h) {
                    Some(ref border_ch) => {
                        s.push_str(border_ch);
                        1
                    },
                    None => {
                        s.push_str(" ");
                        1
                    }
                };
            }
            y += 1;
        }
        s
    }

    fn get_border_char(&self, w: u16, h: u16) -> Option<&str> {
        let h_cell_border = h % (self.cell_height + 1) == 0;
        let v_cell_border = w % (self.cell_width + 1) == 0;

        if w == 0 && h == 0 {
            Some(chars::DOUBLE_BORDER_TOP_LEFT)
        } else if w == self.width - 1 && h == 0 {
            Some(chars::DOUBLE_BORDER_TOP_RIGHT)
        } else if w == 0 && h == self.height -1  {
            Some(chars::DOUBLE_BORDER_BOTTOM_LEFT)
        } else if w == self.width - 1 && h == self.height -1 {
            Some(chars::DOUBLE_BORDER_BOTTOM_RIGHT)
        } else if h == 0  {
            if self.cell_borders && v_cell_border {
                Some(chars::DOUBLE_BORDER_JOIN_UP)
            } else {
                Some(chars::DOUBLE_BORDER_HOR_LINE)
            }
        } else if h == self.height -1  {
            if self.cell_borders && v_cell_border {
                Some(chars::DOUBLE_BORDER_JOIN_DOWN)
            } else {
                Some(chars::DOUBLE_BORDER_HOR_LINE)
            }
        } else if w == 0 {
            if self.cell_borders && h_cell_border {
                Some(chars::DOUBLE_BORDER_JOIN_LEFT)
            } else {
                Some(chars::DOUBLE_BORDER_VERT_LINE)
            }
        } else if w == self.width - 1 {
            if self.cell_borders && h_cell_border {
                Some(chars::DOUBLE_BORDER_JOIN_RIGHT)
            } else {
                Some(chars::DOUBLE_BORDER_VERT_LINE)
            }
        } else if self.cell_borders {
            if h_cell_border && v_cell_border {
                Some(chars::SINGLE_BORDER_CROSS)
            } else if h_cell_border {
                Some(chars::SINGLE_BORDER_HOR_LINE)
            } else if v_cell_border {
                Some(chars::SINGLE_BORDER_VERT_LINE)
            } else {
                None
            }
        } else {
            None
        }
    }
}
