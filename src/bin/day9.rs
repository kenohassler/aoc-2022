use std::{collections::HashSet, fmt};

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn from_str(input: &str) -> Self {
        match input {
            "U" => Direction::Up,
            "D" => Direction::Down,
            "L" => Direction::Left,
            "R" => Direction::Right,
            e => panic!("invalid direction: {e}"),
        }
    }
}

struct Grid {
    head: Point,
    tail: Point,
    visited: HashSet<Point>,
}

impl Grid {
    fn new() -> Self {
        let head = Point { x: 0, y: 0 };
        let tail = Point { x: 0, y: 0 };
        let visited = HashSet::new();

        Grid {
            head,
            tail,
            visited,
        }
    }

    fn move_head(&mut self, instr: &str) {
        let (dir, dist) = instr.split_once(' ').unwrap();
        let dist: usize = dist.parse().unwrap();
        let dir = Direction::from_str(dir);

        for _ in 0..dist {
            match dir {
                Direction::Up => {
                    self.head.y += 1;
                    // tail.y is equal, 1 below, or 2 below.
                    if self.tail.y + 2 == self.head.y {
                        // diagonal move if tail.x != head.x
                        self.tail.y += 1;
                        self.tail.x = self.head.x;
                    }
                }
                Direction::Down => {
                    self.head.y -= 1;
                    // tail.y is equal, 1 above, or 2 above.
                    if self.tail.y - 2 == self.head.y {
                        // diagonal move if tail.x != head.x
                        self.tail.y -= 1;
                        self.tail.x = self.head.x;
                    }
                }
                Direction::Left => {
                    self.head.x -= 1;
                    // tail.x is equal, 1 to the right, or 2 to the right.
                    if self.tail.x - 2 == self.head.x {
                        // diagonal move if tail.y != head.y
                        self.tail.x -= 1;
                        self.tail.y = self.head.y;
                    }
                }
                Direction::Right => {
                    self.head.x += 1;
                    // tail.x is equal, 1 to the left, or 2 to the left.
                    if self.tail.x + 2 == self.head.x {
                        // diagonal move if tail.y != head.y
                        self.tail.x += 1;
                        self.tail.y = self.head.y;
                    }
                }
            }
            self.visited.insert(self.tail);
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "head: {}, tail: {} visited:", self.head, self.tail)?;
        for p in &self.visited {
            write!(f, "{} ", p)?;
        }
        Ok(())
    }
}

fn main() {
    let example = aoc_2022::example(9);
    parse(&example);

    let input = aoc_2022::input(9);
    parse(&input);
}

fn parse(input: &str) {
    let mut g = Grid::new();
    for ll in input.lines() {
        g.move_head(ll);
    }
    println!("{g}");
    println!("{}", g.visited.len());
}
