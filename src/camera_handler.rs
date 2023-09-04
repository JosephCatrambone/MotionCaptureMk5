use nokhwa::{Camera, pixel_format};
use nokhwa::utils::{CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution};
use std::ops::DerefMut;
use std::sync::{Arc, mpsc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;
use image;

pub struct CameraHandler {
	camera_idx: u32,
	ring_buffer_size: usize,
	ring_buffer: Arc<RwLock<Vec<image::RgbImage>>>, // Maybe we could have a buffer of images and a buffer of views?
	ring_buffer_write_position: Arc<AtomicUsize>,
	ring_buffer_read_position: Arc<AtomicUsize>,
	camera: Arc<Mutex<Option<Camera>>>,
	enumerated_cameras: Vec<String>,
	worker_thread_run: Arc<AtomicBool>,
	worker_thread: thread::JoinHandle<()>,
	//image_rx: mpsc::Receiver<(u64, image::RgbImage)>,
}

impl Default for CameraHandler {
	fn default() -> Self {
		Self::new()
	}
}

impl Drop for CameraHandler {
	fn drop(&mut self) {
		self.worker_thread_run.store(false, Ordering::Relaxed); // TODO: Right ordering for this?
		self.swap_camera(None);
		//_ = self.worker_thread.expect("Worker thread is null on shutdown.").join();
	}
}

impl CameraHandler {
	pub fn new() -> Self {
		let ring_buffer_size = 100;
		let ring_buffer_data: Vec<image::RgbImage> = (0..ring_buffer_size).into_iter().map(|_| { image::RgbImage::new(1, 1) }).collect();

		let camera_ref = Arc::new(Mutex::new(None::<Camera>));
		let run_thread = Arc::<AtomicBool>::new(true.into());
		let ring_buffer = Arc::new(RwLock::new(ring_buffer_data));
		let read_head = Arc::<AtomicUsize>::new(0.into());
		let write_head = Arc::<AtomicUsize>::new(1.into()); // Start writing at one ahead of read.

		// Ring-buffer writer!
		let worker_thread_handle = {
			let camera_ref = camera_ref.clone();
			let run = run_thread.clone();
			let mut ring_buffer = ring_buffer.clone();
			let read_head = read_head.clone();
			let mut write_head = write_head.clone();
			thread::spawn(move || {
				let mut count = 0;
				loop {
					if let Ok(mut camera_lock) = camera_ref.lock() {
						if let Some(ref mut camera) = &mut camera_lock.deref_mut() {
							// Instead of streaming to a channel, read and write to a circular buffer.
							// This is the only thing that writes so we're safe to just check if it's full.
							// The thread MAY report the buffer as full and be emptied shortly thereafter, but it will never report less than full when it's full.

							// Check if full:
							let mut wh = 0;
							loop {
								// Read head + 1 == write head -> empty.
								// Write head == read head -> full
								wh = write_head.load(Ordering::Relaxed);
								if wh == read_head.load(Ordering::Relaxed) {
									println!("DEBUG: Buffer full while writing.  Read faster.");
									thread::sleep(Duration::default());
								} else {
									break;
								}
							}

							// Write to the next position.
							let write_lock: &mut image::RgbImage = &mut ring_buffer.write().expect("Unable to lock ring buffer for writing!")[wh];
							camera.write_frame_to_buffer::<pixel_format::RgbFormat>(write_lock.as_mut());
							write_head.store((wh + 1) % ring_buffer_size, Ordering::Relaxed);
							count += 1;
						}
					}
					// Check if we should shut down:
					if !run.load(Ordering::Relaxed) {
						return;
					}
				}
			})
		};

		Self {
			camera_idx: 0,
			camera: camera_ref.clone(),
			enumerated_cameras: vec![],
			ring_buffer_size: ring_buffer_size, // We could, in theory, just read the length of the buffer, but having to lock to read hurt.
			ring_buffer: ring_buffer,
			ring_buffer_read_position: read_head,
			ring_buffer_write_position: write_head,
			worker_thread_run: run_thread.clone(),
			worker_thread: worker_thread_handle,
		}
	}

	/// Return a list of strings that describe cameras.
	/// The index of a camera should correspond to the device id.
	pub fn get_cameras(&self) -> Vec<String> {
		vec!["Default".to_string(),]
	}
	
	pub fn get_camera_idx(&self) -> u32 { self.camera_idx }

	fn reallocate_ring_buffer(&mut self, width: u32, height: u32) {
		match &mut self.ring_buffer.write() {
			Ok(write_lock) => {
				let buffer_size = write_lock.len();
				// Clear all old entries.
				write_lock.clear();
				// Allocate new ones of the right size.
				for _ in 0..buffer_size {
					write_lock.push(image::RgbImage::new(width, height));
				}
			},
			Err(_) => {
				panic!("Failed to establish write lock on ring buffer.");
			}
		}
	}

	pub fn read_next_frame_blocking<'a>(&'a mut self) -> &'a image::RgbImage {
		// Important safety tip: WE DO NOT RELEASE THE REFERENCE TO THE PREVIOUS RGBIMAGE UNTIL THIS READ FRAME IS CALLED AGAIN.
		// Read head + 1 == write head -> empty.
		// Write head == read head -> full
		let rh = (self.ring_buffer_read_position.load(Ordering::Relaxed) + 1) % self.ring_buffer_size;
		self.ring_buffer_read_position.store(rh, Ordering::Relaxed);
		loop {
			if self.ring_buffer_write_position.load(Ordering::Relaxed) == rh {
				// Buffer is empty.
				thread::sleep(Duration::default());
			} else {
				break;
			}
		}
		let ring_buffer_read_lock = self.ring_buffer.read().expect("Unable to get read lock on ring buffer.");
		let image_ref = ring_buffer_read_lock.get(rh).expect("Failed to unwrap image at read head. This can't happen.");
		image_ref
	}

	/// Close the old camera stream and set the value to the new one.
	fn swap_camera(&mut self, mut new_camera: Option<Camera>) {
		let mut new_resolution: Option<Resolution> = None;
		match &mut self.camera.lock() {
			Ok(mutex_guard) => {
				let mut maybe_old_camera = mutex_guard.take();
				if let Some(mut old_camera) = maybe_old_camera {
					if let Err(e) = old_camera.stop_stream() {
						println!("Soft error closing previous camera: {}", e);
					}
				}

				if let Some(mut camera) = new_camera.as_mut() {
					camera.open_stream().expect("Failed to open new camera.");
					new_resolution = Some(camera.resolution());
				}
				(*mutex_guard.deref_mut()) = new_camera
			},
			Err(e) => {
				panic!("Mutex poisoned: {}", e);
			}
		}
		// Reallocate the ring buffer if our resolution changed:
		if let Some(new_res) = new_resolution {
			self.reallocate_ring_buffer(new_res.width_x, new_res.height_y);
		}
	}

	/// Close the old camera stream if it exists and replace it with the given config.
	fn set_camera(&mut self, new_idx: u32, camera_config: Option<RequestedFormatType>) {
		let index = CameraIndex::Index(new_idx);
		let requested = RequestedFormat::new::<pixel_format::RgbFormat>(match camera_config {
			Some(config) => config,
			None => RequestedFormatType::AbsoluteHighestFrameRate
		});
		println!("Opening camera.");
		let camera = Camera::new(index, requested);
		println!("Opened");
		if let Err(problem) = camera {
			panic!("{}", problem);
		}
		println!("Swapping camera.");
		self.swap_camera(camera.ok());
		println!("Swapped");
	}

	pub fn request_open_camera(&mut self, device_idx: u32, width: u32, height: u32, fps: u32) {
		self.set_camera(device_idx, Some(RequestedFormatType::Closest(
			CameraFormat::new_from(width, height, FrameFormat::MJPEG, fps)
		)));
	}
	
	pub fn request_open_camera_highest_fps(&mut self, device_idx: u32) {
		self.set_camera(device_idx, Some(RequestedFormatType::AbsoluteHighestFrameRate));
	}
}
