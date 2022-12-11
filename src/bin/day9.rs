use std::{collections::HashSet, fmt, fs::read_to_string};

#[derive(Eq, Hash, PartialEq, Clone)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn move_head(&mut self, dir: &Direction) {
        match dir {
            Direction::Up => self.y += 1,
            Direction::Down => self.y -= 1,
            Direction::Left => self.x -= 1,
            Direction::Right => self.x += 1,
        }
    }

    fn follow(&mut self, other: &Point) {
        let dist_x = other.x - self.x;
        let dist_y = other.y - self.y;

        if dist_x.abs() == 2 || dist_y.abs() == 2 {
            self.x += dist_x.signum();
            self.y += dist_y.signum();
        }
    }
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
    rope: Vec<Point>,
    visited: HashSet<Point>,
}

impl Grid {
    fn new(len: usize) -> Self {
        let rope = vec![Point { x: 0, y: 0 }; len];
        let visited = HashSet::new();

        Grid { rope, visited }
    }

    fn do_move(&mut self, instruction: &str) {
        let (dir, dist) = instruction.split_once(' ').unwrap();
        let dist: usize = dist.parse().unwrap();
        let dir = Direction::from_str(dir);

        for _ in 0..dist {
            self.rope.first_mut().unwrap().move_head(&dir);
            for i in 1..self.rope.len() {
                // iterate through all pairs of knots in the rope
                let (head, tail) = self.rope.split_at_mut(i);
                tail.first_mut().unwrap().follow(head.last().unwrap());
            }
            self.visited.insert(self.rope.last().unwrap().clone());
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "head --> ")?;
        for p in &self.rope {
            write!(f, "{} ", p)?;
        }
        write!(f, "<-- tail")
    }
}

fn main() {
    let example = aoc_2022::example(9);
    parse(&example, 2);
    parse(&example, 10);

    // bigger example for the second part
    let big_example = read_to_string("inputs/day9_example_big.txt").unwrap();
    parse(&big_example, 10);

    let input = aoc_2022::input(9);
    parse(&input, 2);
    parse(&input, 10);
}

fn parse(input: &str, len: usize) {
    let mut g = Grid::new(len);
    for ll in input.lines() {
        g.do_move(ll);
    }
    println!("{g}");
    println!("{}", g.visited.len());
}
