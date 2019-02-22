use std::io::{self, Read, Write};
use std::cell::RefCell;
use std::rc::Rc;

use termion::event::Key;
use termion::color;

use gameboard::{Board, ResourceTable, Cell, Game, InputListener, Cursor, Position,
                CellUpdates};

const START_POSITION: Position = Position(1, 1);

const CELL_EMPTY: u8 = 0;
const CELL_X: u8 = 1;
const CELL_O: u8 = 2;

const TEXT_GAME_RESULT_WIN: &'static str = "|^|You win.";
const TEXT_GAME_RESULT_LOSE: &'static str = "|^|You lose.";
const TEXT_GAME_RESULT_DRAW: &'static str = "|^|Draw.";
const TEXT_REPLAY: &'static str = "|^|Press 'r' to replay.";
const TEXT_QUIT: &'static str = "|^|Press 'q' to quit.";

fn create_resources() -> ResourceTable {
    let mut res = ResourceTable::new();
    res.insert(0, String::from("    OOO      O   O    O     O    O   O      OOO   "));
    res.insert(1, String::from("   X   X      X X        X        X X      X   X  "));
    res
}

#[derive(PartialEq, Eq)]
enum GameResult {
    Unknown = 0,
    HumanWin,
    ComputerWin,
    Draw
}

struct App {
    cursor_position: Position,
    board: [u8; 9],
    turn_num: u8,
    game_over: bool,
    result: GameResult,
    exit: bool
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
                    // No need to call game.hide_message(), because after game stop
                    // board will be recreated and redrawn anyway.
                    game.stop();
                }
            },
            Key::Char('j') => {
                if let Some(updates) = self.process_user_turn() {
                    game.update_cells(updates);
                }
                if self.game_over {
                    let game_res = if self.result == GameResult::HumanWin {
                        TEXT_GAME_RESULT_WIN
                    } else if self.result == GameResult::ComputerWin {
                        TEXT_GAME_RESULT_LOSE
                    } else {
                        TEXT_GAME_RESULT_DRAW
                    };
                    game.show_message(&[
                        game_res,
                        "",
                        TEXT_REPLAY,
                        TEXT_QUIT,
                    ]);
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
        App {
            cursor_position: START_POSITION,
            board: [CELL_EMPTY; 9],
            turn_num: 0,
            game_over: false,
            result: GameResult::Unknown,
            exit: false,
        }
    }

    fn reset(&mut self) {
        self.cursor_position = START_POSITION;
        self.board = [CELL_EMPTY; 9];
        self.turn_num = 0;
        self.game_over = false;
        self.result = GameResult::Unknown;
    }

    fn process_user_turn(&mut self) -> Option<CellUpdates> {
        let Position(x, y) = self.cursor_position;
        if self.get(x, y) == CELL_EMPTY {
            // Add X to the cell. This is user's turn.
            self.set(x, y, CELL_X);
            let mut updates = CellUpdates::with_capacity(2);
            updates.push((Cell::ResourceId(1), Position(x, y)));

            if self.is_user_win() {
                self.game_over = true;
                self.result = GameResult::HumanWin;
            } else if !self.is_empty_cells() {
                self.game_over = true;
                self.result = GameResult::Draw;
            } else {
                // Computer makes turn.
                self.make_turn(&mut updates);
            }
            Some(updates)
        } else {
            None
        }
    }

    fn make_turn(&mut self, updates: &mut CellUpdates) {
        let mut new_pos = Position(1, 1); // this value will never be set
        if let Some(pos) = self.find_two_in_line(CELL_O) {
            // Check if we can win. Finish game if we can.
            new_pos = pos;
            self.game_over = true;
            self.result = GameResult::ComputerWin;
        } else if let Some(pos) = self.find_two_in_line(CELL_X) {
            // Check if user can win. Don't let user win.
            new_pos = pos;
        } else if self.get(1, 1) == CELL_EMPTY {
            // If center cell is empty, put 'O' in it.
            new_pos = Position(1, 1);
        } else if self.turn_num == 1 && self.get(1, 1) == CELL_O &&
                  ((self.get(0, 0) == CELL_X && self.get(2, 2) == CELL_X) ||
                   (self.get(2, 0) == CELL_X && self.get(0, 2) == CELL_X)) {
            // Handle special cases:
            //  ..x      x..
            //  .o.  or  .o.
            //  x..      ..x
            new_pos = Position(0, 1);
        } else if let Some(pos) = self.find_fork() {
            // Check if user can make fork. Don't let user do this.
            new_pos = pos;
        } else {
            // Put 'O' in any corner, otherwise in any cell.
            let indexes = [Position(0, 0), Position(0, 2), Position(2, 0), Position(2, 2),
                           Position(0, 1), Position(1, 0), Position(1, 2), Position(2, 1)];
            for &Position(x, y) in &indexes {
                if self.get(x, y) == CELL_EMPTY {
                    new_pos = Position(x, y);
                    break;
                }
            }
        }
        self.set(new_pos.0, new_pos.1, CELL_O);
        self.turn_num += 1;
        updates.push((Cell::ResourceId(0), new_pos));
    }

    // Find 2 X's or O's in line and return position of 3rd cell to complete the line.
    fn find_two_in_line(&self, value: u8) -> Option<Position> {
        // Check columns
        for x in 0..3 {
            let (val_num, empty_num, empty_pos) =
                self.check_line(value, [Position(x, 0), Position(x, 1), Position(x, 2)]);
            if val_num == 2 && empty_num == 1 {
                return Some(empty_pos)
            }
        }
        // Check rows
        for y in 0..3 {
            let (val_num, empty_num, empty_pos) =
                self.check_line(value, [Position(0, y), Position(1, y), Position(2, y)]);
            if val_num == 2 && empty_num == 1 {
                return Some(empty_pos)
            }
        }
        // Check diagonals
        let (val_num, empty_num, empty_pos) =
            self.check_line(value, [Position(0, 0), Position(1, 1), Position(2, 2)]);
        if val_num == 2 && empty_num == 1 {
            return Some(empty_pos)
        }
        let (val_num, empty_num, empty_pos) =
            self.check_line(value, [Position(0, 2), Position(1, 1), Position(2, 0)]);
        if val_num == 2 && empty_num == 1 {
            return Some(empty_pos)
        }
        None
    }

    fn find_fork(&self) -> Option<Position> {
        let (val_num, empty_num, _) =
            self.check_line(CELL_X, [Position(0, 0), Position(0, 1), Position(0, 2)]);
        let row0 = val_num == 1 && empty_num == 2;

        let (val_num, empty_num, _) =
            self.check_line(CELL_X, [Position(2, 0), Position(2, 1), Position(2, 2)]);
        let row2 = val_num == 1 && empty_num == 2;

        let (val_num, empty_num, _) =
            self.check_line(CELL_X, [Position(0, 0), Position(1, 0), Position(2, 0)]);
        let col0 = val_num == 1 && empty_num == 2;

        let (val_num, empty_num, _) =
            self.check_line(CELL_X, [Position(0, 2), Position(1, 2), Position(2, 2)]);
        let col2 = val_num == 1 && empty_num == 2;

        if row0 && col0 && self.get(0, 0) == CELL_EMPTY {
            Some(Position(0, 0))
        } else if row0 && col2 && self.get(0, 2) == CELL_EMPTY {
            Some(Position(0, 2))
        } else if row2 && col0 && self.get(2, 0) == CELL_EMPTY {
            Some(Position(2, 0))
        } else if row2 && col2 && self.get(2, 2) == CELL_EMPTY {
            Some(Position(2, 2))
        } else {
            None
        }
    }

    fn is_user_win(&self) -> bool {
        // Check columns
        for x in 0..3 {
            let (val_num, ..) =
                self.check_line(CELL_X, [Position(x, 0), Position(x, 1), Position(x, 2)]);
            if val_num == 3 {
                return true
            }
        }
        // Check rows
        for y in 0..3 {
            let (val_num, ..) =
                self.check_line(CELL_X, [Position(0, y), Position(1, y), Position(2, y)]);
            if val_num == 3 {
                return true
            }
        }
        // Check diagonals
        let (val_num, ..) =
            self.check_line(CELL_X, [Position(0, 0), Position(1, 1), Position(2, 2)]);
        if val_num == 3 {
            return true
        }
        let (val_num, ..) =
            self.check_line(CELL_X, [Position(0, 2), Position(1, 1), Position(2, 0)]);
        if val_num == 3 {
            return true
        }
        false
    }

    // Check line for values and return
    // (number of values, number of empty cells, empty cell position).
    // Empty cell position makes sense for 1 empty cell in line only.
    fn check_line(&self, value: u8, indexes: [Position; 3]) -> (usize, usize, Position) {
        let mut val_num = 0;
        let mut empty_num = 0;
        let mut empty_pos = Position(0, 0);
        for &Position(x, y) in &indexes {
            let v = self.get(x, y);
            if v == value {
                val_num += 1;
            } else if v == CELL_EMPTY {
                empty_num += 1;
                empty_pos = Position(x, y);
            }
        }
        (val_num, empty_num, empty_pos)
    }

    fn is_empty_cells(&self) -> bool {
        for x in 0..2 {
            for y in 0..2 {
                if self.get(x, y) == CELL_EMPTY {
                    return true;
                }
            }
        }
        return false;
    }

    fn get(&self, x: usize, y: usize) -> u8 {
        self.board[y * 3 + x]
    }

    fn set(&mut self, x: usize, y: usize, val: u8) {
        self.board[y * 3 + x] = val;
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
        let cursor = Cursor::new(color::Rgb(0, 0, 200), START_POSITION, true, None);
        let mut board = Board::new(3, 3, 10, 5, true, Some(create_resources()));
        board.init_from_vec(&vec![Cell::Empty, Cell::Empty, Cell::Empty,
                                  Cell::Empty, Cell::Empty, Cell::Empty,
                                  Cell::Empty, Cell::Empty, Cell::Empty,],
                            Some(cursor));
        game.borrow_mut().init(board, None);
        game.borrow_mut().start();
    }
}
