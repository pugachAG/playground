use std::collections::HashSet;
use rand::thread_rng;
use rand::seq::SliceRandom;
use sha2::Digest;

use test::Bencher;

extern crate test;

type Data = [u8; 32];

fn convert(i: usize) -> Data {
    sha2::Sha256::digest(i.to_string().as_bytes()).into()
}


fn bench_hash_set(b: &mut Bencher) {
    const CNT: usize = 1000;
    const SZ: usize = 40_000;
    let mut sets = (0..CNT).map(|_| HashSet::<Data>::with_capacity(SZ/2)).collect::<Vec<_>>();
    let mut data: Vec<_> = (0..CNT*SZ).map(|i| convert(i)).collect();
    for (i, v) in data.iter().step_by(2).enumerate() {
        sets[i%CNT].insert(*v);
    }
    data.shuffle(&mut thread_rng());
    let mut i = 0;
    b.iter(|| {
        let v = data[i];
        i = (i + 1) % data.len();
        for set in &sets {
            let _ = set.contains(&v);
        }
    });
}