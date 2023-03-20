# finalizable

This crate provides a type (`Finalizable`) for values that can be finalized,
with methods that operate on working values but leave finalized values unchanged.

This package provides one optional feature, `try`. Enabling it requires nightly Rust.
It implements the `Try` trait on `Finalizable`.
