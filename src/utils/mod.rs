use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::sync::{Arc, Mutex};

pub fn init_ringbuffer(sampling_rate: usize) -> Arc<Mutex<AllocRingBuffer<f32>>> {
    let mut buf = AllocRingBuffer::new((5 * sampling_rate).next_power_of_two());
    buf.fill(0.0);
    Arc::new(Mutex::new(buf))
}
