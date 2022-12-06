use itertools::Itertools;

fn main() {
    let example = aoc_2022::example(6);
    for ll in example.lines() {
        find_marker_long(ll, 4);
        find_marker_long(ll, 14);
    }

    let input = aoc_2022::input(6);
    find_marker_long(input.trim(), 4);
    find_marker_long(input.trim(), 14);
}

fn find_marker_long(stream: &str, size: usize) -> usize {
    let bytes: Vec<char> = stream.chars().collect();
    'start: for i in 0..bytes.len() - size + 1 {
        for j in 0..size - 1 {
            for k in j + 1..size {
                if bytes[i + j] == bytes[i + k] {
                    // one pair of chars equal, continue to next window
                    continue 'start;
                }
            }
        }
        // all chars in this window unequal
        println!("marker {} at {}", &stream[i..i + size], i + size);
        return i + size;
    }
    panic!("no marker found");
}

#[allow(dead_code)]
fn find_marker(stream: &str) -> usize {
    println!("input: {}", stream);
    for (num, (c1, c2, c3, c4)) in stream.chars().tuple_windows().enumerate() {
        if c1 != c2 && c1 != c3 && c1 != c4 && c2 != c3 && c2 != c4 && c3 != c4 {
            println!("marker {c1}{c2}{c3}{c4} found, position {}", num + 4);
            return num + 4;
        }
    }
    panic!("no marker found");
}
