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
    AppConfig, DownloadState, PackageItem, PackageKind, ServiceType, VersionManifest,
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
    active_download: Option<(String, DownloadState)>,
    download_rx: Option<mpsc::Receiver<DownloadState>>,
    search_log_filter: String,
    selected_service_filter: Option<ServiceType>,
    selected_kind_filter: Option<PackageKind>,
    new_php_server_name: String,
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

        let mut app = Self {
            current_tab: ActiveTab::Dashboard,
            i18n: I18n::new(&lang),
            config,
            config_path,
            settings_status: None,
            status_bar_message: Some(("Sistema pronto e inicializado.".to_string(), false)),
            service_manager,
            manifest,
            installed_versions: Vec::new(),
            active_download: None,
            download_rx: None,
            search_log_filter: String::new(),
            selected_service_filter: None,
            selected_kind_filter: None,
            new_php_server_name: String::new(),
            new_php_server_port: 8000,
            new_php_server_root: "www".to_string(),
            tokio_handle,
        };

        app.refresh_installed_versions();
        app
    }

    fn refresh_installed_versions(&mut self) {
        if let Ok(sm) = self.service_manager.lock() {
            self.installed_versions = vec![
                (ServiceType::Apache, sm.list_installed_versions(ServiceType::Apache)),
                (ServiceType::Php, sm.list_installed_versions(ServiceType::Php)),
                (ServiceType::Mariadb, sm.list_installed_versions(ServiceType::Mariadb)),
                (ServiceType::Mysql, sm.list_installed_versions(ServiceType::Mysql)),
                (ServiceType::Postgresql, sm.list_installed_versions(ServiceType::Postgresql)),
            ];
        }
    }

    fn start_package_download(&mut self, item: PackageItem) {
        let (tx, rx) = mpsc::channel(64);
        let pkg_label = format!("{} v{}", item.id, item.version);
        self.active_download = Some((pkg_label.clone(), DownloadState::Downloading {
            downloaded_bytes: 0,
            total_bytes: item.size_bytes,
            percent: 0.0,
        }));
        self.download_rx = Some(rx);
        self.status_bar_message = Some((format!("A descarregar {}", pkg_label), false));

        let destination = self.config.bin_dir.clone();
        self.tokio_handle.spawn(async move {
            let _ = PackageManager::download_and_extract(&item, &destination, Some(tx)).await;
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
                if let Ok(mut sm) = self.service_manager.lock() {
                    match sm.start_service(st, "default", port) {
                        Ok(()) => {
                            self.status_bar_message = Some((
                                format!("{} iniciado com sucesso na porta {}", st.display_name(), port),
                                false,
                            ));
                        }
                        Err(err) => {
                            self.status_bar_message = Some((
                                format!("Falha ao iniciar {}: {}", st.display_name(), err),
                                true,
                            ));
                        }
                    }
                }
            }
            ServiceAction::Stop(st) => {
                if let Ok(mut sm) = self.service_manager.lock() {
                    let _ = sm.stop_service(st);
                    self.status_bar_message = Some((format!("{} parado.", st.display_name()), false));
                }
            }
            ServiceAction::Restart(st) => {
                let port = ports.iter().find(|(t, _)| *t == st).map(|(_, p)| *p).unwrap_or(st.default_port());
                if let Ok(mut sm) = self.service_manager.lock() {
                    let _ = sm.stop_service(st);
                    match sm.start_service(st, "default", port) {
                        Ok(()) => {
                            self.status_bar_message = Some((
                                format!("{} reiniciado na porta {}", st.display_name(), port),
                                false,
                            ));
                        }
                        Err(err) => {
                            self.status_bar_message = Some((
                                format!("Falha ao reiniciar {}: {}", st.display_name(), err),
                                true,
                            ));
                        }
                    }
                }
            }
            ServiceAction::StartAll => {
                let mut errors = Vec::new();
                if let Ok(mut sm) = self.service_manager.lock() {
                    for (st, port) in ports {
                        if let Err(err) = sm.start_service(st, "default", port) {
                            errors.push(format!("{}: {}", st.display_name(), err));
                        }
                    }
                }
                if errors.is_empty() {
                    self.status_bar_message = Some(("Todos os serviços iniciados com sucesso.".to_string(), false));
                } else {
                    self.status_bar_message = Some((format!("Erros ao iniciar: {}", errors.join("; ")), true));
                }
            }
            ServiceAction::StopAll => {
                if let Ok(mut sm) = self.service_manager.lock() {
                    sm.stop_all();
                    self.status_bar_message = Some(("Todos os serviços foram parados.".to_string(), false));
                }
            }
        }
    }

    fn handle_php_server_action(&mut self, action: PhpServerAction) {
        match action {
            PhpServerAction::Start(id) => {
                if let Some(inst) = self.config.php_servers.iter().find(|s| s.id == id).cloned() {
                    let php_ver = self.config.active_php_version.as_deref();
                    if let Ok(mut sm) = self.service_manager.lock() {
                        match sm.start_php_server(&inst, php_ver) {
                            Ok(()) => {
                                self.status_bar_message = Some((
                                    format!("Servidor PHP '{}' ativo em http://{}:{}", inst.name, inst.host, inst.port),
                                    false,
                                ));
                            }
                            Err(err) => {
                                self.status_bar_message = Some((
                                    format!("Falha ao iniciar servidor PHP '{}': {}", inst.name, err),
                                    true,
                                ));
                            }
                        }
                    }
                }
            }
            PhpServerAction::Stop(id) => {
                if let Ok(mut sm) = self.service_manager.lock() {
                    let _ = sm.stop_php_server(&id);
                    self.status_bar_message = Some(("Servidor PHP parado.".to_string(), false));
                }
            }
            PhpServerAction::Add(instance) => {
                let name = instance.name.clone();
                self.config.php_servers.push(instance);
                let _ = self.config.save(&self.config_path);
                self.status_bar_message = Some((format!("Nova instância '{}' criada.", name), false));
            }
            PhpServerAction::Remove(id) => {
                if let Ok(mut sm) = self.service_manager.lock() {
                    let _ = sm.stop_php_server(&id);
                }
                self.config.php_servers.retain(|s| s.id != id);
                let _ = self.config.save(&self.config_path);
                self.status_bar_message = Some(("Instância de servidor PHP removida.".to_string(), false));
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
                if let Some((name, current_state)) = &mut self.active_download {
                    *current_state = state.clone();
                    if matches!(state, DownloadState::Completed) {
                        self.status_bar_message = Some((format!("✓ Download de {} concluído com sucesso!", name), false));
                        should_refresh_after_dl = true;
                    } else if let DownloadState::Failed(err) = &state {
                        self.status_bar_message = Some((format!("⚠ Erro no download de {}: {}", name, err), true));
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
                                    RichText::new("📜 Ver Logs")
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
                            RichText::new("AMP Control v0.1.0 • Pronto")
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
                        RichText::new("Cross-Platform • 100% Configurable")
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
        let mut refresh_versions_requested = false;

        // Central Content Area
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut services_info = Vec::new();
            let mut logs = Vec::new();

            if let Ok(sm) = self.service_manager.lock() {
                services_info.push(sm.get_service_info(ServiceType::Apache));
                services_info.push(sm.get_service_info(ServiceType::Php));
                services_info.push(sm.get_service_info(ServiceType::Mariadb));
                services_info.push(sm.get_service_info(ServiceType::Mysql));
                services_info.push(sm.get_service_info(ServiceType::Postgresql));
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
                        if let Ok(sm) = self.service_manager.lock() {
                            sm.is_php_server_running(id)
                        } else {
                            false
                        }
                    };

                    render_php_servers(
                        ui,
                        &self.i18n,
                        &self.config.php_servers,
                        is_running_checker,
                        &mut php_actions,
                        &mut self.new_php_server_name,
                        &mut self.new_php_server_port,
                        &mut self.new_php_server_root,
                    );
                }
                ActiveTab::Versions => {
                    render_versions(
                        ui,
                        &self.i18n,
                        self.manifest.as_ref(),
                        &self.installed_versions,
                        self.active_download.as_ref(),
                        &mut self.selected_service_filter,
                        &mut self.selected_kind_filter,
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
                    let log_entries: Vec<amp_core::service_manager::LogMessage> = logs;
                    render_logs(ui, &self.i18n, &log_entries, &mut self.search_log_filter);
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

        for action in settings_actions {
            match action {
                SettingsAction::Save => {
                    let _ = self.config.save(&self.config_path);
                    self.settings_status = Some(self.i18n.t("settings_saved").to_string());
                    self.status_bar_message = Some(("Definições guardadas com sucesso.".to_string(), false));
                }
                SettingsAction::ResetDefaults => {
                    self.config.reset_to_defaults();
                    let _ = self.config.save(&self.config_path);
                    self.i18n.set_language(&self.config.language);
                    self.settings_status = Some("Valores por omissão restaurados!".to_string());
                    self.status_bar_message = Some(("Valores por omissão restaurados.".to_string(), false));
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
