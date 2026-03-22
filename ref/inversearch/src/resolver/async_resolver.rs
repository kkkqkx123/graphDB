use crate::resolver::{Resolver, ResolverOptions, ResolverResult, ResolverError};
use crate::r#type::{IntermediateSearchResults, SearchResults};

#[derive(Clone)]
pub struct AsyncResolver {
    resolver: Resolver,
}

impl AsyncResolver {
    pub fn new(resolver: Resolver) -> Self {
        AsyncResolver { resolver }
    }

    pub fn from_options(options: ResolverOptions) -> ResolverResult<Self> {
        let resolver = Resolver::from_options(options, None)?;
        Ok(AsyncResolver::new(resolver))
    }

    pub fn limit(&mut self, limit: usize) -> &mut Self {
        self.resolver.limit(limit);
        self
    }

    pub fn offset(&mut self, offset: usize) -> &mut Self {
        self.resolver.offset(offset);
        self
    }

    pub fn boost(&mut self, boost: i32) -> &mut Self {
        self.resolver.boost(boost);
        self
    }

    pub fn get(&mut self) -> SearchResults {
        self.resolver.get()
    }

    pub async fn resolve(&mut self, limit: Option<usize>, offset: Option<usize>, enrich: bool) -> SearchResults {
        self.resolver.resolve(limit, offset, enrich)
    }

    pub async fn resolve_with_callback<F>(&mut self, limit: Option<usize>, offset: Option<usize>, enrich: bool, callback: F)
    where
        F: FnOnce(SearchResults),
    {
        let result = self.resolve(limit, offset, enrich).await;
        callback(result);
    }

    pub fn and(&mut self, other: IntermediateSearchResults) -> &mut Self {
        self.resolver.and(other);
        self
    }

    pub fn or(&mut self, other: IntermediateSearchResults) -> &mut Self {
        self.resolver.or(other);
        self
    }

    pub fn not(&mut self, other: IntermediateSearchResults) -> &mut Self {
        self.resolver.not(other);
        self
    }

    pub fn xor(&mut self, other: IntermediateSearchResults) -> &mut Self {
        self.resolver.xor(other);
        self
    }

    pub fn with_index(mut self, index: crate::Index) -> Self {
        self.resolver.index = Some(index);
        self
    }

    pub fn with_result(mut self, result: IntermediateSearchResults) -> Self {
        self.resolver.result = result;
        self
    }

    pub fn into_inner(self) -> Resolver {
        self.resolver
    }

    pub fn borrow(&self) -> &Resolver {
        &self.resolver
    }

    pub fn borrow_mut(&mut self) -> &mut Resolver {
        &mut self.resolver
    }
}

#[cfg(feature = "async")]
impl AsyncResolver {
    pub async fn search_with_query(
        &mut self,
        query: String,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> ResolverResult<SearchResults> {
        let index = self.resolver.index.as_ref()
            .ok_or(ResolverError::IndexNotSet)?;

        let options = crate::r#type::SearchOptions {
            query: Some(query),
            limit,
            offset,
            ..Default::default()
        };

        let search_result = index.search(&options)?;
        let result = search_result.results;
        self.resolver.result = vec![result.clone()];

        Ok(result)
    }

    pub async fn execute_chain(
        &mut self,
        operations: Vec<(String, ResolverOptions)>,
    ) -> ResolverResult<SearchResults> {
        let mut current = self.resolver.clone();

        for (method, options) in operations {
            let index = current.index.as_ref()
                .ok_or(ResolverError::IndexNotSet)?;

            let limit = options.limit().unwrap_or(0);
            let offset = options.offset().unwrap_or(0);
            let enrich = options.enrich().unwrap_or(false);
            let resolve = options.resolve().unwrap_or(false);
            let suggest = options.suggest().unwrap_or(false);

            if let Some(query_str) = options.query().cloned() {
                let search_options = crate::r#type::SearchOptions {
                    query: Some(query_str),
                    limit: Some(limit),
                    offset: Some(offset),
                    enrich: Some(enrich),
                    resolve: Some(resolve),
                    suggest: Some(suggest),
                    boost: options.boost(),
                    ..Default::default()
                };

                let search_result = index.search(&search_options)?;
                let search_ids = search_result.results;

                match method.as_str() {
                    "and" => {
                        let op_results: Vec<IntermediateSearchResults> = vec![vec![search_ids.clone()]];
                        crate::resolver::handler::Handler::handle_and(&mut current, op_results, limit, offset, enrich, resolve, suggest);
                    }
                    "or" => {
                        let op_results: Vec<IntermediateSearchResults> = vec![vec![search_ids.clone()]];
                        crate::resolver::handler::Handler::handle_or(&mut current, op_results, limit, offset, enrich, resolve, suggest);
                    }
                    "not" => {
                        let op_results: Vec<IntermediateSearchResults> = vec![vec![search_ids.clone()]];
                        crate::resolver::handler::Handler::handle_not(&mut current, op_results, limit, offset, enrich, resolve, suggest);
                    }
                    "xor" => {
                        let op_results: Vec<IntermediateSearchResults> = vec![vec![search_ids.clone()]];
                        crate::resolver::handler::Handler::handle_xor(&mut current, op_results, limit, offset, enrich, resolve, suggest);
                    }
                    _ => {
                        current.result = vec![search_ids];
                    }
                }
            }
        }

        self.resolver.result = current.result;
        Ok(self.resolver.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_resolver_new() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3]];
        let resolver = Resolver::new(result, None);
        let async_resolver = AsyncResolver::new(resolver);

        assert_eq!(async_resolver.borrow().result.len(), 1);
    }

    #[test]
    fn test_async_resolver_builder_methods() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3, 4, 5]];
        let resolver = Resolver::new(result, None);
        let mut async_resolver = AsyncResolver::new(resolver);

        async_resolver.limit(3).offset(1).boost(5);

        let borrowed = async_resolver.borrow();
        assert_eq!(borrowed.boostval, 5);
    }

    #[test]
    fn test_async_resolver_into_inner() {
        let result: IntermediateSearchResults = vec![vec![1, 2, 3]];
        let resolver = Resolver::new(result, None);
        let async_resolver = AsyncResolver::new(resolver);

        let inner = async_resolver.into_inner();
        assert_eq!(inner.result.len(), 1);
    }
}
