use anyhow::{Context, Result};
use itertools::Itertools;
use std::{
    cmp::{max, min},
    fmt,
    str::FromStr,
};

struct Coord {
    x: usize,
    y: usize,
}

impl FromStr for Coord {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s
            .split_once(',')
            .context("coordinates should be comma-separated")?;
        let x = x.parse()?;
        let y = y.parse()?;
        Ok(Coord { x, y })
    }
}

#[derive(Clone)]
enum Point {
    Air,
    Rock,
    SandRest,
    SandSource,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Point::Air => ".",
            Point::Rock => "#",
            Point::SandRest => "o",
            Point::SandSource => "+",
        };
        f.write_str(c)
    }
}

#[derive(Clone)]
struct Grid(Vec<Vec<Point>>);

impl Grid {
    fn new() -> Self {
        let mut grid = Grid(vec![vec![Point::Air; 501]; 1]);
        *grid.at_mut(500, 0).unwrap() = Point::SandSource;
        grid
    }

    fn at(&self, x: usize, y: usize) -> Option<&Point> {
        match self.0.get(y) {
            None => None,
            Some(row) => row.get(x),
        }
    }

    fn at_mut(&mut self, x: usize, y: usize) -> Option<&mut Point> {
        match self.0.get_mut(y) {
            None => None,
            Some(row) => row.get_mut(x),
        }
    }

    fn add_line(&mut self, from: &Coord, to: &Coord) {
        if from.x != to.x {
            // assuming from.y == to.y here (no diagonal lines)
            let start = min(from.x, to.x);
            let end = max(from.x, to.x);
            for i in start..end + 1 {
                self.add_point(i, from.y)
            }
        } else {
            // x coords equal
            let start = min(from.y, to.y);
            let end = max(from.y, to.y);
            for i in start..end + 1 {
                self.add_point(from.x, i)
            }
        }
    }

    fn add_point(&mut self, x: usize, y: usize) {
        // extend right
        if self.0[0].len() <= x {
            let to_add = vec![Point::Air; x + 1 - self.0[0].len()];
            for row in &mut self.0 {
                row.extend(to_add.clone());
            }
        }
        // extend down
        for _ in self.0.len()..y + 1 {
            self.0.push(vec![Point::Air; self.0[0].len()]);
        }

        *self.at_mut(x, y).unwrap() = Point::Rock;
    }

    /// Simulate one unit of sand until it comes to rest.
    /// Returns the resting position, or None if the sand falls into the void.
    fn simulate_step(&self) -> Option<Coord> {
        let mut sand_pos = Coord { x: 500, y: 0 };
        loop {
            // try down
            match self.at(sand_pos.x, sand_pos.y + 1) {
                Some(Point::Air) => {
                    sand_pos.y += 1;
                }
                Some(Point::SandRest) | Some(Point::Rock) => {
                    // try down-left
                    match self.at(sand_pos.x - 1, sand_pos.y + 1) {
                        Some(Point::Air) => {
                            sand_pos.x -= 1;
                            sand_pos.y += 1;
                        }
                        Some(Point::SandRest) | Some(Point::Rock) => {
                            // try down-right
                            match self.at(sand_pos.x + 1, sand_pos.y + 1) {
                                Some(Point::Air) => {
                                    sand_pos.x += 1;
                                    sand_pos.y += 1;
                                }
                                Some(Point::SandRest) | Some(Point::Rock) => {
                                    // sand comes to rest
                                    return Some(sand_pos);
                                }
                                Some(Point::SandSource) => {
                                    unreachable!("sand source cannot be below the falling sand")
                                }
                                None => {
                                    // infinite fall, terminate
                                    return None;
                                }
                            }
                        }
                        Some(Point::SandSource) => {
                            unreachable!("sand source cannot be below the falling sand")
                        }
                        None => {
                            // infinite fall, terminate
                            return None;
                        }
                    }
                }
                Some(Point::SandSource) => {
                    unreachable!("sand source cannot be below the falling sand")
                }
                None => {
                    // infinite fall, terminate
                    return None;
                }
            }
        }
    }

    /// Find the minimum x position that is not air (used for pretty-printing).
    fn x_min(&self) -> usize {
        let start_idx = self
            .0
            .iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .find_or_last(|(_, p)| !matches!(p, Point::Air))
                    .map(|(i, _)| i)
                    .unwrap()
            })
            .min()
            .unwrap();
        start_idx
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start_idx = self.x_min();

        for row in &self.0 {
            for p in row.iter().skip(start_idx) {
                write!(f, "{p}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let example = aoc_2022::example(14);
    let g = build_grid(&example)?;
    simulate(g.clone());
    simulate_finite(g);

    let input = aoc_2022::input(14);
    let g = build_grid(&input)?;
    simulate(g.clone());
    simulate_finite(g);
    Ok(())
}

fn simulate(mut g: Grid) {
    println!("=== INITIAL GRID ===\n{g}");
    let mut count = 0;
    while let Some(sand_pos) = g.simulate_step() {
        count += 1;
        *g.at_mut(sand_pos.x, sand_pos.y).unwrap() = Point::SandRest;
        //println!("{g}");
    }
    println!("=== FINAL GRID ===\n{g}");
    println!("No. steps: {count}");
}

fn simulate_finite(mut g: Grid) {
    let ymax = g.0.len() + 1;
    let mut xmax = g.0[0].len() - 1;
    let mut xmin = g.x_min();
    g.add_line(&Coord { x: xmin, y: ymax }, &Coord { x: xmax, y: ymax });

    println!("=== INITIAL GRID ===\n{g}");
    let mut count = 0;
    loop {
        match g.simulate_step() {
            None => {
                xmax += 1;
                xmin = xmin.saturating_sub(1);
                g.add_point(xmax, ymax);
                g.add_point(xmin, ymax);
            }
            Some(Coord { x, y }) => {
                count += 1;
                *g.at_mut(x, y).unwrap() = Point::SandRest;
                //println!("{g}");
                if x == 500 && y == 0 {
                    break;
                }
            }
        }
    }
    println!("=== FINAL GRID ===\n{g}");
    println!("No. steps: {count}");
}

fn build_grid(input: &str) -> Result<Grid> {
    let mut g = Grid::new();
    for ll in input.lines() {
        let mut last = None;
        for coord in ll.split(" -> ") {
            let next = coord.parse::<Coord>()?;
            if let Some(last) = last {
                g.add_line(&last, &next);
            }
            last = Some(next);
        }
    }
    Ok(g)
}
