use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;

// low for testing
const INITIAL_NBUCKETS:usize = 1;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

impl<K,V> HashMap<K,V> {
    pub fn new() -> Self {
        HashMap {
            // Initialize an empty map. We will allocate only when we have to, the first time we insert.
            buckets: Vec::new(),
            items: 0
        }
    }
}

impl<K,V> HashMap<K,V> 
where
    K:Hash + Eq
{
    // Hashes a key and returns which bucket it belongs to 
    fn bucket(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.buckets.len() as u64) as usize
    }

    // Calls resize if we are empty or almost full.
    // Increases the internal items count.
    // Replaces duplicate keys.
    pub fn insert(&mut self, key : K, value : V) -> Option<V> {
        // If we have no buckets yet or the map is 3/4 full
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }

        let bucket = self.bucket(&key);
        let bucket = &mut self.buckets[bucket];

        self.items += 1;
        // Iterate through the bucket, trying to find any element, that has a key that they user has given us.
        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            if ekey == &key {
                // Tell the user what was replaced with what.
                return Some(mem::replace(evalue, value));
            }
        }
        //if let Some(&mut (ref ekey, ref mut evalue)) = bucket.iter_mut().find(|&mut (ref ekey, _)| ekey == key)

        // We know now the key doesnt exist so just insert it.
        bucket.push((key,value));
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let bucket = self.bucket(key);

        for x in &self.buckets[bucket] {
            if &x.0 == key {
                return Some(&x.1);
            }
        }
        return None;

        // self.buckets[bucket]
        //     .iter()
        //     .find(|&(ref ekey, _)| ekey == key)
        //     .map(|&(_, ref v)| v)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
        
        // let bucket = self.bucket(key);

        // self.buckets[bucket]
        //     .iter()
        //     .find(|&(ref ekey, _)| ekey == key)
        //     .map(|&(_, ref v)| v)
        //     .is_some()
    }
    
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let bucket = self.bucket(key);
        let bucket = &mut self.buckets[bucket];
        let i = bucket.iter().position(|&(ref ekey, _)| ekey == key)?;
        self.items -= 1;
        // swap_remove() first changes the order then truncates so it can be more efficient.
        Some(bucket.swap_remove(i).1)
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
    }

    // 
    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
             0 => INITIAL_NBUCKETS,
             n=> 2*n,
        };
 
        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));

        // Iterate all bucket and for each bucket drain all of they key value pairs
        for (key, value) in self.buckets
            .iter_mut()
            .flat_map(|bucket| bucket.drain(..)) 
        {
             let mut hasher = DefaultHasher::new();
             key.hash(&mut hasher);
             let bucket = (hasher.finish() % new_buckets.len() as u64) as usize;
             new_buckets[bucket].push((key,value));
        }

        mem::replace(&mut self.buckets, new_buckets);
     }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    map: &'a HashMap<K,V>,
    bucket: usize,
    at: usize
}

// impl<'a, K, V> Iter<'a, K, V>{

//     fn new(&'a HashMap<K,V>) -> Self {

//     }
// }

impl <'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    // Return some of an item if there are more things to iterate
    // and None if there are no more elements.
    fn next(&mut self) -> Option<Self::Item> {
        // if we reach the end of the bucket we need to move to the next bucket.
        // if we reach the end of the map we need to return None.
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => {
                    match bucket.get(self.at) {
                        Some(&(ref k, ref v)) => {
                            // move along self.at and self.bucket
                            self.at += 1;
                            // Loops can return with values!
                            break Some((k, v));
                        }
                        None => {
                            // this indicates we've reached the end of the bucket. so start from the beginning of the next one.
                            self.bucket += 1;
                            self.at = 0;
                            continue;
                        }
                    }
                }
                None => {
                    // this indicates we've reached a bucket that's out of bounds meaning the end of the hash map
                    break None
                }
            }
        }
    }
}

// the user has a hash map and they want to iterate over it.
// for loops require the thing after in to implement into_iter()
impl<'a, K, V> IntoIterator for &'a HashMap<K,V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) ->Self::IntoIter {
        Iter {  
            map: self, 
            bucket: 0, 
            at: 0 
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = HashMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        map.insert("foo", 322);
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
        assert_eq!(map.get(&"foo"), Some(&322));
        assert_eq!(map.remove(&"foo"), Some(322));
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        assert_eq!(map.get(&"foo"), None);
    }

    #[test]
    fn iter() {
        let mut map = HashMap::new();
        map.insert("zulul", 322);
        map.insert("peepo", 2);
        map.insert("kek", 3);
        map.insert("butter", 4);
        for (&k, &v) in &map {
            match k {
                "zulul" => assert_eq!(v, 322),
                "peepo" => assert_eq!(v, 2),
                "kek" => assert_eq!(v, 3),
                "butter" => assert_eq!(v, 4),
                _ => unreachable!(),
            }
        }

        assert_eq!((&map).into_iter().count(), 4);
    }
}