impl Solution {
    pub fn sort_array(nums: Vec<i32>) -> Vec<i32> {
        let mut res: Vec<i32> = nums.into();
        
        for _ in 0..res.len() { 
            let mut swaps = 0;
            for i in 1..res.len() { 
                if res[i-1] > res[i] { 
                    res.swap(i -1, i);
                    swaps +=1;
                }
            }
            if swaps == 0 { 
                break;
            }
        }
        res
    }
}