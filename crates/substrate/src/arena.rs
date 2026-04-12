//! Bump allocator that hands out raw `NonNull<T>` pointers.
//!
//! # Why `NonNull<T>` and not `&mut T`
//! A `&mut T` reference asserts Rust's exclusive-reference guarantee for its
//! lifetime. This arena cannot uphold that: calling [`Arena::reset`] invalidates
//! every previously-returned pointer immediately, with no lifetime to enforce it.
//! Returning `NonNull<T>` makes the unsafe contract explicit — the caller must
//! manage pointer validity and dereference inside `unsafe`.
//!
//! A pointer returned by [`Arena::alloc`] or [`Arena::alloc_uninit`] is valid
//! only until the next call to [`Arena::reset`]. For frame-scoped allocations,
//! prefer [`crate::frame_arena::FrameArena`].

use std::alloc::Layout;
use std::ptr::NonNull;

pub struct Arena {
    buf: Vec<u8>,
    offset: usize,
}

#[derive(Debug)]
pub enum ArenaError {
    OutOfMemory,
}

/// Rounds `addr` up to the nearest multiple of `align`.
/// `align` must be a non-zero power of two (guaranteed by `std::alloc::Layout`).
/// Returns `None` on arithmetic overflow.
pub(crate) fn align_up(addr: usize, align: usize) -> Option<usize> {
    let bumped = addr.checked_add(align - 1)?;
    Some(bumped & !(align - 1))
}

impl Arena {
    pub fn new(capacity: usize) -> Self {
        let buf = Vec::with_capacity(capacity);
        Self { buf, offset: 0 }
    }

    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    pub fn used(&self) -> usize {
        self.offset
    }

    pub fn alloc_layout(
        &mut self,
        layout: Layout,
    ) -> Result<NonNull<u8>, ArenaError> {
        let base = self.buf.as_ptr() as usize;
        let current = base + self.offset;

        let aligned =
            align_up(current, layout.align()).ok_or(ArenaError::OutOfMemory)?;
        let new_offset = aligned
            .checked_sub(base)
            .and_then(|o| o.checked_add(layout.size()))
            .ok_or(ArenaError::OutOfMemory)?;

        if new_offset > self.capacity() {
            return Err(ArenaError::OutOfMemory);
        }
        self.offset = new_offset;
        NonNull::new(aligned as *mut u8).ok_or(ArenaError::OutOfMemory)
    }

    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// Allocates space for `T`, writes `value`, and returns a `NonNull<T>`.
    ///
    /// # Safety
    /// The returned pointer is valid only until the next call to [`Arena::reset`].
    /// Dereferencing after `reset()` is undefined behaviour.
    pub fn alloc<T>(&mut self, value: T) -> Result<NonNull<T>, ArenaError> {
        let layout = Layout::new::<T>();
        let ptr = self.alloc_layout(layout)?.as_ptr() as *mut T;
        unsafe { ptr.write(value) };
        Ok(unsafe { NonNull::new_unchecked(ptr) })
    }

    /// Allocates space for `T` without initialising it.
    ///
    /// # Safety
    /// The caller must initialise the allocation before reading it.
    /// The pointer is valid only until the next call to [`Arena::reset`].
    pub fn alloc_uninit<T>(&mut self) -> Result<NonNull<T>, ArenaError> {
        let layout = Layout::new::<T>();
        let ptr = self.alloc_layout(layout)?.as_ptr() as *mut T;
        Ok(unsafe { NonNull::new_unchecked(ptr) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_arena_has_correct_capacity_and_zero_offset() {
        let arena = Arena::new(1024);
        assert_eq!(arena.capacity(), 1024);
        assert_eq!(arena.used(), 0);
    }

    #[test]
    fn alloc_layout_returns_ptr_to_advanced_offset_when_enough_capacity() {
        let mut arena = Arena::new(1024);
        let layout =
            Layout::from_size_align(8, 8).expect("Should be valid layout.");
        let result = arena.alloc_layout(layout);
        let _ptr = result.unwrap();
        assert_eq!(arena.used(), 8);
    }

    #[test]
    fn alloc_layout_returns_err_outofmemory_when_new_offset_exceeds_capacity() {
        let mut arena = Arena::new(8);
        let layout =
            Layout::from_size_align(9, 8).expect("Should be valid layout.");
        let result = arena.alloc_layout(layout);
        assert!(result.is_err());
        assert_eq!(arena.used(), 0);
    }

    #[test]
    fn reset_sets_offset_to_zero() {
        let mut arena = Arena::new(16);
        let layout =
            Layout::from_size_align(9, 8).expect("Should be valid layout.");
        _ = arena.alloc_layout(layout);
        arena.reset();
        assert_eq!(arena.used(), 0);
    }

    #[test]
    fn alloc_u64_writes_u64_value() {
        let mut arena = Arena::new(64);
        let ptr = arena.alloc::<u64>(42).expect("Should be enough room");
        assert_eq!(unsafe { *ptr.as_ptr() }, 42);
    }

    #[test]
    fn align_up_returns_none_on_overflow() {
        assert_eq!(align_up(usize::MAX, 8), None);
        assert_eq!(align_up(usize::MAX - 6, 8), None);
        assert_eq!(align_up(0, 8), Some(0));
        assert_eq!(align_up(1, 8), Some(8));
    }

    #[test]
    fn alloc_uninit_round_trip() {
        let mut arena = Arena::new(64);
        let ptr = arena.alloc_uninit::<u64>().expect("Should be enough room");
        unsafe {
            ptr.as_ptr().write(99u64);
            assert_eq!(ptr.as_ptr().read(), 99u64);
        }
    }
}
