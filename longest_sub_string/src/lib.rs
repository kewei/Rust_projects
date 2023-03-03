use std::collections::{HashMap, VecDeque};
use std::cmp::max;
use std::process::id;

pub struct Solution { }

impl Solution {
    pub fn length_of_longest_substring(s: String) -> i32 {
        let mut long_sub_map: HashMap<char, usize> = HashMap::new();
        let mut start_idx = 0;
        let mut largest_len = 0;

        for (idx, c) in s.char_indices() {
            if long_sub_map.contains_key(&c) {
                if long_sub_map[&c] >= start_idx {
                    largest_len = max(largest_len, idx - long_sub_map[&c]);
                    start_idx = long_sub_map[&c] + 1;
                }
                else { largest_len = max(largest_len, idx - start_idx + 1); }
            }
            else { largest_len = max(largest_len, idx - start_idx + 1); }
            long_sub_map.insert(c, idx);
        }
        largest_len as i32
    }

    pub fn length_of_longest_substring_entry_and_modify(s: String) -> i32 {
        let mut long_sub_map: HashMap<char, usize> = HashMap::new();
        let mut start_idx = 0;
        let mut largest_len = 0;

        for (idx, c) in s.char_indices() {
            long_sub_map.entry(c)
                .and_modify(|old_idx| {
                    if *old_idx >= start_idx {
                        largest_len = max(largest_len, idx - *old_idx);
                        start_idx = *old_idx + 1;
                    }
                    else { largest_len = max(largest_len, idx - start_idx + 1); }
                    *old_idx = idx;
                })
                .or_insert(idx);
            largest_len = max(largest_len, idx - start_idx + 1);
        }
        largest_len as i32
    }

    pub fn length_of_longest_substring_vecdeque(s: String) -> i32 {
        let (max_len, _) = s.chars().fold(
            (0, VecDeque::with_capacity(s.len())),
            |(max_len, mut sub_vec_deque), ch| {
                if sub_vec_deque.contains(&ch) {
                    while sub_vec_deque.pop_back() != Some(ch) { }
                }
                sub_vec_deque.push_front(ch);
                (max(max_len, sub_vec_deque.len()), sub_vec_deque)
            }
        );
        max_len as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_of_longest_substring() {
        let s1 = String::from("abcabcbb");
        let len1 = Solution::length_of_longest_substring(s1);
        assert_eq!(len1, 3);
        let s2 = String::from("bbbbb");
        let len2 = Solution::length_of_longest_substring(s2);
        assert_eq!(len2, 1);
        let s3 = String::from("pwwkew");
        let len3 = Solution::length_of_longest_substring(s3);
        assert_eq!(len3, 3);
        let s4 = String::from("");
        let len4 = Solution::length_of_longest_substring(s4);
        assert_eq!(len4, 0);
        let s5 = String::from(" ");
        let len5 = Solution::length_of_longest_substring(s5);
        assert_eq!(len5, 1);
    }

    #[test]
    fn test_length_of_longest_substring_entry_and_modify() {
        let s1 = String::from("abcabcbb");
        let len1 = Solution::length_of_longest_substring_entry_and_modify(s1);
        assert_eq!(len1, 3);
        let s2 = String::from("bbbbb");
        let len2 = Solution::length_of_longest_substring_entry_and_modify(s2);
        assert_eq!(len2, 1);
        let s3 = String::from("pwwkew");
        let len3 = Solution::length_of_longest_substring_entry_and_modify(s3);
        assert_eq!(len3, 3);
        let s4 = String::from("");
        let len4 = Solution::length_of_longest_substring_entry_and_modify(s4);
        assert_eq!(len4, 0);
        let s5 = String::from(" ");
        let len5 = Solution::length_of_longest_substring_entry_and_modify(s5);
        assert_eq!(len5, 1);
    }

    #[test]
    fn test_length_of_longest_substring_vecdeque() {
        let s1 = String::from("abcabcbb");
        let len1 = Solution::length_of_longest_substring_vecdeque(s1);
        assert_eq!(len1, 3);
        let s2 = String::from("bbbbb");
        let len2 = Solution::length_of_longest_substring_vecdeque(s2);
        assert_eq!(len2, 1);
        let s3 = String::from("pwwkew");
        let len3 = Solution::length_of_longest_substring_vecdeque(s3);
        assert_eq!(len3, 3);
        let s4 = String::from("");
        let len4 = Solution::length_of_longest_substring_vecdeque(s4);
        assert_eq!(len4, 0);
        let s5 = String::from(" ");
        let len5 = Solution::length_of_longest_substring_vecdeque(s5);
        assert_eq!(len5, 1);
    }
}