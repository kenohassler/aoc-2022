use std::{cell::RefCell, collections::VecDeque, fmt, ops::DerefMut};

use num_bigint::BigUint;

struct Item(BigUint);

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct Monkey {
    items: VecDeque<Item>,
    operation: Box<dyn Fn(&mut Item)>,
    test: Box<dyn Fn(&Item) -> bool>,
    monkey_true: usize,
    monkey_false: usize,
    item_counter: u32,
}

impl Monkey {
    fn new(input: &str, idx: usize) -> Self {
        let mut lines = input.lines();
        // first line: monkey ID
        let id: usize = lines
            .next()
            .expect("id")
            .split_once(":")
            .unwrap()
            .0
            .split_once(" ")
            .unwrap()
            .1
            .parse()
            .unwrap();
        assert_eq!(id, idx);

        // second line: items
        let items: VecDeque<Item> = lines
            .next()
            .expect("items")
            .split_once(":")
            .unwrap()
            .1
            .split(",")
            .map(|num| match num.trim().parse() {
                Ok(i) => Item(i),
                Err(_) => panic!("not a number: {num}"),
            })
            .collect();

        // third line: operation
        let mut op_text = lines
            .next()
            .expect("operation")
            .split_once(":")
            .unwrap()
            .1
            .split_ascii_whitespace();
        assert_eq!(op_text.next().unwrap(), "new");
        assert_eq!(op_text.next().unwrap(), "=");
        let left = op_text.next().unwrap().to_owned();
        let op = op_text.next().unwrap().to_owned();
        let right = op_text.next().unwrap().to_owned();

        let op_fun = move |item: &mut Item| {
            let left = match left.as_str() {
                "old" => item.0.clone(),
                num => num.parse().expect("Expected a number"),
            };
            let right = match right.as_str() {
                "old" => item.0.clone(),
                num => num.parse().expect("Expected a number"),
            };
            match op.as_str() {
                "*" => item.0 = left * right,
                "+" => item.0 = left + right,
                e => panic!("unsupported operation: {e}"),
            }
        };
        let operation = Box::new(op_fun);

        // fourth line: test
        let mut test_text = lines
            .next()
            .expect("test")
            .split_once(":")
            .unwrap()
            .1
            .split_ascii_whitespace();
        assert_eq!(test_text.next().unwrap(), "divisible");
        assert_eq!(test_text.next().unwrap(), "by");
        let num: u32 = test_text
            .next()
            .unwrap()
            .parse()
            .expect("Expected a number");
        let test_fun = move |item: &Item| -> bool { &item.0 % num == 0.try_into().unwrap() };
        let test = Box::new(test_fun);

        // fifth line: monkey_true
        let monkey_true = lines
            .next()
            .expect("monkey_true")
            .split_once(":")
            .unwrap()
            .1
            .split_ascii_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap();

        // sixth line: monkey_false
        let monkey_false = lines
            .next()
            .expect("monkey_false")
            .split_once(":")
            .unwrap()
            .1
            .split_ascii_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap();

        Monkey {
            items,
            operation,
            test,
            monkey_true,
            monkey_false,
            item_counter: 0,
        }
    }

    fn process_items(&mut self, all_monkeys: &[RefCell<Monkey>], decreasing: bool) {
        while let Some(mut item) = self.items.pop_front() {
            // println!(
            //     "  Monkey inspects an item with a worry level of {}.",
            //     item.0
            // );
            (self.operation)(&mut item);
            // println!("    new worry level {}", item.0);
            if decreasing {
                item.0 /= 3u32;
                // println!("    monkey bored, worry level {}", item.0);
            }
            if (self.test)(&item) {
                all_monkeys[self.monkey_true]
                    .borrow_mut()
                    .items
                    .push_back(item);
                // println!("    item sent to {}", self.monkey_true);
            } else {
                all_monkeys[self.monkey_false]
                    .borrow_mut()
                    .items
                    .push_back(item);
                // println!("    item sent to {}", self.monkey_false);
            }
            self.item_counter += 1;
        }
    }
}

impl fmt::Display for Monkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut items = self.items.iter();
        if let Some(mut last) = items.next() {
            while let Some(cur) = items.next() {
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
    do_rounds(20, &example_monkeys, true);
    println!();

    let input_monkeys = parse(&input);
    do_rounds(20, &input_monkeys, true);
    println!();

    let example_monkeys = parse(&example);
    do_rounds(20, &example_monkeys, false);

    let example_monkeys = parse(&example);
    do_rounds(1000, &example_monkeys, false);
}

fn parse(input: &str) -> Vec<RefCell<Monkey>> {
    let mut monkeys = Vec::new();
    for (idx, one_input) in input.split("\n\n").enumerate() {
        let monkey = RefCell::new(Monkey::new(one_input, idx));
        //println!("{monkey:?}");
        monkeys.push(monkey);
    }
    monkeys
}

fn do_rounds(rounds: u32, monkeys: &[RefCell<Monkey>], decreasing: bool) {
    for round in 0..rounds {
        for monkey in monkeys {
            monkey
                .borrow_mut()
                .deref_mut()
                .process_items(&monkeys, decreasing);
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
}
