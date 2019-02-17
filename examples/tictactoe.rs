use std::io::{self, Read, Write};
use std::cell::RefCell;
use std::rc::Rc;

use termion::event::Key;
use termion::color;

use gameboard::{Board, ResourceTable, Cell, Game, GameState, InputListener,
                Cursor, Position};

fn create_resources() -> ResourceTable {
    let mut res = ResourceTable::new();
    res.insert(0, String::from("    OOO      O   O    O     O    O   O      OOO   "));
    res.insert(1, String::from("   X   X      X X        X        X X      X   X  "));
    res
}

struct App {
}

impl<R: Read, W: Write> InputListener<R, W> for App {
    fn handle_key(&mut self, key: Key, game: &mut Game<R, W, Self>) {
        match key {
            Key::Char('q') => game.stop(),
            Key::Char('p') => {
                let state = game.get_state();
                if state == GameState::Started {
                    game.pause(Key::Char('p'));
                } else if state == GameState::Paused {
                    game.resume();
                }
            },
            _ => {}
        }
    }

    fn cursor_moved(&mut self, _position: Position, _game: &mut Game<R, W, Self>) {
    }
}

impl App {
    fn new() -> Self {
        App {}
    }
}

fn main() {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let stdout = io::stdout();
    let stdout = stdout.lock();

    let app = Rc::new(RefCell::new(App::new()));

    let cursor = Cursor::new(color::Rgb(0, 0, 200), Position(0, 0), true, None);

    let mut board = Board::new(3, 3, 10, 5, true, Some(create_resources()));
    board.init_from_vec(&vec![Cell::ResourceId(0), Cell::ResourceId(0), Cell::Empty,
                              Cell::Empty, Cell::ResourceId(1), Cell::Empty,
                              Cell::Char('a'), Cell::Empty, Cell::ResourceId(1)],
                        Some(cursor));

    let game = Rc::new(RefCell::new(Game::new(stdin, stdout, Rc::clone(&app))));
    game.borrow_mut().init(board, None);
    game.borrow_mut().start();
}
