use nokhwa::{Camera, pixel_format};
use nokhwa::utils::{CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType};
use std::ops::DerefMut;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use image;

pub struct CameraHandler {
	camera_idx: u32,
	camera: Arc<Mutex<Option<Camera>>>,
	enumerated_cameras: Vec<String>,
	worker_thread_run: Arc<AtomicBool>,
	worker_thread: thread::JoinHandle<()>,
	image_rx: mpsc::Receiver<(u64, Arc<Mutex<image::RgbImage>>)>,
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
		let max_buffer_size: usize = 10;
		let camera_ref = Arc::new(Mutex::new(None::<Camera>));
		let run_thread = Arc::<AtomicBool>::new(true.into());
		let (tx, rx) = mpsc::channel();

		// Ring-buffer writer!
		let worker_thread_handle = {
			let camera_ref = camera_ref.clone();
			let run = run_thread.clone();
			thread::spawn(move || {
				let mut allocated_images: Vec<Arc<Mutex<image::RgbImage>>> = vec![];
				let mut count = 0;
				loop {
					if let Ok(mut camera_lock) = camera_ref.lock() {
						if let Some(ref mut camera) = &mut camera_lock.deref_mut() {
							let camera_resolution = camera.resolution();
							let camera_fps = camera.frame_rate();
							// Check if we have any images that were deallocated.  Reuse them.
							let img = {
								loop {
									let maybe_image = allocated_images.iter().find(|&i| { Arc::strong_count(i) < 2 });
									if allocated_images.len() < max_buffer_size && maybe_image.is_none() {
										println!("Allocating new image.");
										let i = Arc::new(Mutex::new(image::RgbImage::new(camera_resolution.width(), camera_resolution.height())));
										allocated_images.push(i.clone());
										break i
									} else if let Some(i) = maybe_image {
										println!("Reusing image.");
										break i.clone()
									} else {
										// TODO: Check for quit.  Drain buffer.
										thread::yield_now();
										continue;
									}
								}
							};
							// Image may have the wrong size for this camera resolution.
							// TODO: Double check image size.

							// Write to the next position.
							camera.write_frame_to_buffer::<pixel_format::RgbFormat>(img.lock().unwrap().deref_mut());
							count += 1;
							match tx.send((count, img)) {
								Ok(_) => {
									println!("Sent image {}", &count);
									thread::sleep(Duration::from_secs_f32(1.0f32 / camera_fps as f32));
									println!("FPS: {}", &camera_fps);
									//thread::yield_now();
								}
								Err(_) => {
									panic!("Failed to send. Channel may have closed.");
								}
							}
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
			worker_thread_run: run_thread.clone(),
			worker_thread: worker_thread_handle,
			image_rx: rx
		}
	}

	/// Return a list of strings that describe cameras.
	/// The index of a camera should correspond to the device id.
	pub fn get_cameras(&self) -> Vec<String> {
		vec!["Default".to_string(),]
	}
	
	pub fn get_camera_idx(&self) -> u32 { self.camera_idx }

	pub fn read_next_frame(&mut self) -> Arc<Mutex<image::RgbImage>> {
		self.image_rx.recv().expect("Failed to pop image from buffer.").1
	}

	/// Close the old camera stream and set the value to the new one.
	fn swap_camera(&mut self, mut new_camera: Option<Camera>) {
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
				}
				(*mutex_guard.deref_mut()) = new_camera
			},
			Err(e) => {
				panic!("Mutex poisoned: {}", e);
			}
		}
	}

	/// Close the old camera stream if it exists and replace it with the given config.
	fn set_camera(&mut self, new_idx: u32, camera_config: Option<RequestedFormatType>) {
		let index = CameraIndex::Index(new_idx);
		let requested = RequestedFormat::new::<pixel_format::RgbFormat>(match camera_config {
			Some(config) => config,
			None => RequestedFormatType::AbsoluteHighestFrameRate
		});
		let camera = Camera::new(index, requested);
		if let Err(problem) = camera {
			panic!("{}", problem);
		}
		let camera = camera.unwrap();
		println!("Opened camera with resolution {:?} and frame rate {:?}", &camera.resolution(), &camera.frame_rate());
		self.swap_camera(Some(camera));
	}

	/// Maybe calls the function with a camera if one is open.
	/// Returns 'true' if there was an open function or false otherwise.
	/// Functions as a noop if the camera is not open.
	fn with_camera<F, R>(&self, func: F) -> Option<R> where F: Fn(&mut Camera) -> R {
		let mut mutex_guard = self.camera.lock().expect("Unable to unwrap mutex around camera -- lock may have been poisoned.  Did the program crash?");
		let mut cam = mutex_guard.take();
		if let Some(mut cam) = cam {
			return Some(func(&mut cam));
		}
		None
	}

	pub fn get_resolution(&self) -> Option<(u32, u32)> {
		let mut res: Option<(u32, u32)> = self.with_camera(|c| {
			return (c.resolution().width_x, c.resolution().height_y);
		});
		res
	}

	/// Return a copy of the next frame.
	/// This is a convenience method but should not be used in favor of read_next_frame.
	pub fn get_frame(&mut self) -> Option<image::RgbImage> {
		let f = self.read_next_frame();
		let frame = f.lock().unwrap().clone();
		Some(frame)
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
