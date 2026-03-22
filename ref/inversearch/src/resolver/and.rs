use crate::r#type::IntermediateSearchResults;
use crate::intersect::core::intersect_simple;

pub fn intersect_and(arrays: Vec<IntermediateSearchResults>, limit: usize) -> IntermediateSearchResults {
    if arrays.is_empty() {
        return vec![];
    }

    if arrays.len() == 1 {
        return arrays[0].clone();
    }

    let mut resolution = 0;
    for arr in &arrays {
        let len = arr.iter().map(|a| a.len()).sum();
        if resolution < len {
            resolution = len;
        } else if resolution == 0 {
            return vec![];
        }
    }

    if resolution == 0 {
        return vec![];
    }

    let flat_arrays: Vec<Vec<u64>> = arrays.into_iter().flatten().collect();
    let intersection = intersect_simple(&flat_arrays);

    vec![intersection]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersect_and_empty() {
        let arrays: Vec<IntermediateSearchResults> = vec![];
        let result = intersect_and(arrays, 100);
        assert!(result.is_empty());
    }

    #[test]
    fn test_intersect_and_single() {
        let arrays: Vec<IntermediateSearchResults> = vec![vec![vec![1, 2, 3]]];
        let result = intersect_and(arrays, 100);
        assert_eq!(result, vec![vec![1, 2, 3]]);
    }

    #[test]
    fn test_intersect_and_multiple() {
        let arrays: Vec<IntermediateSearchResults> = vec![
            vec![vec![1, 2, 3]],
            vec![vec![2, 3, 4]],
        ];
        let result = intersect_and(arrays, 100);
        assert_eq!(result, vec![vec![2, 3]]);
    }

    #[test]
    fn test_intersect_and_no_overlap() {
        let arrays: Vec<IntermediateSearchResults> = vec![
            vec![vec![1, 2, 3]],
            vec![vec![4, 5, 6]],
        ];
        let result = intersect_and(arrays, 100);
        assert!(result[0].is_empty());
    }
}
