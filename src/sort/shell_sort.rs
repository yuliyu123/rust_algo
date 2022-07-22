impl Solution {
    pub fn sort_array(nums: Vec<i32>) -> Vec<i32> {
        let l = nums.len();
        let mut gap = l/2;
        let mut res: Vec<i32> = nums.into();
        
        while gap > 0 { 
            for i in gap..l { 
                let tmp = res[i].clone();
                let mut j = i;
                while j >= gap && res[j - gap] > tmp { 
                    res[j] = res[j - gap].clone();
                    j -= gap;
                }
                res[j] = tmp;
            }
            gap /= 2;
        }
        res
    }
}