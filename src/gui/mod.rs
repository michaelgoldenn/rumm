// src/gui/gui.rs


use eframe::egui;
use eframe::egui::{Ui, WidgetText};
use egui_dock::{DockArea, DockState, Style, TabViewer};

use crate::thunderstore::ModList;
use crate::user_info::{Config, LocalModOptions};

mod thunderstore_browser;
mod local_mod_list;
//use crate::gui::ThunderstoreBrowser;
//use crate:gui::LocalModList;

use thunderstore_browser::draw_thunderstore_browser;
use local_mod_list::LocalModsTab;

pub fn start_gui(mod_list: ModList) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
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
}

/// This custom tab viewer delegates each tab’s UI to the respective module.
struct MyTabViewer;

impl TabViewer for MyTabViewer {
    type Tab = CustomTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            CustomTab::ThunderstoreBrowser(_) => "Mod Browser".into(),
            CustomTab::LocalModList(_) => "Mods".into(),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            CustomTab::ThunderstoreBrowser(list) => draw_thunderstore_browser(ui, list),
            CustomTab::LocalModList(tab) => tab.ui(ui),
        }
    }
}

struct MyTabs {
    dock_state: DockState<CustomTab>,
}

impl MyTabs {
    pub fn new(thunderstore_mod_list: ModList) -> Self {
        // Create initial tabs using the mod list.
        let local_options = LocalModOptions::new(&Config::new());
        let tabs = vec![
            CustomTab::ThunderstoreBrowser(thunderstore_mod_list.clone()),
            CustomTab::LocalModList(LocalModsTab::new(&thunderstore_mod_list, local_options)),
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
