use anyhow::{bail, Context, Result};
use itertools::Itertools;

struct Crt {
    cycles: Vec<i32>,
}

impl Crt {
    fn new(instructions: &str) -> Result<Self> {
        let mut x = 1;
        let mut cycles = vec![x];

        for ll in instructions.lines() {
            let mut words = ll.split_whitespace();
            match words.next() {
                Some("addx") => {
                    cycles.push(x);
                    cycles.push(x);
                    x += words
                        .next()
                        .context("addx expects an immediate value")?
                        .parse::<i32>()?;
                }
                Some("noop") => {
                    cycles.push(x);
                }
                Some(e) => bail!("unsupported instruction: {e}"),
                None => bail!("empty line"),
            }
        }
        Ok(Self { cycles })
    }

    fn sig_strength(&self) {
        let strength_20 = self.cycles[20] * 20;
        let strength_60 = self.cycles[60] * 60;
        let strength_100 = self.cycles[100] * 100;
        let strength_140 = self.cycles[140] * 140;
        let strength_180 = self.cycles[180] * 180;
        let strength_220 = self.cycles[220] * 220;

        let sum =
            strength_20 + strength_60 + strength_100 + strength_140 + strength_180 + strength_220;
        println!(
            "{} + {} + {} + {} + {} + {} = {}",
            strength_20, strength_60, strength_100, strength_140, strength_180, strength_220, sum
        );
    }

    fn draw(&self) {
        for iter in &self.cycles.iter().skip(1).chunks(40) {
            let mut line = String::new();
            for (i, x) in iter.enumerate() {
                let i: i32 = i.try_into().unwrap(); // lines are len 40, should never fail
                if *x == i || x - 1 == i || x + 1 == i {
                    line.push('#');
                } else {
                    line.push('.');
                }
            }
            println!("line: {}", line);
        }
    }
}

fn main() -> Result<()> {
    let example = aoc_2022::example(10);
    let crt = Crt::new(&example)?;
    crt.sig_strength();
    crt.draw();

    let input = aoc_2022::input(10);
    let crt = Crt::new(&input)?;
    crt.sig_strength();
    crt.draw();

    Ok(())
}
