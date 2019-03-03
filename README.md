# gameboard

Rust library for creating game board for text UI games. Gameboard draws board in the terminal screen, updates cells, 
handles user IO and allows you to concentrate on the game logic. It can be used for classic table games with board with
cell: tic-tac-toe, scrabble, sudoku, or for any text UI game where you can implement game field as a cell board.

Gameboard uses [termion](https://github.com/redox-os/termion) crate for terminal input/output.

## Usage
Add to your `Cargo.toml`:
```
[dependencies]
gameboard = "0.1.0"
```

Game creation:
```rust
use std::io::{self, Read, Write};
use std::cell::RefCell;
use std::rc::Rc;
use termion::event::Key;
use gameboard::{Board, Game, InputListener};

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
    let game = Rc::new(RefCell::new(Game::new(stdin, stdout, Rc::clone(&app))));
    game.borrow_mut().init(board, None);
    game.borrow_mut().start();
}
```

[Here](./examples) you can see more examples of usage.

## License
This project is licensed under the terms of the [MIT](LICENSE) license.