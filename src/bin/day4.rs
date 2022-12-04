struct Work {
    lower: u32,
    upper: u32,
}

impl Work {
    fn parse(s_range: &str) -> Self {
        let (l, r) = s_range.split_once('-').unwrap();
        let lower = l.parse().unwrap();
        let upper = r.parse().unwrap();
        Self { lower, upper }
    }

    fn contains(&self, other: &Work) -> bool {
        if self.lower <= other.lower && self.upper >= other.upper {
            return true;
        }
        false
    }

    fn overlaps(&self, other: &Work) -> bool {
        if self.lower <= other.lower && self.upper >= other.lower {
            return true;
        }
        false
    }
}

fn main() {
    let input = aoc_2022::example(4);
    count_overlaps(input);

    let input = aoc_2022::input(4);
    count_overlaps(input);
}

fn count_overlaps(input: String) {
    let mut full = 0;
    let mut part = 0;
    for ll in input.lines() {
        let (elf1, elf2) = ll.split_once(',').unwrap();
        let w1 = Work::parse(elf1);
        let w2 = Work::parse(elf2);
        if w1.contains(&w2) || w2.contains(&w1) {
            //println!("{elf1} and {elf2} overlap fully");
            full += 1;
        }
        if w1.overlaps(&w2) || w2.overlaps(&w1) {
            //println!("{elf1} and {elf2} overlap partly");
            part += 1;
        }
    }
    println!("{full}");
    println!("{part}");
}
