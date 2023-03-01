use longest_sub_string::Solution;

fn main() {
    let s = String::from("dhfwkhjafa");
    let mut larg_size = Solution::length_of_longest_substring(s.clone());
    let mut larg_size2 = Solution::length_of_longest_substring_entry_and_modify(s.clone());
    println!("The size of longest substring: {}", larg_size);
    println!("The size of longest substring v2: {}", larg_size2);
}
