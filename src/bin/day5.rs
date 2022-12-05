#[derive(Clone, Debug)]
struct Cargoship {
    stacks: Vec<String>,
}

impl Cargoship {
    fn new(cargo: &str) -> Self {
        let mut stacks = Vec::new();
        let mut lines = cargo.lines().rev();

        // first line has stack numbers, use for initialization
        if let Some(ll) = lines.next() {
            for _ in ll.split_ascii_whitespace() {
                stacks.push(String::new());
            }
        }

        // push cargo onto the stacks
        for ll in lines {
            for (i, s) in stacks.iter_mut().enumerate() {
                let chr = ll.chars().nth(i * 4 + 1).unwrap();
                if chr != ' ' {
                    s.push(chr);
                }
            }
        }

        Cargoship { stacks }
    }

    fn rearrange(&mut self, orders: &str, multi_move: bool) {
        for ll in orders.lines() {
            let words: Vec<&str> = ll.split_ascii_whitespace().collect();
            let num = words[1].parse::<usize>().unwrap();
            let from = words[3].parse::<usize>().unwrap() - 1;
            let to = words[5].parse::<usize>().unwrap() - 1;

            let mut cargo = String::new();
            for _ in 0..num {
                cargo.push(self.stacks[from].pop().unwrap());
            }

            // CrateMover 9001 moves multiple crates at once (in-order)
            if multi_move {
                cargo = cargo.chars().rev().collect();
            }

            self.stacks[to].push_str(&cargo);
        }
    }

    fn tops(&self) -> String {
        let mut tops = String::with_capacity(self.stacks.len());
        for s in &self.stacks {
            tops.push(s.chars().last().expect("Expected a crate on this stack."));
        }
        tops
    }
}

fn main() {
    let example = aoc_2022::example(5);
    parse(&example);

    let input = aoc_2022::input(5);
    parse(&input);
}

fn parse(input: &str) {
    let (cargo, orders) = input
        .split_once("\n\n")
        .expect("There should be an empty line between stacks and instructions.");

    let ship = Cargoship::new(cargo);
    eprintln!("input: {ship:?}");

    let mut part1 = ship.clone();
    part1.rearrange(orders, false);
    println!("{}", part1.tops());

    let mut part2 = ship;
    part2.rearrange(orders, true);
    println!("{}", part2.tops());
}
