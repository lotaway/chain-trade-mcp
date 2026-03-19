use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

pub fn show_info(message: &str) {
    println!("show info: {}", message);
}

pub fn find_sub_str_index(haystack: String, needle: String) -> i32 {
    let size = haystack.len();
    let c_size = needle.len();
    if size == 0 || c_size == 0 {
        return -1;
    }
    let mut index = 0;
    let mut it = needle.chars();
    let chars: Vec<char> = haystack.chars().collect();
    while index < size && c_size <= size - index {
        let mut c_count = 0;
        while let Option::Some(ch) = it.next() {
            if ch != chars[index + c_count] {
                it = needle.chars();
                break;
            }
            c_count = c_count + 1;
        }
        if c_count == c_size {
            return index as i32;
        }
        index = index + 1;
    }
    -1
}

pub fn get_config_file(file_name: &str) -> String {
    let prev_fix = "/config";
    let path = Path::new(prev_fix).join(file_name);
    let result = get_file(&path);
    let mut content = match result {
        Ok(_content) => _content,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create(file_name) {
                Ok(mut n_file) => {
                    let mut _content = String::new();
                    let result = n_file.read_to_string(&mut _content);
                    println!("result: {}", result.unwrap());
                    _content
                }
                Err(err) => panic!("{}", err.to_string()),
            },
            _ => panic!("{}", error.to_string()),
        },
    };
    if content.is_empty() {
        std::io::stdin()
            .read_line(&mut content)
            .expect("error in get config file.");
    }
    content
}

#[derive(Debug)]
enum GetFileError {
    NoContent,
    IoError(std::io::Error),
}

fn get_file(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut content = String::new();
    // let size = File::open(path).and_then(|mut file| file.read_to_string(&mut content))?;
    // Ok(content)
    File::open(path)?
        .read_to_string(&mut content)
        .map(|_| content)
}

pub fn climb_stairs(n: i32) -> i32 {
    if n <= 2 {
        return n;
    }
    let mut sums = vec![0; n as usize + 1];
    sums[1] = 1;
    sums[2] = 2;
    for i in 3usize..n as usize + 1 {
        sums[i] = sums[i - 1] + sums[i - 2];
    }
    sums[n as usize]
}

#[cfg(test)]
pub mod utils_tests {
    use crate::utils::utils::{climb_stairs, find_sub_str_index};

    #[test]
    fn test_find_sub_str_index() {
        let result = find_sub_str_index(String::from("sadbutsad"), String::from("sad"));
        assert_eq!(result, 0, "find sub str index failed 1");
        let result2 = find_sub_str_index(String::from("leetcode"), String::from("leeto"));
        assert_eq!(result2, -1, "find sub str index failed 2");
        let result3 = find_sub_str_index(String::from("mississippi"), String::from("issip"));
        assert_eq!(result3, 4, "find sub str index failed 3");
    }

    #[test]
    fn test_climb_stairs() {
        assert_eq!(climb_stairs(1), 1);
        assert_eq!(climb_stairs(2), 2);
        assert_eq!(climb_stairs(3), 3);
        assert_eq!(climb_stairs(5), 8);
        assert_eq!(climb_stairs(45), 1836311903);
    }
}
