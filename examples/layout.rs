use std::io;

use termion::raw::IntoRawMode;

use gameboard::{Board, Game, Info, InfoLayout};

fn main() {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let stdout = stdout.into_raw_mode().unwrap();

    let board = Board::new(4, 4, 3, false);
    let info = Info::new(5, InfoLayout::Top);
    let mut game = Game::new(stdout, stdin);
    game.init(board, Some(info));
}
