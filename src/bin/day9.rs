use std::{collections::HashSet, fmt, fs::read_to_string};

#[derive(Eq, Hash, PartialEq, Clone)]
struct Point {
    x: i32,
    y: i32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone)]
enum Direction {
    // parsable directions
    Up,
    Down,
    Left,
    Right,
    // tail-only directions
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
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
            let mut cur_dir = dir.clone();
            for i in 0..self.rope.len() {
                // iterate through all pairs of knots in the rope
                let (head, tail) = self.rope.split_at_mut(i + 1);
                match Grid::move_point(head.last_mut().unwrap(), tail.first(), &cur_dir) {
                    Some(new_dir) => cur_dir = new_dir,
                    None => break,
                }
            }
            self.visited.insert(self.rope.last().unwrap().clone());
        }
    }

    /// Move head in the given direction and calculate the move direction for tail.
    /// Returns None if no tail was given or no move is needed to catch up with head.
    fn move_point(
        head: &mut Point,
        maybe_tail: Option<&Point>,
        dir: &Direction,
    ) -> Option<Direction> {
        match dir {
            Direction::Up => {
                head.y += 1;
                if let Some(tail) = maybe_tail {
                    // tail.y is equal, 1 below, or 2 below.
                    if tail.y + 2 == head.y {
                        // diagonal move if tail.x != head.x
                        if tail.x + 1 == head.x {
                            return Some(Direction::UpRight);
                        } else if tail.x - 1 == head.x {
                            return Some(Direction::UpLeft);
                        } else if tail.x == head.x {
                            return Some(Direction::Up);
                        } else {
                            unreachable!("{head} -- {tail}");
                        }
                        // tail.y += 1;
                        // tail.x = head.x;
                    }
                }
            }
            Direction::Down => {
                head.y -= 1;
                if let Some(tail) = maybe_tail {
                    // tail.y is equal, 1 above, or 2 above.
                    if tail.y - 2 == head.y {
                        // diagonal move if tail.x != head.x
                        if tail.x + 1 == head.x {
                            return Some(Direction::DownRight);
                        } else if tail.x - 1 == head.x {
                            return Some(Direction::DownLeft);
                        } else if tail.x == head.x {
                            return Some(Direction::Down);
                        } else {
                            unreachable!("{head} -- {tail}");
                        }
                        // tail.y -= 1;
                        // tail.x = head.x;
                    }
                }
            }
            Direction::Left => {
                head.x -= 1;
                if let Some(tail) = maybe_tail {
                    // tail.x is equal, 1 to the right, or 2 to the right.
                    if tail.x - 2 == head.x {
                        // diagonal move if tail.y != head.y
                        if tail.y + 1 == head.y {
                            return Some(Direction::UpLeft);
                        } else if tail.y - 1 == head.y {
                            return Some(Direction::DownLeft);
                        } else if tail.y == head.y {
                            return Some(Direction::Left);
                        } else {
                            unreachable!("{head} -- {tail}");
                        }
                        // tail.x -= 1;
                        // tail.y = head.y;
                    }
                }
            }
            Direction::Right => {
                head.x += 1;
                if let Some(tail) = maybe_tail {
                    // tail.x is equal, 1 to the left, or 2 to the left.
                    if tail.x + 2 == head.x {
                        // diagonal move if tail.y != head.y
                        if tail.y + 1 == head.y {
                            return Some(Direction::UpRight);
                        } else if tail.y - 1 == head.y {
                            return Some(Direction::DownRight);
                        } else if tail.y == head.y {
                            return Some(Direction::Right);
                        } else {
                            unreachable!("{head} -- {tail}");
                        }
                        // tail.x += 1;
                        // tail.y = head.y;
                    }
                }
            }
            Direction::UpLeft => {
                head.x -= 1;
                head.y += 1;
                if let Some(tail) = maybe_tail {
                    // need to move if either y is 2 away or x is 2 away
                    if tail.y + 2 == head.y {
                        if tail.x == head.x {
                            // straight move
                            return Some(Direction::Up);
                        } else {
                            // diagonal move
                            return Some(Direction::UpLeft);
                        }
                    }
                    if tail.x - 2 == head.x {
                        if tail.y == head.y {
                            // straight move
                            return Some(Direction::Left);
                        } else {
                            // diagonal move
                            return Some(Direction::UpLeft);
                        }
                    }
                }
            }
            Direction::UpRight => {
                head.x += 1;
                head.y += 1;
                if let Some(tail) = maybe_tail {
                    // need to move if either y is 2 away or x is 2 away
                    if tail.y + 2 == head.y {
                        if tail.x == head.x {
                            // straight move
                            return Some(Direction::Up);
                        } else {
                            // diagonal move
                            return Some(Direction::UpRight);
                        }
                    }
                    if tail.x + 2 == head.x {
                        if tail.y == head.y {
                            // straight move
                            return Some(Direction::Right);
                        } else {
                            // diagonal move
                            return Some(Direction::UpRight);
                        }
                    }
                }
            }
            Direction::DownLeft => {
                head.x -= 1;
                head.y -= 1;
                if let Some(tail) = maybe_tail {
                    // need to move if either y is 2 away or x is 2 away
                    if tail.y - 2 == head.y {
                        if tail.x == head.x {
                            // straight move
                            return Some(Direction::Down);
                        } else {
                            // diagonal move
                            return Some(Direction::DownLeft);
                        }
                    }
                    if tail.x - 2 == head.x {
                        if tail.y == head.y {
                            // straight move
                            return Some(Direction::Left);
                        } else {
                            // diagonal move
                            return Some(Direction::DownLeft);
                        }
                    }
                }
            }
            Direction::DownRight => {
                head.x += 1;
                head.y -= 1;
                if let Some(tail) = maybe_tail {
                    // need to move if either y is 2 away or x is 2 away
                    if tail.y - 2 == head.y {
                        if tail.x == head.x {
                            // straight move
                            return Some(Direction::Down);
                        } else {
                            // diagonal move
                            return Some(Direction::DownRight);
                        }
                    }
                    if tail.x + 2 == head.x {
                        if tail.y == head.y {
                            // straight move
                            return Some(Direction::Right);
                        } else {
                            // diagonal move
                            return Some(Direction::DownRight);
                        }
                    }
                }
            }
        }
        None
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
