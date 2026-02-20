//! arena.rs

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
        let next = (current + layout.align() - 1) & !(layout.align() - 1);

        let new_offset = next - base + layout.size();
        if new_offset > self.capacity() {
            return Err(ArenaError::OutOfMemory);
        }
        self.offset = new_offset;

        NonNull::new(next as *mut u8).ok_or(ArenaError::OutOfMemory)
    }

    pub fn reset(&mut self) {
        self.offset = 0;
    }

    pub fn alloc<T: Copy>(&mut self, value: T) -> Result<&mut T, ArenaError> {
        let layout = Layout::new::<T>();
        let ptr = self.alloc_layout(layout)?.as_ptr() as *mut T;
        unsafe { ptr.write(value) };
        return unsafe { ptr.as_mut().ok_or(ArenaError::OutOfMemory) };
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
        assert_eq!(*ptr, 42);
    }
}
