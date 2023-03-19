#![no_std]
#![cfg_attr(feature = "try", feature(try_trait_v2))]
//! This crate provides a type ([`Finalizable`]) for values that can be finalized,
//! with methods that operate on working values but leave finalized values unchanged.

#[cfg(feature = "try")]
use core::ops::{ControlFlow, FromResidual, Try};

pub use Finalizable::*;

/// A value that can be a working value or a finalized value.
/// All operations on a single [`Finalizable<T>`] do not modify a finalized value.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Finalizable<T> {
    /// A working value.
    Working(T),
    /// A finalized value.
    Finalized(T),
}

impl<T> Finalizable<T> {
    /// Create a new finalizable value from a value and a boolean
    /// that determines if it is a finalized or working value.
    pub fn new(value: T, finalized: bool) -> Self {
        match finalized {
            true => Finalized(value),
            false => Working(value),
        }
    }
    /// Finalize a value. Returns a finalized version of the value.
    pub fn finalize(self) -> Self {
        Finalized(self.get())
    }
    /// Get the value, whether working or finalized.
    pub fn get(self) -> T {
        match self {
            Working(x) => x,
            Finalized(x) => x,
        }
    }
    /// Get the value from a reference to a finalizable value,
    /// whether working or finalized, as a reference to the underlying value.
    pub fn get_as_ref(&self) -> &T {
        self.as_ref().get()
    }
    /// Get the value from a mutable reference to a working value
    /// as a mutable reference. Returns [`None`] if the value is a finalized value.
    pub fn try_get_mut(&mut self) -> Option<&mut T> {
        match self {
            Working(x) => Some(x),
            Finalized(_) => None,
        }
    }
    /// Override a working value. Does nothing to a finalized value.
    pub fn set(self, value: T) -> Self {
        match self {
            Working(_) => Working(value),
            a @ Finalized(_) => a,
        }
    }
    /// Check if a value is a working value.
    pub fn is_working(&self) -> bool {
        matches!(self, Working(_))
    }
    /// Check if a value is a finalized value.
    pub fn is_finalized(&self) -> bool {
        matches!(self, Finalized(_))
    }
    /// Get the value, but only if it is a working value.
    /// Returns [`None`] if the value is a finalized value.
    pub fn working_or_none(self) -> Option<T> {
        match self {
            Working(x) => Some(x),
            Finalized(_) => None,
        }
    }
    /// Get the value, but only if it is a finalized value.
    /// Returns [`None`] if the value is a working value.
    pub fn finalized_or_none(self) -> Option<T> {
        match self {
            Working(_) => None,
            Finalized(x) => Some(x),
        }
    }
    /// Get the value, but only if it is a finalized value.
    /// Returns `default` if the value is a working value.
    pub fn finalized_or(self, default: T) -> T {
        match self {
            Working(_) => default,
            Finalized(x) => x,
        }
    }
    /// Get the value, but only if it is a finalized value.
    /// Calls `default` and returns its result if the value is a working value.
    pub fn finalized_or_else<F: FnOnce(T) -> T>(self, op: F) -> T {
        match self {
            Working(x) => op(x),
            Finalized(x) => x,
        }
    }
    /// Turn a reference to a finalizable value into a finalizable reference.
    pub fn as_ref(&self) -> Finalizable<&T> {
        match self {
            Working(x) => Working(x),
            Finalized(x) => Finalized(x),
        }
    }
    /// Apply a function to a working value. Does nothing to a finalized value.
    pub fn map<F: FnOnce(T) -> T>(self, op: F) -> Self {
        match self {
            Working(x) => Working(op(x)),
            a @ Finalized(_) => a,
        }
    }
    /// Apply a function to a working value and finalize it.
    /// Does nothing to a finalized value.
    pub fn map_and_finalize<F: FnOnce(T) -> T>(self, op: F) -> Self {
        self.map(op).finalize()
    }
    /// Get a finalized value, panicking with `msg` if the value is a working value.
    pub fn expect_finalized(self, msg: &str) -> T {
        match self {
            Working(x) => x,
            Finalized(_) => panic!("{}", msg),
        }
    }
    /// Return `fin` if the value is a working value, returning a finalized value unchanged.
    pub fn and(self, fin: Self) -> Self {
        match self {
            Working(_) => fin,
            a @ Finalized(_) => a,
        }
    }
    /// Call `op` on the value if it is a working value,
    /// returning a finalized value unchanged.
    pub fn and_then<F: FnOnce(T) -> Self>(self, op: F) -> Self {
        match self {
            Working(x) => op(x),
            a @ Finalized(_) => a,
        }
    }
    /// Call `op` on the value if it is a working value,
    /// creating a new finalizable value by using the returned tuple
    /// as the arguments to [`new`], returning a finalized value unchanged.
    ///
    /// [`new`]: Finalizable::new
    pub fn and_then_new<F: FnOnce(T) -> (T, bool)>(self, op: F) -> Self {
        self.and_then(|x| {
            let (value, finalized) = op(x);
            Finalizable::new(value, finalized)
        })
    }
}

impl<T> Finalizable<&T> {
    /// Make a copy of a finalizable value by copying the underlying value.
    pub fn copied(self) -> Finalizable<T>
    where
        T: Copy,
    {
        match self {
            Working(x) => Working(*x),
            Finalized(x) => Finalized(*x),
        }
    }
    /// Make a clone of a finalizable value by cloning the underlying value.
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

#[cfg(feature = "try")]
/// Acts like [`ControlFlow<T, T>`].
/// Finalized values ([`Finalized`]) break,
/// working values ([`Working`]) continue.
impl<T> Try for Finalizable<T> {
    type Output = T;
    type Residual = Residual<T>;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Working(x) => ControlFlow::Continue(x),
            Finalized(x) => ControlFlow::Break(Residual(x)),
        }
    }
    fn from_output(output: Self::Output) -> Self {
        Working(output)
    }
}

#[cfg(feature = "try")]
impl<T> FromResidual for Finalizable<T> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        Finalized(residual.0)
    }
}

#[cfg(feature = "try")]
/// The residual from applying `?` to a finalized value ([`Finalized`]).
/// Used in the implementation of [`Try`] for [`Finalizable`].
pub struct Residual<T>(pub T);
