
use egui;
use image;
use std::default::Default;
use std::ops::Deref;
use crate::camera_handler::CameraHandler;

mod camera_handler;
mod depth_model;

#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MainApp {
	#[serde(skip)]
	camera_manager: CameraHandler,

	#[serde(skip)]
	display_texture: Option<egui::TextureHandle>,

	#[serde(skip)]
	frame_buffer: Vec<u8>,
}

impl MainApp {
	/// Called once before the first frame.
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// This is also where you can customize the look and feel of egui using
		// `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

		// Load previous app state (if any).
		// Note that you must enable the `persistence` feature for this to work.
		if let Some(storage) = cc.storage {
			return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
		}

		Self {
			..Default::default()
		}
	}
}

impl eframe::App for MainApp {
	/// Called by the frame work to save state before shutdown.
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}

	/// Called each time the UI needs repainting, which may be many times per second.
	/// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		//let Self { preferred_camera_idx, camera } = self;

		// Examples of how to create different panels and windows.
		// Pick whichever suits you.
		// Tip: a good default choice is to just keep the `CentralPanel`.
		// For inspiration and more examples, go to https://emilk.github.io/egui

		#[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			// The top panel is often a good place for a menu bar:
			egui::menu::bar(ui, |ui| {
				ui.menu_button("File", |ui| {
					if ui.button("Quit").clicked() {
						_frame.close();
					}
				});
			});
		});

		egui::SidePanel::left("side_panel").show(ctx, |ui| {
			ui.heading("Side Panel");

			//ui.horizontal(|ui| { ui.label("Write something: "); ui.text_edit_singleline(label); });

			// TODO: Dropdown camera select.
			self.draw_camera_select_ui(ctx, ui);
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			// The central panel the region left after adding TopPanel's and SidePanel's
			//ui.heading("eframe template");
			if ui.button("Burn image").clicked() {
				let frame = self.camera_manager.read_next_frame();
				if let Some(ref mut handle) = &mut self.display_texture {
					let lock = frame.lock().unwrap();
					handle.set(egui::ColorImage::from_rgb([lock.width() as usize, lock.height() as usize], lock.as_raw()), egui::TextureOptions::default());
				}
			}
			if let Some(tex) = &self.display_texture {
				ui.image(tex.id(), tex.size_vec2());
			}

			egui::warn_if_debug_build(ui);
		});

		if false {
			egui::Window::new("Window").show(ctx, |ui| {
				ui.label("Windows can be moved by dragging them.");
				ui.label("They are automatically sized based on contents.");
				ui.label("You can turn on resizing and scrolling if you like.");
				ui.label("You would normally choose either panels OR windows.");
			});
		}
	}
}

impl MainApp {
	fn draw_camera_select_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
		ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
			ui.horizontal(|ui| {
				ui.spacing_mut().item_spacing.x = 0.0;
				ui.label("Camera IDX:");
				let mut idx = self.camera_manager.get_camera_idx() as usize;
				let options = self.camera_manager.get_cameras();
				if egui::ComboBox::from_label("Camera:").show_index(
					ui, &mut idx, options.len(), |i| &options[i]
				).changed() {
					self.camera_manager.request_open_camera_highest_fps(idx as u32);
					let frame = self.camera_manager.get_frame().unwrap();
					let img = egui::ColorImage::from_rgb([frame.width() as usize, frame.height() as usize], frame.as_raw());
					self.display_texture = Some(ctx.load_texture("display-texture", img, Default::default()));
				};
			});

			ui.hyperlink_to(
				"Source Repo (GitHub)",
				"https://github.com/JosephCatrambone/MotionCaptureMk5",
			);
		});
	}
}

// Some slightly ugly conversions from egui::RgbImage to Colorframe
// TODO: Replace all the funky casting above with this.
struct ImageRgbImageWrapper(image::RgbImage);

impl Deref for ImageRgbImageWrapper {
	type Target = image::RgbImage;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<ImageRgbImageWrapper> for egui::ColorImage {
	fn from(value: ImageRgbImageWrapper) -> Self {
		egui::ColorImage::from_rgb([value.width() as usize, value.height() as usize], &value.as_raw())
	}
}

/*
fn imagedata_to_egui_image<T>(frame: &mut eframe::Frame, image_data: T) -> Option<(egui::Vec2, egui::TextureId)> where T: GenericImageView {
	// Load the image:
	let image = image::load_from_memory(image_data).expect("Failed to load image");
	let image_buffer = image.to_rgba8();

	let size = (image.width() as usize, image.height() as usize);
	let pixels = image_buffer.into_vec();
	assert_eq!(size.0 * size.1 * 4, pixels.len());
	let pixels: Vec<_> = pixels
		.chunks_exact(4)
		.map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
		.collect();

	// Allocate a texture:
	let texture = frame
		.tex_allocator()
		.alloc_srgba_premultiplied(size, &pixels);
	let size = egui::Vec2::new(size.0 as f32, size.1 as f32);
	Some((size, texture))
}
*/