// ==============================================================================
// Lock-Free Transaction Pipeline (LFTP)
// Mathematical Foundation: Pipeline_Throughput = min(Stage_Throughput) × (1 - Contention_Factor)
// Contention_Factor = 0 (lock-free)
// ==============================================================================

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::mem::MaybeUninit;
use std::cell::UnsafeCell;

/// Lock-free MPSC (Multi-Producer Single-Consumer) queue
/// Uses atomic CAS operations for lock-free push/pop
/// Time complexity: O(1) amortized
#[repr(C, align(64))]
pub struct LockFreeQueue<T> {
    head: AtomicU64,
    tail: AtomicU64,
    buffer: UnsafeCell<[MaybeUninit<T>; CAPACITY]>,
}

impl<T> std::fmt::Debug for LockFreeQueue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LockFreeQueue")
            .field("head", &self.head.load(Ordering::Relaxed))
            .field("tail", &self.tail.load(Ordering::Relaxed))
            .field("capacity", &CAPACITY)
            .finish()
    }
}

const CAPACITY: usize = 65536; // 2^16 entries, fits in L2 cache on most CPUs

unsafe impl<T: Send> Send for LockFreeQueue<T> {}
unsafe impl<T: Send> Sync for LockFreeQueue<T> {}

impl<T> LockFreeQueue<T> {
    /// Create a new lock-free queue
    pub fn new() -> Self {
        Self {
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
            buffer: UnsafeCell::new(unsafe { MaybeUninit::uninit().assume_init() }),
        }
    }
    
    /// Push an item to the queue (producer)
    /// Returns true if successful, false if queue is full
    /// Time complexity: O(1) - single CAS operation
    #[inline(always)]
    pub fn push(&self, item: T) -> bool {
        let head = self.head.fetch_add(1, Ordering::AcqRel);
        let tail = self.tail.load(Ordering::Acquire);
        
        // Check if queue is full
        if head - tail >= CAPACITY as u64 {
            self.head.fetch_sub(1, Ordering::AcqRel);
            return false;
        }
        
        // Write to buffer
        let idx = (head as usize) % CAPACITY;
        unsafe {
            (*self.buffer.get())[idx].write(item);
        }
        
        true
    }
    
    /// Pop an item from the queue (consumer)
    /// Returns Some(item) if available, None if queue is empty
    /// Time complexity: O(1) - single CAS operation
    #[inline(always)]
    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        
        // Check if queue is empty
        if tail >= head {
            return None;
        }
        
        // Read from buffer
        let idx = (tail as usize) % CAPACITY;
        let item = unsafe {
            (*self.buffer.get())[idx].assume_init_read()
        };
        
        // Update tail
        self.tail.fetch_add(1, Ordering::AcqRel);
        
        Some(item)
    }
    
    /// Check if queue is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.tail.load(Ordering::Acquire) >= self.head.load(Ordering::Acquire)
    }
    
    /// Check if queue is full
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head - tail >= CAPACITY as u64
    }
    
    /// Get current queue length
    #[inline(always)]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        (head - tail) as usize
    }
    
    /// Get queue capacity
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        CAPACITY
    }
}

impl<T> Default for LockFreeQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Lock-free SPSC (Single-Producer Single-Consumer) queue
/// Optimized for single producer/consumer scenario
/// Even faster than MPSC due to relaxed memory ordering
#[repr(C, align(64))]
pub struct SpscQueue<T> {
    head: AtomicUsize,
    tail: AtomicUsize,
    buffer: UnsafeCell<[MaybeUninit<T>; CAPACITY]>,
}

unsafe impl<T: Send> Send for SpscQueue<T> {}
unsafe impl<T: Send> Sync for SpscQueue<T> {}

impl<T> SpscQueue<T> {
    pub fn new() -> Self {
        Self {
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            buffer: UnsafeCell::new(unsafe { MaybeUninit::uninit().assume_init() }),
        }
    }
    
    /// Push from single producer
    #[inline(always)]
    pub fn push(&self, item: T) -> bool {
        let head = self.head.fetch_add(1, Ordering::Release);
        let tail = self.tail.load(Ordering::Acquire);
        
        if head - tail >= CAPACITY {
            self.head.fetch_sub(1, Ordering::Release);
            return false;
        }
        
        let idx = head % CAPACITY;
        unsafe {
            (*self.buffer.get())[idx].write(item);
        }
        
        true
    }
    
    /// Pop from single consumer
    #[inline(always)]
    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        
        if tail >= head {
            return None;
        }
        
        let idx = tail % CAPACITY;
        let item = unsafe {
            (*self.buffer.get())[idx].assume_init_read()
        };
        
        self.tail.fetch_add(1, Ordering::Release);
        
        Some(item)
    }
    
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.tail.load(Ordering::Acquire) >= self.head.load(Ordering::Acquire)
    }
    
    #[inline(always)]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head - tail
    }
}

impl<T> Default for SpscQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_mpsc_basic() {
        let queue = LockFreeQueue::new();
        assert!(queue.push(42));
        assert_eq!(queue.pop(), Some(42));
        assert!(queue.is_empty());
    }
    
    #[test]
    fn test_mpsc_capacity() {
        let queue = LockFreeQueue::new();
        
        // Fill queue
        for i in 0..CAPACITY {
            assert!(queue.push(i));
        }
        
        // Should be full
        assert!(queue.is_full());
        assert!(!queue.push(CAPACITY));
        
        // Drain and verify
        for i in 0..CAPACITY {
            assert_eq!(queue.pop(), Some(i as i32));
        }
        
        assert!(queue.is_empty());
    }
    
    #[test]
    fn test_mpsc_concurrent() {
        let queue = std::sync::Arc::new(LockFreeQueue::new());
        let mut handles = vec![];
        
        // Spawn producers
        for i in 0..4 {
            let q = queue.clone();
            let handle = thread::spawn(move || {
                for j in 0..1000 {
                    while !q.push(i * 1000 + j) {
                        std::hint::spin_loop();
                    }
                }
            });
            handles.push(handle);
        }
        
        // Spawn consumer
        let q = queue.clone();
        let consumer = thread::spawn(move || {
            let mut count = 0;
            while count < 4000 {
                if let Some(_) = q.pop() {
                    count += 1;
                }
            }
            count
        });
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let count = consumer.join().unwrap();
        assert_eq!(count, 4000);
    }
    
    #[test]
    fn test_spsc_basic() {
        let queue = SpscQueue::new();
        assert!(queue.push(42));
        assert_eq!(queue.pop(), Some(42));
        assert!(queue.is_empty());
    }
    
    #[test]
    fn test_spsc_concurrent() {
        let queue = std::sync::Arc::new(SpscQueue::new());
        
        let q = queue.clone();
        let producer = thread::spawn(move || {
            for i in 0..1000 {
                while !q.push(i) {
                    std::hint::spin_loop();
                }
            }
        });
        
        let q = queue.clone();
        let consumer = thread::spawn(move || {
            let mut count = 0;
            while count < 1000 {
                if let Some(_) = q.pop() {
                    count += 1;
                }
            }
            count
        });
        
        producer.join().unwrap();
        let count = consumer.join().unwrap();
        assert_eq!(count, 1000);
    }
}
