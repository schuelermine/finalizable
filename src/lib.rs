#![no_std]

use core::iter::{once, FusedIterator, Once};
use Finalizable::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Finalizable<T> {
    Working(T),
    Finalized(T),
}

impl<T> Finalizable<T> {
    pub fn finalized(self) -> Self {
        Finalized(self.get())
    }
    pub fn and_then_finalized<F: FnOnce(T) -> T>(self, op: F) -> Self {
        self.map(op).finalized()
    }
    pub fn get(self) -> T {
        match self {
            Working(x) => x,
            Finalized(x) => x,
        }
    }
    pub fn get_as_ref(&self) -> &T {
        self.as_ref().get()
    }
    pub fn get_as_mut(&mut self) -> &mut T {
        self.as_mut().get()
    }
    pub fn set(self, value: T) -> Self {
        match self {
            Working(_) => Working(value),
            a @ Finalized(_) => a,
        }
    }
    pub fn is_working(&self) -> bool {
        matches!(self, Working(_))
    }
    pub fn is_finalized(&self) -> bool {
        matches!(self, Finalized(_))
    }
    pub fn working_or_none(self) -> Option<T> {
        match self {
            Working(x) => Some(x),
            Finalized(_) => None,
        }
    }
    pub fn finalized_or_none(self) -> Option<T> {
        match self {
            Working(_) => None,
            Finalized(x) => Some(x),
        }
    }
    pub fn finalized_or(self, default: T) -> T {
        match self {
            Working(_) => default,
            Finalized(x) => x,
        }
    }
    pub fn finalized_or_else<F: FnOnce(T) -> T>(self, op: F) -> T {
        match self {
            Working(x) => op(x),
            Finalized(x) => x,
        }
    }
    pub fn as_ref(&self) -> Finalizable<&T> {
        match self {
            Working(x) => Working(x),
            Finalized(x) => Finalized(x),
        }
    }
    pub fn as_mut(&mut self) -> Finalizable<&mut T> {
        match self {
            Working(x) => Working(x),
            Finalized(x) => Finalized(x),
        }
    }
    pub fn map<F: FnOnce(T) -> T>(self, op: F) -> Self {
        match self {
            Working(x) => Working(op(x)),
            a @ Finalized(_) => a,
        }
    }
    pub fn iter(&self) -> Iter<'_, T> {
        match self {
            Working(x) => Iter { inner: once(x) },
            Finalized(x) => Iter { inner: once(x) },
        }
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        match self {
            Working(x) => IterMut { inner: once(x) },
            Finalized(x) => IterMut { inner: once(x) },
        }
    }
    pub fn expect_finalized(self, msg: &str) -> T {
        match self {
            Working(x) => x,
            Finalized(_) => panic!("{}", msg),
        }
    }
    pub fn and(self, fin: Self) -> Self {
        match self {
            Working(_) => fin,
            a @ Finalized(_) => a,
        }
    }
    pub fn and_then<F: FnOnce(T) -> Self>(self, op: F) -> Self {
        match self {
            Working(x) => op(x),
            a @ Finalized(_) => a,
        }
    }
}

impl<T> Finalizable<&T> {
    pub fn copied(self) -> Finalizable<T>
    where
        T: Copy,
    {
        match self {
            Working(x) => Working(*x),
            Finalized(x) => Finalized(*x),
        }
    }

    pub fn cloned(self) -> Finalizable<T>
    where
        T: Clone,
    {
        match self {
            Working(x) => Working(x.clone()),
            Finalized(x) => Finalized(x.clone()),
        }
    }
}

impl<'a, T> IntoIterator for &'a Finalizable<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Finalizable<T> {
    type IntoIter = IterMut<'a, T>;
    type Item = &'a mut T;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> IntoIterator for Finalizable<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: once(self.get()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Iter<'a, T> {
    inner: Once<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T> FusedIterator for Iter<'a, T> {}

#[derive(Debug)]
pub struct IterMut<'a, T> {
    inner: Once<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {}

impl<'a, T> FusedIterator for IterMut<'a, T> {}

#[derive(Clone, Debug)]
pub struct IntoIter<T> {
    inner: Once<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> FusedIterator for IntoIter<T> {}
