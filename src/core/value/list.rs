//! List Type Module
//!
//! This module defines the List type and its associated operations.

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Simple list representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Encode, Decode)]
pub struct List {
    pub values: Vec<super::types::Value>,
}

impl List {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, super::types::Value> {
        self.values.iter()
    }

    pub fn push(&mut self, value: super::types::Value) {
        self.values.push(value);
    }

    pub fn remove(&mut self, index: usize) -> super::types::Value {
        self.values.remove(index)
    }

    pub fn pop(&mut self) -> Option<super::types::Value> {
        self.values.pop()
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.values.swap(a, b);
    }

    pub fn get(&self, index: usize) -> Option<&super::types::Value> {
        self.values.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut super::types::Value> {
        self.values.get_mut(index)
    }

    pub fn insert(&mut self, index: usize, value: super::types::Value) {
        self.values.insert(index, value);
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.values.reserve(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.values.shrink_to_fit();
    }

    pub fn truncate(&mut self, len: usize) {
        self.values.truncate(len);
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&super::types::Value) -> bool,
    {
        self.values.retain(f);
    }

    pub fn dedup(&mut self)
    where
        super::types::Value: PartialEq,
    {
        self.values.dedup();
    }

    pub fn sort(&mut self)
    where
        super::types::Value: Ord,
    {
        self.values.sort();
    }

    pub fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut(&super::types::Value, &super::types::Value) -> std::cmp::Ordering,
    {
        self.values.sort_by(f);
    }

    pub fn reverse(&mut self) {
        self.values.reverse();
    }

    pub fn contains(&self, value: &super::types::Value) -> bool
    where
        super::types::Value: PartialEq,
    {
        self.values.contains(value)
    }

    pub fn append(&mut self, other: &mut Self) {
        self.values.append(&mut other.values);
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = super::types::Value>,
    {
        self.values.extend(iter);
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        Self {
            values: self.values.split_off(at),
        }
    }

    pub fn as_slice(&self) -> &[super::types::Value] {
        self.values.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [super::types::Value] {
        self.values.as_mut_slice()
    }

    pub fn into_vec(self) -> Vec<super::types::Value> {
        self.values
    }

    pub fn from_vec(values: Vec<super::types::Value>) -> Self {
        Self { values }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }
}

impl std::ops::Index<usize> for List {
    type Output = super::types::Value;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl std::ops::IndexMut<usize> for List {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index]
    }
}

impl std::ops::Index<std::ops::Range<usize>> for List {
    type Output = [super::types::Value];

    fn index(&self, range: std::ops::Range<usize>) -> &Self::Output {
        &self.values[range]
    }
}

impl std::ops::Index<std::ops::RangeFrom<usize>> for List {
    type Output = [super::types::Value];

    fn index(&self, range: std::ops::RangeFrom<usize>) -> &Self::Output {
        &self.values[range]
    }
}

impl std::ops::Index<std::ops::RangeTo<usize>> for List {
    type Output = [super::types::Value];

    fn index(&self, range: std::ops::RangeTo<usize>) -> &Self::Output {
        &self.values[range]
    }
}

impl std::ops::Index<std::ops::RangeFull> for List {
    type Output = [super::types::Value];

    fn index(&self, range: std::ops::RangeFull) -> &Self::Output {
        &self.values[range]
    }
}

impl IntoIterator for List {
    type Item = super::types::Value;
    type IntoIter = std::vec::IntoIter<super::types::Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a> IntoIterator for &'a List {
    type Item = &'a super::types::Value;
    type IntoIter = std::slice::Iter<'a, super::types::Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

impl From<Vec<super::types::Value>> for List {
    fn from(values: Vec<super::types::Value>) -> Self {
        Self { values }
    }
}

impl From<List> for Vec<super::types::Value> {
    fn from(list: List) -> Self {
        list.values
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}
