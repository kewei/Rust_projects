use std::mem;

pub struct Solution {}

#[derive(Debug, Clone)]
pub struct ListNode {
    val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        Self {
            val,
            next: None,
        }
    }
}

impl Solution {
    pub fn merge_two_lists_recursion(list1: Option<Box<ListNode>>, list2: Option<Box<ListNode>>)
        -> Option<Box<ListNode>> {
        match (list1, list2) {
            (None, None) => None,
            (Some(l1), None) => Some(l1),
            (None, Some(l2)) => Some(l2),
            (Some(l1), Some(l2)) =>
                match l1.val <= l2.val {
                    true => Some(Box::new(ListNode{
                        val: l1.val,
                        next: Solution::merge_two_lists_recursion(l1.next, Some(l2)),
                    })),
                    false => Some(Box::new(ListNode {
                        val: l2.val,
                        next: Solution::merge_two_lists_recursion(Some(l1), l2.next),
                    }))
                }
        }
    }

    pub fn merge_two_lists_mem(mut list1: Option<Box<ListNode>>, mut list2: Option<Box<ListNode>>)
        -> Option<Box<ListNode>> {
        let mut head = None;
        let mut p_next = &mut head;

        while list1.is_some() && list2.is_some() {
            let l1 = &mut list1;
            let l2 = &mut list2;

            let ll = if l1.as_ref().unwrap().val <=
                                                l2.as_ref().unwrap().val { l1 } else { l2 };
            mem::swap(p_next, ll);
            mem::swap(ll, &mut p_next.as_mut().unwrap().next);
            p_next = &mut p_next.as_mut().unwrap().next;
        }
        mem::swap(p_next, if list1.is_none() {&mut list2} else {&mut list1});
        head
    }
}
