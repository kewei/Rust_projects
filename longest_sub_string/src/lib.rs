use std::collections::HashMap;
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

    pub fn length_of_longest_substring_v2(s: String) -> i32 {
        let mut long_sub_map: HashMap<char, usize> = HashMap::new();
        let mut start_idx = 0;
        let mut largest_len = 0;

        for (idx, c) in s.char_indices() {
            
        }
    }
}