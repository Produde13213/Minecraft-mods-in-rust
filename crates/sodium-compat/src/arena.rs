//! CPU-side range allocator for chunk vertex/index buffers.
//! Allocations are stable until explicitly released. Adjacent free ranges coalesce,
//! and all arithmetic is checked so malformed mesh sizes cannot wrap offsets.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Allocation {
    pub offset: u64,
    pub length: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArenaError {
    ZeroLength,
    OutOfSpace,
    InvalidAllocation,
    ArithmeticOverflow,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArenaStats {
    pub capacity: u64,
    pub used: u64,
    pub free: u64,
    pub largest_free_range: u64,
    pub free_ranges: usize,
}

#[derive(Debug)]
pub struct MeshArena {
    capacity: u64,
    used: u64,
    free: BTreeMap<u64, u64>,
    live: BTreeMap<u64, u64>,
}

impl MeshArena {
    pub fn new(capacity: u64) -> Self {
        let mut free = BTreeMap::new();
        if capacity > 0 { free.insert(0, capacity); }
        Self { capacity, used: 0, free, live: BTreeMap::new() }
    }

    /// First-fit allocation. The returned range is measured in caller-defined units
    /// (normally vertices or bytes); alignment is handled by the caller.
    pub fn allocate(&mut self, length: u64) -> Result<Allocation, ArenaError> {
        if length == 0 { return Err(ArenaError::ZeroLength); }
        let Some((&offset, &available)) = self.free.iter().find(|(_, n)| **n >= length) else {
            return Err(ArenaError::OutOfSpace);
        };
        let end = offset.checked_add(length).ok_or(ArenaError::ArithmeticOverflow)?;
        self.free.remove(&offset);
        if available > length {
            self.free.insert(end, available - length);
        }
        self.live.insert(offset, length);
        self.used = self.used.checked_add(length).ok_or(ArenaError::ArithmeticOverflow)?;
        Ok(Allocation { offset, length })
    }

    /// Releases exactly the allocation previously returned by allocate.
    pub fn release(&mut self, allocation: Allocation) -> Result<(), ArenaError> {
        if self.live.get(&allocation.offset).copied() != Some(allocation.length) {
            return Err(ArenaError::InvalidAllocation);
        }
        self.live.remove(&allocation.offset);
        self.used -= allocation.length;
        let mut start = allocation.offset;
        let mut length = allocation.length;

        if let Some((&previous, &previous_len)) = self.free.range(..start).next_back() {
            if previous.checked_add(previous_len) == Some(start) {
                self.free.remove(&previous);
                start = previous;
                length = length.checked_add(previous_len).ok_or(ArenaError::ArithmeticOverflow)?;
            }
        }
        let end = start.checked_add(length).ok_or(ArenaError::ArithmeticOverflow)?;
        if let Some(next_len) = self.free.remove(&end) {
            length = length.checked_add(next_len).ok_or(ArenaError::ArithmeticOverflow)?;
        }
        self.free.insert(start, length);
        Ok(())
    }

    /// Grow without moving existing allocations.
    pub fn grow(&mut self, new_capacity: u64) -> Result<(), ArenaError> {
        if new_capacity < self.capacity { return Err(ArenaError::InvalidAllocation); }
        let additional = new_capacity - self.capacity;
        if additional == 0 { return Ok(()); }
        let old_capacity = self.capacity;
        self.capacity = new_capacity;
        if let Some((&last, &last_len)) = self.free.last_key_value() {
            if last.checked_add(last_len) == Some(old_capacity) {
                self.free.insert(last, last_len.checked_add(additional).ok_or(ArenaError::ArithmeticOverflow)?);
                return Ok(());
            }
        }
        self.free.insert(old_capacity, additional);
        Ok(())
    }

    pub fn stats(&self) -> ArenaStats {
        let free = self.capacity - self.used;
        ArenaStats {
            capacity: self.capacity,
            used: self.used,
            free,
            largest_free_range: self.free.values().copied().max().unwrap_or(0),
            free_ranges: self.free.len(),
        }
    }

    pub fn contains(&self, allocation: Allocation) -> bool {
        self.live.get(&allocation.offset).copied() == Some(allocation.length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_first_fit_and_reuses_released_ranges() {
        let mut arena = MeshArena::new(100);
        let a = arena.allocate(20).unwrap();
        let b = arena.allocate(30).unwrap();
        assert_eq!((a.offset, b.offset), (0, 20));
        arena.release(a).unwrap();
        let c = arena.allocate(12).unwrap();
        assert_eq!(c.offset, 0);
        assert_eq!(arena.stats().used, 42);
    }

    #[test]
    fn adjacent_frees_coalesce_into_one_range() {
        let mut arena = MeshArena::new(128);
        let a = arena.allocate(32).unwrap();
        let b = arena.allocate(32).unwrap();
        let c = arena.allocate(32).unwrap();
        arena.release(b).unwrap();
        arena.release(a).unwrap();
        assert_eq!(arena.stats().largest_free_range, 64);
        arena.release(c).unwrap();
        assert_eq!(arena.stats().free_ranges, 1);
        assert_eq!(arena.stats().largest_free_range, 128);
    }

    #[test]
    fn grow_preserves_allocations_and_extends_tail() {
        let mut arena = MeshArena::new(16);
        let a = arena.allocate(12).unwrap();
        arena.grow(40).unwrap();
        let b = arena.allocate(20).unwrap();
        assert_eq!(a.offset, 0);
        assert_eq!(b.offset, 12);
        assert!(arena.contains(a));
    }

    #[test]
    fn rejects_double_free_and_invalid_lengths() {
        let mut arena = MeshArena::new(10);
        assert_eq!(arena.allocate(0), Err(ArenaError::ZeroLength));
        let a = arena.allocate(5).unwrap();
        arena.release(a).unwrap();
        assert_eq!(arena.release(a), Err(ArenaError::InvalidAllocation));
    }

    #[test]
    fn reports_fragmentation_separately_from_total_free_space() {
        let mut arena = MeshArena::new(100);
        let a = arena.allocate(25).unwrap();
        let b = arena.allocate(25).unwrap();
        let c = arena.allocate(25).unwrap();
        arena.release(a).unwrap();
        arena.release(c).unwrap();
        let stats = arena.stats();
        assert_eq!(stats.free, 75);
        assert_eq!(stats.largest_free_range, 50);
        assert_eq!(stats.free_ranges, 2);
        assert_eq!(arena.allocate(60), Err(ArenaError::OutOfSpace));
        arena.release(b).unwrap();
        assert_eq!(arena.stats().largest_free_range, 100);
    }
}
