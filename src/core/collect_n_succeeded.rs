//! Collect N Succeeded - Async operation aggregation tool
//!
//! This module provides functionality similar to NebulaGraph's CollectNSucceeded,
//! collecting a specified number of successful futures from a collection of futures.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

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
    Evaluator: Fn(&Output) -> bool, // Returns true if the output is considered successful
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

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 使用安全的方式获取可变引用
        let this = self.as_mut().get_mut();

        // 收集需要处理的索引列表，以避免借用冲突
        let mut completed_indices = Vec::new();

        // Poll 每个未完成的future
        for (idx, future_option) in this.futures.iter_mut().enumerate() {
            if let Some(fut) = future_option {
                // 使用安全的Pin::new方式而不是unsafe版本
                let mut pinned_fut = Pin::new(fut);
                if let Poll::Ready(output) = pinned_fut.as_mut().poll(cx) {
                    completed_indices.push((idx, output));
                }
            }
        }

        // 处理已完成的futures
        for (idx, output) in completed_indices {
            let _ = this.futures[idx].take();
            this.completed_count += 1;

            // 根据评估器检查输出是否被认为成功
            if (this.evaluator)(&output) {
                this.results.push((idx, output));

                // 检查是否达到目标成功数
                if this.results.len() >= this.target_success_count {
                    return Poll::Ready(CollectResult::Success(
                        this.results
                            .iter()
                            .map(|(_, output)| output.clone())
                            .collect(),
                    ));
                }
            }
        }

        // 如果所有futures已完成但仍未获得足够的成功
        if this.completed_count == this.futures.len() {
            return Poll::Ready(CollectResult::Partial(
                this.results
                    .iter()
                    .map(|(_, output)| output.clone())
                    .collect(),
            ));
        }

        // 仍在等待更多结果
        Poll::Pending
    }
}

/// Alternative implementation using futures crate utilities
pub async fn collect_n_succeeded<Fut, Evaluator, Output>(
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
pub async fn collect_n_ok<T, Fut, E>(
    futures: Vec<Fut>,
    target_success_count: usize,
) -> CollectResult<T>
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
        // 使用futures创建统一类型的async blocks
        async fn make_value(v: i32) -> i32 {
            v
        }

        let futures = vec![
            make_value(1),
            make_value(2),
            make_value(3),
            make_value(4),
            make_value(5),
        ];

        // 评估器认为所有值都成功
        let evaluator = |_: &i32| true;

        let result = collect_n_succeeded(futures, 3, evaluator).await;

        match result {
            CollectResult::Success(values) => {
                assert_eq!(values.len(), 3);
                assert!(values.contains(&1));
            }
            CollectResult::Partial(_) => panic!("期望 Success"),
        }
    }

    #[tokio::test]
    async fn test_collect_n_with_evaluator() {
        // 使用统一函数创建async futures
        async fn make_pair(v: i32, b: bool) -> (i32, bool) {
            (v, b)
        }

        let futures = vec![
            make_pair(1, true),  // 成功
            make_pair(2, false), // 失败
            make_pair(3, true),  // 成功
            make_pair(4, false), // 失败
            make_pair(5, true),  // 成功
        ];

        // 评估器只认为第二个元素为true时成功
        let evaluator = |value: &(i32, bool)| value.1;

        let result = collect_n_succeeded(futures, 2, evaluator).await;

        match result {
            CollectResult::Success(values) => {
                assert_eq!(values.len(), 2);
                for val in &values {
                    assert!(val.1); // 第二个元素应该为true
                }
            }
            CollectResult::Partial(_) => panic!("期望 Success"),
        }
    }

    #[tokio::test]
    async fn test_collect_n_ok() {
        // 使用统一函数创建async futures
        async fn make_result(v: i32, ok: bool) -> Result<i32, ()> {
            if ok {
                Ok(v)
            } else {
                Err(())
            }
        }

        let futures = vec![
            make_result(1, true),  // ok
            make_result(2, false), // err - 这个会被忽略
            make_result(3, true),  // ok
            make_result(4, false), // err - 这个会被忽略
            make_result(5, true),  // ok
        ];

        let result = collect_n_ok(futures, 2).await;

        match result {
            CollectResult::Success(values) => {
                assert_eq!(values.len(), 2);
                assert!(values.contains(&1));
                assert!(values.contains(&3) || values.contains(&5));
            }
            CollectResult::Partial(_) => panic!("期望 Success"),
        }
    }

    #[tokio::test]
    async fn test_collect_n_partial() {
        // 使用统一函数创建async futures
        async fn make_pair(v: i32, b: bool) -> (i32, bool) {
            (v, b)
        }

        let futures = vec![
            make_pair(1, false), // 失败
            make_pair(2, false), // 失败
            make_pair(3, false), // 失败
        ];

        // 评估器只认为true值成功
        let evaluator = |value: &(i32, bool)| value.1;

        let result = collect_n_succeeded(futures, 3, evaluator).await;

        match result {
            CollectResult::Partial(values) => {
                assert_eq!(values.len(), 0); // 没有成功的值
            }
            CollectResult::Success(_) => panic!("期望 Partial"),
        }
    }
}
