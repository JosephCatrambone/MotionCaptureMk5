use nokhwa::{Camera, pixel_format};
use nokhwa::utils::{CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType};
use std::ops::DerefMut;
use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use image;
use nokhwa::pixel_format::RgbFormat;

pub struct CameraHandler {
	camera_idx: u32,
	camera: Arc<Mutex<Option<Camera>>>,
	worker_thread: Option<thread::JoinHandle<()>>,
	image_tx: mpsc::Sender<(u64, image::RgbImage)>,
	image_rx: mpsc::Receiver<(u64, image::RgbImage)>,
}

impl Default for CameraHandler {
	fn default() -> Self {
		let (tx, rx) = mpsc::channel();
		Self {
			camera_idx: 0,
			camera: Arc::new(Mutex::new(None)),
			worker_thread: None,
			image_tx: tx,
			image_rx: rx,
		}
	}
}

impl Drop for CameraHandler {
	fn drop(&mut self) {
		self.swap_camera(None);
		if let Some(t) = &mut self.worker_thread {
			if let Err(e) = t.join() {
				eprintln!("Failed to join and close thread on shutdown: {:?}", e);
			}
		}
	}
}

impl CameraHandler {
	/// Return a list of strings that describe cameras.
	/// The index of a camera should correspond to the device id.
	pub fn get_cameras(&self) -> Vec<String> {
		vec![]
	}

	fn get_frame(&mut self) {
		let mut c = self.camera.unwrap();
		let frame = c.frame().unwrap();
		let decoded = frame.decode_image::<RgbFormat>();
		if let Ok(f) = decoded {

		}
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
				(*mutex_guard.deref_mut()) = new_camera;
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
}

fn image_fetch_worker(camera_ref: Arc<Mutex<Option<Camera>>>, transmit: mpsc::Sender<(u64, image::RgbImage)>) {
	let mut count = 0;
	loop {
		if let Ok(mut camera_lock) = camera_ref.lock() {
			if let Some(mut camera) = &mut camera_lock.deref_mut() {
				if let Ok(frame) = camera.frame() {
					if let Ok(buf) = frame.decode_image() {
						transmit.send((count, buf));
						count += 1;
					}
				}
			}
		}
	}
}
