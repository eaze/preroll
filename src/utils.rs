//! Miscellaneous utilities.

/// This function is useful for inspecting variables that rust-analyzer has trouble extracting type information for,
/// namely returns from awaited futures.
///
/// Literally copied from rustc's [std::any::type_name_of_val][].
/// Currently "experimental" in nightly rust because the name is undecided.
/// Docs: [https://doc.rust-lang.org/std/any/fn.type_name_of_val.html](https://doc.rust-lang.org/std/any/fn.type_name_of_val.html)
/// Tracking issue: [https://github.com/rust-lang/rust/issues/66359](https://github.com/rust-lang/rust/issues/66359)
pub fn type_name_of<T: ?Sized>(_val: &T) -> &'static str {
    std::any::type_name::<T>()
}
