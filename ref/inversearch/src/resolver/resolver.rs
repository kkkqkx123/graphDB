use crate::r#type::{IntermediateSearchResults, SearchResults, SearchOptions as TypeSearchOptions};

#[derive(Debug, Clone)]
pub enum ResolverError {
    IndexNotSet,
    QueryExecutionFailed(String),
    InvalidOptions(String),
    EmptyResult,
    InvalidParameter(String),
}

impl std::fmt::Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolverError::IndexNotSet => write!(f, "Index is not set"),
            ResolverError::QueryExecutionFailed(msg) => write!(f, "Query execution failed: {}", msg),
            ResolverError::InvalidOptions(msg) => write!(f, "Invalid options: {}", msg),
            ResolverError::EmptyResult => write!(f, "Result is empty"),
            ResolverError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
        }
    }
}

impl std::error::Error for ResolverError {}

impl From<crate::InversearchError> for ResolverError {
    fn from(error: crate::InversearchError) -> Self {
        ResolverError::QueryExecutionFailed(error.to_string())
    }
}

pub type ResolverResult<T> = Result<T, ResolverError>;

#[derive(Clone)]
pub struct ResolverOptions {
    pub options: TypeSearchOptions,
    pub index: Option<crate::Index>,
}

impl Default for ResolverOptions {
    fn default() -> Self {
        ResolverOptions {
            options: TypeSearchOptions::default(),
            index: None,
        }
    }
}

impl ResolverOptions {
    pub fn new() -> Self {
        ResolverOptions::default()
    }

    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.options.query = Some(query.into());
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.options.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.options.offset = Some(offset);
        self
    }

    pub fn with_enrich(mut self, enrich: bool) -> Self {
        self.options.enrich = Some(enrich);
        self
    }

    pub fn with_boost(mut self, boost: i32) -> Self {
        self.options.boost = Some(boost);
        self
    }

    pub fn with_index(mut self, index: crate::Index) -> Self {
        self.index = Some(index);
        self
    }

    pub fn query(&self) -> Option<&String> {
        self.options.query.as_ref()
    }

    pub fn limit(&self) -> Option<usize> {
        self.options.limit
    }

    pub fn offset(&self) -> Option<usize> {
        self.options.offset
    }

    pub fn enrich(&self) -> Option<bool> {
        self.options.enrich
    }

    pub fn boost(&self) -> Option<i32> {
        self.options.boost
    }

    pub fn resolve(&self) -> Option<bool> {
        self.options.resolve
    }

    pub fn suggest(&self) -> Option<bool> {
        self.options.suggest
    }

    pub fn index(&self) -> Option<&crate::Index> {
        self.index.as_ref()
    }

    pub fn into_options(self) -> TypeSearchOptions {
        self.options
    }
}

#[derive(Clone)]
pub struct Resolver {
    pub index: Option<crate::Index>,
    pub result: IntermediateSearchResults,
    pub boostval: i32,
    pub resolved: bool,
    pub options: Option<ResolverOptions>,
}

impl std::fmt::Debug for Resolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resolver")
            .field("index", &self.index.as_ref().map(|_| "Index"))
            .field("result", &self.result)
            .field("boostval", &self.boostval)
            .field("resolved", &self.resolved)
            .finish()
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Resolver {
            index: None,
            result: Vec::new(),
            boostval: 0,
            resolved: false,
            options: None,
        }
    }
}

impl Resolver {
    pub fn new(result: IntermediateSearchResults, index: Option<crate::Index>) -> Self {
        Resolver {
            index,
            result,
            boostval: 0,
            resolved: false,
            options: None,
        }
    }

    pub fn from_options(options: ResolverOptions, index: Option<crate::Index>) -> ResolverResult<Self> {
        let boost = options.boost().unwrap_or(0);

        let mut resolver = Resolver {
            index: options.index().cloned().or(index),
            result: Vec::new(),
            boostval: boost,
            resolved: false,
            options: Some(options),
        };

        if let Some(ref query) = resolver.options.as_ref().unwrap().query() {
            if query.is_empty() {
                resolver.result = vec![];
            } else {
                resolver.result = vec![vec![]];
            }
        } else {
            resolver.result = vec![];
        }

        Ok(resolver)
    }

    pub fn limit(&mut self, limit: usize) -> &mut Self {
        if !self.result.is_empty() {
            let mut final_result: IntermediateSearchResults = Vec::new();
            let mut remaining_limit = limit;

            for ids in &self.result {
                if ids.is_empty() {
                    continue;
                }

                if ids.len() <= remaining_limit {
                    final_result.push(ids.clone());
                    remaining_limit -= ids.len();
                } else if remaining_limit > 0 {
                    final_result.push(ids[..remaining_limit].to_vec());
                    break;
                }
            }

            self.result = final_result;
        }
        self
    }

    pub fn offset(&mut self, offset: usize) -> &mut Self {
        if !self.result.is_empty() && offset > 0 {
            let mut final_result: IntermediateSearchResults = Vec::new();
            let mut current_offset = offset;

            for ids in &self.result {
                if ids.is_empty() {
                    continue;
                }

                if ids.len() <= current_offset {
                    current_offset -= ids.len();
                } else {
                    final_result.push(ids[current_offset..].to_vec());
                }
            }

            self.result = final_result;
        }
        self
    }

    pub fn boost(&mut self, boost: i32) -> &mut Self {
        self.boostval += boost;
        self
    }

    pub fn get(&mut self) -> SearchResults {
        if !self.resolved {
            self.resolved = true;
            
            if self.result.is_empty() {
                return Vec::new();
            }
            
            let mut flattened = Vec::new();
            for array in &self.result {
                flattened.extend_from_slice(array);
            }
            
            flattened
        } else {
            Vec::new()
        }
    }

    pub fn resolve(&mut self, limit: Option<usize>, offset: Option<usize>, enrich: bool) -> SearchResults {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        
        if self.result.is_empty() {
            self.resolved = true;
            return Vec::new();
        }
        
        let result = resolve_default(&self.result, limit, offset, enrich);
        
        self.resolved = true;
        
        result
    }

    pub fn and(&mut self, other: IntermediateSearchResults) -> &mut Self {
        if !self.result.is_empty() && !other.is_empty() {
            let current = self.result.clone();
            let arrays = vec![current, other];
            
            let simple_arrays: Vec<Vec<u64>> = arrays.into_iter().flatten().collect();
            let intersection_result = crate::intersect::core::intersect_simple(&simple_arrays);
            
            self.result = vec![intersection_result];
        } else if !self.result.is_empty() {
        } else if !other.is_empty() {
            self.result = other;
        }
        self
    }

    pub fn or(&mut self, other: IntermediateSearchResults) -> &mut Self {
        if !self.result.is_empty() && !other.is_empty() {
            let mut combined = self.result.clone();
            combined.extend(other);
            
            let mut seen = std::collections::HashMap::new();
            let mut unique_result = Vec::new();
            
            for array in combined {
                let mut unique_array = Vec::new();
                for &id in &array {
                    if !seen.contains_key(&id) {
                        seen.insert(id, true);
                        unique_array.push(id);
                    }
                }
                if !unique_array.is_empty() {
                    unique_result.push(unique_array);
                }
            }
            
            self.result = unique_result;
        } else if self.result.is_empty() {
            self.result = other;
        }
        self
    }

    pub fn not(&mut self, other: IntermediateSearchResults) -> &mut Self {
        if !self.result.is_empty() {
            let current_flat = self.flatten_results();
            let other_flat: SearchResults = other.into_iter().flatten().collect();
            
            let mut check = std::collections::HashMap::new();
            for &id in &other_flat {
                check.insert(id, true);
            }

            let mut result: SearchResults = Vec::new();
            for &id in &current_flat {
                if !check.contains_key(&id) {
                    result.push(id);
                }
            }

            self.result = vec![result];
        }
        self
    }

    pub fn xor(&mut self, other: IntermediateSearchResults) -> &mut Self {
        if !self.result.is_empty() && !other.is_empty() {
            let mut counts: std::collections::HashMap<u64, u32> = std::collections::HashMap::new();
            
            for ids in &self.result {
                for &id in ids {
                    *counts.entry(id).or_insert(0) += 1;
                }
            }
            
            for ids in &other {
                for &id in ids {
                    *counts.entry(id).or_insert(0) += 1;
                }
            }
            
            let mut xor_result: SearchResults = Vec::new();
            for ids in &self.result {
                for &id in ids {
                    if counts.get(&id) == Some(&1) {
                        xor_result.push(id);
                    }
                }
            }
            
            for ids in &other {
                for &id in ids {
                    if counts.get(&id) == Some(&1) {
                        if !xor_result.contains(&id) {
                            xor_result.push(id);
                        }
                    }
                }
            }
            
            self.result = vec![xor_result];
        } else if !self.result.is_empty() {
        } else if !other.is_empty() {
            self.result = other;
        }
        self
    }

    fn flatten_results(&self) -> SearchResults {
        let mut flattened = Vec::new();
        for array in &self.result {
            flattened.extend_from_slice(array);
        }
        flattened
    }

    pub fn with_result(mut self, result: IntermediateSearchResults) -> Self {
        self.result = result;
        self
    }

    pub fn with_index(mut self, index: crate::Index) -> Self {
        self.index = Some(index);
        self
    }
}

pub fn resolve_default(
    result: &IntermediateSearchResults,
    limit: usize,
    offset: usize,
    _enrich: bool,
) -> SearchResults {
    if result.is_empty() {
        return Vec::new();
    }

    if result.len() == 1 {
        let mut final_result = result[0].clone();
        if offset > 0 || final_result.len() > limit {
            final_result = final_result.into_iter().skip(offset).take(limit).collect();
        }
        return final_result;
    }

    let mut final_result: SearchResults = Vec::new();
    let mut current_limit = limit;
    let mut current_offset = offset;

    for arr in result {
        if arr.is_empty() {
            continue;
        }

        let mut processed_arr = arr.clone();

        if current_offset > 0 {
            if current_offset >= processed_arr.len() {
                current_offset -= processed_arr.len();
                continue;
            }
            processed_arr = processed_arr[current_offset..].to_vec();
            current_offset = 0;
        }

        if processed_arr.len() > current_limit {
            processed_arr = processed_arr[..current_limit].to_vec();
        }

        if final_result.is_empty() && processed_arr.len() >= current_limit {
            return processed_arr;
        }

        final_result.extend_from_slice(&processed_arr);
        current_limit -= processed_arr.len();

        if current_limit == 0 {
            break;
        }
    }

    if final_result.is_empty() {
        return Vec::new();
    }

    final_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#type::SearchOptions;

    #[test]
    fn test_resolver_new() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3]];
        let resolver = Resolver::new(result, None);
        assert_eq!(resolver.result.len(), 1);
        assert_eq!(resolver.boostval, 0);
        assert!(!resolver.resolved);
    }

    #[test]
    fn test_resolver_from_options_basic() {
        let mut options = ResolverOptions::default();
        options.options.query = Some("test".to_string());
        options.options.limit = Some(10);
        options.options.offset = Some(5);
        
        let resolver = Resolver::from_options(options, None).unwrap();
        assert_eq!(resolver.boostval, 0);
        assert!(!resolver.result.is_empty());
    }

    #[test]
    fn test_resolver_limit() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
        let mut resolver = Resolver::new(result, None);
        resolver.limit(3);
        
        let flattened = resolver.get();
        assert_eq!(flattened.len(), 3);
        assert_eq!(flattened, vec![1, 2, 3]);
    }

    #[test]
    fn test_resolver_offset() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
        let mut resolver = Resolver::new(result, None);
        resolver.offset(2);
        
        let flattened = resolver.get();
        assert_eq!(flattened.len(), 3);
        assert_eq!(flattened, vec![3, 4, 5]);
    }

    #[test]
    fn test_resolver_boost() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3]];
        let mut resolver = Resolver::new(result, None);
        resolver.boost(5).boost(3);
        
        assert_eq!(resolver.boostval, 8);
    }

    #[test]
    fn test_resolve_default_single_array() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
        let resolved = resolve_default(&result, 3, 1, false);
        assert_eq!(resolved, vec![2, 3, 4]);
    }

    #[test]
    fn test_resolve_default_multiple_arrays() {
        let result: IntermediateSearchResults = vec![vec![1, 2], vec![3, 4, 5]];
        let resolved = resolve_default(&result, 10, 0, false);
        assert_eq!(resolved, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_resolve_default_with_offset() {
        let result: IntermediateSearchResults = vec![vec![1, 2], vec![3, 4, 5]];
        let resolved = resolve_default(&result, 10, 2, false);
        assert_eq!(resolved, vec![3, 4, 5]);
    }

    #[test]
    fn test_resolve_default_with_limit() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
        let resolved = resolve_default(&result, 3, 0, false);
        assert_eq!(resolved, vec![1, 2, 3]);
    }
}
