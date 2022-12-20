use anyhow::{Context, Result};
use std::{collections::HashSet, str::FromStr};

struct Coord {
    x: i32,
    y: i32,
}

impl Coord {
    fn new(x: i32, y: i32) -> Self {
        Coord { x, y }
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

    fn covered(&self, line: i32) -> Vec<i32> {
        let dist = line.abs_diff(self.position.y);
        let mut res = Vec::new();
        if let Some(num) = self.range().checked_sub(dist) {
            res.push(self.position.x);
            for i in 1..num + 1 {
                res.push(self.position.x.checked_sub(i.try_into().unwrap()).unwrap());
                res.push(self.position.x.checked_add(i.try_into().unwrap()).unwrap());
            }
        }
        res
    }
}

fn main() -> Result<()> {
    let example = aoc_2022::example(15);
    let sensors = parse(&example)?;
    let cov = covered_in_line(&sensors, 10);
    println!("{cov}");

    let input = aoc_2022::input(15);
    let sensors = parse(&input)?;
    let cov = covered_in_line(&sensors, 2000000);
    println!("{cov}");
    Ok(())
}

fn covered_in_line(sensors: &[Sensor], line: i32) -> usize {
    let mut covered = HashSet::new();
    for sensor in sensors {
        covered.extend(sensor.covered(line));
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
    Ok(sensors)
}
