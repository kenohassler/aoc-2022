use std::fs;

const IN_DIR: &str = "inputs";

pub fn get_example(day: u8) -> String {
    fs::read_to_string(format!("{IN_DIR}/day{day}_example.txt")).unwrap()
}

pub fn get_input(day: u8) -> String {
    fs::read_to_string(format!("{IN_DIR}/day{day}.txt")).unwrap()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
