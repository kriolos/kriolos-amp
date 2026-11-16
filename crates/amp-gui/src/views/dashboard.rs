use crate::i18n::I18n;
use amp_core::models::{AppConfig, ServiceInfo, ServiceStatus, ServiceType};
use amp_core::PortChecker;
use egui::{Color32, RichText, Ui};

pub enum ServiceAction {
    Start(ServiceType),
    Stop(ServiceType),
    Restart(ServiceType),
    StartAll,
    StopAll,
}

pub fn render_dashboard(
    ui: &mut Ui,
    i18n: &I18n,
    services: &[ServiceInfo],
    config: &AppConfig,
    actions: &mut Vec<ServiceAction>,
) {
    ui.add_space(8.0);

    // Global quick actions banner
    ui.horizontal(|ui| {
        ui.heading(RichText::new("📊 ").size(20.0));
        ui.heading(RichText::new(i18n.t("dashboard")).strong().size(20.0));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let stop_all_label = format!("⏹ {}", i18n.t("stop_all"));
            if ui
                .button(RichText::new(stop_all_label).color(Color32::from_rgb(239, 68, 68)).strong())
                .clicked()
            {
                actions.push(ServiceAction::StopAll);
            }

            let start_all_label = format!("▶ {}", i18n.t("start_all"));
            if ui
                .button(RichText::new(start_all_label).color(Color32::from_rgb(34, 197, 94)).strong())
                .clicked()
            {
                actions.push(ServiceAction::StartAll);
            }
        });
    });

    ui.add_space(14.0);

    // Compact & Perfectly Aligned 3-Column Grid
    egui::Grid::new("services_cards_grid")
        .num_columns(3)
        .spacing([24.0, 16.0])
        .show(ui, |ui| {
            // Row 1: Apache, PHP, MariaDB
            if let Some(info) = services.iter().find(|s| s.service_type == ServiceType::Apache) {
                render_service_card(ui, i18n, info, actions);
            } else {
                ui.label("");
            }

            if let Some(info) = services.iter().find(|s| s.service_type == ServiceType::Php) {
                render_service_card(ui, i18n, info, actions);
            } else {
                ui.label("");
            }

            if let Some(info) = services.iter().find(|s| s.service_type == ServiceType::Mariadb) {
                render_service_card(ui, i18n, info, actions);
            } else {
                ui.label("");
            }
            ui.end_row();

            // Row 2: MySQL (under Apache), PostgreSQL (under PHP)
            if let Some(info) = services.iter().find(|s| s.service_type == ServiceType::Mysql) {
                render_service_card(ui, i18n, info, actions);
            } else {
                ui.label("");
            }

            if let Some(info) = services.iter().find(|s| s.service_type == ServiceType::Postgresql) {
                render_service_card(ui, i18n, info, actions);
            } else {
                ui.label("");
            }
            ui.end_row();
        });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(12.0);

    // Quick shortcuts
    ui.heading(RichText::new(i18n.t("quick_actions")).size(16.0));
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button(format!("📁 {}", i18n.t("open_www"))).clicked() {
            let _ = open::that(&config.root_www_dir);
        }
        if ui.button(format!("🌐 Localhost (: {})", config.apache_port)).clicked() {
            let _ = open::that(format!("http://localhost:{}", config.apache_port));
        }
        if ui.button(format!("🐬 phpMyAdmin (Apache)")).clicked() {
            let _ = open::that(format!("http://localhost:{}/phpmyadmin", config.apache_port));
        }
        if ui.button(format!("🐘 phpinfo()")).clicked() {
            let _ = open::that(format!("http://localhost:{}/phpinfo.php", config.apache_port));
        }
    });

    ui.add_space(18.0);
    ui.separator();
    ui.add_space(12.0);

    // Monitored ports status
    ui.heading(RichText::new(i18n.t("ports_status")).size(16.0));
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        render_port_pill(ui, i18n, "Apache HTTP", config.apache_port);
        ui.add_space(16.0);
        render_port_pill(ui, i18n, "Apache SSL", config.apache_ssl_port);
        ui.add_space(16.0);
        render_port_pill(ui, i18n, "MariaDB", config.mariadb_port);
        ui.add_space(16.0);
        render_port_pill(ui, i18n, "MySQL", config.mysql_port);
        ui.add_space(16.0);
        render_port_pill(ui, i18n, "PostgreSQL", config.postgresql_port);
    });
}

fn render_service_card(
    ui: &mut Ui,
    i18n: &I18n,
    service: &ServiceInfo,
    actions: &mut Vec<ServiceAction>,
) {
    egui::Frame::group(ui.style())
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_width(260.0);

            let icon = match service.service_type {
                ServiceType::Apache => "🌐",
                ServiceType::Php => "🐘",
                ServiceType::Mariadb => "🗄️",
                ServiceType::Mysql => "🐬",
                ServiceType::Postgresql => "🐘",
                ServiceType::Tool => "🛠️",
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(18.0));
                ui.heading(RichText::new(service.service_type.display_name()).size(14.0));
            });

            ui.add_space(8.0);

            // Status Row
            ui.horizontal(|ui| {
                ui.label(format!("{}:", i18n.t("status")));
                match &service.status {
                    ServiceStatus::Running { pid, .. } => {
                        let text = if *pid > 0 {
                            format!("● {} (PID: {})", i18n.t("online"), pid)
                        } else {
                            format!("● {}", i18n.t("online"))
                        };
                        ui.label(RichText::new(text).color(Color32::from_rgb(34, 197, 94)).strong());
                    }
                    ServiceStatus::Starting => {
                        ui.label(
                            RichText::new(format!("⏳ {}", i18n.t("starting")))
                                .color(Color32::from_rgb(234, 179, 8))
                                .strong(),
                        );
                    }
                    ServiceStatus::Stopped => {
                        ui.label(
                            RichText::new(format!("○ {}", i18n.t("offline")))
                                .color(Color32::from_rgb(156, 163, 175)),
                        );
                    }
                    ServiceStatus::Error(_) => {
                        ui.label(
                            RichText::new(format!("⚠ {}", i18n.t("error")))
                                .color(Color32::from_rgb(239, 68, 68))
                                .strong(),
                        );
                    }
                }
            });

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(format!("{}:", i18n.t("port")));
                ui.label(RichText::new(service.port.to_string()).strong());
            });

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(format!("{}:", i18n.t("version")));
                if let Some(ver) = &service.active_version {
                    ui.label(RichText::new(ver).monospace());
                } else {
                    ui.label(RichText::new(i18n.t("no_version_installed")).italics());
                }
            });

            ui.add_space(10.0);

            // Action buttons directly underneath
            ui.horizontal(|ui| {
                let is_running = matches!(service.status, ServiceStatus::Running { .. });

                if is_running {
                    if ui.button(format!("⏹ {}", i18n.t("stop"))).clicked() {
                        actions.push(ServiceAction::Stop(service.service_type));
                    }
                    if ui.button(format!("🔄 {}", i18n.t("restart"))).clicked() {
                        actions.push(ServiceAction::Restart(service.service_type));
                    }
                } else {
                    if ui
                        .button(RichText::new(format!("▶ {}", i18n.t("start"))).strong())
                        .clicked()
                    {
                        actions.push(ServiceAction::Start(service.service_type));
                    }
                }
            });
        });
}

fn render_port_pill(ui: &mut Ui, i18n: &I18n, name: &str, port: u16) {
    let in_use = PortChecker::is_port_in_use(port);
    let (color, status_text) = if in_use {
        (Color32::from_rgb(34, 197, 94), i18n.t("port_in_use"))
    } else {
        (Color32::from_rgb(148, 163, 184), i18n.t("port_available"))
    };

    ui.horizontal(|ui| {
        ui.label(format!("{}:", name));
        ui.label(RichText::new(format!(":{}", port)).strong());
        ui.label(RichText::new(format!("({})", status_text)).color(color));
    });
}
