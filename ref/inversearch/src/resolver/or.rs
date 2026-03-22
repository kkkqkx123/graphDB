use crate::r#type::IntermediateSearchResults;

pub fn union_op(arrays: Vec<IntermediateSearchResults>, _boost: i32) -> IntermediateSearchResults {
    if arrays.is_empty() {
        return vec![];
    }

    if arrays.len() == 1 {
        return arrays[0].clone();
    }

    let mut seen = std::collections::HashMap::new();
    let mut unique_result: IntermediateSearchResults = Vec::new();
    let mut has_overlap = false;

    for array in arrays {
        let mut unique_array: Vec<u64> = Vec::new();
        for ids in array {
            for &id in &ids {
                if !seen.contains_key(&id) {
                    seen.insert(id, true);
                    unique_array.push(id);
                } else {
                    has_overlap = true;
                }
            }
        }
        if !unique_array.is_empty() {
            unique_result.push(unique_array);
        }
    }

    if has_overlap {
        let mut merged: Vec<u64> = Vec::new();
        for array in unique_result {
            merged.extend(array);
        }
        merged.sort();
        vec![merged]
    } else {
        unique_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union_op_empty() {
        let arrays: Vec<IntermediateSearchResults> = vec![];
        let result = union_op(arrays, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_union_op_single() {
        let inner: IntermediateSearchResults = vec![vec![1, 2, 3]];
        let arrays: Vec<IntermediateSearchResults> = vec![inner];
        let result = union_op(arrays, 0);
        let expected: IntermediateSearchResults = vec![vec![1, 2, 3]];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_union_op_multiple_no_overlap() {
        let part1: IntermediateSearchResults = vec![vec![1, 2, 3]];
        let part2: IntermediateSearchResults = vec![vec![4, 5, 6]];
        let arrays: Vec<IntermediateSearchResults> = vec![part1, part2];
        let result = union_op(arrays, 0);
        let expected: IntermediateSearchResults = vec![vec![1, 2, 3], vec![4, 5, 6]];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_union_op_with_overlap() {
        let part1: IntermediateSearchResults = vec![vec![1, 2, 3]];
        let part2: IntermediateSearchResults = vec![vec![3, 4, 5]];
        let arrays: Vec<IntermediateSearchResults> = vec![part1, part2];
        let result = union_op(arrays, 0);
        let expected: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_union_op_multiple_arrays_with_overlap() {
        let part1: IntermediateSearchResults = vec![vec![1, 2]];
        let part2: IntermediateSearchResults = vec![vec![2, 3]];
        let part3: IntermediateSearchResults = vec![vec![3, 4]];
        let arrays: Vec<IntermediateSearchResults> = vec![part1, part2, part3];
        let result = union_op(arrays, 0);
        let expected: IntermediateSearchResults = vec![vec![1, 2, 3, 4]];
        assert_eq!(result, expected);
    }
}
