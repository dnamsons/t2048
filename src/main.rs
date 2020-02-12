use std::process;
use t2048::Game;

fn main() {
    let mut game = Game::new().unwrap_or_else(|err| {
        eprintln!("Problem starting the game: {}", err);
        process::exit(1);
    });

    game.run().unwrap_or_else(|err| {
        eprintln!("Error while running the game: {}", err);
        process::exit(1);
    });
}
