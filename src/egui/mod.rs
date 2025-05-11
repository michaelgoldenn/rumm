use eframe::egui;
use eframe::egui::{Ui, WidgetText};
use egui_dock::{DockArea, DockState, Style, TabViewer};
use settings_ui::draw_settings_ui;
use color_eyre::eyre::Result;

use crate::thunderstore::ModList;
use crate::user_info::{Config, LocalModOptions};

mod thunderstore_browser_ui;
mod local_mod_list_ui;
mod settings_ui;

use thunderstore_browser_ui::draw_thunderstore_browser;
use local_mod_list_ui::LocalModsTab;

pub fn start_gui(mod_list: ModList) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "rumm",
        options,
        Box::new(move |cc| {
            // Install image loaders, etc.
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
        egui::CentralPanel::default().show(ctx, |ui| {
            self.tabs.ui(ui);
        });
    }
}

pub enum CustomTab {
    ThunderstoreBrowser(ModList),
    LocalModList(LocalModsTab),
    Settings(Config)
}

/// This custom tab viewer delegates each tab's UI to the respective module.
struct MyTabViewer {
    error_popup: Option<String>,
}

impl MyTabViewer {
    fn new() -> Self {
        Self {
            error_popup: None,
        }
    }
    
    fn show_error_popup(&mut self, ui: &mut Ui) {
        if let Some(error_message) = &self.error_popup.clone() {
            // Show a popup window with the error message
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(error_message);
                    if ui.button("OK").clicked() {
                        self.error_popup = None;
                    }
                });
        }
    }
}

impl TabViewer for MyTabViewer {
    type Tab = CustomTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            CustomTab::ThunderstoreBrowser(_) => "Mod Browser".into(),
            CustomTab::LocalModList(_) => "Mods".into(),
            CustomTab::Settings(_) => "Settings".into(),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        self.show_error_popup(ui);
        let result = match tab {
            CustomTab::ThunderstoreBrowser(list) => draw_thunderstore_browser(ui, list),
            CustomTab::LocalModList(tab) => tab.ui(ui),
            CustomTab::Settings(config) => draw_settings_ui(ui, config),
        };
        if let Err(err) = result {
            self.error_popup = Some(format!("Error: {}", err));
        }
    }
}

struct MyTabs {
    dock_state: DockState<CustomTab>,
    tab_viewer: MyTabViewer,
}

impl MyTabs {
    pub fn new(thunderstore_mod_list: ModList) -> Self {
        // Create initial tabs using the mod list.
        let local_options = LocalModOptions::new(&Config::new());
        let tabs = vec![
            CustomTab::LocalModList(LocalModsTab::new(&thunderstore_mod_list, local_options)),
            CustomTab::ThunderstoreBrowser(thunderstore_mod_list.clone()),
            CustomTab::Settings(Config::new())
        ];
        let dock_state = DockState::new(tabs);
        Self { 
            dock_state,
            tab_viewer: MyTabViewer::new(),
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        // make vertical layout to fit tabs and error messages
        egui::ScrollArea::vertical().show(ui, |ui| {
            let available_height = ui.available_height();
            // Leave some space for the label
            let dock_height = available_height - 20.0;
            ui.allocate_ui(eframe::egui::Vec2::new(ui.available_width(), dock_height), |ui| {
                DockArea::new(&mut self.dock_state)
                    .style(Style::from_egui(ui.style().as_ref()))
                    .show_inside(ui, &mut self.tab_viewer);
            });
            ui.label("Test!");
        });
    }
}