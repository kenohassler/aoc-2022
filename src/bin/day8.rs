use std::fmt;

use itertools::Itertools;

struct Tree {
    height: u32,
    visible: bool,
    scenic_score: usize,
}

impl Tree {
    fn new(height: u32) -> Self {
        Tree {
            height,
            visible: true,
            scenic_score: 0,
        }
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.visible {
            f.write_fmt(format_args!("{}", self.height))
        } else {
            // print hidden trees in green
            f.write_fmt(format_args!("\x1b[32m{}\x1b[39m", self.height))
        }
    }
}

struct Forest(Vec<Vec<Tree>>);

impl Forest {
    fn parse_trees(input: &str) -> Self {
        let mut trees: Vec<Vec<Tree>> = Vec::new();
        for ll in input.lines() {
            trees.push(
                ll.chars()
                    .map(|c| Tree::new(c.to_digit(10).unwrap()))
                    .collect(),
            );
        }
        Forest(trees)
    }

    fn north_of(&self, i: usize, j: usize) -> impl Iterator<Item = (usize, &Tree)> {
        self.0
            .iter()
            .map(move |row| &row[j]) // tree column
            .enumerate()
            .filter(move |(idx, _)| *idx < i) // left part (north)
            .rev()
    }

    fn south_of(&self, i: usize, j: usize) -> impl Iterator<Item = (usize, &Tree)> {
        self.0
            .iter()
            .map(move |row| &row[j]) // tree column
            .enumerate()
            .filter(move |(idx, _)| *idx > i) // right part (south)
    }

    fn west_of(&self, i: usize, j: usize) -> impl Iterator<Item = (usize, &Tree)> {
        self.0[i]
            .iter() // tree row
            .enumerate()
            .filter(move |(idx, _)| *idx < j) // left part (west)
            .rev()
    }

    fn east_of(&self, i: usize, j: usize) -> impl Iterator<Item = (usize, &Tree)> {
        self.0[i]
            .iter() // tree row
            .enumerate()
            .filter(move |(idx, _)| *idx > j) // right part (east)
    }

    fn calc_visible(&mut self) {
        // iterate through inner trees (outer rows are always visible)
        for i in 1..self.0.len() - 1 {
            for j in 1..self.0[i].len() - 1 {
                let higher_tree = |(_, tree): (usize, &Tree)| tree.height >= self.0[i][j].height;

                let north_blocked = self.north_of(i, j).any(higher_tree);
                let south_blocked = self.south_of(i, j).any(higher_tree);

                let west_blocked = self.west_of(i, j).any(higher_tree);
                let east_blocked = self.east_of(i, j).any(higher_tree);

                self.0[i][j].visible =
                    !(north_blocked && south_blocked && west_blocked && east_blocked);
            }
        }
    }

    fn calc_scenic(&mut self) {
        // iterate through inner trees (outer rows are always zero)
        for i in 1..self.0.len() - 1 {
            for j in 1..self.0[i].len() - 1 {
                let higher_tree = |(_, tree): &(usize, &Tree)| tree.height >= self.0[i][j].height;

                let north_score = self
                    .north_of(i, j)
                    .find_or_last(higher_tree)
                    .map(|(idx, _)| i - idx)
                    .unwrap();

                let south_score = self
                    .south_of(i, j)
                    .find_or_last(higher_tree)
                    .map(|(idx, _)| idx - i)
                    .unwrap();

                let west_score = self
                    .west_of(i, j)
                    .find_or_last(higher_tree)
                    .map(|(idx, _)| j - idx)
                    .unwrap();

                let east_score = self
                    .east_of(i, j)
                    .find_or_last(higher_tree)
                    .map(|(idx, _)| idx - j)
                    .unwrap();

                self.0[i][j].scenic_score = north_score * south_score * west_score * east_score;
            }
        }
    }

    fn count_visible(&self) -> usize {
        self.0.iter().flatten().filter(|t| t.visible).count()
    }

    fn max_scenic(&self) -> usize {
        self.0
            .iter()
            .flatten()
            .map(|t| t.scenic_score)
            .max()
            .unwrap()
    }
}

impl fmt::Display for Forest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.0 {
            for tree in row {
                f.write_fmt(format_args!("{tree}"))?;
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

fn main() {
    let example = aoc_2022::example(8);
    let mut forest = Forest::parse_trees(&example);
    forest.calc_visible();
    // println!("{forest}");
    forest.calc_scenic();
    println!("{}", forest.count_visible());
    println!("{}", forest.max_scenic());

    let input = aoc_2022::input(8);
    let mut forest = Forest::parse_trees(&input);
    forest.calc_visible();
    // println!("{forest}");
    forest.calc_scenic();
    println!("{}", forest.count_visible());
    println!("{}", forest.max_scenic());
}
