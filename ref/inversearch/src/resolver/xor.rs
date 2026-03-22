use crate::r#type::IntermediateSearchResults;

pub fn xor_op(arrays: Vec<IntermediateSearchResults>, boost: i32) -> IntermediateSearchResults {
    if arrays.is_empty() {
        return vec![];
    }

    if arrays.len() == 1 {
        return arrays[0].clone();
    }

    let mut counts: std::collections::HashMap<u64, u32> = std::collections::HashMap::new();
    let mut max_len = 0;

    for arr in &arrays {
        let arr_len: usize = arr.iter().map(|a| a.len()).sum();
        if max_len < arr_len {
            max_len = arr_len;
        }
        for ids in arr {
            for &id in ids {
                *counts.entry(id).or_insert(0) += 1;
            }
        }
    }

    let mut result: IntermediateSearchResults = Vec::new();
    for arr in arrays {
        let mut xor_array: Vec<u64> = Vec::new();
        for ids in arr {
            for &id in &ids {
                if let Some(&1) = counts.get(&id) {
                    xor_array.push(id);
                }
            }
        }
        if !xor_array.is_empty() {
            result.push(xor_array);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_op_empty() {
        let arrays: Vec<IntermediateSearchResults> = vec![];
        let result = xor_op(arrays, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_xor_op_single() {
        let arrays: Vec<IntermediateSearchResults> = vec![vec![vec![1, 2, 3]]];
        let result = xor_op(arrays, 0);
        assert_eq!(result, vec![vec![1, 2, 3]]);
    }

    #[test]
    fn test_xor_op_no_overlap() {
        let arrays: Vec<IntermediateSearchResults> = vec![
            vec![vec![1, 2]],
            vec![vec![3, 4]],
        ];
        let result = xor_op(arrays, 0);
        assert_eq!(result, vec![vec![1, 2], vec![3, 4]]);
    }

    #[test]
    fn test_xor_op_with_overlap() {
        let arrays: Vec<IntermediateSearchResults> = vec![
            vec![vec![1, 2, 3]],
            vec![vec![2, 3, 4]],
        ];
        let result = xor_op(arrays, 0);
        assert_eq!(result, vec![vec![1], vec![4]]);
    }

    #[test]
    fn test_xor_op_all_overlap() {
        let arrays: Vec<IntermediateSearchResults> = vec![
            vec![vec![1, 2]],
            vec![vec![1, 2]],
        ];
        let result = xor_op(arrays, 0);
        assert!(result.iter().all(|arr| arr.is_empty()));
    }
}
