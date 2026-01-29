// Pluely windows speaker input and stream
use super::AudioDevice;
use anyhow::Result;
use futures_util::Stream;
use std::collections::VecDeque;
use std::sync::{mpsc, Arc, Mutex};
use std::task::{Poll, Waker};
use std::thread;
use std::time::Duration;
use tracing::error;
use wasapi::{get_default_device, DeviceCollection, Direction, SampleType, StreamMode, WaveFormat};

pub fn get_input_devices() -> Result<Vec<AudioDevice>> {
    let mut devices = Vec::new();

    let default_device = get_default_device(&Direction::Capture).ok();
    let default_id = default_device.as_ref().and_then(|d| d.get_id().ok());

    let collection = DeviceCollection::new(&Direction::Capture)?;
    let count = collection.get_nbr_devices()?;

    for i in 0..count {
        if let Ok(device) = collection.get_device_at_index(i) {
            let name = device
                .get_friendlyname()
                .unwrap_or_else(|_| format!("Microphone {}", i));
            let id = device
                .get_id()
                .unwrap_or_else(|_| format!("windows_input_{}", i));
            let is_default = default_id.as_ref().map(|def| def == &id).unwrap_or(false);

            devices.push(AudioDevice {
                id,
                name,
                is_default,
            });
        }
    }

    Ok(devices)
}

pub fn get_output_devices() -> Result<Vec<AudioDevice>> {
    let mut devices = Vec::new();

    let default_device = get_default_device(&Direction::Render).ok();
    let default_id = default_device.as_ref().and_then(|d| d.get_id().ok());

    let collection = DeviceCollection::new(&Direction::Render)?;
    let count = collection.get_nbr_devices()?;

    for i in 0..count {
        if let Ok(device) = collection.get_device_at_index(i) {
            let name = device
                .get_friendlyname()
                .unwrap_or_else(|_| format!("Speaker {}", i));
            let id = device
                .get_id()
                .unwrap_or_else(|_| format!("windows_output_{}", i));
            let is_default = default_id.as_ref().map(|def| def == &id).unwrap_or(false);

            devices.push(AudioDevice {
                id,
                name,
                is_default,
            });
        }
    }

    Ok(devices)
}

pub struct SpeakerInput {
    device_id: Option<String>,
}

impl SpeakerInput {
    pub fn new(device_id: Option<String>) -> Result<Self> {
        let device_id = device_id.filter(|id| !id.is_empty() && id != "default");
        Ok(Self { device_id })
    }

    pub fn stream(self) -> SpeakerStream {
        let sample_queue = Arc::new(Mutex::new(VecDeque::new()));
        let waker_state = Arc::new(Mutex::new(WakerState {
            waker: None,
            has_data: false,
            shutdown: false,
        }));
        let (init_tx, init_rx) = mpsc::channel();

        SpeakerStream {
            sample_queue,
            waker_state,
            capture_thread: None, // Stub
            actual_sample_rate: 44100,
        }
    }
}

struct WakerState {
    waker: Option<Waker>,
    has_data: bool,
    shutdown: bool,
}

pub struct SpeakerStream {
    sample_queue: Arc<Mutex<VecDeque<f32>>>,
    waker_state: Arc<Mutex<WakerState>>,
    capture_thread: Option<thread::JoinHandle<()>>,
    actual_sample_rate: u32,
}

impl SpeakerStream {
    pub fn sample_rate(&self) -> u32 {
        self.actual_sample_rate
    }
}

impl Stream for SpeakerStream {
    type Item = f32;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}
