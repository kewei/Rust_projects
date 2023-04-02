use merge_link_nodes::Solution;
use merge_link_nodes::ListNode;

fn create_list_node(vec: Vec<i32>) -> Option<Box<ListNode>> {
    let mut head = None;
    let mut current = &mut head;

    for v in vec {
        *current = Some(Box::new(ListNode::new(v)));
        current = &mut current.as_mut().unwrap().next;
    }
    head
}

fn main() {
    let v1 = vec![1_i32, 2, 4];
    let v2 = vec![1_i32, 3, 4];

    let list1 = create_list_node(v1);
    let list2 = create_list_node(v2);

    let list_res = Solution::merge_two_lists_recursion(list1.clone(), list2.clone());
    println!("{:?}", list_res.unwrap());

    let list_res_mem = Solution::merge_two_lists_mem(list1, list2);
    println!("{:?}", list_res_mem.unwrap());
}
