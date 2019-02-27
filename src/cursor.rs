//! Simple cursor implementation.
//!
//! You don't have to use this module. This implementation is very simple. Cursor handles 4 base
//! movements and marks current position with background color. If you need more sophisticated
//! cursor behavior, implement your own cursor.

use termion::color;
use termion::event::Key;

use crate::cell::Cell;
use crate::board::CellUpdates;
use crate::cell_grid::CellGrid;
use crate::game::Position;

/// Result of handling key press be cursor.
pub(crate) enum KeyHandleResult {
    /// Key not handled.
    NotHandled,
    /// Key handled, but position is not changed. No need to handle this key.
    Consumed,
    /// Cursor is moved to new position.
    NewPosition(Position)
}

/// Cursor move direction.
pub enum Direction {
    Left = 0,
    Right,
    Up,
    Down,
}

/// Cursor structure.
pub struct Cursor {
    original_cell: Cell,
    background: color::Rgb,
    position: Position,
    wrap_around: bool,
    get_direction: fn(key: Key) -> Option<Direction>,
    rows: usize,
    columns: usize,
}

impl Cursor {
    /// Creates new cursor.
    ///
    /// # Arguments
    ///
    /// `background` - background color of the cell where cursor is placed. Use `termion::color`.
    ///
    /// `position` - cursor start position.
    ///
    /// `wrap_around` - should cursor be wrapped around or not.
    ///
    /// `get_direction` - pointer to key handler function (optional). This function should
    /// translate key into cursor move direction. Function must return `None` if key is not
    /// handled. If function isn't provided the default function is used.
    /// ```
    /// fn get_direction_default(key: Key) -> Option<Direction> {
    ///     match key {
    ///         Key::Char('a') | Key::Left => Some(Direction::Left),
    ///         Key::Char('s') | Key::Down => Some(Direction::Down),
    ///         Key::Char('w') | Key::Up => Some(Direction::Up),
    ///         Key::Char('d') | Key::Right => Some(Direction::Right),
    ///         _ => None,
    ///     }
    /// }
    /// ```
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use termion::color;
    ///
    /// const START_POSITION: Position = Position(1, 1);
    /// let cursor = Cursor::new(color::Rgb(0, 0, 200), START_POSITION, true, None);
    /// ```
    pub fn new(background: color::Rgb, position: Position, wrap_around: bool,
               get_direction: Option<fn(key: Key) -> Option<Direction>>) -> Self {
        let fn_ptr = match get_direction {
            Some(ptr) => ptr,
            None => get_direction_default
        };
        Cursor {
            original_cell: Cell::Empty,
            background,
            position,
            wrap_around,
            get_direction: fn_ptr,
            rows: 0,
            columns: 0,
        }
    }

    pub(crate) fn init(&mut self, rows: usize, columns: usize, grid: &mut CellGrid) {
        self.rows = rows;
        self.columns = columns;
        self.original_cell = grid.update_cell_bg_color(self.position, self.background);
    }

    pub(crate) fn handle_key(&mut self, key: Key, grid: &mut CellGrid) -> KeyHandleResult {
        match (self.get_direction)(key) {
            Some(Direction::Left) => self.left(grid),
            Some(Direction::Right) => self.right(grid),
            Some(Direction::Up) => self.up(grid),
            Some(Direction::Down) => self.down(grid),
            None => KeyHandleResult::NotHandled
        }
    }

    pub(crate) fn check_updates(&mut self, updates: &CellUpdates, grid: &mut CellGrid) {
        for (_, pos) in updates {
            if *pos == self.position {
                // User updated the cell where cursor is placed.
                // We need to add background color for this cell.
                self.original_cell = grid.update_cell_bg_color(self.position, self.background);
                break;
            }
        }
    }

    fn left(&mut self, grid: &mut CellGrid) -> KeyHandleResult {
        let mut x = self.position.0;
        if x == 0 && !self.wrap_around {
            return KeyHandleResult::Consumed;
        } else if x == 0 && self.wrap_around {
            x = self.columns - 1;
        } else {
            x -= 1;
        };
        self.move_cursor(Position(x, self.position.1), grid)
    }

    fn right(&mut self, grid: &mut CellGrid) -> KeyHandleResult {
        let mut x = self.position.0;
        if x == self.columns - 1 && !self.wrap_around {
            return KeyHandleResult::Consumed;
        } else if x == self.columns - 1 && self.wrap_around {
            x = 0;
        } else {
            x += 1;
        };
        self.move_cursor(Position(x, self.position.1), grid)
    }

    fn up(&mut self, grid: &mut CellGrid) -> KeyHandleResult {
        let mut y = self.position.1;
        if y == 0 && !self.wrap_around {
            return KeyHandleResult::Consumed;
        } else if y == 0 && self.wrap_around {
            y = self.rows - 1;
        } else {
            y -= 1;
        };
        self.move_cursor(Position(self.position.0, y), grid)
    }

    fn down(&mut self, grid: &mut CellGrid) -> KeyHandleResult {
        let mut y = self.position.1;
        if y == self.rows - 1 && !self.wrap_around {
            return KeyHandleResult::Consumed;
        } else if y == self.rows - 1 && self.wrap_around {
            y = 0;
        } else {
            y += 1;
        };
        self.move_cursor(Position(self.position.0, y), grid)
    }

    fn move_cursor(&mut self, new_pos: Position, grid: &mut CellGrid) -> KeyHandleResult {
        // Restore original content of current cell.
        grid.update_cell(self.original_cell.clone(), self.position);
        // Move cursor to new position.
        self.position = new_pos;
        // Add bg color to new cell and get original cell from grid.
        self.original_cell = grid.update_cell_bg_color(self.position, self.background);
        KeyHandleResult::NewPosition(self.position)
    }
}

fn get_direction_default(key: Key) -> Option<Direction> {
    match key {
        Key::Char('a') | Key::Left => Some(Direction::Left),
        Key::Char('s') | Key::Down => Some(Direction::Down),
        Key::Char('w') | Key::Up => Some(Direction::Up),
        Key::Char('d') | Key::Right => Some(Direction::Right),
        _ => None,
    }
}
