extern crate termion;

use rand::{thread_rng, Rng};
use std::error::Error;
use std::io::stdin;
use std::io::{stdout, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor};

mod cell;
use cell::Cell;

/// Board height in cells
const BOARD_HEIGHT: usize = 4;
/// Board width in cells
const BOARD_WIDTH: usize = 4;

// TODO: Random additions, got 6 etc. Find the bug
// TODO: unable to loose; fix that

/// Represents the direction of movement
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn from_input(input: termion::event::Key) -> Option<Direction> {
        match input {
            Key::Char('w') | Key::Up => Some(Direction::Up),
            Key::Char('s') | Key::Down => Some(Direction::Down),
            Key::Char('a') | Key::Left => Some(Direction::Left),
            Key::Char('d') | Key::Right => Some(Direction::Right),
            _ => None,
        }
    }
}

enum GameState {
    Running,
    Lost,
}

impl Default for GameState {
    fn default() -> GameState {
        GameState::Running
    }
}

pub struct Game {
    rows: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
    stdout: RawTerminal<Stdout>,
    state: GameState,
}

impl Game {
    /// Creates a starting game block
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut game = Game {
            rows: [[Cell::new(); BOARD_WIDTH]; BOARD_HEIGHT],
            stdout: stdout().into_raw_mode()?,
            state: GameState::default(),
        };
        game.add_block();
        Ok(game)
    }

    /// Starts the game loop
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.draw()?;

        loop {
            let stdin = stdin();

            for c in stdin.keys() {
                match c? {
                    key if Direction::from_input(key).is_some() => {
                        self.update(Direction::from_input(key).unwrap())?
                    }
                    Key::Esc | Key::Char('q') => {
                        write!(self.stdout, "{}{}", clear::All, termion::cursor::Show)?;
                        return Ok(());
                    }
                    _ => (),
                }
            }
        }
    }

    /// Updates the game state if a cell has been moved.
    /// Moves blocks in the direction provided and generates a new block.
    pub fn update(&mut self, direction: Direction) -> Result<(), Box<dyn Error>> {
        let moved: bool = match direction {
            Direction::Up => self.move_up(),
            Direction::Down => self.move_down(),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
        };

        if moved {
            self.add_block();
            if self.no_moves_left() {
                self.state = GameState::Lost;
            }
            self.draw()?;
        }

        Ok(())
    }

    /// Draws the game board
    pub fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        write!(self.stdout, "{}{}", clear::All, termion::cursor::Hide)?;

        for (row_index, row) in self.rows.iter().enumerate() {
            let mut x: u16 = 1;
            let y: u16 = (7 * row_index as u16) + 1;

            for cell in row.iter() {
                write!(self.stdout, "{}{}", cursor::Goto(x, y), cell)?;
                x += 14;
            }
            write!(self.stdout, "\n\r")?;
        }

        let row_text = match self.state {
            GameState::Running => "Use the arrow or WASD keys to move.",
            GameState::Lost => "You Lost!",
        };
        write!(self.stdout, "\n\r{}\n\r", row_text)?;
        write!(self.stdout, "Press q or ESC to quit the game.\n\r")?;

        self.stdout.flush()?;
        Ok(())
    }

    /// Randomly adds a new value of either `2` or `4` to one of the empty cells.
    fn add_block(&mut self) {
        let mut coord_list: Vec<(usize, usize)> = Vec::new();

        for ri in 0..BOARD_HEIGHT {
            for ci in 0..BOARD_WIDTH {
                if self.rows[ri][ci].is_empty() {
                    coord_list.push((ri, ci));
                }
            }
        }

        if coord_list.is_empty() {
            return;
        }

        let mut rng = thread_rng();
        let coord_list_index = rng.gen_range(0, coord_list.len());
        let (i, j) = coord_list[coord_list_index];

        if rng.gen::<f64>() < 0.9 {
            self.rows[i][j].value = 2;
        } else {
            self.rows[i][j].value = 4;
        }
    }

    fn no_moves_left(&mut self) -> bool {
        self.rows.iter().enumerate().all(|(i, row)| {
            row.iter().enumerate().skip(1).all(|(j, cell)| {
                !cell.is_empty()
                    && cell != &self.rows[i][j - 1]
                    && (i == 0 || cell != &self.rows[i - 1][j])
            })
        })
    }

    fn can_move_into(&mut self, from: Cell, to: Cell, initial_value: u32) -> bool {
        to.is_empty() || (from == to && from == initial_value)
    }

    fn move_up(&mut self) -> bool {
        let mut moved = false;

        for j in 0..BOARD_WIDTH {
            for i in 1..BOARD_HEIGHT {
                if self.rows[i][j].is_empty() {
                    continue;
                }

                let initial_value: u32 = self.rows[i][j].value;

                for l in (0..i).rev() {
                    if self.can_move_into(self.rows[l + 1][j], self.rows[l][j], initial_value) {
                        self.rows[l][j].move_from(self.rows[l + 1][j]);
                        self.rows[l + 1][j].clear();
                        moved = true;
                    } else {
                        break;
                    }
                }
            }
        }

        moved
    }

    fn move_down(&mut self) -> bool {
        let mut moved = false;

        for j in 0..BOARD_WIDTH {
            for i in (0..BOARD_HEIGHT - 1).rev() {
                if self.rows[i][j].is_empty() {
                    continue;
                }

                let initial_value: u32 = self.rows[i][j].value;

                for l in i + 1..BOARD_HEIGHT {
                    if self.can_move_into(self.rows[l - 1][j], self.rows[l][j], initial_value) {
                        self.rows[l][j].move_from(self.rows[l - 1][j]);
                        self.rows[l - 1][j].clear();
                        moved = true;
                    } else {
                        break;
                    }
                }
            }
        }

        moved
    }

    fn move_left(&mut self) -> bool {
        let mut moved = false;

        for i in 0..BOARD_HEIGHT {
            for j in 1..BOARD_WIDTH {
                if self.rows[i][j].is_empty() {
                    continue;
                }

                let initial_value: u32 = self.rows[i][j].value;

                for l in (0..j).rev() {
                    if self.can_move_into(self.rows[i][l + 1], self.rows[i][l], initial_value) {
                        self.rows[i][l].move_from(self.rows[i][l + 1]);
                        self.rows[i][l + 1].clear();
                        moved = true;
                    } else {
                        break;
                    }
                }
            }
        }

        moved
    }

    fn move_right(&mut self) -> bool {
        let mut moved = false;

        for i in 0..BOARD_HEIGHT {
            for j in (0..BOARD_WIDTH - 1).rev() {
                if self.rows[i][j].is_empty() {
                    continue;
                }

                let initial_value: u32 = self.rows[i][j].value;

                for l in j + 1..BOARD_WIDTH {
                    if self.rows[i][l].is_empty()
                        || (self.rows[i][l] == self.rows[i][l - 1]
                            && self.rows[i][l - 1] == initial_value)
                    {
                        self.rows[i][l].move_from(self.rows[i][l - 1]);
                        self.rows[i][l - 1].clear();
                        moved = true;
                    } else {
                        break;
                    }
                }
            }
        }

        moved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_with_grid(grid_values: Vec<Vec<u32>>) -> Result<Game, Box<dyn Error>> {
        let mut game = Game::new()?;

        game.rows = build_rows(grid_values);
        Ok(game)
    }

    fn build_rows(grid_values: Vec<Vec<u32>>) -> [[Cell; BOARD_WIDTH]; BOARD_HEIGHT] {
        let mut grid = [[Cell::new(); BOARD_WIDTH]; BOARD_HEIGHT];
        for (r_i, row) in grid_values.iter().enumerate() {
            for (c_i, cell_value) in row.iter().enumerate() {
                grid[r_i][c_i].value = *cell_value;
            }
        }

        grid
    }

    #[test]
    fn it_creates_an_empty_grid_with_one_value() {
        let game = Game::new().unwrap();

        assert!(
            game.rows
                .iter()
                .flat_map(|row| row.iter().filter(|cell| cell.value != 0))
                .count()
                == 1
        );
    }

    #[test]
    fn add_block_randomly_adds_a_new_value() {
        let mut game = Game::new().unwrap();
        game.add_block();

        assert!(
            game.rows
                .iter()
                .flat_map(|row| row.iter().filter(|cell| cell.value != 0))
                .count()
                == 2
        );
    }

    #[test]
    fn add_block_with_full_grid_does_not_add_a_value() {
        let full_grid = vec![vec![8; 4]; 4];
        let mut game = setup_with_grid(full_grid).unwrap();
        game.add_block();

        assert_eq!(game.rows, [[8; 4]; 4]);
    }

    #[test]
    fn move_returns_false_if_no_move_happened() {
        let mut game = setup_with_grid(vec![
            vec![2, 4, 2, 4],
            vec![4, 2, 4, 2],
            vec![2, 4, 2, 4],
            vec![4, 2, 4, 2],
        ])
        .unwrap();

        assert!(!game.move_down());
    }

    #[test]
    fn no_moves_left_works() {
        let mut game = setup_with_grid(vec![
            vec![2, 4, 2, 4],
            vec![4, 2, 4, 2],
            vec![2, 4, 2, 4],
            vec![4, 2, 4, 2],
        ])
        .unwrap();

        assert!(game.no_moves_left())
    }

    #[test]
    fn it_moves_properly_when_there_are_blocks_to_move_to() {
        let mut game = setup_with_grid(vec![
            vec![0, 2, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![2, 0, 0, 0],
        ])
        .unwrap();
        let expected = build_rows(vec![
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![2, 2, 0, 0],
        ]);

        assert!(game.move_down());
        assert_eq!(game.rows, expected);
    }

    #[test]
    fn it_adds_cell_values_when_they_are_the_same() {
        let mut game = setup_with_grid(vec![
            vec![0, 0, 2, 2],
            vec![0, 0, 4, 2],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ])
        .unwrap();
        let expected = build_rows(vec![
            vec![0, 0, 0, 4],
            vec![0, 0, 4, 2],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ]);

        assert!(game.move_right());
        assert_eq!(game.rows, expected);
    }

    #[test]
    fn it_does_not_add_more_than_two_cell_values_and_starts_addition_from_the_end() {
        let mut game = setup_with_grid(vec![
            vec![2, 2, 2, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ])
        .unwrap();
        let expected = build_rows(vec![
            vec![4, 2, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ]);

        assert!(game.move_left());
        assert_eq!(game.rows, expected);
    }

    #[test]
    fn it_both_moves_and_adds_cell_values_in_one_move() {
        let mut game = setup_with_grid(vec![
            vec![2, 4, 0, 0],
            vec![0, 0, 8, 16],
            vec![0, 4, 0, 16],
            vec![2, 0, 8, 0],
        ])
        .unwrap();
        let expected = build_rows(vec![
            vec![4, 8, 16, 32],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ]);

        assert!(game.move_up());
        assert_eq!(game.rows, expected);
    }

    #[test]
    fn it_does_not_add_a_newly_created_cell_value_in_the_same_move() {
        let mut game = setup_with_grid(vec![
            vec![4, 0, 0, 8],
            vec![2, 0, 0, 4],
            vec![2, 0, 0, 4],
            vec![0, 0, 0, 0],
        ])
        .unwrap();
        let expected = build_rows(vec![
            vec![4, 0, 0, 8],
            vec![4, 0, 0, 8],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ]);

        assert!(game.move_up());
        assert_eq!(game.rows, expected);
    }
}
