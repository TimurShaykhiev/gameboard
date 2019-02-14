use std::collections::HashMap;

use termion::{cursor};

use crate::chars;
use crate::cell::Cell;

pub type ResourceTable = HashMap<u16, String>;
pub type CellUpdates = Vec<(Cell, usize, usize)>;

pub struct Board {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    rows: usize,
    columns: usize,
    cell_width: usize,
    cell_height: usize,
    cell_borders: bool,
    cells: Vec<Cell>,
    resources: Option<ResourceTable>,
    update_all: bool,
    updates: Vec<usize>,
}

impl Board {
    pub fn new(width: usize, height: usize, cell_width: usize, cell_height: usize,
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
            rows: height,
            columns: width,
            cell_width,
            cell_height,
            cell_borders,
            cells: vec![Cell::Empty; width * height],
            resources,
            update_all: true,
            updates: vec![],
        }
    }

    pub fn reset(&mut self) {
        self.cells = vec![Cell::Empty; self.rows * self.columns];
        self.update_all = true;
    }

    pub fn init_from_vec(&mut self, cells: &Vec<Cell>) {
        if cells.len() != self.rows * self.columns {
            panic!("Invalid number of cells.");
        }
        self.cells = cells.clone();
        self.update_all = true;
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
        self.update_all = true;
    }

    pub(crate) fn get_width(&self) -> usize {
        self.width
    }

    pub(crate) fn get_height(&self) -> usize {
        self.height
    }

    pub(crate) fn set_position(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    pub(crate) fn get_border(&self) -> String {
        let mut y = self.y as u16;
        // Add 16 chars to row width for Goto sequences
        let mut res = String::with_capacity((self.width + 16) * self.height);

        for h in 0..self.height {
            res.push_str(&format!("{}", cursor::Goto(self.x as u16, y)));
            for w in 0..self.width {
                match self.get_border_char(w, h) {
                    Some(border_ch) => {
                        res.push(border_ch);
                    },
                    None => {
                        res.push(' ');
                    }
                };
            }
            y += 1;
        }
        res
    }

    pub(crate) fn get_updates(&mut self) -> Option<String> {
        if !self.update_all && self.updates.len() == 0 {
            return None
        }
        let update_all = self.update_all;
        let capacity;
        if self.update_all {
            self.update_all = false;
            capacity = self.width * self.height * 2;
        } else {
            capacity = self.updates.len() * self.cell_width * self.cell_height * 2;
        }
        let mut res = String::with_capacity(capacity);

        if update_all && self.cell_width == 1 && self.cell_height == 1 && !self.cell_borders {
            // If we need to update all cells and board has 1x1 cells and no borders,
            // we can simplify the process.
            for cell in &self.cells {
                cell.add_value_to_str(&mut res, self.resources.as_ref());
            }
        } else if update_all {
            for (i, cell) in (&self.cells).iter().enumerate() {
                let (x, y) = self.get_cell_top_left(i);
                res.push_str(
                    &cell.get_content(self.cell_width, self.cell_height, x, y,
                                      self.resources.as_ref())
                );
            }
        } else {
            for pos in &self.updates {
                let (x, y) = self.get_cell_top_left(*pos);
                let cell = &self.cells[*pos];
                res.push_str(
                    &cell.get_content(self.cell_width, self.cell_height, x, y,
                                      self.resources.as_ref())
                );
            }
        }
        self.updates = vec![];
        Some(res)
    }

    pub(crate) fn update_cells(&mut self, updates: CellUpdates) {
        for (cell, x, y) in updates {
            let pos = self.get_cell_pos(x, y);
            self.cells[pos] = cell.clone();
            self.updates.push(pos);
        }
    }

    fn get_border_char(&self, w: usize, h: usize) -> Option<char> {
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

    fn get_cell_pos(&self, x: usize, y: usize) -> usize {
        y * self.columns + x
    }

    fn get_cell_top_left(&self, pos: usize) -> (u16, u16) {
        let start_x = self.x + 1;
        let start_y = self.y + 1;
        let step_x = if self.cell_borders {
            self.cell_width + 1
        } else {
            self.cell_width
        };
        let step_y = if self.cell_borders {
            self.cell_height + 1
        } else {
            self.cell_height
        };
        let x = start_x + (pos % self.columns) * step_x;
        let y = start_y + (pos / self.rows) * step_y;
        (x as u16, y as u16)
    }
}
