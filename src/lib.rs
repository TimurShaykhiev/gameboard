//! Gameboard is a small library to create text UI for board games (like tic-tac-toe,
//! scrabble etc.). It allows you to easily draw and update board in the terminal.
//!
//! Board must be rectangular and must contain rectangular cells. Also information board is supported.
//!
//! Library uses [termion] crate for terminal input/output.
//!
//! [termion]: https://github.com/redox-os/termion
//!

pub use board::{Board, ResourceTable, CellUpdates};
pub use cell::Cell;
pub use game::{Game, GameState, InputListener, Position};
pub use info::{Info, InfoLayout};
pub use cursor::Cursor;

pub mod board;
pub mod game;
pub mod info;
pub mod cell;
pub mod cursor;
mod chars;
mod cell_grid;
mod str_utils;
