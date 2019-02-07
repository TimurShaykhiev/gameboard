use std::io;

use gameboard::{Board, Game, Info, InfoLayout};

fn main() {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let board = Board::new(5, 5, 10, 5, true);
    let info = Info::new(15, InfoLayout::Top);
    let mut game = Game::new(stdin, stdout);
    game.init(board, Some(info));
    game.start();
}
