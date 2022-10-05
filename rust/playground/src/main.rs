struct Solution;

struct Dsu {
    p: Vec<usize>
}

impl Dsu {
    fn new(n: usize) -> Dsu {
        Dsu {
            p: (0..n).collect()
        }
    }

    fn parent(&mut self, i: usize) -> usize {
        if self.p[i] != i {
            self.p[i] = self.parent(self.p[i]);
        }
        self.p[i]
    }

    fn join(&mut self, i: usize, j: usize) {
        let pi = self.parent(i);
        let pj = self.parent(j);
        if pi != pj {
            self.p[pj] = pi;
        }
    }
}


fn main() {
}
