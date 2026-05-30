use tantivy_stacker::ArenaHashMap;

const ALICE_WORDS: &[&str] = &[
    "Alice", "was", "beginning", "to", "get", "very", "tired", "of", "sitting",
    "by", "her", "sister", "on", "the", "bank", "and", "of", "having", "nothing",
    "to", "do", "once", "or", "twice", "she", "had", "peeped", "into", "the",
    "book", "her", "sister", "was", "reading",
];

fn main() {
    create_hash_map((0..100_000_000).map(|el| el.to_string()));

    for _ in 0..1000 {
        create_hash_map(ALICE_WORDS.iter().copied());
    }
}

fn create_hash_map<T: AsRef<str>>(terms: impl Iterator<Item = T>) -> ArenaHashMap {
    let mut map = ArenaHashMap::with_capacity(4);
    for term in terms {
        map.mutate_or_create(term.as_ref().as_bytes(), |val| {
            if let Some(mut val) = val {
                val += 1;
                val
            } else {
                1u64
            }
        });
    }

    map
}
