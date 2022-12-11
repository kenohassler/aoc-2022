use std::{cell::RefCell, collections::VecDeque, fmt, ops::DerefMut};

use itertools::Itertools;

struct Item(u64);

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct Monkey {
    items: VecDeque<Item>,
    left: String,
    right: String,
    op: String,
    divisor: u64,
    monkey_true: usize,
    monkey_false: usize,
    item_counter: u64,
}

impl Monkey {
    fn new(input: &str, idx: usize) -> Self {
        let mut lines = input.lines();
        let mut line_helper = || lines.next().unwrap().split_once(':').unwrap();

        // first line: monkey ID
        let id: usize = line_helper().0.split_once(' ').unwrap().1.parse().unwrap();
        assert_eq!(id, idx);

        // second line: items
        let items: VecDeque<Item> = line_helper()
            .1
            .split(',')
            .map(|num| Item(num.trim().parse().unwrap()))
            .collect();

        // third line: operation
        let mut op_text = line_helper().1.split_ascii_whitespace();
        assert_eq!(op_text.next().unwrap(), "new");
        assert_eq!(op_text.next().unwrap(), "=");
        let left = op_text.next().unwrap().to_owned();
        let op = op_text.next().unwrap().to_owned();
        let right = op_text.next().unwrap().to_owned();

        // fourth line: test
        let mut test_text = line_helper().1.split_ascii_whitespace();
        assert_eq!(test_text.next().unwrap(), "divisible");
        assert_eq!(test_text.next().unwrap(), "by");
        let divisor = test_text.next().unwrap().parse().unwrap();

        // fifth line: monkey_true
        let monkey_true = line_helper()
            .1
            .split_ascii_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap();

        // sixth line: monkey_false
        let monkey_false = line_helper()
            .1
            .split_ascii_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap();

        Monkey {
            items,
            left,
            right,
            op,
            divisor,
            monkey_true,
            monkey_false,
            item_counter: 0,
        }
    }

    fn operation(&self, item: &mut Item) {
        let left = match self.left.as_str() {
            "old" => item.0,
            num => num.parse().expect("Expected a number"),
        };
        let right = match self.right.as_str() {
            "old" => item.0,
            num => num.parse().expect("Expected a number"),
        };
        match self.op.as_str() {
            "*" => item.0 = left * right,
            "+" => item.0 = left + right,
            e => panic!("unsupported operation: {e}"),
        }
    }

    fn test(&self, item: &mut Item, modulus: Option<u64>) -> bool {
        // modular arithmetic limits size
        if let Some(modulus) = modulus {
            item.0 %= modulus;
        }
        // the actual test
        item.0 % self.divisor == 0
    }

    fn process_items(&mut self, all_monkeys: &[RefCell<Monkey>], modulus: Option<u64>) {
        while let Some(mut item) = self.items.pop_front() {
            // perform the monkey's calculation
            self.operation(&mut item);

            // decrease worry level (for part 1)
            if modulus.is_none() {
                item.0 /= 3u64;
            }

            // test divisibility (does modular reduction as well)
            if self.test(&mut item, modulus) {
                all_monkeys[self.monkey_true]
                    .borrow_mut()
                    .items
                    .push_back(item);
            } else {
                all_monkeys[self.monkey_false]
                    .borrow_mut()
                    .items
                    .push_back(item);
            }

            self.item_counter += 1;
        }
    }
}

impl fmt::Display for Monkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut items = self.items.iter();
        if let Some(mut last) = items.next() {
            for cur in items {
                write!(f, "{last}, ")?;
                last = cur;
            }
            write!(f, "{last}")?;
        }
        Ok(())
    }
}

fn main() {
    let example = aoc_2022::example(11);
    let input = aoc_2022::input(11);

    let example_monkeys = parse(&example);
    let counts = do_rounds(20, &example_monkeys, true);
    println!("{}", monkey_business(counts));

    let input_monkeys = parse(&input);
    let counts = do_rounds(20, &input_monkeys, true);
    println!("{}", monkey_business(counts));

    // part 2
    let example_monkeys = parse(&example);
    let counts = do_rounds(20, &example_monkeys, false);
    println!("{}", monkey_business(counts));

    let example_monkeys = parse(&example);
    let counts = do_rounds(10000, &example_monkeys, false);
    println!("{}", monkey_business(counts));

    let input_monkeys = parse(&input);
    let counts = do_rounds(10000, &input_monkeys, false);
    println!("{}", monkey_business(counts));
}

fn monkey_business(counts: Vec<u64>) -> u64 {
    let mut sorted = counts.iter().sorted_unstable().rev();
    let first = sorted.next().unwrap();
    let second = sorted.next().unwrap();

    first * second
}

fn parse(input: &str) -> Vec<RefCell<Monkey>> {
    let mut monkeys = Vec::new();
    for (idx, one_input) in input.split("\n\n").enumerate() {
        let monkey = RefCell::new(Monkey::new(one_input, idx));
        monkeys.push(monkey);
    }
    monkeys
}

/// Euclidean algorithm for gcd, used as proxy for least common multiple
fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let tmp = b;
        b = a % b;
        a = tmp;
    }
    a
}

fn do_rounds(rounds: usize, monkeys: &[RefCell<Monkey>], decreasing: bool) -> Vec<u64> {
    let gcd = monkeys
        .iter()
        .map(|m| m.borrow().divisor)
        .reduce(gcd)
        .unwrap();
    let modulus = match decreasing {
        // fun fact: the divisors are all prime, so gcd is always 1 here -.-
        true => None,
        false => Some(monkeys.iter().map(|m| m.borrow().divisor).product::<u64>() / gcd),
    };

    for _round in 0..rounds {
        for monkey in monkeys {
            monkey
                .borrow_mut()
                .deref_mut()
                .process_items(monkeys, modulus);
        }

        // pretty printing
        /*
        println!("After round {}, the monkeys are holding items with these worry levels:", round + 1);
        for (i, monkey) in monkeys.iter().enumerate() {
            println!("Monkey {i}: {}", monkey.borrow());
        }
        println!();
        */
    }

    // print inspection counter
    for (i, monkey) in monkeys.iter().enumerate() {
        println!(
            "Monkey {i} inspected items {} times.",
            monkey.borrow().item_counter
        );
    }
    monkeys.iter().map(|m| m.borrow().item_counter).collect()
}
