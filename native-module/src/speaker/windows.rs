// Windows WASAPI speaker/system audio capture using loopback mode
// Based on the pluely Windows implementation using wasapi 0.19.0
use anyhow::Result;
use ringbuf::{
    traits::{Producer, Split},
    HeapCons, HeapProd, HeapRb,
};
use std::collections::VecDeque;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use wasapi::{get_default_device, DeviceCollection, Direction, SampleType, StreamMode, WaveFormat};

/// Shared state for signaling shutdown
struct CaptureState {
    shutdown: bool,
}

pub struct SpeakerInput {
    device_id: Option<String>,
}

pub struct SpeakerStream {
    consumer: Option<HeapCons<f32>>,
    capture_state: Arc<Mutex<CaptureState>>,
    capture_thread: Option<thread::JoinHandle<()>>,
    actual_sample_rate: u32,
}

impl SpeakerStream {
    pub fn sample_rate(&self) -> u32 {
        self.actual_sample_rate
    }

    /// Take the consumer for lock-free audio sample reading.
    /// This can only be called once - subsequent calls return None.
    pub fn take_consumer(&mut self) -> Option<HeapCons<f32>> {
        self.consumer.take()
    }
}

/// Helper to find device by ID
fn find_device_by_id(direction: &Direction, device_id: &str) -> Option<wasapi::Device> {
    let collection = DeviceCollection::new(direction).ok()?;
    let count = collection.get_nbr_devices().ok()?;

    for i in 0..count {
        if let Ok(device) = collection.get_device_at_index(i) {
            if let Ok(id) = device.get_id() {
                if id == device_id {
                    return Some(device);
                }
            }
        }
    }
    None
}

pub fn list_output_devices() -> Result<Vec<(String, String)>> {
    let collection = DeviceCollection::new(&Direction::Render)?;
    let count = collection.get_nbr_devices()?;
    let mut list = Vec::new();

    for i in 0..count {
        if let Ok(device) = collection.get_device_at_index(i) {
            let id = device.get_id().unwrap_or_default();
            let name = device.get_friendlyname().unwrap_or_default();
            if !id.is_empty() {
                list.push((id, name));
            }
        }
    }
    Ok(list)
}

impl SpeakerInput {
    pub fn new(device_id: Option<String>) -> Result<Self> {
        let device_id = device_id.filter(|id| !id.is_empty() && id != "default");
        Ok(Self { device_id })
    }

    pub fn stream(self) -> SpeakerStream {
        // Create ring buffer for lock-free audio transfer (128KB = 131072 samples)
        let buffer_size = 131072;
        let rb = HeapRb::<f32>::new(buffer_size);
        let (producer, consumer) = rb.split();

        let capture_state = Arc::new(Mutex::new(CaptureState { shutdown: false }));
        let (init_tx, init_rx) = mpsc::channel();

        let state_clone = capture_state.clone();
        let device_id = self.device_id;

        let capture_thread = thread::spawn(move || {
            if let Err(e) = Self::capture_audio_loop(producer, state_clone, init_tx, device_id) {
                eprintln!("[Windows Audio] Capture loop failed: {}", e);
            }
        });

        let actual_sample_rate = match init_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(rate)) => {
                println!("[Windows Audio] Initialized at {} Hz", rate);
                rate
            }
            Ok(Err(e)) => {
                eprintln!("[Windows Audio] Initialization failed: {}", e);
                44100
            }
            Err(_) => {
                eprintln!("[Windows Audio] Initialization timeout");
                44100
            }
        };

        SpeakerStream {
            consumer: Some(consumer),
            capture_state,
            capture_thread: Some(capture_thread),
            actual_sample_rate,
        }
    }

    fn capture_audio_loop(
        mut producer: HeapProd<f32>,
        capture_state: Arc<Mutex<CaptureState>>,
        init_tx: mpsc::Sender<Result<u32>>,
        device_id: Option<String>,
    ) -> Result<()> {
        let init_result = (|| -> Result<_> {
            // Get the render device (for loopback capture of system audio)
            let device = match device_id {
                Some(ref id) => match find_device_by_id(&Direction::Render, id) {
                    Some(d) => {
                        println!(
                            "[Windows Audio] Using device: {}",
                            d.get_friendlyname().unwrap_or_else(|_| "Unknown".to_string())
                        );
                        d
                    }
                    None => {
                        println!("[Windows Audio] Device not found, using default");
                        get_default_device(&Direction::Render)?
                    }
                },
                None => get_default_device(&Direction::Render)?,
            };

            let device_name = device
                .get_friendlyname()
                .unwrap_or_else(|_| "Unknown".to_string());
            println!("[Windows Audio] Capturing from: {}", device_name);

            let mut audio_client = device.get_iaudioclient()?;
            let device_format = audio_client.get_mixformat()?;
            let actual_rate = device_format.get_samplespersec();

            // Request mono f32 format for easier processing
            let desired_format = WaveFormat::new(
                32,                   // bits per sample
                32,                   // valid bits
                &SampleType::Float,
                actual_rate as usize,
                1,                    // mono
                None,
            );

            let (_def_time, min_time) = audio_client.get_device_period()?;

            // Use shared loopback mode with auto-conversion
            let mode = StreamMode::EventsShared {
                autoconvert: true,
                buffer_duration_hns: min_time,
            };

            // Initialize for loopback capture (capture from render device)
            audio_client.initialize_client(&desired_format, &Direction::Capture, &mode)?;

            let h_event = audio_client.set_get_eventhandle()?;
            let capture_client = audio_client.get_audiocaptureclient()?;
            audio_client.start_stream()?;

            Ok((h_event, capture_client, actual_rate))
        })();

        match init_result {
            Ok((h_event, capture_client, sample_rate)) => {
                let _ = init_tx.send(Ok(sample_rate));

                let mut consecutive_drops = 0u32;
                let max_buffer_size = 131072usize;

                loop {
                    // Check shutdown signal
                    {
                        let state = capture_state.lock().unwrap();
                        if state.shutdown {
                            println!("[Windows Audio] Shutdown signal received");
                            break;
                        }
                    }

                    // Wait for audio data event
                    if h_event.wait_for_event(3000).is_err() {
                        eprintln!("[Windows Audio] Event timeout, stopping capture");
                        break;
                    }

                    // Read available audio data into deque
                    let mut temp_queue: VecDeque<u8> = VecDeque::new();
                    if let Err(e) = capture_client.read_from_device_to_deque(&mut temp_queue) {
                        eprintln!("[Windows Audio] Failed to read audio: {}", e);
                        continue;
                    }

                    if temp_queue.is_empty() {
                        continue;
                    }

                    // Convert bytes to f32 samples (4 bytes per sample)
                    let mut samples = Vec::new();
                    while temp_queue.len() >= 4 {
                        let bytes = [
                            temp_queue.pop_front().unwrap(),
                            temp_queue.pop_front().unwrap(),
                            temp_queue.pop_front().unwrap(),
                            temp_queue.pop_front().unwrap(),
                        ];
                        let sample = f32::from_le_bytes(bytes);
                        samples.push(sample);
                    }

                    if !samples.is_empty() {
                        // Push to ring buffer (lock-free)
                        let pushed = producer.push_slice(&samples);

                        if pushed < samples.len() {
                            consecutive_drops += 1;
                            if consecutive_drops == 25 {
                                eprintln!(
                                    "[Windows Audio] Warning: Buffer overflow - system may be overloaded"
                                );
                            }
                            if consecutive_drops > 50 {
                                eprintln!("[Windows Audio] Critical: Stopping due to persistent overflow");
                                break;
                            }
                        } else {
                            consecutive_drops = 0;
                        }
                    }
                }
            }
            Err(e) => {
                let _ = init_tx.send(Err(e));
            }
        }

        println!("[Windows Audio] Capture loop ended");
        Ok(())
    }
}

impl Drop for SpeakerStream {
    fn drop(&mut self) {
        // Signal shutdown
        if let Ok(mut state) = self.capture_state.lock() {
            state.shutdown = true;
        }

        // Wait for capture thread to finish
        if let Some(handle) = self.capture_thread.take() {
            let _ = handle.join();
        }
    }
}
