use std::collections::BinaryHeap;
impl Solution {
    pub fn sort_array(nums: Vec<i32>) -> Vec<i32> {
        let mut maxheap = BinaryHeap::from(nums);
        maxheap.into_sorted_vec()
    }
}