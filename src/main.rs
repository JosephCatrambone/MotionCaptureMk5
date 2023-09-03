#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

const APP_TITLE: &str = "MotionCaptureMk5";

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
	env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

	let native_options = eframe::NativeOptions::default();
	eframe::run_native(
		APP_TITLE,
		native_options,
		Box::new(|cc| Box::new(motion_capture_mk5::MainApp::new(cc))),
	)
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
fn main() {
	// Redirect `log` message to `console.log` and friends:
	eframe::WebLogger::init(log::LevelFilter::Debug).ok();

	let web_options = eframe::WebOptions::default();

	wasm_bindgen_futures::spawn_local(async {
		eframe::WebRunner::new()
			.start(
				"the_canvas_id", // hardcode it
				web_options,
				Box::new(|cc| Box::new(motion_capture_mk5::MainApp::new(cc))),
			)
			.await
			.expect("failed to start eframe");
	});
}