use std::io::{self, Read, Write};
use std::cell::RefCell;
use std::rc::Rc;

use termion::event::Key;
use termion::color;
use rand::{thread_rng, Rng};
use rand::distributions::Uniform;

use gameboard::{Board, Info, InfoLayout, Game, InputListener, Cell, Cursor, Position, CellUpdates};

const FIELD_WIDTH: usize = 50;
const FIELD_HEIGHT: usize = 20;
const START_POSITION: Position = Position(FIELD_WIDTH / 2, FIELD_HEIGHT / 2);
const BOMB_TOTAL: usize = 100;

const CELL_MASK_BOMB: u8 = 0x1;
const CELL_MASK_FLAG: u8 = 0x2;
const CELL_MASK_OPEN: u8 = 0x4;

const MINE: char = '*';
const FLAG: char = 'F';
const CONCEALED: char = '▒';

const TEXT_WIN: &'static str = "You WIN";
const TEXT_LOSE: &'static str = "You LOSE";
const TEXT_BOMBS_LEFT: &'static str = "Bombs left";
const TEXT_KEYS: &'static str = "Move: asdw/arrows. Open: j. Flag: i. Exit: q.";
const TEXT_REPLAY: &'static str = "Press r to replay. Press q to exit game.";

#[derive(PartialEq, Eq)]
enum GameResult {
    Unknown = 0,
    Win,
    Lose,
}

struct App {
    cursor_position: Position,
    board: [u8; FIELD_WIDTH * FIELD_HEIGHT],
    result: GameResult,
    exit: bool,
    concealed: usize,
    flags: usize,
}

impl<R: Read, W: Write> InputListener<R, W> for App {
    fn handle_key(&mut self, key: Key, game: &mut Game<R, W, Self>) {
        match key {
            Key::Char('q') => {
                game.stop();
                self.exit = true;
            },
            Key::Char('r') => {
                if self.result != GameResult::Unknown {
                    game.stop();
                }
            },
            Key::Char('i') => {
                if self.result == GameResult::Unknown {
                    if let Some(updates) = self.set_flag() {
                        game.update_cells(updates);

                        let bomb_left = if self.flags <= BOMB_TOTAL {
                            BOMB_TOTAL - self.flags
                        } else {
                            0
                        };
                        game.update_info(&[
                            "",
                            &format!("{:^width$}",
                                     &format!("{} {}", TEXT_BOMBS_LEFT, bomb_left),
                                     width = FIELD_WIDTH),
                            "",
                            &format!("{:^width$}", TEXT_KEYS, width = FIELD_WIDTH),
                        ]);
                    }
                }
            },
            Key::Char('j') => {
                if self.result == GameResult::Unknown {
                    if let Some(updates) = self.reveal() {
                        game.update_cells(updates);
                    }
                    if self.result != GameResult::Unknown {
                        let s = if self.result == GameResult::Win {
                            TEXT_WIN
                        } else {
                            TEXT_LOSE
                        };
                        game.update_info(&[
                            "",
                            &format!("{:^width$}", &s, width = FIELD_WIDTH),
                            "",
                            &format!("{:^width$}", TEXT_REPLAY, width = FIELD_WIDTH),
                        ]);
                    }
                }
            },
            _ => {}
        }
    }

    fn cursor_moved(&mut self, position: Position, _game: &mut Game<R, W, Self>) {
        self.cursor_position = position;
    }
}

impl App {
    fn new() -> Self {
        let mut app = App {
            cursor_position: START_POSITION,
            board: [0; FIELD_WIDTH * FIELD_HEIGHT],
            result: GameResult::Unknown,
            exit: false,
            concealed: FIELD_WIDTH * FIELD_HEIGHT,
            flags: 0,
        };
        app.setup_board();
        app
    }

    fn reset(&mut self) {
        self.cursor_position = START_POSITION;
        self.board = [0; FIELD_WIDTH * FIELD_HEIGHT];
        self.result = GameResult::Unknown;
        self.concealed = FIELD_WIDTH * FIELD_HEIGHT;
        self.flags = 0;
        self.setup_board();
    }

    fn setup_board(&mut self) {
        let range = Uniform::new(0, FIELD_WIDTH * FIELD_HEIGHT);
        let bomb_idx: Vec<usize> = thread_rng().sample_iter(&range).take(BOMB_TOTAL).collect();
        for i in bomb_idx {
            self.board[i] |= CELL_MASK_BOMB;
        }
    }

    fn set_flag(&mut self) -> Option<CellUpdates> {
        let Position(x, y) = self.cursor_position;
        if self.is_open(x, y) {
            return None
        };

        let new_cell = if self.is_flag(x, y) {
            self.flags -= 1;
            Cell::Char(CONCEALED)
        } else {
            self.flags += 1;
            Cell::Char(FLAG)
        };
        self.toggle_flag(x, y);
        let mut updates = CellUpdates::with_capacity(1);
        updates.push((new_cell, Position(x, y)));
        Some(updates)
    }

    fn reveal(&mut self) -> Option<CellUpdates> {
        let Position(x, y) = self.cursor_position;
        if self.is_open(x, y) {
            return None
        };

        let mut updates = CellUpdates::with_capacity(8);
        if self.is_bomb(x, y) {
            self.result = GameResult::Lose;
            self.show_all_bombs(&mut updates);
        } else {
            self.open_free_cells(x, y, &mut updates);
            if self.concealed == BOMB_TOTAL {
                self.result = GameResult::Win;
            }
        }
        Some(updates)
    }

    fn open_free_cells(&mut self, x: usize, y: usize, updates: &mut CellUpdates) {
        self.set_open(x, y);
        self.concealed -= 1;
        let val = self.get_bomb_num(x, y);
        if val == 0 {
            // Cell is free, no bombs around.
            updates.push((Cell::Empty, Position(x, y)));

            // Recursively open cells around until non-free cell is reached.
            for &(i, j) in self.cells_around(x, y).iter() {
                if !self.is_open(i, j) && !self.is_bomb(i, j) {
                    self.open_free_cells(i, j, updates);
                }
            }
        } else {
            // Cell isn't free. Print the bomb number.
            updates.push((Cell::Content(val.to_string()), Position(x, y)));
        }
    }

    fn show_all_bombs(&self, updates: &mut CellUpdates) {
        for h in 0..FIELD_HEIGHT {
            for w in 0..FIELD_WIDTH {
                if self.is_bomb(w, h) && !self.is_flag(w, h) {
                    updates.push((Cell::Char(MINE), Position(w, h)));
                }
            }
        }
    }

    fn get_bomb_num(&self, x: usize, y: usize) -> u8 {
        self.cells_around(x, y).iter().map(|&(i, j)| if self.is_bomb(i, j) { 1 } else { 0 }).sum()
    }

    fn cells_around(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut v = Vec::with_capacity(8);
        if y > 0 {
            // up
            v.push((x, y - 1));
            if x > 0 {
                // left up
                v.push((x - 1, y - 1));
            }
            if x < FIELD_WIDTH - 1 {
                // right up
                v.push((x + 1, y - 1));
            }
        }
        if x > 0 {
            // left
            v.push((x - 1, y));
        }
        if x < FIELD_WIDTH - 1 {
            // right
            v.push((x + 1, y));
        }
        if y < FIELD_HEIGHT - 1 {
            // down
            v.push((x, y + 1));
            if x > 0 {
                // left down
                v.push((x - 1, y + 1));
            }
            if x < FIELD_WIDTH - 1 {
                // right down
                v.push((x + 1, y + 1));
            }
        }
        v
    }

    fn is_bomb(&self, x: usize, y: usize) -> bool {
        self.board[y * FIELD_WIDTH + x] & CELL_MASK_BOMB != 0
    }

    fn is_flag(&self, x: usize, y: usize) -> bool {
        self.board[y * FIELD_WIDTH + x] & CELL_MASK_FLAG != 0
    }

    fn is_open(&self, x: usize, y: usize) -> bool {
        self.board[y * FIELD_WIDTH + x] & CELL_MASK_OPEN != 0
    }

    fn set_open(&mut self, x: usize, y: usize) {
        self.board[y * FIELD_WIDTH + x] |= CELL_MASK_OPEN;
    }

    fn toggle_flag(&mut self, x: usize, y: usize) {
        self.board[y * FIELD_WIDTH + x] ^= CELL_MASK_FLAG;
    }
}

fn main() {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let stdout = io::stdout();
    let stdout = stdout.lock();

    let app = Rc::new(RefCell::new(App::new()));
    let game = Rc::new(RefCell::new(Game::new(stdin, stdout, Rc::clone(&app))));

    while !app.borrow().exit {
        app.borrow_mut().reset();
        let cursor = Cursor::new(color::Rgb(0, 0, 255), START_POSITION, false, None);
        let mut board = Board::new(FIELD_WIDTH, FIELD_HEIGHT, 1, 1, false, None);
        let info = Info::new(6, InfoLayout::Top, &[
            "",
            &format!("{:^width$}",
                     &format!("{} {}", TEXT_BOMBS_LEFT, BOMB_TOTAL), width = FIELD_WIDTH),
            "",
            &format!("{:^width$}", TEXT_KEYS, width = FIELD_WIDTH),
        ]);
        board.init_from_str(&CONCEALED.to_string().repeat(FIELD_WIDTH * FIELD_HEIGHT),
                            Some(cursor));
        game.borrow_mut().init(board, Some(info));
        game.borrow_mut().start();
    }
}
