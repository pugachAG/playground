use std::collections::HashMap;

struct Solution;

impl Solution {
    fn find_start(pairs: &Vec<Vec<i32>>) -> i32 {
        let mut cnt = HashMap::<i32, i32>::new();
        let mut upd = |v: i32, d: i32| {
            *cnt.entry(v).or_insert(0) += d;
        };
        for pr in pairs {
            upd(pr[0], 1);
            upd(pr[1], -1);
        }
        for (i, pr) in pairs.iter().enumerate() {
            let v = &pr[0];
            if cnt[v] == 1 {
                return *v;
            }
        }
        pairs[0][0]
    }

    fn dfs(v: i32, g: &mut HashMap<i32, Vec<i32>>, ans: &mut Vec<i32>) {
        while let Some(u) = g.get_mut(&v).and_then(|nxt| nxt.pop()) {
            Self::dfs(u, g, ans);
        }
        ans.push(v);
    }

    pub fn valid_arrangement(pairs: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
        let mut g = HashMap::new();
        for (i, pr) in pairs.iter().enumerate() {
            g.entry(pr[0]).or_insert_with(Vec::new).push(pr[1]);
        }
        let mut ord = Vec::new();
        Self::dfs(Self::find_start(&pairs), &mut g, &mut ord);
        ord.reverse();
        let mut ans = Vec::new();
        for i in 1..ord.len() {
            ans.push(vec![ord[i - 1], ord[i]]);
        }
        ans
    }
}

fn main() {
    let ans = Solution::valid_arrangement(vec![vec![2, 3], vec![2, 0], vec![0, 2], vec![3, 1]]);
    dbg!(ans);
}
