use crate::r#type::IntermediateSearchResults;

pub fn combine_search_results(results: Vec<IntermediateSearchResults>) -> IntermediateSearchResults {
    if results.is_empty() {
        return vec![];
    }

    let mut combined: IntermediateSearchResults = Vec::new();
    
    for result in results {
        for arr in result {
            if !arr.is_empty() {
                combined.push(arr);
            }
        }
    }
    
    combined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_empty() {
        let results: Vec<IntermediateSearchResults> = vec![];
        let combined = combine_search_results(results);
        assert!(combined.is_empty());
    }

    #[test]
    fn test_combine_single() {
        let inner: IntermediateSearchResults = vec![vec![1, 2, 3]];
        let results: Vec<IntermediateSearchResults> = vec![inner];
        let combined = combine_search_results(results);
        let expected: IntermediateSearchResults = vec![vec![1, 2, 3]];
        assert_eq!(combined, expected);
    }

    #[test]
    fn test_combine_multiple() {
        let part1: IntermediateSearchResults = vec![vec![1, 2]];
        let part2: IntermediateSearchResults = vec![vec![3, 4], vec![5, 6]];
        let results: Vec<IntermediateSearchResults> = vec![part1, part2];
        let combined = combine_search_results(results);
        let expected: IntermediateSearchResults = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        assert_eq!(combined, expected);
    }

    #[test]
    fn test_combine_with_empty() {
        let part1: IntermediateSearchResults = vec![vec![1, 2]];
        let part2: IntermediateSearchResults = vec![];
        let part3: IntermediateSearchResults = vec![vec![3, 4]];
        let results: Vec<IntermediateSearchResults> = vec![part1, part2, part3];
        let combined = combine_search_results(results);
        let expected: IntermediateSearchResults = vec![vec![1, 2], vec![3, 4]];
        assert_eq!(combined, expected);
    }
}
