#[derive(Debug, Copy, Clone)]
enum Move {
    Rock = 1,
    Paper = 2,
    Scissors = 3,
}

fn score(moves: &(Move, Move)) -> i32 {
    use Move::{Paper, Rock, Scissors};
    let score = match moves {
        (Rock, Rock) | (Paper, Paper) | (Scissors, Scissors) => 3, /* draw */
        (Rock, Paper) | (Paper, Scissors) | (Scissors, Rock) => 6, /* win  */
        (Rock, Scissors) | (Paper, Rock) | (Scissors, Paper) => 0, /* loss */
    };
    score + moves.1 as i32
}

fn main() {
    let score_acc = |acc, moves| acc + score(moves);

    // example
    let input = aoc_2022::example(2);
    let guide = make_guide(&input);
    let total = guide.iter().fold(0, score_acc);

    let guide2 = make_guide2(&input);
    let total2 = guide2.iter().fold(0, score_acc);
    println!("{}", total);
    println!("{}", total2);

    // real input
    let input = aoc_2022::input(2);
    let guide = make_guide(&input);
    let total = guide.iter().fold(0, score_acc);

    let guide2 = make_guide2(&input);
    let total2 = guide2.iter().fold(0, score_acc);
    println!("{}", total);
    println!("{}", total2);
}

fn make_guide(input: &str) -> Vec<(Move, Move)> {
    use Move::{Paper, Rock, Scissors};
    let mut guide = Vec::<(Move, Move)>::new();

    for ll in input.lines() {
        let (opp, me) = ll.split_once(' ').unwrap();

        let opp_move = match opp {
            "A" => Rock,
            "B" => Paper,
            "C" => Scissors,
            _ => unreachable!(),
        };
        let my_move = match me {
            "X" => Rock,
            "Y" => Paper,
            "Z" => Scissors,
            _ => unreachable!(),
        };

        guide.push((opp_move, my_move));
    }
    guide
}

fn make_guide2(input: &str) -> Vec<(Move, Move)> {
    use Move::{Paper, Rock, Scissors};
    let mut guide = Vec::<(Move, Move)>::new();

    for ll in input.lines() {
        let (opp, me) = ll.split_once(' ').unwrap();

        let opp_move = match opp {
            "A" => Rock,
            "B" => Paper,
            "C" => Scissors,
            _ => unreachable!(),
        };
        let my_move = match me {
            "X" => match opp_move {
                // need to lose
                Rock => Scissors,
                Paper => Rock,
                Scissors => Paper,
            },
            "Y" => opp_move, // need draw
            "Z" => match opp_move {
                // need to win
                Rock => Paper,
                Paper => Scissors,
                Scissors => Rock,
            },
            _ => unreachable!(),
        };

        guide.push((opp_move, my_move));
    }
    guide
}
