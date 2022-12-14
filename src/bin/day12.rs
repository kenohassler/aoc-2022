use std::{collections::VecDeque, fmt};

use anyhow::{ensure, Context, Result};

#[derive(PartialEq, Clone, Copy)]
struct Coord {
    x: usize,
    y: usize,
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl fmt::Debug for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

struct Node {
    elevation: u8,
    next: Option<Coord>,
}

impl Node {
    fn new(elevation: u8) -> Self {
        Node {
            elevation,
            next: None,
        }
    }
}

struct Grid {
    rows: Vec<Vec<Node>>,
    start: Coord,
    dest: Coord,
}

impl Grid {
    fn new(input: &str) -> Result<Self> {
        let mut rows = Vec::new();
        let mut start = None;
        let mut dest = None;

        for (y, ll) in input.lines().enumerate() {
            let mut row = Vec::new();
            for (x, c) in ll.chars().enumerate() {
                match c {
                    // either lowercase, current position (S) or destination (E)
                    'S' => {
                        ensure!(start.is_none(), "only one start node allowed");
                        start = Some(Coord { x, y });
                        row.push(Node::new(1));
                    }
                    'E' => {
                        ensure!(dest.is_none(), "only one destination node allowed");
                        dest = Some(Coord { x, y });
                        row.push(Node::new(26))
                    }
                    lc => {
                        ensure!(lc.is_ascii_lowercase(), "non-allowed char");
                        let b = lc as u8 - 96;
                        row.push(Node::new(b));
                    }
                }
            }
            rows.push(row);
        }

        Ok(Grid {
            rows,
            dest: dest.context("no destination node")?,
            start: start.context("no start node")?,
        })
    }

    fn at(&self, pos: &Coord) -> &Node {
        &self.rows[pos.y][pos.x]
    }

    fn at_mut(&mut self, pos: &Coord) -> &mut Node {
        &mut self.rows[pos.y][pos.x]
    }

    fn inplace_bfs(&mut self) -> Option<Coord> {
        let mut q = VecDeque::new();
        q.push_back(self.dest);
        let mut shortest = None;

        while let Some(pos) = q.pop_front() {
            if self.at(&pos).elevation == 1 && shortest.is_none() {
                shortest = Some(pos);
            }

            for n in self.neighbours(&pos) {
                if self.at(&n).elevation + 1 >= self.at(&pos).elevation
                    && self.at(&n).next.is_none()
                {
                    self.at_mut(&n).next = Some(pos);
                    q.push_back(n);
                }
            }
        }
        shortest
    }

    fn path(&self, from: Coord) -> usize {
        let mut path = vec![from];
        let mut cur = path.last().unwrap();
        while let Some(pos) = &self.rows[cur.y][cur.x].next {
            if *pos == self.dest {
                break;
            }
            path.push(*pos);
            cur = path.last().unwrap();
        }

        println!("start --> {:?} <-- dest", path);
        path.len()
    }

    fn neighbours(&self, pos: &Coord) -> Vec<Coord> {
        let mut res = Vec::new();
        let x = pos.x;
        let y = pos.y;

        if pos.y > 0 {
            res.push(Coord { x, y: y - 1 });
        }
        if pos.y < self.rows.len() - 1 {
            res.push(Coord { x, y: y + 1 });
        }
        if pos.x > 0 {
            res.push(Coord { x: x - 1, y });
        }
        if pos.x < self.rows[pos.y].len() - 1 {
            res.push(Coord { x: x + 1, y });
        }

        res
    }
}

fn main() -> Result<()> {
    let example = aoc_2022::example(12);

    let mut grid = Grid::new(&example)?;
    let shortest = grid.inplace_bfs().context("no path found")?;
    let part1 = grid.path(grid.start);
    let part2 = grid.path(shortest);
    println!("shortest S -- E path {part1}");
    println!("shortest a -- E path {part2}");

    let input = aoc_2022::input(12);

    let mut grid = Grid::new(&input)?;
    let shortest = grid.inplace_bfs().context("no path found")?;
    let part1 = grid.path(grid.start);
    let part2 = grid.path(shortest);
    println!("shortest S -- E path {part1}");
    println!("shortest a -- E path {part2}");

    Ok(())
}
