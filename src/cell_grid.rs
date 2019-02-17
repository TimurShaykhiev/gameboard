use std::rc::Rc;
use std::collections::HashSet;
use std::slice::Iter;

use termion::color;

use crate::game::Position;
use crate::board::{ResourceTable, CellUpdates};
use crate::cell::Cell;

const DEFAULT_UPDATES_CAPACITY: usize = 16;

pub(crate) struct CellGrid {
    _rows: usize,
    columns: usize,
    cell_width: usize,
    cell_height: usize,
    cells: Vec<Cell>,
    resources: Rc<Option<ResourceTable>>,
    update_all: bool,
    updates: HashSet<usize>,
}

impl CellGrid {
    pub(crate) fn new(columns: usize, rows: usize, cell_width: usize, cell_height: usize,
                      resources: Rc<Option<ResourceTable>>) -> Self {
        CellGrid {
            _rows: rows,
            columns,
            cell_width,
            cell_height,
            cells: vec![Cell::Empty; columns * rows],
            resources,
            update_all: true,
            updates: HashSet::with_capacity(DEFAULT_UPDATES_CAPACITY),
        }
    }

    pub(crate) fn init_from_vec(&mut self, cells: &Vec<Cell>) {
        self.cells = cells.clone();
        self.update_all = true;
    }

    pub(crate) fn init_from_str(&mut self, cells: &str) {
        for (i, ch) in cells.chars().enumerate() {
            self.cells[i] = Cell::Char(ch);
        }
        self.update_all = true;
    }

    pub(crate) fn has_updates(&self) -> bool {
        self.update_all || self.updates.len() > 0
    }

    pub(crate) fn need_update_all(&self) -> bool {
        self.update_all
    }

    pub(crate) fn iter(&self) -> Iter<Cell> {
        self.cells.iter()
    }

    pub(crate) fn updated_iter(&self) -> UpdatedIterator {
        UpdatedIterator {
            cells: &self.cells,
            updates: self.updates.iter().cloned().collect()
        }
    }

    pub(crate) fn update_complete(&mut self) {
        self.updates.clear();
        self.update_all = false;
    }

    pub(crate) fn update_cells(&mut self, updates: &CellUpdates) {
        for (cell, cell_pos) in updates {
            let pos = self.get_cell_pos(*cell_pos);
            self.cells[pos] = cell.clone();
            self.updates.insert(pos);
        }
    }

    // This method is for Cursor only.
    pub(crate) fn update_cell(&mut self, cell: Cell, pos: Position) {
        let pos = self.get_cell_pos(pos);
        self.cells[pos] = cell.clone();
        self.updates.insert(pos);
    }

    // This method is for Cursor only.
    pub(crate) fn update_cell_bg_color(&mut self, pos: Position, bg_color: color::Rgb) -> Cell {
        let pos = self.get_cell_pos(pos);
        let original_cell = self.cells[pos].clone();
        self.cells[pos] = original_cell.with_bg_color(self.cell_width, self.cell_height,
                                                      Rc::clone(&self.resources), bg_color);
        self.updates.insert(pos);
        original_cell
    }

    fn get_cell_pos(&self, pos: Position) -> usize {
        pos.1 * self.columns + pos.0
    }
}

pub(crate) struct UpdatedIterator<'a> {
    cells: &'a Vec<Cell>,
    updates: Vec<usize>
}

impl <'a> Iterator for UpdatedIterator<'a> {
  type Item = (&'a Cell, usize);

  fn next(&mut self) -> Option<(&'a Cell, usize)> {
      match self.updates.pop() {
          Some(idx) => Some((&self.cells[idx], idx)),
          None => None
      }
  }
}
