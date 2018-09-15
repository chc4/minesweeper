use std::io;
use std::io::BufRead;
use std::io::Write;
use std::error::Error;

extern crate rand;
use rand::prelude::*;

#[derive(Clone)]
enum Slot {
    Mine(bool), // tripped
    Empty(bool, i32), // visible
}

fn print_slot(slot: &Slot) -> String {
    match *slot {
        Slot::Mine(true) => "!".to_string(),
        Slot::Mine(false) => "?".to_string(),
        Slot::Empty(true, count) => count.to_string(),
        Slot::Empty(false, _) => "?".to_string()
    }
}

fn read_input<T>(message: &'static str) -> Result<T,Box<Error>>
    where T: std::str::FromStr,
          <T as std::str::FromStr>::Err: std::fmt::Debug
{
    // all of this is really nasty - i'd use an actual library for input
    let mut stdout = std::io::stdout();
    let mut stdin = std::io::stdin();
    stdout.write(message.to_string().as_bytes())?;
    stdout.flush()?;
    let mut buff = String::new();
    stdin.lock().read_line(&mut buff)?;
    Ok(buff.trim().parse::<T>().expect("Couldn't parse your input!"))
}

struct Game {
    board: Vec<Vec<Slot>>,
    done: bool
}

#[derive(Clone,Copy)]
struct Pos(i32, i32);

static NEIGHBORS: &[(i32,i32)] =
    &[
        (-1,1),  (0,1),  (1,1),
        (-1,0),          (1,0),
        (-1,-1), (0,-1), (1,-1)
    ];

impl Game {
    fn new(x: usize, y: usize) -> Game {
        let mut board = Vec::with_capacity(y);
        for i in 0..y {
            let mut row = Vec::with_capacity(x);
            for j in 0..x {
                row.push(Slot::Empty(false, 0));
            }
            board.push(row);
        }
        Game {
            board: board,
            done: false
        }
    }

    fn is_mine(&self, pos: Pos) -> bool {
        if pos.1 < 0 || pos.1 as usize >= self.board.len() {
            return false;
        } else if pos.0 < 0 || pos.0 as usize >= self.board[0].len() {
            return false;
        }

        if let Slot::Mine(_) = self.get(pos) {
            return true;
        }
        false
    }

    fn populate(&mut self, safe: Pos, difficulty: f32) {
        self.add_mines(safe, difficulty);
        for y in 0..self.board.len() {
            for x in 0..self.board[0].len() {
                self.board[y][x] = self.count(Pos(x as i32,y as i32));
            }
        }
    }

    fn add_mines(&mut self, safe: Pos, difficulty: f32) {
        for (y,row) in self.board.iter_mut().enumerate() {
            for (x,entry) in row.iter_mut().enumerate() {
                if safe.0 != (x as i32) && safe.1 != (y as i32)
                   && random::<f32>() < difficulty {
                    *entry = Slot::Mine(false);
                }
            }
        }
    }

    fn get(&self, pos: Pos) -> &Slot {
        &self.board[pos.1 as usize][pos.0 as usize]
    }

    fn count(&self, pos: Pos) -> Slot {
        let slot = self.get(pos.clone());
        if let Slot::Empty(visible,count) = slot {
            let mut new_count = 0;
            for offset in NEIGHBORS {
                if self.is_mine(Pos(pos.0 + offset.0,pos.1 + offset.1)) {
                        new_count = new_count + 1;
                }
            }
            return Slot::Empty(*visible,new_count);
        }
        return slot.clone();
    }

    fn still_mines(&mut self) -> bool {
        let mut any_left = false;
        for row in self.board.iter() {
            for entry in row {
                if let Slot::Mine(false) = entry {
                    any_left = true;
                }
            }
        }

        any_left
    }

    fn reveal(&mut self, pos: Pos) {
        // do flood-fill reveal of 0 neighbor slots
        if pos.1 < 0 || pos.1 as usize >= self.board.len() {
            return;
        } else if pos.0 < 0 || pos.0 as usize >= self.board[0].len() {
            return;
        }
        if let &Slot::Empty(false, count) = self.get(pos) {
            self.board[pos.1 as usize][pos.0 as usize] = Slot::Empty(true, count);
            if count != 0 {
                return;
            }
            self.reveal(Pos(pos.0 - 1,pos.1));
            self.reveal(Pos(pos.0 + 1,pos.1));
            self.reveal(Pos(pos.0,pos.1 - 1));
            self.reveal(Pos(pos.0,pos.1 + 1));
        }
    }

    fn print(&mut self) {
        {
            let width = self.board[0].len();
            print!("   ");
            for i in 0..width {
                if i % 5 == 0 {
                    print!("{}", i);
                } else {
                    print!(" ");
                }
            }
        }
        print!("\n");

        for (num, row) in self.board.iter().enumerate() {
            if (num) % 5 == 0 {
                print!("{:2} ", num);
            } else {
                print!("   ");
            }

            for entry in row {
                print!("{}",print_slot(&entry));
            }
            print!("\n");
        }

    }
}

fn main() -> Result<(),Box<std::error::Error>> {
    let stdin = std::io::stdin();

    let width: usize = read_input("Enter board width: ")?;
    let height: usize = read_input("Enter board height: ")?;

    let mut game = Game::new(width, height);

    let difficulty: f32 = read_input("Enter mine density (eg 0.10): ")?;

    game.print();

    let mut started = false;
    while !game.done {
        let x: usize = read_input("Enter your move's x-coord: ")?;
        let y: usize = read_input("Enter your move's y-coord: ")?;
        let pos = Pos(x as i32,y as i32);
        if !started {
            started = true;
            game.populate(pos,difficulty);
        }
        let picked = game.get(pos.clone());
        let mut blown = false;
        game.board[y][x] = match picked.clone() {
            Slot::Mine(true) => panic!("How did you blow up twice?"),
            Slot::Mine(false) => {
                game.done = true;
                blown = true;
                Slot::Mine(true)
            },
            slot @ Slot::Empty(true, _) => {
                println!("That slot was already revealed!");
                slot
            },
            Slot::Empty(false, count) => {
                if count == 0 {
                    game.reveal(pos);
                }
                Slot::Empty(true, count)
            }
        };

        if !game.still_mines() {
            println!("\n\nCongratulations! You won!");
            game.done = true;
        } else if blown {
            println!("\n\nOh no! You blew up!");
        }

        game.print();
    }

    Ok(())
}













