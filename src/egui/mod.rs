use std::sync::Arc;

use color_eyre::eyre::Result;
use eframe::egui;
use eframe::egui::{Ui, WidgetText};
use egui_dock::{DockArea, DockState, Style, TabViewer};
use settings_ui::draw_settings_ui;
use tokio::runtime::{Handle, Runtime};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::mod_cache::ModCache;
use crate::thunderstore::{Mod, ModList};
use crate::user_info::{Config, LocalModOptions};

mod local_mod_list_ui;
mod settings_ui;
mod thunderstore_browser_ui;

use local_mod_list_ui::LocalModsTab;
use thunderstore_browser_ui::draw_thunderstore_browser;

pub enum AppCommand {
    UpdateMod(Mod),
    UpdateAllMods,
    CacheModByID(Uuid, Option<String>),
}

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
    cache: Arc<RwLock<ModCache>>,
    runtime: RuntimeGuard,
    handle: Handle,
    runtime_commands: UnboundedSender<AppCommand>,
}

impl MyApp {
    fn new(mods: ModList) -> Self {
        let runtime = start_runtime();
        let (runtime_commands, mut cmd_rx) = mpsc::unbounded_channel::<AppCommand>();
        let cache = Arc::new(RwLock::new(ModCache::new(&mods)));
        // worker
        {
            let cache = cache.clone();
            let handle = runtime.handle();
            handle.spawn(async move {
                while let Some(cmd) = cmd_rx.recv().await {
                    let mut cache = cache.write().await;
                    match cmd {
                        AppCommand::UpdateMod(mod_to_update) => {
                            println!("updating mod!");
                            if let Err(e) = cache.update_mod(&Config::new(), &mod_to_update).await {
                                eprintln!("update_all_mods: {e}");
                            }
                        } 
                        AppCommand::UpdateAllMods => {
                            if let Err(e) = cache.update_all_mods(&Config::new()).await {
                                eprintln!("update_all_mods: {e}");
                            }
                        }
                        AppCommand::CacheModByID( id, version ) => {
                            if let Err(e) = cache.cache_mod_by_mod_id(&id.to_string(), version.as_ref()).await {
                                eprintln!("cache_mod_by_mod_id: {e}");
                            }
                        }
                    }
                }
            });
        }
    
        Self {
            cache,
            tabs: MyTabs::new(mods, runtime_commands.clone()),
            handle: runtime.handle(),
            runtime,
            runtime_commands,
        }
    }    
}

/// Spawns a multi-thread runtime and gives you a `Handle` you can clone.
/// Dropping `RuntimeGuard` will shut it down gracefully.
pub fn start_runtime() -> RuntimeGuard {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("create tokio runtime");

    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();

    // park the runtime on a dedicated thread so egui’s paint thread stays clean
    std::thread::spawn({
        let handle = runtime.handle().clone();
        move || {
            handle.block_on(async move {
                tokio::select! {
                    _ = &mut stop_rx => { /* shutdown requested */ }
                    else => unreachable!(),
                }
            });
        }
    });

    RuntimeGuard { rt: runtime, stop: Some(stop_tx) }
}

pub struct RuntimeGuard {
    rt: tokio::runtime::Runtime,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
}
impl RuntimeGuard {
    pub fn handle(&self) -> tokio::runtime::Handle { self.rt.handle().clone() }
}
impl Drop for RuntimeGuard {
    fn drop(&mut self) { let _ = self.stop.take().map(|s| s.send(())); }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            self.tabs.ui(ui);
        });
    }
}
pub type TabResult = Result<Option<AppCommand>, color_eyre::eyre::Report>;

pub enum CustomTab {
    ThunderstoreBrowser(ModList),
    LocalModList(LocalModsTab),
    Settings(Config),
}

/// This custom tab viewer delegates each tab's UI to the respective module.
struct MyTabViewer {
    error_popup: Option<String>,
    runtime_commands: mpsc::UnboundedSender<AppCommand>,
}

impl MyTabViewer {
    fn new(runtime_commands: mpsc::UnboundedSender<AppCommand>) -> Self {
        Self {
            error_popup: None,
            runtime_commands,
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
        if let Err(err) = &result {
            self.error_popup = Some(format!("Error: {}", err));
        }
        if let Ok(Some(cmd)) = result {
            let _ = self.runtime_commands.send(cmd);
        }        
    }
}

struct MyTabs {
    dock_state: DockState<CustomTab>,
    tab_viewer: MyTabViewer,
}

impl MyTabs {
    pub fn new(thunderstore_mod_list: ModList, runtime_commands: mpsc::UnboundedSender<AppCommand>) -> Self {
        // Create initial tabs using the mod list.
        let local_options = LocalModOptions::new(&Config::new());
        let tabs = vec![
            CustomTab::LocalModList(LocalModsTab::new(&thunderstore_mod_list, local_options)),
            CustomTab::ThunderstoreBrowser(thunderstore_mod_list.clone()),
            CustomTab::Settings(Config::new()),
        ];
        let dock_state = DockState::new(tabs);
        Self {
            dock_state,
            tab_viewer: MyTabViewer::new(runtime_commands),
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        // make vertical layout to fit tabs and error messages
        egui::ScrollArea::vertical().show(ui, |ui| {
            let available_height = ui.available_height();
            // Leave some space for the label
            let dock_height = available_height - 20.0;
            ui.allocate_ui(
                eframe::egui::Vec2::new(ui.available_width(), dock_height),
                |ui| {
                    DockArea::new(&mut self.dock_state)
                        .style(Style::from_egui(ui.style().as_ref()))
                        .show_inside(ui, &mut self.tab_viewer);
                },
            );
            ui.label("Test!");
        });
    }
}
