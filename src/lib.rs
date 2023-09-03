
use egui;
use std::sync::{Arc, Mutex};

mod camera;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MainApp {
	preferred_camera_idx: usize,

	#[serde(skip)]


	#[serde(skip)]
	frame_buffer: Vec<u8>,
}

impl Default for MainApp {
	fn default() -> Self {
		Self {
			// Example stuff:
			preferred_camera_idx: 0,
			camera: Arc::new(Mutex::new(None))
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

			ui.heading("eframe template");
			ui.hyperlink("https://github.com/emilk/eframe_template");
			ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
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
				let options: Vec<String> = (0u32..10).into_iter().map(|idx| { format!("Camera {}", idx) }).collect();
				if egui::ComboBox::from_label("Camera:").show_index(
					ui, &mut self.preferred_camera_idx, options.len(), |i| &options[i]
				).changed() {
					// Maybe close this camera...
					self.set_camera(self.preferred_camera_idx as u32, None);
				};
			});

			ui.hyperlink_to(
				"Source Repo (GitHub)",
				"https://github.com/JosephCatrambone/MotionCaptureMk5",
			);
		});
	}


}
