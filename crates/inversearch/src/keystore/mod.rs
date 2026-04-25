use std::collections::{HashMap, HashSet};

pub type DocId = u64;
pub type ResolutionSlot = Vec<DocId>;
pub type TermIndex = HashMap<usize, ResolutionSlot>;
pub type ContextIndex = HashMap<String, TermIndex>;

#[derive(Clone, Debug)]
pub struct KeystoreMap<K, V> {
    pub index: HashMap<usize, HashMap<K, V>>,
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
            size: 0,
            bit: bitlength,
        }
    }

    fn crc(&self, key: &K) -> usize {
        // Uses the same hash algorithm as Index::keystore_hash_str
        let key_str = key.to_string();
        let mut crc: u32 = 0;
        for c in key_str.chars() {
            crc = (crc << 8) ^ (crc >> (32 - 8)) ^ (c as u32);
        }
        (crc as usize) % (1 << self.bit)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let address = self.crc(key);
        self.index.get(&address).and_then(|map| map.get(key))
    }

    pub fn set(&mut self, key: K, value: V) {
        let address = self.crc(&key);
        let map = self.index.entry(address).or_default();
        let old_size = map.len();
        map.insert(key, value);
        if map.len() > old_size {
            self.size += 1;
        }
    }

    pub fn batch_set(&mut self, items: Vec<(K, V)>) {
        for (key, value) in items {
            let address = self.crc(&key);
            let map = self.index.entry(address).or_default();
            let old_size = map.len();
            map.insert(key, value);
            if map.len() > old_size {
                self.size += 1;
            }
        }
    }

    pub fn has(&self, key: &K) -> bool {
        let address = self.crc(key);
        self.index
            .get(&address)
            .is_some_and(|map| map.contains_key(key))
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
        self.size = 0;
    }

    pub fn keys(&self) -> Vec<K> {
        let mut keys = Vec::new();
        for map in self.index.values() {
            for key in map.keys() {
                keys.push(key.clone());
            }
        }
        keys
    }

    pub fn values(&self) -> Vec<&V> {
        let mut values = Vec::new();
        for map in self.index.values() {
            for value in map.values() {
                values.push(value);
            }
        }
        values
    }

    pub fn entries(&self) -> Vec<(K, &V)> {
        let mut entries = Vec::new();
        for map in self.index.values() {
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
        self.size = 0;
    }
}

#[derive(Clone, Debug)]
pub struct KeystoreSet<T> {
    pub index: HashMap<usize, HashSet<T>>,
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
            size: 0,
            bit: bitlength,
        }
    }

    fn crc(&self, key: &T) -> usize {
        // Uses the same hash algorithm as Index::keystore_hash_str
        let key_str = key.to_string();
        let mut crc: u32 = 0;
        for c in key_str.chars() {
            crc = (crc << 8) ^ (crc >> (32 - 8)) ^ (c as u32);
        }
        (crc as usize) % (1 << self.bit)
    }

    pub fn add(&mut self, key: T) {
        let address = self.crc(&key);
        let set = self.index.entry(address).or_default();
        let old_size = set.len();
        set.insert(key);
        if set.len() > old_size {
            self.size += 1;
        }
    }

    pub fn batch_add(&mut self, items: Vec<T>) {
        for key in items {
            let address = self.crc(&key);
            let set = self.index.entry(address).or_default();
            let old_size = set.len();
            set.insert(key);
            if set.len() > old_size {
                self.size += 1;
            }
        }
    }

    pub fn has(&self, key: &T) -> bool {
        let address = self.crc(key);
        self.index
            .get(&address)
            .is_some_and(|set| set.contains(key))
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
        self.size = 0;
    }

    pub fn keys(&self) -> Vec<T> {
        self.index
            .values()
            .flat_map(|set| set.iter().cloned())
            .collect()
    }

    pub fn values(&self) -> Vec<T> {
        self.index
            .values()
            .flat_map(|set| set.iter().cloned())
            .collect()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn values_iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.index.values().flat_map(|set| set.iter())
    }

    pub fn destroy(&mut self) {
        self.index.clear();
        self.size = 0;
    }
}

#[derive(Clone, Debug)]
pub struct KeystoreArray<T> {
    pub data: Vec<T>,
    pub size: usize,
}

impl<T> KeystoreArray<T>
where
    T: Clone,
{
    pub fn new(_bitlength: usize) -> Self {
        KeystoreArray {
            data: Vec::new(),
            size: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        self.data.push(value);
        self.size += 1;
    }

    pub fn batch_push(&mut self, items: Vec<T>) {
        self.data.extend(items);
        self.size = self.data.len();
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn set(&mut self, index: usize, value: T) {
        if index < self.data.len() {
            self.data[index] = value;
        } else if index == self.data.len() {
            self.data.push(value);
            self.size += 1;
        } else {
            // Extend with default values if needed
            while self.data.len() < index {
                self.data.push(value.clone());
            }
            self.data.push(value);
            self.size = self.data.len();
        }
    }

    pub fn length(&self) -> usize {
        self.size
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.size = 0;
    }

    pub fn includes(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.data.contains(value)
    }

    pub fn index_of(&self, value: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        self.data.iter().position(|x| x == value)
    }

    pub fn pop(&mut self) -> Option<T> {
        let result = self.data.pop();
        if result.is_some() {
            self.size -= 1;
        }
        result
    }

    pub fn slice(&self, start: usize, end: usize) -> Vec<T>
    where
        T: Clone,
    {
        let start = start.min(self.data.len());
        let end = end.min(self.data.len());
        self.data[start..end].to_vec()
    }

    pub fn destroy(&mut self) {
        self.data.clear();
        self.size = 0;
    }
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

        assert!(map.has(&"key1".to_string()));
        assert!(!map.has(&"key3".to_string()));

        map.delete(&"key1".to_string());
        assert!(!map.has(&"key1".to_string()));
    }

    #[test]
    fn test_keystore_set() {
        let mut set = KeystoreSet::new(8);
        set.add(1);
        set.add(2);
        set.add(3);

        assert!(set.has(&1));
        assert!(set.has(&2));
        assert!(!set.has(&4));

        set.delete(&2);
        assert!(!set.has(&2));
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
        assert!(arr.includes(&2));
        assert!(!arr.includes(&4));
        assert_eq!(arr.index_of(&2), Some(1));

        arr.push(4);
        assert_eq!(arr.length(), 4);
        assert!(arr.includes(&4));

        assert_eq!(arr.pop(), Some(4));
        assert_eq!(arr.length(), 3);

        let slice = arr.slice(1, 3);
        assert_eq!(slice, vec![2, 3]);
    }
}
