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
        capacity: usize,
        write_offset: usize,

        buf1: *mut T,
        buf1_offset: Arc<AtomicUsize>,
        buf1_started_writing: Arc<AtomicBool>,
        buf1_finished_reading: Arc<AtomicBool>,

        buf2: *mut T,
        buf2_offset: Arc<AtomicUsize>,
        buf2_started_writing: Arc<AtomicBool>,
        buf2_finished_reading: Arc<AtomicBool>,
    }

    impl<T> Sender<T> {
        // TODO: maybe we could do with immutable ref? what would be benefit?
        pub fn send(&mut self, val: T) -> () {
            if self.write_offset == self.capacity {
                // spin lock until buf2 is ready for us
                while !self.buf2_finished_reading.load(Ordering::Acquire) {
                    if Arc::strong_count(&self.buf1_offset) < 2 {
                        panic!("Receiver hung up!");
                    }
                }

                self.write_offset = 0;
                mem::swap(&mut self.buf1, &mut self.buf2);
                mem::swap(&mut self.buf1_offset, &mut self.buf2_offset);
                mem::swap(&mut self.buf1_started_writing, &mut self.buf2_started_writing);
                mem::swap(&mut self.buf1_finished_reading, &mut self.buf2_finished_reading);

                self.buf1_offset.store(0, Ordering::Release);
                self.buf1_started_writing.store(true, Ordering::Release);
                // TODO: is this final ordering correct?
                self.buf2_started_writing.store(false, Ordering::Relaxed);
            }

            // always write to buf1
            unsafe {
                self.buf1.offset(self.write_offset as isize).write(val);
            }
            self.write_offset += 1;
            self.buf1_offset.store(self.write_offset, Ordering::Release);
        }
    }

    pub struct Receiver<T> {
        capacity: usize,
        read_offset: usize,
        next_offset: usize,

        buf1: *const T,
        buf1_offset: Arc<AtomicUsize>,
        buf1_started_writing: Arc<AtomicBool>,
        buf1_finished_reading: Arc<AtomicBool>,

        buf2: *const T,
        buf2_offset: Arc<AtomicUsize>,
        buf2_started_writing: Arc<AtomicBool>,
        buf2_finished_reading: Arc<AtomicBool>,
    }

    impl<T> Receiver<T> {
        pub fn recv(&mut self) -> Option<T> {
            // always load from buf1

            // spinlock on start of start write
            if self.read_offset == self.capacity {
                while !self.buf2_started_writing.load(Ordering::Acquire) {
                    if Arc::strong_count(&self.buf1_offset) < 2 {
                        if !self.buf2_started_writing.load(Ordering::Acquire) {
                            return None;
                        }
                    }
                }

                self.read_offset = 0;
                self.next_offset = 0;
                mem::swap(&mut self.buf1, &mut self.buf2);
                mem::swap(&mut self.buf1_offset, &mut self.buf2_offset);
                mem::swap(&mut self.buf1_started_writing, &mut self.buf2_started_writing);
                mem::swap(&mut self.buf1_finished_reading, &mut self.buf2_finished_reading);
                
                // TODO: check ordering
                self.buf1_finished_reading.store(false, Ordering::Relaxed);
            }

            // spinlock on next value
            while self.read_offset == self.next_offset {
                if Arc::strong_count(&self.buf1_offset) < 2 {
                    if self.buf1_offset.load(Ordering::Acquire) == self.read_offset {
                        return None;
                    }
                }
                self.next_offset = self.buf1_offset.load(Ordering::Acquire);
            }

            let res = unsafe {
                Some(self.buf1.offset(self.read_offset as isize).read())
            };
            self.read_offset += 1;

            if self.read_offset == self.capacity {
                self.buf1_finished_reading.store(true, Ordering::Relaxed);
            }

            res
        }
    }

    /// Creates a single consumer, single produced channel with bounded capacity.
    pub fn bounded<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        assert!(capacity > 0);

        let buf1 = Vec::with_capacity(capacity);
        let buf1_offset: Arc<AtomicUsize> = Arc::new(Default::default());
        let buf1_started_writing = Arc::new(AtomicBool::new(false));
        let buf1_finished_reading = Arc::new(AtomicBool::new(true));

        let buf2 = Vec::with_capacity(capacity);
        let buf2_offset: Arc<AtomicUsize> = Arc::new(Default::default());
        let buf2_started_writing = Arc::new(AtomicBool::new(false));
        let buf2_finished_reading = Arc::new(AtomicBool::new(true));

        let sender = Sender {
            capacity,
            write_offset: 0,

            buf1: buf1.as_ptr() as *mut T,
            buf1_offset: buf1_offset.clone(),
            buf1_started_writing: buf1_started_writing.clone(),
            buf1_finished_reading: buf1_finished_reading.clone(),

            buf2: buf2.as_ptr() as *mut T,
            buf2_offset: buf2_offset.clone(),
            buf2_started_writing: buf2_started_writing.clone(),
            buf2_finished_reading: buf2_finished_reading.clone(),
        };
        let receiver = Receiver {
            capacity,
            read_offset: 0,
            next_offset: 0,

            buf1: buf1.as_ptr(),
            buf1_offset,
            buf1_started_writing,
            buf1_finished_reading,
        
            buf2: buf2.as_ptr(),
            buf2_offset,
            buf2_started_writing,
            buf2_finished_reading,
        };

        mem::forget(buf1);
        mem::forget(buf2);

        (sender, receiver)
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn simple_send() {
            let (mut snd, mut recv) = bounded(2);
            snd.send(15u32);
            assert_eq!(recv.recv().unwrap(), 15u32);
            snd.send(5u32);
            snd.send(7u32);

            assert_eq!(recv.recv().unwrap(), 5u32);
            assert_eq!(recv.recv().unwrap(), 7u32);

            snd.send(1u32);
            snd.send(1u32);
            snd.send(1u32);

            assert_eq!(recv.recv().unwrap(), 1u32);
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