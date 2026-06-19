//! Native cpal audio output. The emulator worker pushes samples into a ring
//! buffer; the device callback drains it at the hardware sample rate. The
//! buffer fill (`buffered`) lets the worker pace emulation to the audio clock —
//! no IPC, no main-thread timers, so playback is immune to WebView throttling
//! (e.g. when the window is minimized).
//!
//! `cpal::Stream` is `!Send`, so `Audio` is created and owned entirely by the
//! worker thread and never crosses a thread boundary. Only `ring` (an `Arc`)
//! is shared — with the real-time audio callback, which is the sole consumer.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct Audio {
    pub sample_rate: f64,
    ring: Arc<Mutex<VecDeque<f32>>>,
    _stream: cpal::Stream,
}

impl Audio {
    /// Open the default output device (F32). Returns `None` on any failure so
    /// the emulator runs silently (wall-clock paced) rather than crashing.
    pub fn new() -> Option<Audio> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        let config = device.default_output_config().ok()?;
        if config.sample_format() != cpal::SampleFormat::F32 {
            eprintln!("# audio: non-F32 device format, running silent");
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
                        let s = buf.pop_front().unwrap_or(0.0); // underrun → silence
                        for c in frame.iter_mut() {
                            *c = s; // mono → duplicated across channels
                        }
                    }
                },
                |err| eprintln!("# audio stream error: {err}"),
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

    /// Samples currently queued (drained by the callback at the device rate).
    /// The worker runs frames until this reaches its target, locking emulation
    /// speed to the sound card's stable clock.
    pub fn buffered(&self) -> usize {
        self.ring.lock().unwrap().len()
    }

    /// Queue samples, dropping the oldest if the buffer overruns (bounds latency
    /// under fast-forward, where emulation outpaces playback).
    pub fn queue(&self, samples: &[f32]) {
        let mut buf = self.ring.lock().unwrap();
        let max = (self.sample_rate as usize) / 8; // ~125 ms cap
        if buf.len() > max {
            let drop = buf.len() - max;
            buf.drain(0..drop);
        }
        buf.extend(samples.iter().copied());
    }
}
