
use egui;
use crate::camera_handler::CameraHandler;

mod camera_handler;
mod depth_model;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MainApp {
	#[serde(skip)]
	camera_manager: camera_handler::CameraHandler,

	#[serde(skip)]
	frame_buffer: Vec<u8>,
}

impl Default for MainApp {
	fn default() -> Self {
		Self {
			camera_manager: CameraHandler::default(),
			frame_buffer: vec![]
		}
	}
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

		Default::default()
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
				let _ = self.camera_manager.read_next_frame();
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
	fn draw_camera_select_ui(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
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
				};
			});

			ui.hyperlink_to(
				"Source Repo (GitHub)",
				"https://github.com/JosephCatrambone/MotionCaptureMk5",
			);
		});
	}


}
