use std::collections::HashMap;

use termion::{cursor};

use crate::chars;
use crate::cell::Cell;

pub type ResourceTable = HashMap<u16, String>;

pub struct Board {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    rows: usize,
    columns: usize,
    cell_width: u16,
    cell_height: u16,
    cell_borders: bool,
    cells: Vec<Cell>,
    resources: Option<ResourceTable>
}

impl Board {
    pub fn new(width: u16, height: u16, cell_width: u16, cell_height: u16,
               cell_borders: bool, resources: Option<ResourceTable>) -> Board {
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
            rows: height as usize,
            columns: width as usize,
            cell_width,
            cell_height,
            cell_borders,
            cells: vec![Cell::Empty; width as usize * height as usize],
            resources
        }
    }

    pub fn reset(&mut self) {
        self.cells = vec![Cell::Empty; self.rows * self.columns];
    }

    pub fn init_from_vec(&mut self, cells: &Vec<Cell>) {
        if cells.len() != self.rows * self.columns {
            panic!("Invalid number of cells.");
        }
        self.cells = cells.clone();
    }

    pub fn init_from_str(&mut self, cells: &str) {
        if cells.len() != self.rows * self.columns {
            panic!("Invalid number of cells.");
        }
        if self.cell_width != 1 && self.cell_height != 1 {
            panic!("You can initialize cells from string for board with 1x1 cells only.");
        }
        for (i, ch) in cells.chars().enumerate() {
            self.cells[i] = Cell::Char(ch);
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
                        let pos = self.cell_pos_from_screen_pos(w, h);
                        let cell = &self.cells[pos];
                        cell.add_line_to_str(&mut s, self.cell_width as usize,
                                             self.get_cell_line_num(h),
                                             self.resources.as_ref());
                        self.cell_width
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

    // This method is applicable for lines inside cell only
    fn get_cell_line_num(&self, h: u16) -> usize {
        if self.cell_borders {
            ((h + self.cell_height) % (self.cell_height + 1)) as usize
        } else {
            ((h + self.cell_height - 1) % self.cell_height) as usize
        }
    }

    fn cell_pos(&self, x: u16, y: u16) -> usize {
        y as usize * self.columns + x as usize
    }

    // This method is applicable for screen positions inside cell only
    fn cell_pos_from_screen_pos(&self, x: u16, y: u16) -> usize {
        let cell_x: u16;
        let cell_y: u16;
        if self.cell_borders {
            cell_x = x / (self.cell_width + 1);
            cell_y = y / (self.cell_height + 1);
        } else {
            cell_x = (x - 1) / self.cell_width;
            cell_y = (y - 1) / self.cell_height;
        }
        cell_y as usize * self.columns + cell_x as usize
    }
}
