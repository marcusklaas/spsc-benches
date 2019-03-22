pub fn produce_value(difficulty: usize) -> usize {
    (1..difficulty).into_iter().fold(1, |x, y| x * y)
}

pub fn consume_value(difficulty: usize) -> usize {
    (1..difficulty).into_iter().fold(1, |x, y| x * y)
}

pub mod spinlock_spsc {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, AtomicBool, AtomicPtr, Ordering};
    use std::mem;

    pub struct Sender<T> {
        capacity_mask: usize,
        offset: usize,
        other_offset: usize,

        buf1: *const T,
        // write offset is offset of next write.
        // TODO: these should probably be isizes
        buf1_write_offset: Arc<AtomicUsize>,
        // read offset is offset of next read
        buf1_read_offset: Arc<AtomicUsize>,
    }

    unsafe impl<T> Send for Sender<T> {}

    impl<T> Sender<T> {
        // TODO: maybe we could do with immutable ref? what would be benefit?
        pub fn send(&mut self, val: T) -> () {
            let next_offset = (self.offset + 1) & self.capacity_mask;

            if next_offset == self.other_offset {
                loop {
                    self.other_offset = self.buf1_read_offset.load(Ordering::Relaxed);
                    if next_offset == self.other_offset {
                        if Arc::strong_count(&self.buf1_write_offset) < 2 {
                            panic!("Receiver hung up!");
                        }
                    } else {
                        break;
                    }
                }
            }

            unsafe {
                (self.buf1 as *mut T).offset(self.offset as isize).write(val);
            }
            self.offset = next_offset;
            self.buf1_write_offset.store(self.offset, Ordering::Release);
        }
    }

    pub struct Receiver<T> {
        capacity_mask: usize,
        offset: usize,
        other_offset: usize,

        buf1: *const T,
        buf1_write_offset: Arc<AtomicUsize>,
        buf1_read_offset: Arc<AtomicUsize>,
    }

    unsafe impl<T> Send for Receiver<T> {}

    impl<T> Receiver<T> {
        pub fn recv(&mut self) -> Option<T> {
            if self.offset == self.other_offset {
                loop {
                    self.other_offset = self.buf1_write_offset.load(Ordering::Relaxed);
                    if self.offset == self.other_offset {
                        if Arc::strong_count(&self.buf1_write_offset) < 2 {
                            if self.offset == self.buf1_write_offset.load(Ordering::Acquire) {
                                return None;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }

            let res = unsafe {
                Some(self.buf1.offset(self.offset as isize).read())
            };
            self.offset = (self.offset + 1) & self.capacity_mask;
            self.buf1_read_offset.store(self.offset, Ordering::Release);

            res
        }
    }

    /// Creates a single consumer, single produced channel with bounded capacity.
    pub fn bounded<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        assert!(capacity > 0);

        // least power of 2 greater than capacity
        let real_capacity = 1 << ((mem::size_of::<usize>() * 8) - capacity.leading_zeros() as usize - 1);

        let buf1 = Vec::with_capacity(real_capacity);
        let buf1_write_offset: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
        let buf1_read_offset: Arc<AtomicUsize> = Arc::new(Default::default());

        let sender = Sender {
            capacity_mask: real_capacity - 1,
            offset: 0,
            other_offset: 0,

            buf1: buf1.as_ptr(),
            buf1_write_offset: buf1_write_offset.clone(),
            buf1_read_offset: buf1_read_offset.clone(),
        };
        let receiver = Receiver {
            capacity_mask: real_capacity - 1,
            offset: 0,
            other_offset: 0,

            buf1: buf1.as_ptr(),
            buf1_write_offset: buf1_write_offset,
            buf1_read_offset: buf1_read_offset,
        };

        mem::forget(buf1);

        (sender, receiver)
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn simple_send() {
            let (mut snd, mut recv) = bounded(4);
            snd.send(15u32);
            assert_eq!(recv.recv().unwrap(), 15u32);
            snd.send(5u32);
            snd.send(7u32);

            assert_eq!(recv.recv().unwrap(), 5u32);
            assert_eq!(recv.recv().unwrap(), 7u32);

            snd.send(1u32);
            snd.send(1u32);

            assert_eq!(recv.recv().unwrap(), 1u32);
            assert_eq!(recv.recv().unwrap(), 1u32);
            snd.send(1u32);
        }

        #[test]
        fn hangup() {
            let (mut snd, mut recv) = bounded::<isize>(2);
            snd.send(10);
            mem::drop(snd);
            assert_eq!(recv.recv().unwrap(), 10);
            assert!(recv.recv().is_none());
        }
    }
}