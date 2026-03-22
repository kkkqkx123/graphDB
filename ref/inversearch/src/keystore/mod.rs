use std::collections::{HashMap, HashSet};

pub type DocId = u64;
pub type ResolutionSlot = Vec<DocId>;
pub type TermIndex = HashMap<usize, ResolutionSlot>;
pub type ContextIndex = HashMap<String, TermIndex>;

#[derive(Clone, Debug)]
pub struct KeystoreMap<K, V> {
    pub index: HashMap<usize, HashMap<K, V>>,
    pub refs: Vec<HashMap<K, V>>,
    pub size: usize,
    pub bit: usize,
}

impl<K, V> KeystoreMap<K, V>
where
    K: std::hash::Hash + Eq + Clone + std::fmt::Display,
    V: Clone,
{
    pub fn new(bitlength: usize) -> Self {
        KeystoreMap {
            index: HashMap::new(),
            refs: Vec::new(),
            size: 0,
            bit: bitlength,
        }
    }

    fn crc(&self, key: &K) -> usize {
        let key_str = key.to_string();
        if self.bit > 32 {
            lcg64(&key_str, self.bit as u32)
        } else {
            lcg(&key_str, self.bit as u32)
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let address = self.crc(key);
        self.index.get(&address).and_then(|map| map.get(key))
    }

    pub fn set(&mut self, key: K, value: V) {
        let address = self.crc(&key);
        let map = self.index.entry(address).or_insert_with(HashMap::new);
        let old_size = map.len();
        map.insert(key.clone(), value);
        if map.len() > old_size {
            self.size += 1;
            self.refs.push(map.clone());
        }
    }

    pub fn has(&self, key: &K) -> bool {
        let address = self.crc(key);
        self.index.get(&address).map_or(false, |map| map.contains_key(key))
    }

    pub fn delete(&mut self, key: &K) -> bool {
        let address = self.crc(key);
        if let Some(map) = self.index.get_mut(&address) {
            let removed = map.remove(key).is_some();
            if removed {
                self.size -= 1;
            }
            removed
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.index.clear();
        self.refs.clear();
        self.size = 0;
    }

    pub fn keys(&self) -> Vec<K> {
        let mut keys = Vec::new();
        for (_, map) in &self.index {
            for key in map.keys() {
                keys.push(key.clone());
            }
        }
        keys
    }

    pub fn values(&self) -> Vec<&V> {
        let mut values = Vec::new();
        for (_, map) in &self.index {
            for value in map.values() {
                values.push(value);
            }
        }
        values
    }

    pub fn entries(&self) -> Vec<(K, &V)> {
        let mut entries = Vec::new();
        for (_, map) in &self.index {
            for (key, value) in map {
                entries.push((key.clone(), value));
            }
        }
        entries
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn values_iter(&self) -> impl Iterator<Item = &V> + '_ {
        self.index.values().flat_map(|map| map.values())
    }

    pub fn keys_iter(&self) -> impl Iterator<Item = &K> + '_ {
        self.index.values().flat_map(|map| map.keys())
    }

    pub fn entries_iter(&self) -> impl Iterator<Item = (&K, &V)> + '_ {
        self.index.values().flat_map(|map| map.iter())
    }

    pub fn destroy(&mut self) {
        self.index.clear();
        self.refs.clear();
        self.size = 0;
    }
}

#[derive(Clone, Debug)]
pub struct KeystoreSet<T> {
    pub index: HashMap<usize, HashSet<T>>,
    pub refs: Vec<HashSet<T>>,
    pub size: usize,
    pub bit: usize,
}

impl<T> KeystoreSet<T>
where
    T: std::hash::Hash + Eq + Clone + std::fmt::Display,
{
    pub fn new(bitlength: usize) -> Self {
        KeystoreSet {
            index: HashMap::new(),
            refs: Vec::new(),
            size: 0,
            bit: bitlength,
        }
    }

    fn crc(&self, key: &T) -> usize {
        let key_str = key.to_string();
        if self.bit > 32 {
            lcg64(&key_str, self.bit as u32)
        } else {
            lcg(&key_str, self.bit as u32)
        }
    }

    pub fn add(&mut self, key: T) {
        let address = self.crc(&key);
        let set = self.index.entry(address).or_insert_with(HashSet::new);
        let old_size = set.len();
        set.insert(key.clone());
        if set.len() > old_size {
            self.size += 1;
            self.refs.push(set.clone());
        }
    }

    pub fn has(&self, key: &T) -> bool {
        let address = self.crc(key);
        self.index.get(&address).map_or(false, |set| set.contains(key))
    }

    pub fn delete(&mut self, key: &T) -> bool {
        let address = self.crc(key);
        if let Some(set) = self.index.get_mut(&address) {
            let removed = set.remove(key);
            if removed {
                self.size -= 1;
            }
            removed
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.index.clear();
        self.refs.clear();
        self.size = 0;
    }

    pub fn keys(&self) -> Vec<T> {
        let mut keys = Vec::new();
        for set in self.index.values() {
            for key in set {
                keys.push(key.clone());
            }
        }
        keys
    }

    pub fn values(&self) -> Vec<T> {
        let mut values = Vec::new();
        for set in self.index.values() {
            for value in set {
                values.push(value.clone());
            }
        }
        values
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn values_iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.index.values().flat_map(|set| set.iter())
    }

    pub fn destroy(&mut self) {
        self.index.clear();
        self.refs.clear();
        self.size = 0;
    }
}

#[derive(Clone, Debug)]
pub struct KeystoreArray<T> {
    pub index: HashMap<usize, Vec<T>>,
    pub refs: Vec<Vec<T>>,
    pub size: usize,
    pub bit: usize,
}

impl<T> KeystoreArray<T>
where
    T: Clone,
{
    pub fn new(bitlength: usize) -> Self {
        KeystoreArray {
            index: HashMap::new(),
            refs: Vec::new(),
            size: 0,
            bit: bitlength,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.refs.is_empty() {
            self.refs.push(Vec::new());
        }
        if let Some(last_vec) = self.refs.last_mut() {
            last_vec.push(value);
            self.size += 1;
        } else {
            eprintln!("Keystore: Failed to get last reference vector");
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let mut global_index = 0;
        for vec in &self.refs {
            if index < global_index + vec.len() {
                return Some(&vec[index - global_index]);
            }
            global_index += vec.len();
        }
        None
    }

    pub fn set(&mut self, index: usize, value: T) {
        let mut global_index = 0;
        for vec in &mut self.refs {
            if index < global_index + vec.len() {
                vec[index - global_index] = value;
                return;
            }
            global_index += vec.len();
        }
        // If index is beyond current size, extend with new arrays
        while global_index <= index {
            self.refs.push(Vec::new());
            global_index += 1 << self.bit;
        }
        let last_vec = match self.refs.last_mut() {
            Some(vec) => vec,
            None => {
                eprintln!("Keystore: No reference vector available");
                return;
            }
        };
        let local_index = index - (global_index - (1 << self.bit));
        if local_index < last_vec.len() {
            last_vec[local_index] = value;
        } else {
            last_vec.resize(local_index + 1, value.clone());
            last_vec[local_index] = value;
        }
    }

    pub fn length(&self) -> usize {
        self.size
    }

    pub fn clear(&mut self) {
        self.index.clear();
        self.refs.clear();
        self.size = 0;
    }

    pub fn includes(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        for vec in &self.refs {
            if vec.contains(value) {
                return true;
            }
        }
        false
    }

    pub fn index_of(&self, value: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        let mut global_index = 0;
        for vec in &self.refs {
            if let Some(local_index) = vec.iter().position(|x| x == value) {
                return Some(global_index + local_index);
            }
            global_index += vec.len();
        }
        None
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }
        
        if let Some(last_vec) = self.refs.last_mut() {
            let result = last_vec.pop();
            if last_vec.is_empty() {
                self.refs.pop();
            }
            self.size -= 1;
            result
        } else {
            None
        }
    }

    pub fn slice(&self, start: usize, end: usize) -> Vec<T>
    where
        T: Clone,
    {
        let mut result = Vec::new();
        let mut global_index = 0;
        
        // Iterate through refs in order to maintain correct sequence
        for vec in &self.refs {
            for item in vec {
                if global_index >= start && global_index < end {
                    result.push(item.clone());
                }
                global_index += 1;
                if global_index >= end {
                    return result;
                }
            }
        }
        result
    }

    pub fn destroy(&mut self) {
        self.index.clear();
        self.refs.clear();
        self.size = 0;
    }
}

fn lcg(key: &str, bit: u32) -> usize {
    let mut hash: u32 = 0;
    for c in key.chars() {
        hash = (hash << 8) ^ (hash >> (32 - 8)) ^ (c as u32);
    }
    (hash % (1 << bit)) as usize
}

fn lcg64(key: &str, bit: u32) -> usize {
    let mut hash: u64 = 0;
    for c in key.chars() {
        hash = (hash << 8) ^ (hash >> (64 - 8)) ^ (c as u64);
    }
    (hash % (1 << bit)) as usize
}

fn lcg_for_number<T: Into<usize>>(num: T, bit: u32) -> usize {
    let num_val: usize = num.into();
    num_val & ((1 << bit) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keystore_map() {
        let mut map = KeystoreMap::new(8);
        map.set("key1".to_string(), "value1".to_string());
        map.set("key2".to_string(), "value2".to_string());
        
        assert_eq!(map.get(&"key1".to_string()), Some(&"value1".to_string()));
        assert_eq!(map.get(&"key2".to_string()), Some(&"value2".to_string()));
        assert_eq!(map.get(&"key3".to_string()), None);
        
        assert_eq!(map.has(&"key1".to_string()), true);
        assert_eq!(map.has(&"key3".to_string()), false);
        
        map.delete(&"key1".to_string());
        assert_eq!(map.has(&"key1".to_string()), false);
    }

    #[test]
    fn test_keystore_set() {
        let mut set = KeystoreSet::new(8);
        set.add(1);
        set.add(2);
        set.add(3);
        
        assert_eq!(set.has(&1), true);
        assert_eq!(set.has(&2), true);
        assert_eq!(set.has(&4), false);
        
        set.delete(&2);
        assert_eq!(set.has(&2), false);
    }

    #[test]
    fn test_keystore_array() {
        let mut arr = KeystoreArray::new(8);
        arr.push(1);
        arr.push(2);
        arr.push(3);
        
        assert_eq!(arr.get(0), Some(&1));
        assert_eq!(arr.get(1), Some(&2));
        assert_eq!(arr.get(2), Some(&3));
        assert_eq!(arr.get(3), None);
        
        assert_eq!(arr.length(), 3);
        assert_eq!(arr.includes(&2), true);
        assert_eq!(arr.includes(&4), false);
        assert_eq!(arr.index_of(&2), Some(1));

        arr.push(4);
        assert_eq!(arr.length(), 4);
        assert_eq!(arr.includes(&4), true);

        assert_eq!(arr.pop(), Some(4));
        assert_eq!(arr.length(), 3);

        let slice = arr.slice(1, 3);
        assert_eq!(slice, vec![2, 3]);
    }

    #[test]
    fn test_lcg() {
        let hash1 = lcg("hello", 8);
        let hash2 = lcg("hello", 8);
        let hash3 = lcg("world", 8);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}