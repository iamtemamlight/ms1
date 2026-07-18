// ==============================================================================
// Zero-Allocation Memory Pool (ZAMP)
// Mathematical Foundation: Allocation_Time = 0, Fragmentation = 0
// ==============================================================================

use std::sync::atomic::{AtomicUsize, Ordering};
use std::alloc::{Layout, alloc, dealloc};
use std::ptr::NonNull;

/// Zero-allocation memory pool with bump pointer allocation
/// Pre-allocates all memory at startup, eliminates runtime allocation overhead
#[repr(C, align(64))]
pub struct MemoryPool {
    buffer: NonNull<u8>,
    capacity: usize,
    current: AtomicUsize,
    watermark: AtomicUsize,
}

unsafe impl Send for MemoryPool {}
unsafe impl Sync for MemoryPool {}

impl std::fmt::Debug for MemoryPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryPool")
            .field("capacity", &self.capacity)
            .field("current", &self.current.load(Ordering::Relaxed))
            .field("watermark", &self.watermark.load(Ordering::Relaxed))
            .finish()
    }
}

impl MemoryPool {
    /// Create a new memory pool with specified capacity in bytes
    /// 
    /// # Safety
    /// Caller must ensure capacity is reasonable for system memory
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be > 0");
        
        let layout = Layout::from_size_align(capacity, 64)
            .expect("Invalid layout");
        
        let buffer = unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                panic!("Failed to allocate memory pool of {} bytes", capacity);
            }
            NonNull::new_unchecked(ptr)
        };
        
        Self {
            buffer,
            capacity,
            current: AtomicUsize::new(0),
            watermark: AtomicUsize::new(0),
        }
    }
    
    /// Allocate memory from pool with bump pointer
    /// Time complexity: O(1) - single atomic add
    /// 
    /// # Safety
    /// Caller must ensure size <= remaining capacity
    #[inline(always)]
    pub fn allocate(&self, size: usize, align: usize) -> *mut u8 {
        // Align current position
        let current = self.current.load(Ordering::Acquire);
        let aligned = (current + align - 1) & !(align - 1);
        let new_current = aligned + size;
        
        assert!(new_current <= self.capacity, 
                "Memory pool exhausted: requested {} bytes, capacity {}", 
                new_current, self.capacity);
        
        // Atomic CAS to reserve space
        let result = self.current.compare_exchange_weak(
            current,
            new_current,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
        
        let actual_current = match result {
            Ok(_) => aligned,
            Err(actual) => {
                // CAS failed, retry with new current
                return self.allocate(size, align);
            }
        };
        
        // Update watermark
        let mut current_watermark = self.watermark.load(Ordering::Acquire);
        while new_current > current_watermark {
            match self.watermark.compare_exchange_weak(
                current_watermark,
                new_current,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(w) => current_watermark = w,
            }
        }
        
        unsafe { self.buffer.as_ptr().add(actual_current) }
    }
    
    /// Allocate with default 64-byte alignment (cache line)
    #[inline(always)]
    pub fn allocate_aligned(&self, size: usize) -> *mut u8 {
        self.allocate(size, 64)
    }
    
    /// Reset pool to initial state (zero-cost)
    /// All previously allocated memory is invalidated
    #[inline(always)]
    pub fn reset(&self) {
        self.current.store(0, Ordering::Release);
    }
    
    /// Get current usage in bytes
    #[inline(always)]
    pub fn usage(&self) -> usize {
        self.current.load(Ordering::Acquire)
    }
    
    /// Get peak usage (watermark) in bytes
    #[inline(always)]
    pub fn watermark(&self) -> usize {
        self.watermark.load(Ordering::Acquire)
    }
    
    /// Get capacity in bytes
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Get utilization percentage
    #[inline(always)]
    pub fn utilization(&self) -> f64 {
        (self.usage() as f64 / self.capacity as f64) * 100.0
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 64)
            .expect("Invalid layout");
        unsafe {
            dealloc(self.buffer.as_ptr(), layout);
        }
    }
}

/// Type-safe allocator wrapper for specific types
pub struct TypedAllocator<T> {
    pool: &'static MemoryPool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> TypedAllocator<T> {
    pub fn new(pool: &'static MemoryPool) -> Self {
        Self {
            pool,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Allocate space for n instances of T
    #[inline(always)]
    pub fn allocate_array(&self, n: usize) -> *mut T {
        let size = std::mem::size_of::<T>() * n;
        let align = std::mem::align_of::<T>();
        self.pool.allocate(size, align) as *mut T
    }
    
    /// Allocate single instance of T
    #[inline(always)]
    pub fn allocate_one(&self) -> *mut T {
        self.allocate_array(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_pool_creation() {
        let pool = MemoryPool::new(1024);
        assert_eq!(pool.capacity(), 1024);
        assert_eq!(pool.usage(), 0);
    }
    
    #[test]
    fn test_basic_allocation() {
        let pool = MemoryPool::new(1024);
        let ptr = pool.allocate_aligned(64);
        assert!(!ptr.is_null());
        assert_eq!(pool.usage(), 64);
    }
    
    #[test]
    fn test_alignment() {
        let pool = MemoryPool::new(1024);
        let ptr1 = pool.allocate(33, 64);
        let ptr2 = pool.allocate(33, 64);
        
        // Check alignment
        assert_eq!(ptr1 as usize % 64, 0);
        assert_eq!(ptr2 as usize % 64, 0);
    }
    
    #[test]
    fn test_watermark() {
        let pool = MemoryPool::new(1024);
        pool.allocate_aligned(100);
        pool.allocate_aligned(200);
        pool.reset();
        pool.allocate_aligned(50);
        
        assert_eq!(pool.watermark(), 300); // Peak was 100 + 200
        assert_eq!(pool.usage(), 50);
    }
    
    #[test]
    fn test_utilization() {
        let pool = MemoryPool::new(1000);
        pool.allocate_aligned(500);
        assert!((pool.utilization() - 50.0).abs() < 0.1);
    }
    
    #[test]
    fn test_typed_allocator() {
        let pool = Box::leak(Box::new(MemoryPool::new(1024)));
        let allocator: TypedAllocator<u64> = TypedAllocator::new(pool);
        
        let ptr = allocator.allocate_array(10);
        assert!(!ptr.is_null());
    }
    
    #[test]
    #[should_panic(expected = "Memory pool exhausted")]
    fn test_exhaustion() {
        let pool = MemoryPool::new(100);
        pool.allocate_aligned(200); // Should panic
    }
}
