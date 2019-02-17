use std::collections::HashMap;
use std::rc::Rc;

use termion::cursor;
use termion::event::Key;

use crate::game::Position;
use crate::chars;
use crate::cell::Cell;
use crate::cell_grid::CellGrid;
use crate::cursor::{Cursor, KeyHandleResult};

pub type ResourceTable = HashMap<u16, String>;
pub type CellUpdates = Vec<(Cell, Position)>;

pub struct Board {
    position: Position,
    width: usize,
    height: usize,
    rows: usize,
    columns: usize,
    cell_width: usize,
    cell_height: usize,
    cell_borders: bool,
    grid: CellGrid,
    resources: Rc<Option<ResourceTable>>,
    cursor: Option<Cursor>,
}

impl Board {
    pub fn new(width: usize, height: usize, cell_width: usize, cell_height: usize,
               cell_borders: bool, resources: Option<ResourceTable>) -> Self {
        let mut w_borders = 2;
        let mut h_borders = 2;
        if cell_borders {
            w_borders += width - 1;
            h_borders += height - 1;
        }
        let w = width * cell_width + w_borders;
        let h = height * cell_height + h_borders;

        let res_table = Rc::new(resources);
        let grid = CellGrid::new(width, height, cell_width, cell_height, Rc::clone(&res_table));

        Board {
            position: Position(1, 1),
            width: w,
            height: h,
            rows: height,
            columns: width,
            cell_width,
            cell_height,
            cell_borders,
            grid,
            resources: Rc::clone(&res_table),
            cursor: None,
        }
    }

    pub fn init_from_vec(&mut self, cells: &Vec<Cell>, cursor: Option<Cursor>) {
        if cells.len() != self.rows * self.columns {
            panic!("Invalid number of cells.");
        }
        self.grid.init_from_vec(cells);
        self.add_cursor(cursor);
    }

    pub fn init_from_str(&mut self, cells: &str, cursor: Option<Cursor>) {
        if cells.len() != self.rows * self.columns {
            panic!("Invalid number of cells.");
        }
        if self.cell_width != 1 && self.cell_height != 1 {
            panic!("You can initialize cells from string for board with 1x1 cells only.");
        }
        self.grid.init_from_str(cells);
        self.add_cursor(cursor);
    }

    fn add_cursor(&mut self, cursor: Option<Cursor>) {
        if let Some(mut cur) = cursor {
            cur.init(self.rows, self.columns, &mut self.grid);
            self.cursor = Some(cur);
        }
    }

    pub(crate) fn get_width(&self) -> usize {
        self.width
    }

    pub(crate) fn get_height(&self) -> usize {
        self.height
    }

    pub(crate) fn set_position(&mut self, pos: Position) {
        self.position = pos;
    }

    pub(crate) fn get_border(&self) -> String {
        let mut y = self.position.1 as u16;
        // Add 16 chars to row width for Goto sequences
        let mut res = String::with_capacity((self.width + 16) * self.height);

        for h in 0..self.height {
            res.push_str(&format!("{}", cursor::Goto(self.position.0 as u16, y)));
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
        if !self.grid.has_updates() {
            return None
        }

        let mut res = String::with_capacity(self.width * self.height);

        if self.grid.need_update_all() && self.cell_width == 1 && self.cell_height == 1 &&
            !self.cell_borders {
            // If we need to update all cells and board has 1x1 cells and no borders,
            // we can simplify the process.
            for cell in self.grid.iter() {
                cell.add_value_to_str(&mut res, Rc::clone(&self.resources));
            }
        } else if self.grid.need_update_all() {
            for (i, cell) in self.grid.iter().enumerate() {
                let (x, y) = self.get_cell_top_left(i);
                res.push_str(
                    &cell.get_content(self.cell_width, self.cell_height, x, y,
                                      Rc::clone(&self.resources))
                );
            }
        } else {
            for (cell, pos) in self.grid.updated_iter() {
                let (x, y) = self.get_cell_top_left(pos);
                res.push_str(
                    &cell.get_content(self.cell_width, self.cell_height, x, y,
                                      Rc::clone(&self.resources))
                );
            }
        }
        self.grid.update_complete();
        Some(res)
    }

    pub(crate) fn update_cells(&mut self, updates: CellUpdates) {
        self.grid.update_cells(&updates);
        if let Some(ref mut cursor) = self.cursor {
            cursor.check_updates(&updates, &mut self.grid)
        }
    }

    pub(crate) fn handle_key(&mut self, key: Key) -> KeyHandleResult {
        match self.cursor {
            Some(ref mut cursor) => cursor.handle_key(key, &mut self.grid),
            None => KeyHandleResult::NotHandled
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

    fn get_cell_top_left(&self, pos: usize) -> (u16, u16) {
        let start_x = self.position.0 + 1;
        let start_y = self.position.1 + 1;
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
