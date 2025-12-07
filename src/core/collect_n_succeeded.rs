//! Collect N Succeeded - Async operation aggregation tool
//!
//! This module provides functionality similar to NebulaGraph's CollectNSucceeded,
//! collecting a specified number of successful futures from a collection of futures.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::future::BoxFuture;
use futures::FutureExt;

/// Result of a collect operation
pub enum CollectResult<T> {
    /// Specified number of successes collected
    Success(Vec<T>),
    /// All futures completed but not enough successes
    Partial(Vec<T>),
}

/// A future that collects N successful results from a collection of futures
pub struct CollectNSucceeded<Fut, Evaluator>
where
    Fut: Future,
{
    futures: Vec<Option<Fut>>,
    evaluator: Evaluator,
    results: Vec<(usize, Fut::Output)>,
    target_success_count: usize,
    completed_count: usize,
}

impl<Fut, Evaluator, Output> CollectNSucceeded<Fut, Evaluator>
where
    Fut: Future<Output = Output>,
    Evaluator: Fn(&Output) -> bool,  // Returns true if the output is considered successful
{
    /// Create a new CollectNSucceeded future
    pub fn new(futures: Vec<Fut>, target_success_count: usize, evaluator: Evaluator) -> Self {
        let futures: Vec<Option<Fut>> = futures.into_iter().map(Some).collect();
        CollectNSucceeded {
            futures,
            evaluator,
            results: Vec::new(),
            target_success_count,
            completed_count: 0,
        }
    }
}

impl<Fut, Evaluator, Output> Future for CollectNSucceeded<Fut, Evaluator>
where
    Fut: Future<Output = Output> + Unpin,
    Evaluator: Fn(&Output) -> bool + Unpin,
    Output: Clone + Unpin,
{
    type Output = CollectResult<Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Poll each future that hasn't completed yet
        for (idx, future_option) in this.futures.iter_mut().enumerate() {
            if let Some(fut) = future_option {
                let fut = unsafe { Pin::new_unchecked(fut) };
                if let Poll::Ready(output) = fut.poll(cx) {
                    // Remove the future from the list as it's completed
                    let _ = this.futures[idx].take();
                    this.completed_count += 1;

                    // Check if the output is considered successful based on the evaluator
                    if (this.evaluator)(&output) {
                        this.results.push((idx, output));

                        // Check if we've reached the target success count
                        if this.results.len() >= this.target_success_count {
                            return Poll::Ready(CollectResult::Success(
                                this.results.iter().map(|(_, output)| output.clone()).collect()
                            ));
                        }
                    }
                }
            }
        }

        // If all futures are completed but we still don't have enough successes
        if this.completed_count == this.futures.len() {
            return Poll::Ready(CollectResult::Partial(
                this.results.iter().map(|(_, output)| output.clone()).collect()
            ));
        }

        // Still waiting for more results
        Poll::Pending
    }
}

/// Alternative implementation using futures crate utilities
pub async fn collect_n_succeeded<T, Fut, Evaluator, Output>(
    futures: Vec<Fut>,
    target_success_count: usize,
    evaluator: Evaluator,
) -> CollectResult<Output>
where
    Fut: Future<Output = Output> + Send + 'static,
    Output: Send + Clone,
    Evaluator: Fn(&Output) -> bool + Send + Sync + 'static,
{
    use futures::stream::{FuturesUnordered, StreamExt};
    
    let mut futures_stream: FuturesUnordered<_> = futures.into_iter().collect();
    let mut results = Vec::new();
    
    while let Some(output) = futures_stream.next().await {
        if evaluator(&output) {
            results.push(output.clone());
            
            if results.len() >= target_success_count {
                return CollectResult::Success(results);
            }
        }
    }
    
    CollectResult::Partial(results)
}

/// A simplified version that treats Ok results as successful in Result types
pub async fn collect_n_ok<T, Fut, E>(futures: Vec<Fut>, target_success_count: usize) -> CollectResult<T>
where
    Fut: Future<Output = Result<T, E>> + Send + 'static,
    T: Send + Clone,
    E: Send,
{
    use futures::stream::{FuturesUnordered, StreamExt};
    
    let mut futures_stream: FuturesUnordered<_> = futures.into_iter().collect();
    let mut results = Vec::new();
    
    while let Some(output) = futures_stream.next().await {
        if let Ok(value) = output {
            results.push(value);
            
            if results.len() >= target_success_count {
                return CollectResult::Success(results);
            }
        }
    }
    
    CollectResult::Partial(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_collect_n_succeeded() {
        // Create some futures that return different values
        let futures = vec![
            async { 1 },
            async { 2 },
            async { 3 },
            async { 4 },
            async { 5 },
        ];

        // Evaluator that considers all values successful
        let evaluator = |_| true;
        
        let result = collect_n_succeeded(futures, 3, evaluator).await;
        
        match result {
            CollectResult::Success(values) => {
                assert_eq!(values.len(), 3);
                assert!(values.contains(&1));
            }
            CollectResult::Partial(_) => panic!("Expected Success"),
        }
    }

    #[tokio::test]
    async fn test_collect_n_with_evaluator() {
        // Create futures with different success values
        let futures = vec![
            async { (1, true) },   // success
            async { (2, false) },  // failure
            async { (3, true) },   // success
            async { (4, false) },  // failure
            async { (5, true) },   // success
        ];

        // Evaluator that only considers second element being true as successful
        let evaluator = |value: &(i32, bool)| value.1;
        
        let result = collect_n_succeeded(futures, 2, evaluator).await;
        
        match result {
            CollectResult::Success(values) => {
                assert_eq!(values.len(), 2);
                for val in &values {
                    assert!(val.1);  // Second element should be true
                }
            }
            CollectResult::Partial(_) => panic!("Expected Success"),
        }
    }

    #[tokio::test]
    async fn test_collect_n_ok() {
        // Test with Result futures
        let futures = vec![
            async { Ok::<i32, ()>(1) },
            async { Err::<i32, ()>(()) },  // This will be ignored
            async { Ok::<i32, ()>(3) },
            async { Err::<i32, ()>(()) },  // This will be ignored
            async { Ok::<i32, ()>(5) },
        ];

        let result = collect_n_ok(futures, 2).await;
        
        match result {
            CollectResult::Success(values) => {
                assert_eq!(values.len(), 2);
                assert!(values.contains(&1));
                assert!(values.contains(&3) || values.contains(&5));
            }
            CollectResult::Partial(_) => panic!("Expected Success"),
        }
    }

    #[tokio::test]
    async fn test_collect_n_partial() {
        // Create futures where we can't get enough successes
        let futures = vec![
            async { (1, false) },  // failure
            async { (2, false) },  // failure
            async { (3, false) },  // failure
        ];

        // Evaluator that only considers true values as successful
        let evaluator = |value: &(i32, bool)| value.1;
        
        let result = collect_n_succeeded(futures, 3, evaluator).await;
        
        match result {
            CollectResult::Partial(values) => {
                assert_eq!(values.len(), 0);  // No successful values
            }
            CollectResult::Success(_) => panic!("Expected Partial"),
        }
    }
}