use std::ops::Add;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Matcher {
    count: i8,
}

impl Add for Matcher {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Matcher {
            count: self.count + other.count
        }
    }
}

impl Matcher {
    pub fn greatest<T: std::cmp::PartialOrd>(li: &[T]) -> &T {
        let mut greatest = &li[0];
        for item in li {
            if greatest < item {
                greatest = item;
            }
        }
        greatest
    }

    pub fn is_match(&self) -> bool {
        true
    }
}

pub struct NameMatcher {
    supper: Matcher,
}

impl NameMatcher {
    fn new() -> Self {
        NameMatcher { supper: Matcher { count: 2 } }
    }
}

#[cfg(test)]
pub mod matcher_tests {
    use std::collections::HashMap;
    use crate::matcher::{Matcher, NameMatcher};
    use crate::utils;

    pub fn test_matcher<T: std::cmp::PartialOrd>(values: &[T]) -> &T {
        let matcher = Matcher { count: 1 };
        let n_matcher = NameMatcher::new();
        let is_m = matcher.is_match();
        if is_m && n_matcher.supper.eq(&matcher) {
            println!("match!{:?}", matcher);
            println!("{:?}", n_matcher.supper);
        }
        Matcher::greatest::<T>(values)
    }

    #[test]
    pub fn test_color_matcher() {
        let mut map: HashMap<&str, i8> = HashMap::new();
        map.insert("Red", 1);
        map.insert("Blue", 3);
        map.insert("Yellow", 2);
        let values = map.values().collect::<Vec<_>>();
        let result = test_matcher(&values);
        let matcher = Matcher { count: 1 };
        println!("Hello world! With result: {result} and is matcher: {}", matcher.count);
        dbg!(&map);
        utils::utils::show_info(map.get("Red").map_or(0i8, |x| *x).to_string().as_str());
        utils::utils::show_info(map["Red"].to_string().as_str())
    }
}