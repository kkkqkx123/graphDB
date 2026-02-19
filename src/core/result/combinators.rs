use crate::core::error::{DBError, QueryError};
use crate::core::result::ResultIterator;
use crate::core::DBResult;

#[derive(Debug)]
pub struct ZipIterator<RA, RB, A, B>
where
    RA: Send + Sync + std::fmt::Debug + 'static,
    RB: Send + Sync + std::fmt::Debug + 'static,
    A: ResultIterator<'static, RA, Row = RA>,
    B: ResultIterator<'static, RB, Row = RB>,
{
    a: A,
    b: B,
    _phantom: std::marker::PhantomData<(RA, RB)>,
}

impl<RA, RB, A, B> ZipIterator<RA, RB, A, B>
where
    RA: Send + Sync + std::fmt::Debug + 'static,
    RB: Send + Sync + std::fmt::Debug + 'static,
    A: ResultIterator<'static, RA, Row = RA>,
    B: ResultIterator<'static, RB, Row = RB>,
{
    pub(crate) fn new(a: A, b: B) -> Self {
        Self {
            a,
            b,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<RA, RB, A, B> ResultIterator<'static, (RA, RB)> for ZipIterator<RA, RB, A, B>
where
    RA: Send + Sync + std::fmt::Debug + 'static,
    RB: Send + Sync + std::fmt::Debug + 'static,
    A: ResultIterator<'static, RA, Row = RA>,
    B: ResultIterator<'static, RB, Row = RB>,
{
    type Row = (RA, RB);

    fn next(&mut self) -> DBResult<Option<(RA, RB)>> {
        match (self.a.next()?, self.b.next()?) {
            (Some(a_row), Some(b_row)) => Ok(Some((a_row, b_row))),
            _ => Ok(None),
        }
    }

    fn peek(&self) -> DBResult<Option<&(RA, RB)>> {
        Err(DBError::Query(QueryError::ExecutionError(
            "peek not supported for ZipIterator".to_string(),
        )))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let a_hint = self.a.size_hint();
        let b_hint = self.b.size_hint();
        let min_upper = match (a_hint.1, b_hint.1) {
            (Some(a), Some(b)) => Some(a.min(b)),
            _ => None,
        };
        (a_hint.0.min(b_hint.0), min_upper)
    }

    fn nth(&mut self, n: usize) -> DBResult<Option<(RA, RB)>> {
        match (self.a.nth(n)?, self.b.nth(n)?) {
            (Some(a_row), Some(b_row)) => Ok(Some((a_row, b_row))),
            _ => Ok(None),
        }
    }

    fn last(&mut self) -> DBResult<Option<(RA, RB)>> {
        let a_last = self.a.last()?;
        let b_last = self.b.last()?;
        match (a_last, b_last) {
            (Some(a), Some(b)) => Ok(Some((a, b))),
            _ => Ok(None),
        }
    }
}

#[derive(Debug)]
pub struct ChainIterator<R, A, B>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    A: ResultIterator<'static, R, Row = R>,
    B: ResultIterator<'static, R, Row = R>,
{
    first: A,
    second: B,
    in_first: bool,
    _phantom: std::marker::PhantomData<R>,
}

impl<R, A, B> ChainIterator<R, A, B>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    A: ResultIterator<'static, R, Row = R>,
    B: ResultIterator<'static, R, Row = R>,
{
    pub(crate) fn new(first: A, second: B) -> Self {
        Self {
            first,
            second,
            in_first: true,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<R, A, B> ResultIterator<'static, R> for ChainIterator<R, A, B>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    A: ResultIterator<'static, R, Row = R>,
    B: ResultIterator<'static, R, Row = R>,
{
    type Row = R;

    fn next(&mut self) -> DBResult<Option<R>> {
        if self.in_first {
            match self.first.next()? {
                Some(row) => return Ok(Some(row)),
                None => {
                    self.in_first = false;
                }
            }
        }
        self.second.next()
    }

    fn peek(&self) -> DBResult<Option<&R>> {
        if self.in_first {
            self.first.peek()
        } else {
            self.second.peek()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let first_hint = self.first.size_hint();
        let second_hint = self.second.size_hint();
        (
            first_hint.0 + second_hint.0,
            first_hint.1.and_then(|a| second_hint.1.map(|b| a + b)),
        )
    }

    fn nth(&mut self, n: usize) -> DBResult<Option<R>> {
        if self.in_first {
            match self.first.nth(n)? {
                Some(row) => return Ok(Some(row)),
                None => {
                    self.in_first = false;
                    self.second.nth(0)
                }
            }
        } else {
            self.second.nth(n)
        }
    }

    fn last(&mut self) -> DBResult<Option<R>> {
        self.second.last()
    }
}

pub struct FilterIterator<R, P, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    P: Fn(&R) -> bool + Send + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    iter: I,
    predicate: P,
    _phantom: std::marker::PhantomData<R>,
}

impl<R, P, I> std::fmt::Debug for FilterIterator<R, P, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    P: Fn(&R) -> bool + Send + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilterIterator")
            .field("iter", &self.iter)
            .finish_non_exhaustive()
    }
}

impl<R, P, I> FilterIterator<R, P, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    P: Fn(&R) -> bool + Send + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    pub(crate) fn new(iter: I, predicate: P) -> Self {
        Self {
            iter,
            predicate,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<R, P, I> ResultIterator<'static, R> for FilterIterator<R, P, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    P: Fn(&R) -> bool + Send + Sync + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    type Row = R;

    fn next(&mut self) -> DBResult<Option<R>> {
        while let Some(row) = self.iter.next()? {
            if (self.predicate)(&row) {
                return Ok(Some(row));
            }
        }
        Ok(None)
    }

    fn peek(&self) -> DBResult<Option<&R>> {
        Err(DBError::Query(QueryError::ExecutionError(
            "peek not supported for FilterIterator".to_string(),
        )))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.iter.size_hint().1)
    }

    fn nth(&mut self, n: usize) -> DBResult<Option<R>> {
        let mut count = 0;
        while let Some(row) = self.next()? {
            if count == n {
                return Ok(Some(row));
            }
            count += 1;
        }
        Ok(None)
    }

    fn last(&mut self) -> DBResult<Option<R>> {
        let mut last = None;
        while let Some(row) = self.next()? {
            last = Some(row);
        }
        Ok(last)
    }
}

pub struct MapIterator<R, B, F, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    B: Send + Sync + std::fmt::Debug + 'static,
    F: Fn(R) -> B + Send + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    iter: I,
    mapper: F,
    _phantom: std::marker::PhantomData<(R, B)>,
}

impl<R, B, F, I> std::fmt::Debug for MapIterator<R, B, F, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    B: Send + Sync + std::fmt::Debug + 'static,
    F: Fn(R) -> B + Send + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapIterator")
            .field("iter", &self.iter)
            .finish_non_exhaustive()
    }
}

impl<R, B, F, I> MapIterator<R, B, F, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    B: Send + Sync + std::fmt::Debug + 'static,
    F: Fn(R) -> B + Send + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    pub(crate) fn new(iter: I, mapper: F) -> Self {
        Self {
            iter,
            mapper,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<R, B, F, I> ResultIterator<'static, B> for MapIterator<R, B, F, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    B: Send + Sync + std::fmt::Debug + 'static,
    F: Fn(R) -> B + Send + Sync + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    type Row = B;

    fn next(&mut self) -> DBResult<Option<B>> {
        match self.iter.next()? {
            Some(row) => Ok(Some((self.mapper)(row))),
            None => Ok(None),
        }
    }

    fn peek(&self) -> DBResult<Option<&B>> {
        Err(DBError::Query(QueryError::ExecutionError(
            "peek not supported for MapIterator".to_string(),
        )))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn nth(&mut self, n: usize) -> DBResult<Option<B>> {
        self.iter.nth(n).map(|opt| opt.map(&self.mapper))
    }

    fn last(&mut self) -> DBResult<Option<B>> {
        self.iter.last().map(|opt| opt.map(&self.mapper))
    }
}

#[derive(Debug)]
pub struct TakeIterator<R, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    iter: I,
    remaining: usize,
    _phantom: std::marker::PhantomData<R>,
}

impl<R, I> TakeIterator<R, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    pub(crate) fn new(iter: I, n: usize) -> Self {
        Self {
            iter,
            remaining: n,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<R, I> ResultIterator<'static, R> for TakeIterator<R, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    type Row = R;

    fn next(&mut self) -> DBResult<Option<R>> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        self.iter.next()
    }

    fn peek(&self) -> DBResult<Option<&R>> {
        if self.remaining > 0 {
            self.iter.peek()
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.remaining.min(self.iter.size_hint().0),
            Some(self.remaining),
        )
    }

    fn nth(&mut self, n: usize) -> DBResult<Option<R>> {
        if n >= self.remaining {
            self.remaining = 0;
            return Ok(None);
        }
        self.remaining -= n + 1;
        self.iter.nth(n)
    }

    fn last(&mut self) -> DBResult<Option<R>> {
        let mut last = None;
        let mut count = 0;
        while count < self.remaining {
            match self.iter.next()? {
                Some(row) => {
                    last = Some(row);
                    count += 1;
                }
                None => break,
            }
        }
        self.remaining = 0;
        Ok(last)
    }
}

#[derive(Debug)]
pub struct SkipIterator<R, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    iter: I,
    target_skip: usize,
    skipped: usize,
    _phantom: std::marker::PhantomData<R>,
}

impl<R, I> SkipIterator<R, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    pub(crate) fn new(iter: I, n: usize) -> Self {
        Self {
            iter,
            target_skip: n,
            skipped: 0,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<R, I> ResultIterator<'static, R> for SkipIterator<R, I>
where
    R: Send + Sync + std::fmt::Debug + 'static,
    I: ResultIterator<'static, R, Row = R>,
{
    type Row = R;

    fn next(&mut self) -> DBResult<Option<R>> {
        while self.skipped < self.target_skip {
            if self.iter.next()?.is_none() {
                return Ok(None);
            }
            self.skipped += 1;
        }
        self.iter.next()
    }

    fn peek(&self) -> DBResult<Option<&R>> {
        Err(DBError::Query(QueryError::ExecutionError(
            "peek not supported during skip".to_string(),
        )))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        (
            lower.saturating_sub(self.target_skip.saturating_sub(self.skipped)),
            upper.map(|u| u.saturating_sub(self.target_skip.saturating_sub(self.skipped))),
        )
    }

    fn nth(&mut self, n: usize) -> DBResult<Option<R>> {
        self.iter.nth(n + self.target_skip.saturating_sub(self.skipped))
    }

    fn last(&mut self) -> DBResult<Option<R>> {
        while self.skipped < self.target_skip {
            if self.iter.next()?.is_none() {
                return Ok(None);
            }
            self.skipped += 1;
        }
        self.iter.last()
    }
}
