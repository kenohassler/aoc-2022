use std::{cmp::Ordering, fmt, str::FromStr};
use thiserror::Error;

#[derive(Debug, Clone)]
enum Value {
    Integer(u32),
    List(Vec<Value>),
}

impl From<u32> for Value {
    fn from(i: u32) -> Self {
        Self::Integer(i)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            (Self::Integer(l0), Self::List(r0)) => vec![Self::Integer(*l0)] == *r0,
            (Self::List(l0), Self::Integer(r0)) => *l0 == vec![Self::Integer(*r0)],
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Integer(l0), Self::Integer(r0)) => l0.cmp(r0),
            (Self::List(l0), Self::List(r0)) => l0.cmp(r0),
            (Self::Integer(l0), Self::List(r0)) => vec![Self::Integer(*l0)].cmp(r0),
            (Self::List(l0), Self::Integer(r0)) => l0.cmp(&vec![Self::Integer(*r0)]),
        }
    }
}

impl FromStr for Value {
    type Err = ParseValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let js = json::parse(s)?;
        if js.is_number() {
            Ok(Value::Integer(
                js.as_u32().ok_or(ParseValueError::InvalidInteger)?,
            ))
        } else if js.is_array() {
            let mut vals = Vec::new();
            for item in js.members() {
                vals.push(item.to_string().parse::<Value>()?);
            }
            Ok(Value::List(vals))
        } else {
            Err(ParseValueError::InvalidType)
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{i}"),
            Value::List(l) => {
                write!(f, "[")?;
                if let Some(last) = l.last() {
                    for elem in &l[0..l.len() - 1] {
                        write!(f, "{},", elem)?;
                    }
                    write!(f, "{}", last)?;
                }
                write!(f, "]")?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Error)]
enum ParseValueError {
    #[error("the input is not valid JSON")]
    JSONError(#[from] json::Error),
    #[error("elements can only be numbers or arrays")]
    InvalidType,
    #[error("not an integer")]
    InvalidInteger,
}

fn main() -> Result<(), ParseValueError> {
    let example = aoc_2022::example(13);
    let pairs = parse_pairs(&example)?;
    let sum = part_1(&pairs);
    let prod = part_2(&pairs);
    println!("sum of indices already sorted: {sum}");
    println!("product of divider indices: {}", prod.0 * prod.1);

    let input = aoc_2022::input(13);
    let pairs = parse_pairs(&input)?;
    let sum = part_1(&pairs);
    let prod = part_2(&pairs);
    println!("sum of indices already sorted: {sum}");
    println!("product of divider indices: {}", prod.0 * prod.1);

    Ok(())
}

fn parse_pairs(input: &str) -> Result<Vec<Vec<Value>>, ParseValueError> {
    input
        .split("\n\n")
        .map(|p| {
            p.lines()
                .map(|ll| ll.parse::<Value>())
                .collect::<Result<Vec<Value>, _>>()
        })
        .collect::<Result<Vec<Vec<Value>>, _>>()
}

fn part_1(pairs: &[Vec<Value>]) -> usize {
    pairs
        .iter()
        .enumerate()
        .filter(|(_, p)| p.first() < p.last())
        .map(|(i, _)| i + 1)
        .sum()
}

fn part_2(pairs: &[Vec<Value>]) -> (usize, usize) {
    let mut list = pairs.iter().flatten().cloned().collect::<Vec<Value>>();

    // insert dividers somewhere
    let divider_1 = "[[2]]".parse::<Value>().unwrap();
    let divider_2 = "[[6]]".parse::<Value>().unwrap();
    list.push(divider_1.clone());
    list.push(divider_2.clone());

    list.sort();
    let pos_1 = list.iter().position(|v| *v == divider_1).unwrap();
    let pos_2 = list.iter().position(|v| *v == divider_2).unwrap();

    (pos_1 + 1, pos_2 + 1)
}

#[cfg(test)]
mod test {
    use crate::Value;

    #[test]
    fn compare_lists() {
        let a = Value::List(vec![1.into(), 1.into(), 3.into(), 1.into(), 1.into()]);
        let b = Value::List(vec![1.into(), 1.into(), 5.into(), 1.into(), 1.into()]);
        assert_ne!(a, b);
        assert!(a < b);
    }

    #[test]
    fn compare_len() {
        let a = Value::List(vec![7.into(), 7.into(), 7.into(), 7.into()]);
        let b = Value::List(vec![7.into(), 7.into(), 7.into()]);
        assert_ne!(a, b);
        assert!(a > b);
        println!("{a} > {b}");
    }

    #[test]
    fn compare_one_empty() {
        let a = Value::List(vec![]);
        let b = Value::List(vec![3.into()]);
        assert_ne!(a, b);
        assert!(a < b);
        println!("{a} < {b}");
    }

    #[test]
    fn compare_empty() {
        let a = Value::List(vec![Value::List(vec![Value::List(vec![])])]);
        let b = Value::List(vec![Value::List(vec![])]);
        assert_ne!(a, b);
        assert!(a > b);
        println!("{a} > {b}");
    }

    #[test]
    fn parse() {
        let input = "[[]]";
        let output = Value::List(vec![Value::List(vec![])]);
        assert_eq!(input.parse::<Value>().unwrap(), output);
        println!("{output}");
    }
}
