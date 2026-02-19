use crate::core::value::Value;
use crate::core::DBResult;

pub trait ColumnAccess {
    fn columns(&self) -> &[String];
    fn get_column_index(&self, name: &str) -> Option<usize>;
    fn get_column(&self, index: usize) -> Option<&Value>;
    fn get_column_by_name(&self, name: &str) -> Option<&Value>;
}

pub trait ResultIterator<'a, T: 'a>:
    Send + Sync + std::fmt::Debug
{
    type Row: std::fmt::Debug + Send + Sync;

    fn next(&mut self) -> DBResult<Option<Self::Row>>;

    fn peek(&self) -> DBResult<Option<&Self::Row>>;

    fn size_hint(&self) -> (usize, Option<usize>);

    fn count(&mut self) -> DBResult<usize>
    where
        Self: Sized,
    {
        let mut count = 0;
        while self.next()?.is_some() {
            count += 1;
        }
        Ok(count)
    }

    fn nth(&mut self, n: usize) -> DBResult<Option<Self::Row>>;

    fn last(&mut self) -> DBResult<Option<Self::Row>>;

    fn collect(&mut self) -> DBResult<Vec<Self::Row>>
    where
        Self: Sized,
    {
        let mut results = Vec::new();
        while let Some(row) = self.next()? {
            results.push(row);
        }
        Ok(results)
    }

    fn for_each<F>(&mut self, mut f: F) -> DBResult<()>
    where
        Self: Sized,
        F: FnMut(Self::Row) -> (),
    {
        while let Some(row) = self.next()? {
            f(row);
        }
        Ok(())
    }

    fn fold<B, F>(&mut self, init: B, mut f: F) -> DBResult<B>
    where
        Self: Sized,
        F: FnMut(B, Self::Row) -> B,
    {
        let mut acc = init;
        while let Some(row) = self.next()? {
            acc = f(acc, row);
        }
        Ok(acc)
    }

    fn try_fold<B, F>(
        &mut self,
        init: B,
        mut f: F,
    ) -> DBResult<B>
    where
        Self: Sized,
        F: FnMut(B, Self::Row) -> DBResult<B>,
    {
        let mut acc = init;
        while let Some(row) = self.next()? {
            acc = f(acc, row)?;
        }
        Ok(acc)
    }

    fn any<P>(&mut self, mut predicate: P) -> DBResult<bool>
    where
        Self: Sized,
        P: FnMut(&Self::Row) -> bool,
    {
        while let Some(row) = self.next()? {
            if predicate(&row) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn all<P>(&mut self, mut predicate: P) -> DBResult<bool>
    where
        Self: Sized,
        P: FnMut(&Self::Row) -> bool,
    {
        while let Some(row) = self.next()? {
            if !predicate(&row) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn find<P>(&mut self, mut predicate: P) -> DBResult<Option<Self::Row>>
    where
        Self: Sized,
        P: FnMut(&Self::Row) -> bool,
    {
        while let Some(row) = self.next()? {
            if predicate(&row) {
                return Ok(Some(row));
            }
        }
        Ok(None)
    }

    fn position<P>(&mut self, mut predicate: P) -> DBResult<Option<usize>>
    where
        Self: Sized,
        P: FnMut(&Self::Row) -> bool,
    {
        let mut index = 0;
        while let Some(row) = self.next()? {
            if predicate(&row) {
                return Ok(Some(index));
            }
            index += 1;
        }
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct EmptyIterator<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> EmptyIterator<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, T> ResultIterator<'a, T> for EmptyIterator<T>
where
    T: Send + Sync + std::fmt::Debug + 'a,
{
    type Row = T;

    fn next(&mut self) -> DBResult<Option<T>> {
        Ok(None)
    }

    fn peek(&self) -> DBResult<Option<&T>> {
        Ok(None)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }

    fn nth(&mut self, _n: usize) -> DBResult<Option<T>> {
        Ok(None)
    }

    fn last(&mut self) -> DBResult<Option<T>> {
        Ok(None)
    }

    fn try_fold<B, F>(&mut self, init: B, _f: F) -> DBResult<B>
    where
        Self: Sized,
        F: FnMut(B, Self::Row) -> DBResult<B>,
    {
        Ok(init)
    }
}

impl<T> Default for EmptyIterator<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct IteratorFactories;

impl IteratorFactories {
    pub fn zip<RA, RB, A, B>(
        a: A,
        b: B,
    ) -> super::combinators::ZipIterator<RA, RB, A, B>
    where
        RA: Send + Sync + std::fmt::Debug + 'static,
        RB: Send + Sync + std::fmt::Debug + 'static,
        A: ResultIterator<'static, RA, Row = RA>,
        B: ResultIterator<'static, RB, Row = RB>,
    {
        super::combinators::ZipIterator::new(a, b)
    }

    pub fn chain<R, A, B>(
        first: A,
        second: B,
    ) -> super::combinators::ChainIterator<R, A, B>
    where
        R: Send + Sync + std::fmt::Debug + 'static,
        A: ResultIterator<'static, R, Row = R>,
        B: ResultIterator<'static, R, Row = R>,
    {
        super::combinators::ChainIterator::new(first, second)
    }

    pub fn filter<R, P, I>(
        iter: I,
        predicate: P,
    ) -> super::combinators::FilterIterator<R, P, I>
    where
        R: Send + Sync + std::fmt::Debug + 'static,
        P: Fn(&R) -> bool + Send + Sync + std::fmt::Debug + 'static,
        I: ResultIterator<'static, R, Row = R>,
    {
        super::combinators::FilterIterator::new(iter, predicate)
    }

    pub fn map<R, B, F, I>(
        iter: I,
        mapper: F,
    ) -> super::combinators::MapIterator<R, B, F, I>
    where
        R: Send + Sync + std::fmt::Debug + 'static,
        B: Send + Sync + std::fmt::Debug + 'static,
        F: Fn(R) -> B + Send + Sync + std::fmt::Debug + 'static,
        I: ResultIterator<'static, R, Row = R>,
    {
        super::combinators::MapIterator::new(iter, mapper)
    }

    pub fn take<R, I>(
        iter: I,
        n: usize,
    ) -> super::combinators::TakeIterator<R, I>
    where
        R: Send + Sync + std::fmt::Debug + 'static,
        I: ResultIterator<'static, R, Row = R>,
    {
        super::combinators::TakeIterator::new(iter, n)
    }

    pub fn skip<R, I>(
        iter: I,
        n: usize,
    ) -> super::combinators::SkipIterator<R, I>
    where
        R: Send + Sync + std::fmt::Debug + 'static,
        I: ResultIterator<'static, R, Row = R>,
    {
        super::combinators::SkipIterator::new(iter, n)
    }
}
