//! cpal audio output: emulator pushes samples into a ring buffer, the audio
//! callback drains it at the device sample rate.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct Audio {
    pub sample_rate: f64,
    ring: Arc<Mutex<VecDeque<f32>>>,
    _stream: cpal::Stream,
}

impl Audio {
    /// Try to open the default output device (F32). Returns None on failure so
    /// the GUI can run silently rather than crash.
    pub fn new() -> Option<Audio> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        let config = device.default_output_config().ok()?;
        if config.sample_format() != cpal::SampleFormat::F32 {
            log::warn!("audio: non-F32 device format, running silent");
            return None;
        }
        let sample_rate = config.sample_rate().0 as f64;
        let channels = config.channels() as usize;
        let ring: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::with_capacity(8192)));
        let ring2 = ring.clone();
        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _| {
                    let mut buf = ring2.lock().unwrap();
                    for frame in data.chunks_mut(channels) {
                        let s = buf.pop_front().unwrap_or(0.0);
                        for c in frame.iter_mut() {
                            *c = s;
                        }
                    }
                },
                |err| log::error!("audio stream error: {err}"),
                None,
            )
            .ok()?;
        stream.play().ok()?;
        Some(Audio {
            sample_rate,
            ring,
            _stream: stream,
        })
    }

    /// Current number of queued samples (drained by the audio callback at the
    /// device sample rate). Used to pace emulation to the audio clock.
    pub fn buffered(&self) -> usize {
        self.ring.lock().unwrap().len()
    }

    /// Queue samples, dropping the oldest if the buffer grows too large (keeps
    /// latency bounded if emulation outruns playback).
    pub fn queue(&self, samples: &[f32]) {
        let mut buf = self.ring.lock().unwrap();
        let max = (self.sample_rate as usize) / 8; // ~125ms cap
        if buf.len() > max {
            let drop = buf.len() - max;
            buf.drain(0..drop);
        }
        buf.extend(samples.iter().copied());
    }
}
