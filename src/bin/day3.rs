use itertools::Itertools;
use std::{fmt::Display, usize};

struct Item(u8);

impl Item {
    fn new(idx: usize) -> Self {
        assert!(idx < 52);
        Self(idx.try_into().unwrap())
    }

    fn from_ascii(chr: &u8) -> Self {
        if *chr >= 65 && *chr <= 90 {
            Self(chr - 65 + 26)
        } else if *chr >= 97 && *chr <= 122 {
            Self(chr - 97)
        } else {
            unreachable!("invalid character")
        }
    }

    fn to_ascii(&self) -> char {
        let chr;
        if self.0 < 26 {
            chr = self.0 + 97;
        } else if self.0 < 52 {
            chr = self.0 - 26 + 65;
        } else {
            unreachable!("invalid character")
        }

        char::from_u32(chr.into()).unwrap()
    }

    fn idx(&self) -> usize {
        self.0.into()
    }

    fn priority(&self) -> i32 {
        (self.0 + 1).into()
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} ({})", self.priority(), self.to_ascii()))
    }
}

struct Rucksack {
    first: [i32; 52],
    second: [i32; 52],
}

impl Rucksack {
    fn new(items: &[u8]) -> Self {
        let fold = |mut acc: [i32; 52], i: &u8| {
            let item = Item::from_ascii(i);
            acc[item.idx()] += 1;
            acc
        };
        let first = items[..items.len() / 2].iter().fold([0; 52], fold);
        let second = items[items.len() / 2..].iter().fold([0; 52], fold);
        Self { first, second }
    }

    fn find_dup(&self) -> Item {
        for i in 0..52 {
            if self.first[i] != 0 && self.second[i] != 0 {
                return Item::new(i);
            }
        }
        unreachable!("no duplicate item");
    }

    fn at(&self, idx: usize) -> i32 {
        self.first[idx] + self.second[idx]
    }
}

fn find_badge(rs1: Rucksack, rs2: Rucksack, rs3: Rucksack) -> Item {
    for i in 0..52 {
        if rs1.at(i) != 0 && rs2.at(i) != 0 && rs3.at(i) != 0 {
            return Item::new(i);
        }
    }
    unreachable!("no common item");
}

fn main() {
    let item_adder = |sum, item: &Item| sum + item.priority();
    // example
    let input = aoc_2022::example(3);
    let sum_dups = dups(&input).iter().fold(0, item_adder);
    let sum_badges = badges(&input).iter().fold(0, item_adder);
    println!("{sum_dups}");
    println!("{sum_badges}");

    // real input
    let input = aoc_2022::input(3);
    let sum_dups = dups(&input).iter().fold(0, item_adder);
    let sum_badges = badges(&input).iter().fold(0, item_adder);
    println!("{sum_dups}");
    println!("{sum_badges}");
}

fn dups(input: &str) -> Vec<Item> {
    let mut dups = Vec::<Item>::new();
    for ll in input.lines() {
        let rucksack = Rucksack::new(ll.as_bytes());
        dups.push(rucksack.find_dup());
    }
    dups
}

fn badges(input: &str) -> Vec<Item> {
    let mut badges = Vec::<Item>::new();
    for group in &input.lines().chunks(3) {
        if let Some((l1, l2, l3)) = group.collect_tuple() {
            let rs1 = Rucksack::new(l1.as_bytes());
            let rs2 = Rucksack::new(l2.as_bytes());
            let rs3 = Rucksack::new(l3.as_bytes());
            let badge = find_badge(rs1, rs2, rs3);
            badges.push(badge);
        } else {
            panic!("grouping by 3 failed")
        }
    }
    badges
}
