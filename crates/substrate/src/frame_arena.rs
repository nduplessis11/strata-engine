//! Frame-scoped arena allocator.
//!
//! Allocations are valid only until the next [`FrameArena::begin_frame`] call.
//! After that, all previously returned pointers are stale and must not be
//! dereferenced — the underlying memory may be overwritten.

use crate::arena::{Arena, ArenaError};
use std::ptr::NonNull;

/// An arena whose lifetime is explicitly divided into frames.
///
/// Allocations from a `FrameArena` are valid only until the next call to
/// [`begin_frame`](FrameArena::begin_frame). After `begin_frame()` returns,
/// all previously returned pointers are **stale** and dereferencing them is
/// **undefined behaviour** — the memory may be overwritten by new allocations
/// in the same frame.
///
/// # Example
///
/// ```rust
/// use substrate::frame_arena::FrameArena;
///
/// struct DrawCmd { mesh_id: u64, material_id: u64 }
///
/// let mut fa = FrameArena::new(4096);
///
/// // --- frame 0 ---
/// let cmd = fa.alloc(DrawCmd { mesh_id: 1, material_id: 2 }).unwrap();
/// // Use `cmd` only within this frame.
/// unsafe { println!("mesh {}", cmd.as_ptr().read().mesh_id); }
///
/// // --- frame 1: previous `cmd` is now invalid ---
/// fa.begin_frame();
/// let cmd2 = fa.alloc(DrawCmd { mesh_id: 3, material_id: 4 }).unwrap();
/// unsafe { println!("mesh {}", cmd2.as_ptr().read().mesh_id); }
/// ```
pub struct FrameArena {
    inner: Arena,
}

impl FrameArena {
    pub fn new(capacity: usize) -> Self {
        Self { inner: Arena::new(capacity) }
    }

    /// Resets the allocator for a new frame. All pointers from the previous
    /// frame become invalid immediately; using them after this call is
    /// undefined behaviour.
    pub fn begin_frame(&mut self) {
        self.inner.reset();
    }

    pub fn alloc<T>(&mut self, value: T) -> Result<NonNull<T>, ArenaError> {
        self.inner.alloc(value)
    }

    pub fn alloc_uninit<T>(&mut self) -> Result<NonNull<T>, ArenaError> {
        self.inner.alloc_uninit()
    }

    pub fn used(&self) -> usize {
        self.inner.used()
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_frame_resets_used_to_zero() {
        let mut fa = FrameArena::new(128);
        fa.alloc::<u64>(1).unwrap();
        fa.alloc::<u64>(2).unwrap();
        assert!(fa.used() > 0);
        fa.begin_frame();
        assert_eq!(fa.used(), 0);
    }

    #[test]
    fn alloc_across_frames_reads_correct_values() {
        let mut fa = FrameArena::new(128);

        let _p0 = fa.alloc::<u64>(10).unwrap();
        fa.begin_frame();

        // After begin_frame, new allocations reuse the same memory region.
        let p1 = fa.alloc::<u64>(20).unwrap();
        assert_eq!(unsafe { *p1.as_ptr() }, 20);
    }
}
