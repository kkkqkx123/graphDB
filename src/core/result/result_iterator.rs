use crate::core::value::Value;
use crate::core::DBResult;

pub trait ResultIterator<'a, T: 'a>: Send + Sync + std::fmt::Debug {
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

    fn try_fold<B, F>(&mut self, init: B, mut f: F) -> DBResult<B>
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
