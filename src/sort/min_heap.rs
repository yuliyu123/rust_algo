//  Heap Sort 
//  Min heap 
use std::collections::BinaryHeap;
use std::cmp::Reverse;
impl Solution {
    pub fn sort_array(nums: Vec<i32>) -> Vec<i32> {
        let l = nums.len();
        let mut minheap = BinaryHeap::with_capacity(l as usize);
        nums.into_iter().for_each(|i| minheap.push(Reverse(i)));
        let mut res = Vec::new();
        while let Some(Reverse(max)) = minheap.pop() { 
            res.push(max);
            if res.len() == l { 
                break
            }
        }
        res
    }
}
