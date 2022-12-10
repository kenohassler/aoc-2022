use itertools::Itertools;

struct Crt {
    cycles: Vec<i32>,
}

impl Crt {
    fn new(instructions: &str) -> Self {
        let mut x = 1;
        let mut cycles = Vec::new();
        cycles.push(x);

        for ll in instructions.lines() {
            let mut words = ll.split_whitespace();
            match words.next() {
                Some("addx") => {
                    cycles.push(x);
                    cycles.push(x);
                    x += words
                        .next()
                        .expect("addx expects an immediate value")
                        .parse::<i32>()
                        .unwrap();
                }
                Some("noop") => {
                    cycles.push(x);
                }
                Some(e) => panic!("unsupported instruction: {e}"),
                None => panic!("empty line"),
            }
        }
        Self { cycles }
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
                if *x == i as i32 || x - 1 == i as i32 || x + 1 == i as i32 {
                    line.push('#');
                } else {
                    line.push('.');
                }
            }
            println!("line: {}", line);
        }
    }
}

fn main() {
    let example = aoc_2022::example(10);
    let crt = Crt::new(&example);
    crt.sig_strength();
    crt.draw();

    let input = aoc_2022::input(10);
    let crt = Crt::new(&input);
    crt.sig_strength();
    crt.draw();
}
