use crate::r#type::{IntermediateSearchResults, SearchResults};

pub fn exclusion(arrays: IntermediateSearchResults, exclude: &SearchResults, limit: usize) -> IntermediateSearchResults {
    let exclude_set: std::collections::HashSet<u64> = exclude.iter().cloned().collect();
    let mut result: IntermediateSearchResults = Vec::new();
    let mut count = 0;

    for ids in arrays {
        let mut filtered: Vec<u64> = Vec::new();
        for &id in &ids {
            if !exclude_set.contains(&id) {
                if count < limit {
                    filtered.push(id);
                    count += 1;
                }
            }
        }
        if !filtered.is_empty() {
            result.push(filtered);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exclusion_basic() {
        let arrays: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
        let exclude: SearchResults = vec![2, 3];
        let result = exclusion(arrays, &exclude, 100);
        assert_eq!(result, vec![vec![1, 4, 5]]);
    }

    #[test]
    fn test_exclusion_with_limit() {
        let arrays: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
        let exclude: SearchResults = vec![];
        let result = exclusion(arrays, &exclude, 4);
        assert_eq!(result, vec![vec![1, 2, 3, 4]]);
    }

    #[test]
    fn test_exclusion_all_excluded() {
        let arrays: IntermediateSearchResults = vec![vec![1, 2, 3]];
        let exclude: SearchResults = vec![1, 2, 3];
        let result = exclusion(arrays, &exclude, 100);
        assert!(result.is_empty() || result[0].is_empty());
    }

    #[test]
    fn test_exclusion_multiple_arrays() {
        let arrays: IntermediateSearchResults = vec![vec![1, 2], vec![3, 4, 5]];
        let exclude: SearchResults = vec![2, 4];
        let result = exclusion(arrays, &exclude, 100);
        assert_eq!(result, vec![vec![1], vec![3, 5]]);
    }
}
