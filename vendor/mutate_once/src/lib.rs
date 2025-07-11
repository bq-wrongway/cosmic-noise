//
// Copyright (c) 2019 KAMADA Ken'ichi.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
// OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
// HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
// LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
// OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGE.
//

//! This library provides interior mutability that can be borrowed
//! as plain immutable references `&T` in exchange for the write-once,
//! read-many restriction.
//!
//! Unlike `std::cell::Cell` or `std::cell::RefCell`, a plain
//! immutable reference `&T` can be taken from `MutOnce<T>`.
//! Once an immutable reference is taken, the value can never be mutated
//! (even after all references are dropped).
//!
//! The use cases include caching getter and delayed evaluation.

use std::cell::{Cell, UnsafeCell};
use std::ops::{Deref, DerefMut, Drop};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum State {
    Unborrowed,
    Updating,
    Fixed,
}

/// A mutable memory location that is write-once and can be borrowed as
/// plain `&T`.
///
/// Initially the value can be mutated through struct `RefMut<T>`
/// obtained by `get_mut` method.
/// When there is no `RefMut` alive, a shared reference `&T` can be
/// taken by `get_ref` method.  Once `get_ref` is called, `get_mut` must
/// not be called again.
///
/// # Examples
///
/// ```
/// use mutate_once::MutOnce;
/// struct Container {
///     expensive: MutOnce<String>,
/// }
/// impl Container {
///     fn expensive(&self) -> &str {
///         if !self.expensive.is_fixed() {
///             let mut ref_mut = self.expensive.get_mut();
///             *ref_mut += "expensive";
///             // Drop `ref_mut` before calling `get_ref`.
///         }
///         // A plain reference can be returned to the caller
///         // unlike `Cell` or `RefCell`.
///         self.expensive.get_ref()
///     }
/// }
/// let container = Container { expensive: MutOnce::new(String::new()) };
/// assert_eq!(container.expensive(), "expensive");
/// ```
#[derive(Debug)]
pub struct MutOnce<T> {
    value: UnsafeCell<T>,
    state: Cell<State>,
}

impl<T> MutOnce<T> {
    /// Creates a new `MutOnce` containing the given `value`.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            state: Cell::new(State::Unborrowed),
        }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` gets dropped.
    /// This method must not be called if another `RefMut` is active or
    /// `get_ref` is ever called.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed or
    /// ever immutably borrowed.
    ///
    /// # Examples
    ///
    /// ```
    /// let mo = mutate_once::MutOnce::new(0);
    /// *mo.get_mut() += 2;
    /// *mo.get_mut() += 5;
    /// assert_eq!(*mo.get_ref(), 7);
    /// ```
    ///
    /// Panics if another mutable borrow is active:
    ///
    /// ```should_panic
    /// let mo = mutate_once::MutOnce::new(0);
    /// let mut ref_mut = mo.get_mut();
    /// *mo.get_mut() += 7;     // Panics because `ref_mut` is still active.
    /// ```
    ///
    /// Panics if `get_ref` is ever called:
    ///
    /// ```should_panic
    /// let mo = mutate_once::MutOnce::new(0);
    /// assert_eq!(*mo.get_ref(), 0);
    /// *mo.get_mut() += 7;     // Panics because `get_ref` is called once.
    /// ```
    #[inline]
    pub fn get_mut(&self) -> RefMut<T> {
        match self.state.get() {
            State::Unborrowed => {
                self.state.replace(State::Updating);
                RefMut { target: self }
            },
            State::Updating => panic!("already mutably borrowed"),
            State::Fixed => panic!("no longer mutable"),
        }
    }

    /// Returns an immutable reference to the value.
    ///
    /// This method must not be called while the value is mutably borrowed.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    ///
    /// # Examples
    ///
    /// ```
    /// let mo = mutate_once::MutOnce::new(0);
    /// *mo.get_mut() += 7;
    /// assert_eq!(*mo.get_ref(), 7);
    /// ```
    ///
    /// Panics if a mutable borrow is active:
    ///
    /// ```should_panic
    /// let mo = mutate_once::MutOnce::new(0);
    /// let mut ref_mut = mo.get_mut();
    /// mo.get_ref();           // Panics because `ref_mut` is still active.
    /// ```
    #[inline]
    pub fn get_ref(&self) -> &T {
        match self.state.get() {
            State::Unborrowed => { self.state.replace(State::Fixed); },
            State::Updating => panic!("still mutably borrowed"),
            State::Fixed => {},
        }
        unsafe { &*self.value.get() }
    }

    /// Returns true if the value can be no longer mutated (in other words,
    /// if `get_ref` is ever called).
    #[inline]
    pub fn is_fixed(&self) -> bool {
        self.state.get() == State::Fixed
    }

    /// Consumes the `MutOnce`, returning the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

impl<T: Default> Default for MutOnce<T> {
    #[inline]
    fn default() -> MutOnce<T> {
        MutOnce::new(T::default())
    }
}

impl<T> From<T> for MutOnce<T> {
    #[inline]
    fn from(t: T) -> MutOnce<T> {
        MutOnce::new(t)
    }
}

/// A wrapper type for a mutably borrowed value from a `MutOnce<T>`.
#[derive(Debug)]
pub struct RefMut<'a, T> {
    target: &'a MutOnce<T>,
}

impl<'a, T> Deref for RefMut<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.target.value.get() }
    }
}

impl<'a, T> DerefMut for RefMut<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.target.value.get() }
    }
}

impl<'a, T> Drop for RefMut<'a, T> {
    #[inline]
    fn drop(&mut self) {
        debug_assert_eq!(self.target.state.get(), State::Updating);
        self.target.state.replace(State::Unborrowed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_muts() {
        let mo = MutOnce::new(Vec::new());
        {
            let mut mutvec = mo.get_mut();
            mutvec.push(1);
            mutvec.push(2);
        }
        {
            let mut mutvec = mo.get_mut();
            mutvec.push(3);
        }
        let vec = mo.get_ref();
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);
    }

    #[test]
    fn multiple_refs() {
        let mo = MutOnce::new(Vec::new());
        {
            let mut mutvec = mo.get_mut();
            mutvec.push(1);
            mutvec.push(2);
            mutvec.push(3);
        }
        let vec1 = mo.get_ref();
        let vec2 = mo.get_ref();
        assert_eq!(vec1[0], 1);
        assert_eq!(vec2[1], 2);
        assert_eq!(vec1[2], 3);
    }

    #[test]
    fn temporary_value() {
        let mo = MutOnce::new(Vec::new());
        mo.get_mut().push(1);
        mo.get_mut().push(2);
        assert_eq!(mo.get_ref()[0], 1);
        assert_eq!(mo.get_ref()[1], 2);
    }

    #[test]
    #[should_panic(expected = "still mutably borrowed")]
    fn ref_while_mut() {
        let mo = MutOnce::new(Vec::new());
        let mut mutvec = mo.get_mut();
        mutvec.push(1);
        assert_eq!(mo.get_ref()[0], 1);
    }

    #[test]
    #[should_panic(expected = "no longer mutable")]
    fn mut_after_ref() {
        let mo = MutOnce::new(Vec::new());
        assert_eq!(mo.get_ref().len(), 0);
        mo.get_mut().push(1);
    }

    #[test]
    #[should_panic(expected = "already mutably borrowed")]
    fn multiple_muts() {
        let mo = MutOnce::new(Vec::new());
        let mut mutvec1 = mo.get_mut();
        let mut mutvec2 = mo.get_mut();
        mutvec1.push(1);
        mutvec2.push(2);
    }

    #[test]
    fn into_inner() {
        let mo = MutOnce::new(Vec::new());
        mo.get_mut().push(1);
        mo.get_mut().push(7);
        assert_eq!(mo.into_inner(), vec![1, 7])
    }

    #[test]
    fn default() {
        let mo = MutOnce::<u32>::default();
        *mo.get_mut() += 9;
        assert_eq!(*mo.get_ref(), 9);
    }

    #[test]
    fn from() {
        let mo: MutOnce<_> = From::from(0);
        *mo.get_mut() += 9;
        assert_eq!(*mo.get_ref(), 9);
        let mo: MutOnce<_> = 0.into();
        *mo.get_mut() += 9;
        assert_eq!(*mo.get_ref(), 9);
    }
}
