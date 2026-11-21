use crate::i18n::I18n;
use crate::views::{
    configs::render_configs,
    dashboard::{render_dashboard, ServiceAction},
    logs::render_logs,
    php_servers::{render_php_servers, PhpServerAction},
    settings::{render_settings, SettingsAction},
    versions::render_versions,
    vhosts::render_vhosts,
};
use amp_core::models::{
    AppConfig, DownloadState, LogEvent, OsFilter, PackageItem, PackageKind, ServiceType, VersionManifest,
};
use amp_core::{PackageManager, ServiceManager};
use egui::{Color32, RichText, Visuals};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Dashboard,
    PhpServers,
    Versions,
    VHosts,
    Configs,
    Logs,
    Settings,
}

pub struct AmpApp {
    current_tab: ActiveTab,
    i18n: I18n,
    config: AppConfig,
    config_path: PathBuf,
    settings_status: Option<String>,
    status_bar_message: Option<(String, bool)>, // (message, is_error)
    service_manager: Arc<Mutex<ServiceManager>>,
    manifest: Option<VersionManifest>,
    installed_versions: Vec<(ServiceType, Vec<String>)>,
    active_download: Option<(String, String, DownloadState)>,
    download_rx: Option<mpsc::Receiver<DownloadState>>,
    search_log_filter: String,
    selected_service_filter: Option<ServiceType>,
    selected_kind_filter: Option<PackageKind>,
    selected_os_filter: OsFilter,
    new_php_server_name: String,
    new_php_server_host: String,
    new_php_server_port: u16,
    new_php_server_root: String,
    tokio_handle: tokio::runtime::Handle,
}

impl AmpApp {
    pub fn new(cc: &eframe::CreationContext<'_>, tokio_handle: tokio::runtime::Handle) -> Self {
        let config_path = PathBuf::from("config.json");
        let config = AppConfig::load_or_default(&config_path);

        // Configure visual styles according to saved theme
        if config.theme == "light" {
            cc.egui_ctx.set_visuals(Visuals::light());
        } else {
            let mut visuals = Visuals::dark();
            visuals.window_rounding = 8.0.into();
            visuals.menu_rounding = 6.0.into();
            cc.egui_ctx.set_visuals(visuals);
        }

        let service_manager = Arc::new(Mutex::new(ServiceManager::new(config.bin_dir.clone())));

        // Load local manifest if available
        let local_manifest_path = PathBuf::from("assets/versions.json");
        let manifest = if local_manifest_path.exists() {
            std::fs::read_to_string(&local_manifest_path)
                .ok()
                .and_then(|s| serde_json::from_str::<VersionManifest>(&s).ok())
        } else {
            None
        };

        let lang = config.language.clone();
        let i18n = I18n::new(&lang);
        let ready_msg = i18n.t("system_ready").to_string();

        let mut app = Self {
            current_tab: ActiveTab::Dashboard,
            i18n,
            config,
            config_path,
            settings_status: None,
            status_bar_message: Some((ready_msg, false)),
            service_manager,
            manifest,
            installed_versions: Vec::new(),
            active_download: None,
            download_rx: None,
            search_log_filter: String::new(),
            selected_service_filter: None,
            selected_kind_filter: None,
            selected_os_filter: OsFilter::All,
            new_php_server_name: String::new(),
            new_php_server_host: String::new(),
            new_php_server_port: amp_core::models::defaults::DEFAULT_PHP_SERVER_PORT,
            new_php_server_root: amp_core::models::defaults::DEFAULT_DOC_ROOT.to_string(),
            tokio_handle,
        };

        app.refresh_installed_versions();
        app
    }

    fn refresh_installed_versions(&mut self) {
        let services = [
            ServiceType::Apache,
            ServiceType::Php,
            ServiceType::Mariadb,
            ServiceType::Mysql,
            ServiceType::Postgresql,
            ServiceType::Tool,
        ];

        let mut config_changed = false;
        let mut installed = Vec::new();

        if let Ok(sm) = self.service_manager.lock() {
            for &st in &services {
                let local_dirs = sm.list_installed_versions(st);
                for ver in &local_dirs {
                    let needs_register = match self.config.get_binary_path(st, ver) {
                        Some(p) => !p.exists(),
                        None => true,
                    };
                    if needs_register {
                        let ver_dir = self.config.bin_dir.join(st.folder_name()).join(ver);
                        if let Some(bin_path) = ServiceManager::discover_binary_path(st, &ver_dir) {
                            self.config.register_version(
                                st,
                                ver.clone(),
                                bin_path.to_string_lossy().to_string(),
                                amp_core::models::VersionSource::Downloaded,
                            );
                            config_changed = true;
                        }
                    }
                }

                // Check system version
                if let Some(sys_ver) = ServiceManager::detect_system_version(st) {
                    let sys_key = format!("system ({})", sys_ver);
                    if self.config.get_binary_path(st, &sys_key).is_none() {
                        let sys_cmd = match st {
                            ServiceType::Apache => "apache2",
                            ServiceType::Php => "php",
                            ServiceType::Mysql => "mysqld",
                            ServiceType::Mariadb => "mariadbd",
                            ServiceType::Postgresql => "postgres",
                            ServiceType::Tool => "",
                        };
                        #[cfg(not(windows))]
                        let path_opt = std::process::Command::new("which").arg(sys_cmd).output().ok().and_then(|out| {
                            if out.status.success() {
                                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                                if !s.is_empty() { Some(s) } else { None }
                            } else { None }
                        });
                        #[cfg(windows)]
                        let path_opt = None;

                        if let Some(path) = path_opt {
                            self.config.register_version(
                                st,
                                sys_key,
                                path,
                                amp_core::models::VersionSource::System,
                            );
                            config_changed = true;
                        }
                    }
                }

                // Collect local dirs + registered versions from config
                let mut all_versions = local_dirs;
                if let Some(entry) = self.config.get_service_entry(st) {
                    for v in entry.versions.keys() {
                        if !all_versions.contains(v) {
                            all_versions.push(v.clone());
                        }
                    }
                }
                all_versions.sort();
                installed.push((st, all_versions));
            }
        }

        if config_changed {
            let _ = self.config.save(&self.config_path);
        }

        self.installed_versions = installed;
    }

    fn start_package_download(&mut self, item: PackageItem) {
        let (tx, rx) = mpsc::channel(64);
        let pkg_label = format!("{} v{}", item.id, item.version);
        let pkg_url = item.url.clone();
        self.active_download = Some((pkg_label.clone(), pkg_url.clone(), DownloadState::Downloading {
            downloaded_bytes: 0,
            total_bytes: item.size_bytes,
            percent: 0.0,
        }));
        self.download_rx = Some(rx);
        self.status_bar_message = Some((format!("{}...", self.i18n.t("downloading")), false));

        let destination = self.config.bin_dir.clone();
        let sm_arc = Arc::clone(&self.service_manager);
        let item_id = item.id.clone();
        let url_clone = item.url.clone();

        if let Ok(sm) = self.service_manager.lock() {
            sm.add_log_event(LogEvent::DownloadStarted {
                item_id: item_id.clone(),
                url: url_clone.clone(),
            });
        }

        self.tokio_handle.spawn(async move {
            let res = PackageManager::download_and_extract(&item, &destination, Some(tx)).await;
            if let Ok(sm) = sm_arc.lock() {
                match res {
                    Ok(_path) => {
                        sm.add_log_event(LogEvent::DownloadCompleted {
                            item_id,
                            url: url_clone,
                        });
                    }
                    Err(err) => {
                        sm.add_log_event(LogEvent::DownloadFailed {
                            item_id,
                            url: url_clone,
                            error: err.to_string(),
                        });
                    }
                }
            }
        });
    }

    fn handle_service_action(&mut self, action: ServiceAction) {
        let ports = [
            (ServiceType::Apache, self.config.apache_port),
            (ServiceType::Php, self.config.php_fpm_port),
            (ServiceType::Mariadb, self.config.mariadb_port),
            (ServiceType::Mysql, self.config.mysql_port),
            (ServiceType::Postgresql, self.config.postgresql_port),
        ];

        match action {
            ServiceAction::Start(st) => {
                let port = ports.iter().find(|(t, _)| *t == st).map(|(_, p)| *p).unwrap_or(st.default_port());
                let ver = self.config.get_active_version(st).unwrap_or("default");
                let custom_bin = self.config.get_active_binary_path(st).map(|p| p.to_string_lossy().to_string());
                if let Ok(mut sm) = self.service_manager.lock() {
                    match sm.start_service(st, ver, port, custom_bin.as_deref()) {
                        Ok(()) => {
                            let msg = format!("{} :{}", st.display_name(), port);
                            self.status_bar_message = Some((msg, false));
                        }
                        Err(err) => {
                            let err_msg = self.i18n.format_error(&err);
                            self.status_bar_message = Some((err_msg, true));
                        }
                    }
                }
            }
            ServiceAction::Stop(st) => {
                if let Ok(mut sm) = self.service_manager.lock() {
                    let _ = sm.stop_service(st);
                    let msg = format!("{} {}", st.display_name(), self.i18n.t("offline"));
                    self.status_bar_message = Some((msg, false));
                }
            }
            ServiceAction::Restart(st) => {
                let port = ports.iter().find(|(t, _)| *t == st).map(|(_, p)| *p).unwrap_or(st.default_port());
                let ver = self.config.get_active_version(st).unwrap_or("default");
                let custom_bin = self.config.get_active_binary_path(st).map(|p| p.to_string_lossy().to_string());
                if let Ok(mut sm) = self.service_manager.lock() {
                    let _ = sm.stop_service(st);
                    match sm.start_service(st, ver, port, custom_bin.as_deref()) {
                        Ok(()) => {
                            let msg = format!("{} :{}", st.display_name(), port);
                            self.status_bar_message = Some((msg, false));
                        }
                        Err(err) => {
                            let err_msg = self.i18n.format_error(&err);
                            self.status_bar_message = Some((err_msg, true));
                        }
                    }
                }
            }
            ServiceAction::StartAll => {
                let mut errors = Vec::new();
                if let Ok(mut sm) = self.service_manager.lock() {
                    for (st, port) in ports {
                        let ver = self.config.get_active_version(st).unwrap_or("default");
                        let custom_bin = self.config.get_active_binary_path(st).map(|p| p.to_string_lossy().to_string());
                        if let Err(err) = sm.start_service(st, ver, port, custom_bin.as_deref()) {
                            errors.push(self.i18n.format_error(&err));
                        }
                    }
                }
                if errors.is_empty() {
                    self.status_bar_message = Some((self.i18n.t("all_started").to_string(), false));
                } else {
                    self.status_bar_message = Some((errors.join("; "), true));
                }
            }
            ServiceAction::StopAll => {
                if let Ok(mut sm) = self.service_manager.lock() {
                    sm.stop_all();
                    self.status_bar_message = Some((self.i18n.t("all_stopped").to_string(), false));
                }
            }
        }
    }

    fn handle_php_server_action(&mut self, action: PhpServerAction) {
        match action {
            PhpServerAction::Start(id) => {
                if let Some(inst) = self.config.php_servers.iter().find(|s| s.id == id).cloned() {
                    let php_ver = inst.php_version.as_deref().or_else(|| self.config.get_active_version(ServiceType::Php));
                    let custom_bin = if let Some(pv) = php_ver {
                        self.config.get_binary_path(ServiceType::Php, pv).map(|p| p.to_string_lossy().to_string())
                    } else {
                        self.config.get_active_binary_path(ServiceType::Php).map(|p| p.to_string_lossy().to_string())
                    };
                    if let Ok(mut sm) = self.service_manager.lock() {
                        match sm.start_php_server(&inst, php_ver, custom_bin.as_deref()) {
                            Ok(()) => {
                                self.status_bar_message = Some((
                                    format!("PHP '{}' :{}", inst.name, inst.port),
                                    false,
                                ));
                            }
                            Err(err) => {
                                let err_msg = self.i18n.format_error(&err);
                                self.status_bar_message = Some((err_msg, true));
                            }
                        }
                    }
                }
            }
            PhpServerAction::Stop(id) => {
                if let Ok(mut sm) = self.service_manager.lock() {
                    let _ = sm.stop_php_server(&id);
                    self.status_bar_message = Some((self.i18n.t("offline").to_string(), false));
                }
            }
            PhpServerAction::Add(instance) => {
                self.config.php_servers.push(instance);
                let _ = self.config.save(&self.config_path);
            }
            PhpServerAction::Remove(id) => {
                if let Ok(mut sm) = self.service_manager.lock() {
                    let _ = sm.stop_php_server(&id);
                }
                self.config.php_servers.retain(|s| s.id != id);
                let _ = self.config.save(&self.config_path);
            }
            PhpServerAction::OpenVersions => {
                self.current_tab = ActiveTab::Versions;
                self.selected_service_filter = Some(ServiceType::Php);
            }
            PhpServerAction::SetVersion(id, ver) => {
                if let Some(inst) = self.config.php_servers.iter_mut().find(|s| s.id == id) {
                    inst.php_version = ver;
                    let _ = self.config.save(&self.config_path);
                }
            }
        }
    }
}

impl eframe::App for AmpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll active processes to detect termination and update states
        if let Ok(mut sm) = self.service_manager.lock() {
            sm.poll_active_processes();
        }

        // Poll active download progress
        let mut should_refresh_after_dl = false;
        if let Some(rx) = &mut self.download_rx {
            while let Ok(state) = rx.try_recv() {
                if let Some((name, url, current_state)) = &mut self.active_download {
                    *current_state = state.clone();
                    if matches!(state, DownloadState::Completed) {
                        self.status_bar_message = Some((format!("{} ({})", self.i18n.t("download_complete"), name), false));
                        should_refresh_after_dl = true;
                    } else if let DownloadState::Failed(err) = &state {
                        self.status_bar_message = Some((format!("⚠ {}: {} [{}]", self.i18n.t("error"), err, url), true));
                        should_refresh_after_dl = true;
                    }
                }
            }
        }
        if should_refresh_after_dl {
            self.refresh_installed_versions();
        }

        // Apply theme changes
        if self.config.theme == "light" {
            ctx.set_visuals(Visuals::light());
        } else {
            ctx.set_visuals(Visuals::dark());
        }

        // --- Bottom Status Bar Panel ---
        egui::TopBottomPanel::bottom("bottom_status_bar")
            .exact_height(28.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some((msg, is_error)) = &self.status_bar_message {
                        let dot_color = if *is_error {
                            Color32::from_rgb(239, 68, 68)
                        } else {
                            Color32::from_rgb(34, 197, 94)
                        };

                        ui.label(RichText::new("●").color(dot_color).size(12.0));
                        ui.label(
                            RichText::new(msg)
                                .color(if *is_error {
                                    Color32::from_rgb(248, 113, 113)
                                } else {
                                    Color32::from_rgb(209, 213, 219)
                                })
                                .size(12.0),
                        );

                        if *is_error {
                            if ui
                                .button(
                                    RichText::new(format!("📜 {}", self.i18n.t("view_logs")))
                                        .color(Color32::from_rgb(96, 165, 250))
                                        .underline()
                                        .size(11.0),
                                )
                                .clicked()
                            {
                                self.current_tab = ActiveTab::Logs;
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("AMP Control v0.1.0")
                                .color(Color32::DARK_GRAY)
                                .size(11.0),
                        );
                    });
                });
            });

        // Left Navigation Sidebar
        egui::SidePanel::left("nav_panel")
            .resizable(false)
            .exact_width(215.0)
            .show(ctx, |ui| {
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⚡").size(24.0).color(Color32::from_rgb(234, 179, 8)));
                    ui.heading(RichText::new(self.i18n.t("app_title")).strong().size(18.0));
                });

                ui.add_space(18.0);

                let tabs = [
                    (ActiveTab::Dashboard, "📊", self.i18n.t("dashboard")),
                    (ActiveTab::PhpServers, "🐘", self.i18n.t("php_servers")),
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
                        RichText::new("Cross-Platform • Multi-language")
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
                    RichText::new("Apache • MariaDB • MySQL • PostgreSQL • PHP Environment")
                        .color(Color32::GRAY)
                        .size(12.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.i18n.current_language() == "pt-PT" {
                        if ui.button("🇬🇧 EN").clicked() {
                            self.config.language = "en-US".to_string();
                            self.i18n.set_language("en-US");
                            let _ = self.config.save(&self.config_path);
                        }
                    } else {
                        if ui.button("🇵🇹 PT").clicked() {
                            self.config.language = "pt-PT".to_string();
                            self.i18n.set_language("pt-PT");
                            let _ = self.config.save(&self.config_path);
                        }
                    }
                });
            });
            ui.add_space(6.0);
        });

        // Action queues to avoid borrow conflicts
        let mut service_actions = Vec::new();
        let mut php_actions = Vec::new();
        let mut downloads_to_start = Vec::new();
        let mut settings_actions = Vec::new();
        let mut version_to_activate = None;
        let mut refresh_versions_requested = false;

        // Central Content Area
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut services_info = Vec::new();
            let mut logs = Vec::new();

            if let Ok(sm) = self.service_manager.lock() {
                let all_types = [
                    ServiceType::Apache,
                    ServiceType::Php,
                    ServiceType::Mariadb,
                    ServiceType::Mysql,
                    ServiceType::Postgresql,
                ];
                for st in all_types {
                    let active_v = self.config.get_active_version(st);
                    let source = self.config.get_service_entry(st)
                        .and_then(|e| e.active_version.as_ref().and_then(|v| e.versions.get(v).map(|ve| ve.source)));
                    services_info.push(sm.get_service_info(st, active_v, source));
                }
                logs = sm.get_recent_logs();
            }

            match self.current_tab {
                ActiveTab::Dashboard => {
                    render_dashboard(
                        ui,
                        &self.i18n,
                        &services_info,
                        &self.config,
                        &mut service_actions,
                    );
                }
                ActiveTab::PhpServers => {
                    let is_running_checker = |id: &str| {
                        if let Ok(mut sm) = self.service_manager.lock() {
                            sm.is_php_server_running(id)
                        } else {
                            false
                        }
                    };

                    let php_installed = self
                        .installed_versions
                        .iter()
                        .find(|(st, _)| *st == ServiceType::Php)
                        .map(|(_, v)| v.as_slice())
                        .unwrap_or(&[]);
                    let active_php = self.config.get_active_version(ServiceType::Php);

                    render_php_servers(
                        ui,
                        &self.i18n,
                        &self.config.php_servers,
                        is_running_checker,
                        &mut php_actions,
                        &mut self.new_php_server_name,
                        &mut self.new_php_server_host,
                        &mut self.new_php_server_port,
                        &mut self.new_php_server_root,
                        &self.config.default_host,
                        php_installed,
                        active_php,
                    );
                }
                ActiveTab::Versions => {
                    let active_versions = [
                        (ServiceType::Apache, self.config.get_active_version(ServiceType::Apache).map(|s| s.to_string())),
                        (ServiceType::Php, self.config.get_active_version(ServiceType::Php).map(|s| s.to_string())),
                        (ServiceType::Mariadb, self.config.get_active_version(ServiceType::Mariadb).map(|s| s.to_string())),
                        (ServiceType::Mysql, self.config.get_active_version(ServiceType::Mysql).map(|s| s.to_string())),
                        (ServiceType::Postgresql, self.config.get_active_version(ServiceType::Postgresql).map(|s| s.to_string())),
                    ];

                    render_versions(
                        ui,
                        &self.i18n,
                        self.manifest.as_ref(),
                        &self.installed_versions,
                        self.active_download.as_ref(),
                        &mut self.selected_service_filter,
                        &mut self.selected_kind_filter,
                        &mut self.selected_os_filter,
                        &active_versions,
                        &mut version_to_activate,
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
                        &mut self.config,
                        &self.settings_status,
                        &mut settings_actions,
                    );
                }
            }
        });

        // Execute actions outside UI borrows
        for action in service_actions {
            self.handle_service_action(action);
        }

        for action in php_actions {
            self.handle_php_server_action(action);
        }

        for item in downloads_to_start {
            self.start_package_download(item);
        }

        if let Some((st, ver)) = version_to_activate {
            self.config.set_active_version(st, Some(ver));
            let _ = self.config.save(&self.config_path);
            self.refresh_installed_versions();
        }

        for action in settings_actions {
            match action {
                SettingsAction::Save => {
                    let _ = self.config.save(&self.config_path);
                    self.settings_status = Some(self.i18n.t("settings_saved").to_string());
                    self.status_bar_message = Some((self.i18n.t("settings_saved").to_string(), false));
                }
                SettingsAction::ResetDefaults => {
                    self.config.reset_to_defaults();
                    let _ = self.config.save(&self.config_path);
                    self.i18n.set_language(&self.config.language);
                    self.settings_status = Some(self.i18n.t("defaults_restored").to_string());
                    self.status_bar_message = Some((self.i18n.t("defaults_restored").to_string(), false));
                }
            }
        }

        if refresh_versions_requested {
            self.refresh_installed_versions();
        }

        // Request redraw regularly for smooth UI / log streaming
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}
