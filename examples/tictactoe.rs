use std::io::{self, Read, Write};
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use termion::event::Key;
use termion::color;

use gameboard::{Board, ResourceTable, Cell, Game, GameState, InputListener};

const SIGN_O: &'static str = "    OOO      O   O    O     O    O   O      OOO   ";
const SIGN_X: &'static str = "   X   X      X X        X        X X      X   X  ";

fn create_resources() -> ResourceTable {
    let mut res = HashMap::new();
    res.insert(0, String::from(SIGN_O));
    res.insert(1, String::from(SIGN_X));
    res.insert(2, format!("{}{}", color::Fg(color::LightYellow), SIGN_O));
    res.insert(3, format!("{}{}", color::Fg(color::LightYellow), SIGN_X));
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

    let mut board = Board::new(3, 3, 10, 5, true, Some(create_resources()));
    board.init_from_vec(&vec![Cell::ResourceId(0), Cell::ResourceId(2), Cell::Empty,
                              Cell::Empty, Cell::ResourceId(1), Cell::Empty,
                              Cell::Char('a'), Cell::Empty, Cell::ResourceId(1)]);

    let game = Rc::new(RefCell::new(Game::new(stdin, stdout, Rc::clone(&app))));
    game.borrow_mut().init(board, None);
    game.borrow_mut().start();
}
