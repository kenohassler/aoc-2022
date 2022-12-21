use anyhow::{Context, Result};
use std::{collections::HashSet, fmt, str::FromStr};

struct Coord {
    x: i32,
    y: i32,
}

impl Coord {
    fn new(x: i32, y: i32) -> Self {
        Coord { x, y }
    }

    fn freq(&self) -> Result<usize> {
        let x_big: usize = TryInto::<usize>::try_into(self.x)? * 4000000;
        Ok(x_big + TryInto::<usize>::try_into(self.y)?)
    }
}

impl fmt::Debug for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("").field(&self.x).field(&self.y).finish()
    }
}

impl FromStr for Coord {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s
            .split_once(", ")
            .context("expected ',' between x and y coordinate")?;
        let (_, x) = x.split_once('=').context("expected '=' after x")?;
        let (_, y) = y.split_once('=').context("expected '=' after y")?;
        Ok(Coord::new(x.parse()?, y.parse()?))
    }
}

struct Sensor {
    position: Coord,
    nearest: Coord,
}

impl Sensor {
    fn new(position: Coord, nearest: Coord) -> Self {
        Sensor { position, nearest }
    }

    /// Manhattan distance to the nearest beacon
    fn range(&self) -> u32 {
        self.position.x.abs_diff(self.nearest.x) + self.position.y.abs_diff(self.nearest.y)
    }

    /// Returns the first and the last x coordinate covered in the given line.
    fn covered_bounds(&self, line: i32) -> Option<(i32, i32)> {
        let dist = line.abs_diff(self.position.y);
        if self.range() >= dist {
            let width: i32 = (self.range() - dist).try_into().unwrap();
            let min = self.position.x - width;
            let max = self.position.x + width;
            return Some((min, max));
        }
        None
    }
}

fn main() -> Result<()> {
    let example = aoc_2022::example(15);
    let sensors = parse(&example)?;
    let cov = covered_in_line(&sensors, 10);
    println!("{cov}");
    let beacon = find_uncovered(&sensors, 20).context("beacon not found")?;
    println!("{beacon:?} => freq {}", beacon.freq()?);

    let input = aoc_2022::input(15);
    let sensors = parse(&input)?;
    let cov = covered_in_line(&sensors, 2000000);
    println!("{cov}");
    let beacon = find_uncovered(&sensors, 4000000).context("beacon not found")?;
    println!("{beacon:?} => freq {}", beacon.freq()?);

    Ok(())
}

fn find_uncovered(sensors: &[Sensor], upper: i32) -> Option<Coord> {
    for line in 0..upper + 1 {
        let mut lowest_uncovered = 0;
        for s in sensors {
            if let Some((min, max)) = s.covered_bounds(line) {
                if min <= lowest_uncovered && lowest_uncovered <= max {
                    lowest_uncovered = max + 1;
                }
            }
        }

        if lowest_uncovered <= upper {
            return Some(Coord::new(lowest_uncovered, line));
        }
    }
    None
}

fn covered_in_line(sensors: &[Sensor], line: i32) -> usize {
    let mut covered = HashSet::new();
    for sensor in sensors {
        if let Some((min, max)) = sensor.covered_bounds(line) {
            covered.extend(min..max + 1);
        }
    }
    for sensor in sensors {
        if sensor.nearest.y == line {
            covered.remove(&sensor.nearest.x);
        }
    }
    covered.len()
}

fn parse(example: &str) -> Result<Vec<Sensor>> {
    let mut sensors = Vec::new();
    for ll in example.lines() {
        let (pos, beacon) = ll
            .split_once(':')
            .context("expected ':' between sensor and beacon")?;
        let (_, pos) = pos
            .split_once("at ")
            .context("expected 'at' between text and coordinates")?;
        let (_, beacon) = beacon
            .split_once("at ")
            .context("expected 'at' between text and coordinates")?;
        sensors.push(Sensor::new(pos.parse()?, beacon.parse()?));
    }
    sensors.sort_by(|ls, rs| ls.position.x.cmp(&rs.position.x));
    Ok(sensors)
}
