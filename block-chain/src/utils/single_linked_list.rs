use std::ptr::NonNull;

// type ListNodeValue<Rhs> = dyn PartialEq<Rhs> + Clone;
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SingleLinkedList<T: PartialEq + Clone> {
    pub val: T,
    pub next: Option<NonNull<SingleLinkedList<T>>>,
}

impl<T: PartialEq + Clone> SingleLinkedList<T> {
    #[inline]
    pub fn new(val: T) -> Self {
        SingleLinkedList {
            next: None,
            val,
        }
    }

    pub unsafe fn del_dup(&mut self) {
        let mut node = self;
        let mut cur_val = node.val.clone();
        while let Option::Some(next) = node.next.take() { // 拿走
            if next.as_ref().val == cur_val {
                node.next = next.as_ref().next;
            } else {
                cur_val = next.as_ref().val.clone();
                node.next = Option::Some(next); // 放回
                node = node.next.unwrap().as_mut();
            }
        }
    }
}

impl<T: PartialEq + Clone> From<&[T]> for SingleLinkedList<T> {
    fn from(list: &[T]) -> Self {
        let mut head = Option::None;
        for val in list.iter().rev() {
            let new_node = Box::new(SingleLinkedList {
                val: val.clone(),
                next: head,
            });
            head = NonNull::new(Box::into_raw(new_node));
        }
        // Safety: We know that head is not null because we just created it.
        let head = unsafe { *Box::from_raw(head.unwrap().as_ptr()) };
        head
    }
}

// Definition for singly-linked list.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        ListNode {
            next: Option::None,
            val,
        }
    }
}

pub fn delete_duplicates(head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    if head.is_none() {
        return head;
    }
    let mut head = head;
    let mut node = head.as_mut().unwrap();
    let mut cur = node.val;
    while let Option::Some(next) = node.next.take() {
        if next.val == cur {
            node.next = next.next;
        } else {
            cur = next.val;
            node.next = Option::Some(next);
            node = node.next.as_mut().unwrap();
        }
    }
    head
}

#[cfg(test)]
mod single_linked_list_tests {
    use crate::utils::single_linked_list::{delete_duplicates, ListNode, SingleLinkedList};

    #[test]
    fn test_delete_duplicates_link_node() {
        /*{
            let is_none = None::<Box<SingleLinkedList<i8>>>;
            assert_eq!(SingleLinkedList::del_dup_from_option(is_none.clone()), is_none);
        }
        {
            let no_duplicate = Some(Box::new(SingleLinkedList { val: 0, next: None }));
            assert_eq!(SingleLinkedList::del_dup_from_option(no_duplicate.clone()), no_duplicate);
        }*/
        unsafe {
            let has_duplicate = SingleLinkedList::from(&[0, 1, 1, 2, 2, 2, 3, 4] as &[i32]);
            let mut clone = has_duplicate.clone();
            clone.del_dup();
            assert_eq!(clone, has_duplicate);
        }
    }

    fn build_list_from_slice(s: &[i32]) -> Option<Box<ListNode>> {
        if s.is_empty() {
            return Option::None;
        }
        let head = Box::new(ListNode {
            val: s.first().copied().unwrap(),
            next: build_list_from_slice(&s[1..]),
        });
        Option::Some(head)
    }

    #[test]
    fn test_delete_duplicates() {
        struct TestCase {
            name: &'static str,
            nums: &'static [i32],
            expect: &'static [i32],
        }

        vec![
            TestCase {
                name: "basic",
                nums: &[1, 1, 2, 3, 3],
                expect: &[1, 2, 3],
            },
        ].iter().for_each(|testcase| {
            let head = build_list_from_slice(testcase.nums);
            let actual = delete_duplicates(head);
            let expect = build_list_from_slice(testcase.expect);
            assert_eq!(expect, actual, "{} failed", testcase.name);
        });
    }
}