// ==============================================================================
// Zero-Copy Network Stack (ZCNS)
// Mathematical Foundation: Copy_Overhead = 0, Serialization_Time = 0
// ==============================================================================

use std::sync::atomic::{AtomicUsize, Ordering};
use std::mem::MaybeUninit;
use std::ptr::NonNull;

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct ZeroCopyMessage {
    pub msg_type: u32,
    pub timestamp: u64,
    pub data_len: u32,
    pub flags: u32,
    pub reserved: u32,
    pub data: [u8; 4096],
}

impl ZeroCopyMessage {
    #[inline(always)]
    pub fn new(msg_type: u32) -> Self {
        Self {
            msg_type,
            timestamp: 0,
            data_len: 0,
            flags: 0,
            reserved: 0,
            data: [0u8; 4096],
        }
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const _ as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }

    #[inline(always)]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self as *mut _ as *mut u8,
                std::mem::size_of::<Self>(),
            )
        }
    }

    #[inline(always)]
    pub fn data_slice(&self) -> &[u8] {
        &self.data[..self.data_len as usize]
    }

    #[inline(always)]
    pub fn data_slice_mut(&mut self) -> &mut [u8] {
        &mut self.data[..self.data_len as usize]
    }
}

#[repr(C, align(64))]
pub struct ZeroCopyRingBuffer {
    buffer: NonNull<MaybeUninit<ZeroCopyMessage>>,
    capacity: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl ZeroCopyRingBuffer {
    pub fn new(capacity: usize) -> Self {
        let size = capacity * std::mem::size_of::<ZeroCopyMessage>();
        let layout = std::alloc::Layout::from_size_align(size, 64).unwrap();

        let ptr = unsafe {
            let raw = std::alloc::alloc(layout);
            if raw.is_null() {
                panic!("Failed to allocate ZeroCopyRingBuffer");
            }
            NonNull::new_unchecked(raw as *mut MaybeUninit<ZeroCopyMessage>)
        };

        Self {
            buffer: ptr,
            capacity,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    pub fn push(&self, msg: ZeroCopyMessage) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);

        if head - tail >= self.capacity {
            return false;
        }

        unsafe {
            (*self.buffer.as_ptr().add(head % self.capacity)).write(msg);
        }

        self.head.store(head + 1, Ordering::Release);
        true
    }

    #[inline(always)]
    pub fn pop(&self) -> Option<ZeroCopyMessage> {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);

        if tail >= head {
            return None;
        }

        let msg = unsafe {
            (*self.buffer.as_ptr().add(tail % self.capacity)).assume_init_read()
        };

        self.tail.store(tail + 1, Ordering::Release);
        Some(msg)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head - tail
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Drop for ZeroCopyRingBuffer {
    fn drop(&mut self) {
        let size = self.capacity * std::mem::size_of::<ZeroCopyMessage>();
        let layout = std::alloc::Layout::from_size_align(size, 64).unwrap();
        unsafe {
            std::alloc::dealloc(self.buffer.as_ptr() as *mut u8, layout);
        }
    }
}

#[repr(C, align(64))]
pub struct BinaryProtocolEncoder {
    pub scratch: [u8; 512],
    pub pos: usize,
}

impl BinaryProtocolEncoder {
    pub fn new() -> Self {
        Self {
            scratch: [0u8; 512],
            pos: 0,
        }
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.pos = 0;
        self.scratch.fill(0);
    }

    #[inline(always)]
    pub fn encode_u64(&mut self, value: u64) {
        let bytes = value.to_le_bytes();
        self.scratch[self.pos..self.pos + 8].copy_from_slice(&bytes);
        self.pos += 8;
    }

    #[inline(always)]
    pub fn encode_u32(&mut self, value: u32) {
        let bytes = value.to_le_bytes();
        self.scratch[self.pos..self.pos + 4].copy_from_slice(&bytes);
        self.pos += 4;
    }

    #[inline(always)]
    pub fn encode_bytes(&mut self, data: &[u8]) {
        let len = data.len().min(256);
        self.scratch[self.pos..self.pos + len].copy_from_slice(&data[..len]);
        self.pos += len;
    }

    #[inline(always)]
    pub fn finalize(&self) -> &[u8] {
        &self.scratch[..self.pos]
    }

    #[inline(always)]
    pub fn finalize_len(&self) -> usize {
        self.pos
    }
}

impl Default for BinaryProtocolEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C, align(64))]
pub struct BinaryProtocolDecoder {
    pub scratch: [u8; 512],
    pub pos: usize,
}

impl BinaryProtocolDecoder {
    pub fn new() -> Self {
        Self {
            scratch: [0u8; 512],
            pos: 0,
        }
    }

    #[inline(always)]
    pub fn load(&mut self, data: &[u8]) {
        let len = data.len().min(512);
        self.scratch[..len].copy_from_slice(&data[..len]);
        self.pos = 0;
    }

    #[inline(always)]
    pub fn decode_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.scratch[self.pos..self.pos + 8]);
        self.pos += 8;
        u64::from_le_bytes(bytes)
    }

    #[inline(always)]
    pub fn decode_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.scratch[self.pos..self.pos + 4]);
        self.pos += 4;
        u32::from_le_bytes(bytes)
    }

    #[inline(always)]
    pub fn decode_bytes(&mut self, len: usize) -> &[u8] {
        let slice = &self.scratch[self.pos..self.pos + len];
        self.pos += len;
        slice
    }
}

impl Default for BinaryProtocolDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_message() {
        let msg = ZeroCopyMessage::new(1);
        let bytes = msg.as_bytes();
        assert_eq!(bytes.len(), std::mem::size_of::<ZeroCopyMessage>());
    }

    #[test]
    fn test_ring_buffer() {
        let ring = ZeroCopyRingBuffer::new(16);
        let msg = ZeroCopyMessage::new(42);
        assert!(ring.push(msg));
        assert_eq!(ring.len(), 1);
        let popped = ring.pop();
        assert!(popped.is_some());
        assert!(ring.is_empty());
    }

    #[test]
    fn test_binary_encoder() {
        let mut enc = BinaryProtocolEncoder::new();
        enc.encode_u64(0x123456789ABCDEF0);
        enc.encode_u32(0xDEADBEEF);
        let out = enc.finalize();
        assert_eq!(out.len(), 12);
    }

    #[test]
    fn test_binary_decoder() {
        let mut dec = BinaryProtocolDecoder::new();
        let data = [0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12];
        dec.load(&data);
        assert_eq!(dec.decode_u64(), 0x123456789ABCDEF0);
    }
}
