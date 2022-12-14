use std::{
    cell::RefCell, collections::VecDeque, fmt, num::ParseIntError, ops::DerefMut, str::FromStr,
};

use anyhow::{anyhow, ensure, Context, Result};
use itertools::Itertools;

struct Item(u64);

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

enum Operand {
    OldValue,
    Immediate(u64),
}

impl FromStr for Operand {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "old" => Ok(Self::OldValue),
            num => Ok(Self::Immediate(num.parse()?)),
        }
    }
}

enum Operator {
    Plus,
    Times,
}

impl FromStr for Operator {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "*" => Ok(Self::Times),
            "+" => Ok(Self::Plus),
            e => Err(anyhow!("unsupported operator: {e}")),
        }
    }
}

struct Monkey {
    items: VecDeque<Item>,
    left: Operand,
    right: Operand,
    op: Operator,
    divisor: u64,
    monkey_true: usize,
    monkey_false: usize,
    item_counter: u64,
}

impl Monkey {
    fn new(input: &str, idx: usize) -> Result<Self> {
        let mut lines = input.lines();
        let mut line_helper = || {
            lines
                .next()
                .context("expected six input lines per monkey")?
                .split_once(':')
                .context("expected a colon in each input line")
        };

        // first line: monkey ID
        let id: usize = line_helper()?
            .0
            .split_once(' ')
            .context("no whitespace in monkey name")?
            .1
            .parse()?;
        ensure!(id == idx, "expected monkeys in ascending order");

        // second line: items
        let items = line_helper()?
            .1
            .split(',')
            .map(|num| num.trim().parse().map(Item))
            .collect::<Result<VecDeque<Item>, _>>()?;

        // third line: operation
        let mut op_text = line_helper()?.1.split_ascii_whitespace();
        let left = op_text.nth(2).context("missing lhs")?.parse()?;
        let op = op_text.next().context("missing op")?.parse()?;
        let right = op_text.next().context("missing rhs")?.parse()?;

        // fourth line: test
        let mut test_text = line_helper()?.1.split_ascii_whitespace();
        let divisor = test_text.nth(2).context("missing divisor")?.parse()?;

        // fifth line: monkey_true
        let monkey_true = line_helper()?
            .1
            .split_ascii_whitespace()
            .last()
            .context("no words after the colon")?
            .parse()?;

        // sixth line: monkey_false
        let monkey_false = line_helper()?
            .1
            .split_ascii_whitespace()
            .last()
            .context("no words after the colon")?
            .parse()?;

        Ok(Monkey {
            items,
            left,
            right,
            op,
            divisor,
            monkey_true,
            monkey_false,
            item_counter: 0,
        })
    }

    fn operation(&self, item: &mut Item) {
        let left = match self.left {
            Operand::OldValue => item.0,
            Operand::Immediate(num) => num,
        };
        let right = match self.right {
            Operand::OldValue => item.0,
            Operand::Immediate(num) => num,
        };
        match self.op {
            Operator::Times => item.0 = left * right,
            Operator::Plus => item.0 = left + right,
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

fn main() -> Result<()> {
    let example = aoc_2022::example(11);
    let input = aoc_2022::input(11);

    let example_monkeys = parse(&example)?;
    let counts = do_rounds(20, &example_monkeys, true);
    println!("{}", monkey_business(counts)?);

    let input_monkeys = parse(&input)?;
    let counts = do_rounds(20, &input_monkeys, true);
    println!("{}", monkey_business(counts)?);

    // part 2
    let example_monkeys = parse(&example)?;
    let counts = do_rounds(20, &example_monkeys, false);
    println!("{}", monkey_business(counts)?);

    let example_monkeys = parse(&example)?;
    let counts = do_rounds(10000, &example_monkeys, false);
    println!("{}", monkey_business(counts)?);

    let input_monkeys = parse(&input)?;
    let counts = do_rounds(10000, &input_monkeys, false);
    println!("{}", monkey_business(counts)?);
    Ok(())
}

fn monkey_business(counts: Vec<u64>) -> Result<u64> {
    ensure!(counts.len() >= 2, "need >= 2 monkeys for monkey business");
    let mut sorted = counts.iter().sorted_unstable().rev();
    let first = sorted.next().unwrap();
    let second = sorted.next().unwrap();

    Ok(first * second)
}

fn parse(input: &str) -> Result<Vec<RefCell<Monkey>>> {
    let mut monkeys = Vec::new();
    for (idx, one_input) in input.split("\n\n").enumerate() {
        let monkey = RefCell::new(Monkey::new(one_input, idx)?);
        monkeys.push(monkey);
    }
    Ok(monkeys)
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
        .unwrap_or(1);
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
