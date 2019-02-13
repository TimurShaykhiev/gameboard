use std::io::{self, Read, Write};
use std::cell::RefCell;
use std::rc::Rc;

use termion::event::Key;

use gameboard::{Board, Info, InfoLayout, Game, InputListener};

struct App {}

impl<R: Read, W: Write> InputListener<R, W> for App {
    fn handle_key(&mut self, key: Key, game: &mut Game<R, W, Self>) {
        match key {
            Key::Char('q') => game.stop(),
            _ => {}
        }
    }
}

fn main() {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let app = Rc::new(RefCell::new(App {}));

    let board = Board::new(5, 5, 10, 5, true, None);
    let info = Info::new(15, InfoLayout::Top);
    let game = Rc::new(RefCell::new(Game::new(stdin, stdout, Rc::clone(&app))));
    game.borrow_mut().init(board, Some(info));
    game.borrow_mut().start();
}
