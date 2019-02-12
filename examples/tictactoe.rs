use std::io::{self, Read, Write};
use std::cell::RefCell;
use std::rc::Rc;

use termion::event::Key;

use gameboard::{Board, Game, InputListener};

struct App {
}

impl<R: Read, W: Write> InputListener<R, W> for App {
    fn handle_key(&mut self, key: Key, game: &mut Game<R, W, Self>) {
        match key {
            Key::Char('q') => game.stop(),
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

    let board = Board::new(3, 3, 10, 5, true);
    let game = Rc::new(RefCell::new(Game::new(stdin, stdout, Rc::clone(&app))));
    game.borrow_mut().init(board, None);
    game.borrow_mut().start();
}
