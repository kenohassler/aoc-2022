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

    fn do_move(&mut self, instruction: &str) {
        let (dir, dist) = instruction.split_once(' ').unwrap();
        let dist: usize = dist.parse().unwrap();
        let dir = Direction::from_str(dir);

        for _ in 0..dist {
            Grid::move_point(&mut self.head, &mut self.tail, &dir);
            self.visited.insert(self.tail);
        }
    }

    fn move_point(head: &mut Point, tail: &mut Point, dir: &Direction) {
        match dir {
            Direction::Up => {
                head.y += 1;
                // tail.y is equal, 1 below, or 2 below.
                if tail.y + 2 == head.y {
                    // diagonal move if tail.x != head.x
                    tail.y += 1;
                    tail.x = head.x;
                }
            }
            Direction::Down => {
                head.y -= 1;
                // tail.y is equal, 1 above, or 2 above.
                if tail.y - 2 == head.y {
                    // diagonal move if tail.x != head.x
                    tail.y -= 1;
                    tail.x = head.x;
                }
            }
            Direction::Left => {
                head.x -= 1;
                // tail.x is equal, 1 to the right, or 2 to the right.
                if tail.x - 2 == head.x {
                    // diagonal move if tail.y != head.y
                    tail.x -= 1;
                    tail.y = head.y;
                }
            }
            Direction::Right => {
                head.x += 1;
                // tail.x is equal, 1 to the left, or 2 to the left.
                if tail.x + 2 == head.x {
                    // diagonal move if tail.y != head.y
                    tail.x += 1;
                    tail.y = head.y;
                }
            }
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
        g.do_move(ll);
    }
    println!("{g}");
    println!("{}", g.visited.len());
}
