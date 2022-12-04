use std::fs;

const IN_DIR: &str = "inputs";

pub fn example(day: u8) -> String {
    fs::read_to_string(format!("{IN_DIR}/day{day}_example.txt")).unwrap()
}

pub fn input(day: u8) -> String {
    fs::read_to_string(format!("{IN_DIR}/day{day}.txt")).unwrap()
}
