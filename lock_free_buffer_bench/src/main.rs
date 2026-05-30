use std::cell::UnsafeCell;
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Instant;


const ITERATIONS: usize = 2_000_000;
const CHUNK_SIZE: usize = 1024;
const BUFFER_SIZE: usize = 10240; // Must be power of two

#[derive(Clone, Copy, Default)]
struct Complex32 {
    pub re: f32,
    pub im: f32,
}

// -----------------------------------------------------------------------------
// Lock-Free with Condvar
// -----------------------------------------------------------------------------
struct MulticastRingBufferCondvar {
    buffer: Vec<UnsafeCell<Complex32>>,
    mask: usize,
    head: AtomicUsize,
    // Dummy mutex purely to make the Condvar work
    notifier: Mutex<()>,
    condvar: Condvar,
}

unsafe impl Sync for MulticastRingBufferCondvar {}
unsafe impl Send for MulticastRingBufferCondvar {}

impl MulticastRingBufferCondvar {
    fn new(size: usize) -> Self {
        let mut buffer = Vec::with_capacity(size);
        for _ in 0..size {
            buffer.push(UnsafeCell::new(Complex32::default()));
        }
        Self {
            buffer,
            mask: size - 1,
            head: AtomicUsize::new(0),
            notifier: Mutex::new(()),
            condvar: Condvar::new(),
        }
    }

    fn write_samples(&self, samples: &[Complex32]) {
        let n = samples.len();
        let head = self.head.load(Ordering::Relaxed);
        let physical_start = head & self.mask;
        let buffer_len = self.buffer.len();

        unsafe {
            let ptr = self.buffer.as_ptr() as *mut Complex32;
            let dest = ptr.add(physical_start);

            if physical_start + n <= buffer_len {
                std::ptr::copy_nonoverlapping(samples.as_ptr(), dest, n);
            } else {
                let first_part = buffer_len - physical_start;
                std::ptr::copy_nonoverlapping(samples.as_ptr(), dest, first_part);
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr().add(first_part),
                    ptr,
                    n - first_part,
                );
            }
        }
        // Update atomic head
        self.head.store(head + n, Ordering::Release);
        
        // Notify waiting readers
        let _guard = self.notifier.lock().unwrap();
        self.condvar.notify_all();
    }

    fn copy_to_slice(&self, start: usize, dest: &mut [Complex32]) {
        let n = dest.len();
        let physical_start = start & self.mask;
        let buffer_len = self.buffer.len();

        unsafe {
            let ptr = self.buffer.as_ptr() as *const Complex32;
            let src = ptr.add(physical_start);

            if physical_start + n <= buffer_len {
                std::ptr::copy_nonoverlapping(src, dest.as_mut_ptr(), n);
            } else {
                let first_part = buffer_len - physical_start;
                std::ptr::copy_nonoverlapping(src, dest.as_mut_ptr(), first_part);
                std::ptr::copy_nonoverlapping(
                    ptr,
                    dest.as_mut_ptr().add(first_part),
                    n - first_part,
                );
            }
        }
    }

    fn wait_for_data(&self, target_head: usize) {
        // Fast path: if data is already here, don't lock the mutex at all
        if self.head.load(Ordering::Acquire) >= target_head {
            return;
        }

        // Slow path: go to sleep using OS Condvar
        let mut guard = self.notifier.lock().unwrap();
        while self.head.load(Ordering::Acquire) < target_head {
            guard = self.condvar.wait(guard).unwrap();
        }
    }
}


// -----------------------------------------------------------------------------
// Lock-Free without Condvar
// -----------------------------------------------------------------------------
struct MulticastRingBuffer {
    buffer: Vec<UnsafeCell<Complex32>>,
    mask: usize,
    head: AtomicUsize,
}

// We must explicitly promise the compiler we handle thread safety correctly
unsafe impl Sync for MulticastRingBuffer {}
unsafe impl Send for MulticastRingBuffer {}

impl MulticastRingBuffer {
    fn new(size: usize) -> Self {
        let mut buffer = Vec::with_capacity(size);
        for _ in 0..size {
            buffer.push(UnsafeCell::new(Complex32::default()));
        }
        Self {
            buffer,
            mask: size - 1,
            head: AtomicUsize::new(0),
        }
    }

    fn write_samples(&self, samples: &[Complex32]) {
        let n = samples.len();
        let head = self.head.load(Ordering::Relaxed);
        let physical_start = head & self.mask;
        let buffer_len = self.buffer.len();

        unsafe {
            let ptr = self.buffer.as_ptr() as *mut Complex32;
            let dest = ptr.add(physical_start);

            if physical_start + n <= buffer_len {
                std::ptr::copy_nonoverlapping(samples.as_ptr(), dest, n);
            } else {
                let first_part = buffer_len - physical_start;
                std::ptr::copy_nonoverlapping(samples.as_ptr(), dest, first_part);
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr().add(first_part),
                    ptr,
                    n - first_part,
                );
            }
        }
        self.head.store(head + n, Ordering::Release);
    }

    fn copy_to_slice(&self, start: usize, dest: &mut [Complex32]) {
        let n = dest.len();
        let physical_start = start & self.mask;
        let buffer_len = self.buffer.len();

        unsafe {
            let ptr = self.buffer.as_ptr() as *const Complex32;
            let src = ptr.add(physical_start);

            if physical_start + n <= buffer_len {
                std::ptr::copy_nonoverlapping(src, dest.as_mut_ptr(), n);
            } else {
                let first_part = buffer_len - physical_start;
                std::ptr::copy_nonoverlapping(src, dest.as_mut_ptr(), first_part);
                std::ptr::copy_nonoverlapping(
                    ptr,
                    dest.as_mut_ptr().add(first_part),
                    n - first_part,
                );
            }
        }
    }

    fn get_head(&self) -> usize {
        self.head.load(Ordering::Acquire)
    }
}

// -----------------------------------------------------------------------------
// Mutex
// -----------------------------------------------------------------------------
struct MutexState {
    buffer: Vec<Complex32>,
    head: usize,
    mask: usize,
}

struct MutexRingBuffer {
    state: Mutex<MutexState>,
}

impl MutexRingBuffer {
    fn new(size: usize) -> Self {
        Self {
            state: Mutex::new(MutexState {
                buffer: vec![Complex32::default(); size],
                head: 0,
                mask: size - 1,
            }),
        }
    }

    fn write_samples(&self, samples: &[Complex32]) {
        let mut state = self.state.lock().unwrap();
        let n = samples.len();
        let physical_start = state.head & state.mask;
        let buffer_len = state.buffer.len();

        if physical_start + n <= buffer_len {
            state.buffer[physical_start..physical_start + n].copy_from_slice(samples);
        } else {
            let first_part = buffer_len - physical_start;
            state.buffer[physical_start..].copy_from_slice(&samples[..first_part]);
            state.buffer[..n - first_part].copy_from_slice(&samples[first_part..]);
        }
        state.head += n;
    }

    fn copy_to_slice(&self, start: usize, dest: &mut [Complex32]) {
        let state = self.state.lock().unwrap();
        let n = dest.len();
        let physical_start = start & state.mask;
        let buffer_len = state.buffer.len();

        if physical_start + n <= buffer_len {
            dest.copy_from_slice(&state.buffer[physical_start..physical_start + n]);
        } else {
            let first_part = buffer_len - physical_start;
            dest[..first_part].copy_from_slice(&state.buffer[physical_start..]);
            dest[first_part..].copy_from_slice(&state.buffer[..n - first_part]);
        }
    }

    fn get_head(&self) -> usize {
        self.state.lock().unwrap().head
    }
}

// -----------------------------------------------------------------------------
// Benchmark
// -----------------------------------------------------------------------------
fn main() {
    println!("Benchmarking Ring Buffers...");
    println!("Payload: {} iterations of {} elements", ITERATIONS, CHUNK_SIZE);
    println!("Total Samples Written: {}", ITERATIONS * CHUNK_SIZE);
    println!("--------------------------------------------------");

    bench_lock_free_condvar();
    bench_lock_free_no_condvar();
    bench_mutex();
}

fn bench_lock_free_condvar() {
    let ring = Arc::new(MulticastRingBufferCondvar::new(BUFFER_SIZE));
    let start = Instant::now();

    // Spawn 2 Readers
    let mut reader_handles = vec![];
    for _ in 0..2 {
        let ring_clone = Arc::clone(&ring);
        reader_handles.push(thread::spawn(move || {
            let mut my_read_ptr = 0;
            let target = ITERATIONS * CHUNK_SIZE;
            let mut dest = vec![Complex32::default(); CHUNK_SIZE];

            while my_read_ptr < target {
               // Sleep until data is ready
                ring_clone.wait_for_data(my_read_ptr + CHUNK_SIZE);
                
                ring_clone.copy_to_slice(my_read_ptr, &mut dest);
                my_read_ptr += CHUNK_SIZE;
                black_box(&dest);
            }
        }));
    }

    // Spawn 1 Writer
    let ring_writer = Arc::clone(&ring);
    let writer_handle = thread::spawn(move || {
        let chunk = vec![Complex32::default(); CHUNK_SIZE];
        for _ in 0..ITERATIONS {
            ring_writer.write_samples(&chunk);
        }
    });

    writer_handle.join().unwrap();
    for handle in reader_handles {
        handle.join().unwrap();
    }

    println!("Lock-Free (UnsafeCell + Atomics + Condvar): {:?}", start.elapsed());
}


fn bench_lock_free_no_condvar() {
    let ring = Arc::new(MulticastRingBuffer::new(BUFFER_SIZE));
    let start = Instant::now();

    // Spawn 2 Readers
    let mut reader_handles = vec![];
    for _ in 0..2 {
        let ring_clone = Arc::clone(&ring);
        reader_handles.push(thread::spawn(move || {
            let mut my_read_ptr = 0;
            let target = ITERATIONS * CHUNK_SIZE;
            let mut dest = vec![Complex32::default(); CHUNK_SIZE];

            while my_read_ptr < target {
                let current_head = ring_clone.get_head();
                
                // Spin until a full chunk is available
                if current_head >= my_read_ptr + CHUNK_SIZE {
                    ring_clone.copy_to_slice(my_read_ptr, &mut dest);
                    my_read_ptr += CHUNK_SIZE;
                    black_box(&dest); // Prevent optimization
                } else {
                    std::hint::spin_loop();
                }
            }
        }));
    }

    // Spawn 1 Writer
    let ring_writer = Arc::clone(&ring);
    let writer_handle = thread::spawn(move || {
        let chunk = vec![Complex32::default(); CHUNK_SIZE];
        for _ in 0..ITERATIONS {
            ring_writer.write_samples(&chunk);
        }
    });

    writer_handle.join().unwrap();
    for handle in reader_handles {
        handle.join().unwrap();
    }

    println!("Lock-Free (UnsafeCell + Atomics): {:?}", start.elapsed());
}


fn bench_mutex() {
    let ring = Arc::new(MutexRingBuffer::new(BUFFER_SIZE));
    let start = Instant::now();

    // Spawn 2 Readers
    let mut reader_handles = vec![];
    for _ in 0..2 {
        let ring_clone = Arc::clone(&ring);
        reader_handles.push(thread::spawn(move || {
            let mut my_read_ptr = 0;
            let target = ITERATIONS * CHUNK_SIZE;
            let mut dest = vec![Complex32::default(); CHUNK_SIZE];

            while my_read_ptr < target {
                let current_head = ring_clone.get_head();
                
                // Spin until a full chunk is available
                if current_head >= my_read_ptr + CHUNK_SIZE {
                    ring_clone.copy_to_slice(my_read_ptr, &mut dest);
                    my_read_ptr += CHUNK_SIZE;
                    black_box(&dest); // Prevent optimization
                } else {
                    std::hint::spin_loop();
                }
            }
        }));
    }

    // Spawn 1 Writer
    let ring_writer = Arc::clone(&ring);
    let writer_handle = thread::spawn(move || {
        let chunk = vec![Complex32::default(); CHUNK_SIZE];
        for _ in 0..ITERATIONS {
            ring_writer.write_samples(&chunk);
        }
    });

    writer_handle.join().unwrap();
    for handle in reader_handles {
        handle.join().unwrap();
    }

    println!("Mutex-Based (Safe Rust):          {:?}", start.elapsed());
}
