use alloc::vec;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};

pub struct HashMap<K, V> {
    buckets: Vec<Option<(K, V)>>,
    len: usize,
}

impl<K: Eq + Hash + Clone, V: Clone> HashMap<K, V> {
    pub fn new() -> Self {
        let buckets = vec![None; 16];
        Self { buckets, len: 0 }
    }

    fn hash<Q>(&self, key: &Q) -> usize
    where
        Q: Hash,
    {
        // 注意：我们不能用 std 的 DefaultHasher！
        // 自己写一个极简哈希：
        simple_hash(key) % self.buckets.len()
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.len * 2 >= self.buckets.len() {
            self.resize();
        }
        let mut idx = self.hash(&key);
        loop {
            if self.buckets[idx].is_none() {
                self.buckets[idx] = Some((key, value));
                self.len += 1;
                return;
            } else {
                idx = (idx + 1) % self.buckets.len();
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut idx = self.hash(key);
        loop {
            match &self.buckets[idx] {
                Some((k, v)) if k == key => return Some(v),
                None => return None,
                _ => idx = (idx + 1) % self.buckets.len(),
            }
        }
    }

    fn resize(&mut self) {
        let len = self.buckets.len();
        let old = core::mem::replace(&mut self.buckets, vec![None; len * 2]);
        self.len = 0;
        for entry in old.into_iter() {
            if let Some((k, v)) = entry {
                self.insert(k, v);
            }
        }
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            buckets: &self.buckets,
            index: 0,
        }
    }
}

pub struct Iter<'a, K, V> {
    buckets: &'a [Option<(K, V)>],
    index: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.buckets.len() {
            if let Some((ref k, ref v)) = self.buckets[self.index] {
                self.index += 1;
                return Some((k, v));
            }
            self.index += 1;
        }
        None
    }
}

/// 一个简单的哈希函数示例
fn simple_hash<T: Hash>(t: &T) -> usize {
    // 简单暴力版哈希：乘法和异或
    use core::hash::Hasher;
    let mut h = SimpleHasher(0);
    t.hash(&mut h);
    h.finish() as usize
}

struct SimpleHasher(u64);

impl Hasher for SimpleHasher {
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = self.0.wrapping_mul(31).wrapping_add(*b as u64);
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}
