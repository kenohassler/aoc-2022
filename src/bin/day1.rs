use aoc_2022::{get_example, get_input};

struct Elf(Vec<i32>);

impl Elf {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add(&mut self, snack: i32) {
        self.0.push(snack);
    }

    fn calories(&self) -> i32 {
        self.0.iter().sum()
    }
}

fn get_elves(input: &str) -> Vec<Elf> {
    let mut elves = vec![Elf::new()];
    for ll in input.lines() {
        if ll.is_empty() {
            elves.push(Elf::new());
        } else {
            let cur_elf = elves.last_mut().unwrap();
            cur_elf.add(ll.parse::<i32>().unwrap())
        }
    }
    elves
}

fn max_elf(elves: &[Elf]) -> i32 {
    elves.iter().map(|elf| elf.calories()).max().unwrap()
}

fn top3_elves(elves: &[Elf]) -> i32 {
    let mut calories: Vec<i32> = elves.iter().map(|elf| elf.calories()).collect();
    calories.sort_unstable();
    calories.reverse();
    calories[0] + calories[1] + calories[2]
}

fn main() {
    let example = get_example(1);
    let elves = get_elves(&example);
    println!("{}", max_elf(&elves));
    println!("{}", top3_elves(&elves));

    let day1 = get_input(1);
    let elves = get_elves(&day1);
    println!("{}", max_elf(&elves));
    println!("{}", top3_elves(&elves));
}
