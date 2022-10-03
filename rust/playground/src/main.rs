struct Solution;

impl Solution {
    fn z_function(s: &[u8]) -> Vec<usize> {
        let n = s.len();
        let mut z = vec![0; n];
        let (mut l, mut r) = (0, 0);
        for i in 1..n {
            if i <= r {
                z[i] = std::cmp::min(r - i + 1, z[i - l]);
            }
            while i + z[i] < n && s[z[i]] == s[i + z[i]] {
                z[i] += 1;
            }
            let cur_r = i + z[i] - 1;
            if cur_r > r {
                l = i;
                r = cur_r
            }
        }
        z
    }

    pub fn delete_string(s: String) -> i32 {
        let a = s.as_bytes();
        let n = a.len();
        let mut dp = vec![0; n + 1];
        for i in (0..n).rev() {
            let mut cur = 1;
            let z = Self::z_function(&a[i..]);
            for j in i + 1..n {
                let l = j - i;
                if j + l <= n && z[j - i] >= l {
                    cur = std::cmp::max(cur, dp[j] + 1);
                }
            }
            dp[i] = cur;
            //dbg!(i, dp[i]);
        }
        return dp[0];
    }
}

fn main() {
    //println!("{}", Solution::delete_string(String::from("aaabaab")));
    println!("{}", Solution::delete_string(String::from("abcabcdabc")));
    println!("{}", Solution::delete_string(String::from("aaabaab")));
    println!("{}", Solution::delete_string(String::from("aaaaa")));
}
