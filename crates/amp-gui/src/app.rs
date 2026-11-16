use crate::i18n::I18n;
use crate::views::{
    configs::render_configs,
    dashboard::{render_dashboard, ServiceAction},
    logs::render_logs,
    settings::render_settings,
    versions::render_versions,
    vhosts::render_vhosts,
};
use amp_core::models::{
    DownloadState, PackageItem, ServiceType, VersionManifest,
};
use amp_core::{PackageManager, ServiceManager};
use egui::{Color32, RichText, Visuals};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Dashboard,
    Versions,
    VHosts,
    Configs,
    Logs,
    Settings,
}

pub struct AmpApp {
    current_tab: ActiveTab,
    i18n: I18n,
    service_manager: Arc<Mutex<ServiceManager>>,
    manifest: Option<VersionManifest>,
    installed_versions: Vec<(ServiceType, Vec<String>)>,
    active_download: Option<(String, DownloadState)>,
    download_rx: Option<mpsc::Receiver<DownloadState>>,
    current_theme: String,
    root_dir: String,
    search_log_filter: String,
    tokio_handle: tokio::runtime::Handle,
}

impl AmpApp {
    pub fn new(cc: &eframe::CreationContext<'_>, tokio_handle: tokio::runtime::Handle) -> Self {
        // Configure visual styles
        let mut visuals = Visuals::dark();
        visuals.window_rounding = 8.0.into();
        visuals.menu_rounding = 6.0.into();
        cc.egui_ctx.set_visuals(visuals);

        let base_bin_dir = PathBuf::from("bin");
        let service_manager = Arc::new(Mutex::new(ServiceManager::new(base_bin_dir)));

        // Load local manifest if available
        let local_manifest_path = PathBuf::from("assets/versions.json");
        let manifest = if local_manifest_path.exists() {
            std::fs::read_to_string(&local_manifest_path)
                .ok()
                .and_then(|s| serde_json::from_str::<VersionManifest>(&s).ok())
        } else {
            None
        };

        let mut app = Self {
            current_tab: ActiveTab::Dashboard,
            i18n: I18n::new("pt-PT"),
            service_manager,
            manifest,
            installed_versions: Vec::new(),
            active_download: None,
            download_rx: None,
            current_theme: "dark".to_string(),
            root_dir: "www".to_string(),
            search_log_filter: String::new(),
            tokio_handle,
        };

        app.refresh_installed_versions();
        app
    }

    fn refresh_installed_versions(&mut self) {
        if let Ok(sm) = self.service_manager.lock() {
            self.installed_versions = vec![
                (ServiceType::Apache, sm.list_installed_versions(ServiceType::Apache)),
                (ServiceType::Database, sm.list_installed_versions(ServiceType::Database)),
                (ServiceType::Php, sm.list_installed_versions(ServiceType::Php)),
            ];
        }
    }

    fn start_package_download(&mut self, item: PackageItem) {
        let (tx, rx) = mpsc::channel(64);
        let pkg_label = format!("{} v{}", item.id, item.version);
        self.active_download = Some((pkg_label, DownloadState::Downloading {
            downloaded_bytes: 0,
            total_bytes: item.size_bytes,
            percent: 0.0,
        }));
        self.download_rx = Some(rx);

        let destination = PathBuf::from("bin");
        self.tokio_handle.spawn(async move {
            let _ = PackageManager::download_and_extract(&item, &destination, Some(tx)).await;
        });
    }

    fn handle_service_action(&mut self, action: ServiceAction) {
        if let Ok(mut sm) = self.service_manager.lock() {
            match action {
                ServiceAction::Start(st) => {
                    let _ = sm.start_service(st, "default", st.default_port());
                }
                ServiceAction::Stop(st) => {
                    let _ = sm.stop_service(st);
                }
                ServiceAction::Restart(st) => {
                    let _ = sm.stop_service(st);
                    let _ = sm.start_service(st, "default", st.default_port());
                }
                ServiceAction::StartAll => {
                    let _ = sm.start_service(ServiceType::Apache, "default", ServiceType::Apache.default_port());
                    let _ = sm.start_service(ServiceType::Database, "default", ServiceType::Database.default_port());
                    let _ = sm.start_service(ServiceType::Php, "default", ServiceType::Php.default_port());
                }
                ServiceAction::StopAll => {
                    sm.stop_all();
                }
            }
        }
    }
}

impl eframe::App for AmpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll active download progress
        let mut should_refresh_after_dl = false;
        if let Some(rx) = &mut self.download_rx {
            while let Ok(state) = rx.try_recv() {
                if let Some((_name, current_state)) = &mut self.active_download {
                    *current_state = state.clone();
                    if matches!(state, DownloadState::Completed | DownloadState::Failed(_)) {
                        should_refresh_after_dl = true;
                    }
                }
            }
        }
        if should_refresh_after_dl {
            self.refresh_installed_versions();
        }

        // Apply theme changes
        if self.current_theme == "light" {
            ctx.set_visuals(Visuals::light());
        } else {
            ctx.set_visuals(Visuals::dark());
        }

        // Left Navigation Sidebar
        egui::SidePanel::left("nav_panel")
            .resizable(false)
            .exact_width(210.0)
            .show(ctx, |ui| {
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⚡").size(24.0).color(Color32::from_rgb(234, 179, 8)));
                    ui.heading(RichText::new(self.i18n.t("app_title")).strong().size(18.0));
                });

                ui.add_space(20.0);

                let tabs = [
                    (ActiveTab::Dashboard, "📊", self.i18n.t("dashboard")),
                    (ActiveTab::Versions, "📦", self.i18n.t("versions")),
                    (ActiveTab::VHosts, "🌐", self.i18n.t("vhosts")),
                    (ActiveTab::Configs, "⚙️", self.i18n.t("configs")),
                    (ActiveTab::Logs, "📜", self.i18n.t("logs")),
                    (ActiveTab::Settings, "🔧", self.i18n.t("settings")),
                ];

                for (tab, icon, label) in tabs {
                    let is_active = self.current_tab == tab;
                    let text = format!("{} {}", icon, label);
                    let button = if is_active {
                        egui::Button::new(RichText::new(text).strong().color(Color32::from_rgb(96, 165, 250)))
                            .fill(ui.visuals().faint_bg_color)
                    } else {
                        egui::Button::new(RichText::new(text))
                    };

                    if ui.add_sized([ui.available_width(), 34.0], button).clicked() {
                        self.current_tab = tab;
                    }
                    ui.add_space(4.0);
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("Cross-Platform • Rust 1.97")
                            .color(Color32::DARK_GRAY)
                            .size(10.0),
                    );
                });
            });

        // Top Header
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Apache • MariaDB/MySQL • PHP Control Environment")
                        .color(Color32::GRAY)
                        .size(12.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Quick language toggle in header
                    if self.i18n.current_language() == "pt-PT" {
                        if ui.button("🇬🇧 EN").clicked() {
                            self.i18n.set_language("en-US");
                        }
                    } else {
                        if ui.button("🇵🇹 PT").clicked() {
                            self.i18n.set_language("pt-PT");
                        }
                    }
                });
            });
            ui.add_space(6.0);
        });

        // Collect actions to execute after drawing to avoid borrow conflicts
        let mut service_actions = Vec::new();
        let mut downloads_to_start = Vec::new();
        let mut refresh_versions_requested = false;

        // Central Content Area
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut services_info = Vec::new();
            let mut logs = Vec::new();

            if let Ok(sm) = self.service_manager.lock() {
                services_info.push(sm.get_service_info(ServiceType::Apache));
                services_info.push(sm.get_service_info(ServiceType::Database));
                services_info.push(sm.get_service_info(ServiceType::Php));
                logs = sm.get_recent_logs();
            }

            match self.current_tab {
                ActiveTab::Dashboard => {
                    render_dashboard(
                        ui,
                        &self.i18n,
                        &services_info,
                        &mut service_actions,
                    );
                }
                ActiveTab::Versions => {
                    render_versions(
                        ui,
                        &self.i18n,
                        self.manifest.as_ref(),
                        &self.installed_versions,
                        self.active_download.as_ref(),
                        &mut downloads_to_start,
                        &mut refresh_versions_requested,
                    );
                }
                ActiveTab::VHosts => {
                    render_vhosts(ui, &self.i18n);
                }
                ActiveTab::Configs => {
                    render_configs(ui, &self.i18n);
                }
                ActiveTab::Logs => {
                    render_logs(ui, &self.i18n, &logs, &mut self.search_log_filter);
                }
                ActiveTab::Settings => {
                    render_settings(
                        ui,
                        &mut self.i18n,
                        &mut self.current_theme,
                        &mut self.root_dir,
                    );
                }
            }
        });

        // Execute actions outside UI borrows
        for action in service_actions {
            self.handle_service_action(action);
        }

        for item in downloads_to_start {
            self.start_package_download(item);
        }

        if refresh_versions_requested {
            self.refresh_installed_versions();
        }

        // Request redraw regularly for smooth UI / log streaming
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}
