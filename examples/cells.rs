use std::io::{self, Read, Write};
use std::cell::RefCell;
use std::rc::Rc;

use termion::event::Key;
use termion::{style, color};

use gameboard::{Board, Game, InputListener, Cursor, Cell, Position, ResourceTable};

fn create_resources() -> ResourceTable {
    let mut res = ResourceTable::new();
    res.insert(0, String::from("  OO   O  O   OO  "));
    res.insert(1, String::from(" X  X   XX   X  X "));
    res
}

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

    let cursor = Cursor::new(color::Rgb(0, 0, 200), Position(0, 0), true, None);
    let mut board = Board::new(3, 3, 6, 3, true, Some(create_resources()));
    board.init_from_vec(
        &vec![
            Cell::Empty,
            Cell::ResourceId(0),
            Cell::ResourceId(1),
            Cell::Char('z'),
            Cell::Char('▒'),
            Cell::Content(
                format!("{}aaaaaaaa{}aaaaaaaaaa",
                        color::Fg(color::Red),
                        color::Fg(color::Blue))
            ),
            // this cell breaks cursor highlighting
            Cell::Content(
                format!("{}bbb{}bbbbb{}bbbb{}bbb{}bbb",
                        color::Fg(color::Red),
                        style::Bold,
                        style::Reset,
                        color::Fg(color::Blue),
                        style::Reset)
            ),
            // this cell breaks cursor highlighting
            Cell::Content(
                format!("{}cccccccccccc{}cccccc",
                        color::Bg(color::Red),
                        style::Reset)
            ),
            Cell::Content(
                format!("{}dddddddd{}dddddddddd",
                        color::Fg(color::Red),
                        style::Bold)
            )],
        Some(cursor));
    let game = Rc::new(RefCell::new(Game::new(stdin, stdout, Rc::clone(&app))));
    game.borrow_mut().init(board, None);
    game.borrow_mut().start();
}
