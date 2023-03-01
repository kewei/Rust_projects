use longest_sub_string::Solution;

fn main() {
    let s = String::from("dhfwkhjafa");
    let mut larg_size = Solution::length_of_longest_substring(s);
    println!("The size of longest substring: {}", larg_size);
}
