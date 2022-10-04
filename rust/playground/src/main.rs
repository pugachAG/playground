struct Solution;

impl Solution {
    fn upper_bound(a: &Vec<i32>, v: i32) -> usize {
        let n = a.len();
        if a.last().map_or(true, |x| *x <= v) {
            return n;
        }
        let (mut l, mut r) = (0, n-1);
        while l < r {
            let mid = (l+r)/2;
            if a[mid] <= v {
                l = mid+1;
            } else {
                r = mid;
            }
        }
        l
    }

    fn lis(arr: &Vec<i32>) -> usize {
        let mut a = Vec::<i32>::new();
        for v in arr.iter().copied() {
            let i = Self::upper_bound(&a, v);
            if i == a.len() {
                a.push(v);
            } else {
                a[i] = v;
            }
            dbg!(&a);
        }
        a.len()
    }

    pub fn k_increasing(arr: Vec<i32>, k: usize) -> i32 {
        let n = arr.len();
        let mut ans = 0;
        for i in 0..k {
            let cur: Vec<_> = (i..n).step_by(k).map(|i| arr[i]).collect();
            ans += Self::lis(&cur);
        }
        ans as i32
    }
}

fn main() {
    println!("{}", Solution::k_increasing(vec![1, 4, 3], 1));
}