use eframe::egui;
use eframe::egui::{Ui, WidgetText};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};

use crate::thunderstore::ModList;
use crate::user_info::{Config, EnabledMods};

pub fn start_gui(mod_list: ModList) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(move |cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(MyApp::new(mod_list)))
        }),
    )
}

struct MyApp {
    tabs: MyTabs,
}

impl MyApp {
    fn new(mod_list: ModList) -> Self {
        Self {
            tabs: MyTabs::new(mod_list),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        // Use the full window for the dock area
        egui::CentralPanel::default().show(ctx, |ui| {
            self.tabs.ui(ui);
        });
    }
}

#[derive(Clone)]
enum CustomTab {
    ThunderstoreBrowser(ModList),
    Tab2,
}

struct MyTabViewer;

impl TabViewer for MyTabViewer {
    type Tab = CustomTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            CustomTab::ThunderstoreBrowser(_) => "Mod Browser".into(),
            CustomTab::Tab2 => "Mods".into(),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            CustomTab::ThunderstoreBrowser(mod_list) => {
                // create mod icons

                for new_mod in &mod_list.mods {
                    ui.horizontal(|ui| {
                        ui.image(
                            new_mod
                                .versions
                                .first()
                                .expect("there should always be a first version")
                                .icon
                                .clone(),
                        );
                        ui.label(
                            new_mod
                                .versions
                                .first()
                                .expect("there should always be a first version")
                                .name
                                .clone(),
                        );
                        if ui.add(egui::Button::new("Add Mod")).clicked() {
                            //println!("Button Pressed!");
                            // enable the mod
                            let config = Config::new();
                            let mut enabled_mods = EnabledMods::new(&config);
                            if enabled_mods.enable_mod(new_mod, &config).is_err(){
                                println!("ERROR: could not enable mod: {}", new_mod.name)
                            }
                        }
                    });
                }
            }
            CustomTab::Tab2 => {
                ui.label("Content of Tab 2");
            }
        }
    }
}

struct MyTabs {
    dock_state: DockState<CustomTab>,
}

impl MyTabs {
    pub fn new(thunderstore_mod_list: ModList) -> Self {
        // Create initial tabs
        let tabs = vec![
            CustomTab::ThunderstoreBrowser(thunderstore_mod_list),
            CustomTab::Tab2,
        ];
        let dock_state = DockState::new(tabs);
        Self { dock_state }
    }

    fn ui(&mut self, ui: &mut Ui) {
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut MyTabViewer);
    }
}
