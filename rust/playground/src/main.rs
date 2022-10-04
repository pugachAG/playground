use std::collections::{HashMap, VecDeque, HashSet};

struct Solution;

impl Solution {
    pub fn find_all_recipes(recipes: Vec<String>, ingredients: Vec<Vec<String>>, supplies: Vec<String>) -> Vec<String> {
        let mut g = HashMap::<&String, Vec<&String>>::new();
        let mut rem = HashMap::<&String, HashSet<&String>>::new();
        for (i, ings) in ingredients.iter().enumerate() {
            let rec = &recipes[i];
            rem.insert(rec, ings.iter().collect());
            for ing in ings {
                g.entry(ing)
                    .or_insert_with(|| Vec::new())
                    .push(rec);
            }
        }
        let mut q: VecDeque<&String> = supplies.iter().collect();
        let mut ans = Vec::<String>::new();
        while let Some(ing) = q.pop_front() {
            if let Some(nxt) = g.get(ing) {
                for rec in nxt {
                    let s = rem.get_mut(rec).unwrap();
                    if s.remove(ing) && s.is_empty() {
                        q.push_back(rec);
                        ans.push((*rec).clone())
                    }
                }
            }
        }
        ans
    }
}

fn main() {
}

