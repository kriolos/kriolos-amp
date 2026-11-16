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

    // Service cards: Grid with 2 rows of up to 3 cards
    let primary_services = [ServiceType::Apache, ServiceType::Php, ServiceType::Mariadb];
    ui.columns(3, |columns| {
        for (i, st) in primary_services.iter().enumerate() {
            if let Some(info) = services.iter().find(|s| s.service_type == *st) {
                render_service_card(&mut columns[i], i18n, info, actions);
            }
        }
    });

    ui.add_space(10.0);

    let secondary_services = [ServiceType::Mysql, ServiceType::Postgresql];
    ui.columns(2, |columns| {
        for (i, st) in secondary_services.iter().enumerate() {
            if let Some(info) = services.iter().find(|s| s.service_type == *st) {
                render_service_card(&mut columns[i], i18n, info, actions);
            }
        }
    });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(10.0);

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
    ui.add_space(10.0);

    // Port status indicators
    ui.heading(RichText::new(i18n.t("ports_status")).size(16.0));
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        render_port_pill(ui, i18n, "Apache HTTP", config.apache_port);
        ui.add_space(12.0);
        render_port_pill(ui, i18n, "Apache SSL", config.apache_ssl_port);
        ui.add_space(12.0);
        render_port_pill(ui, i18n, "MariaDB", config.mariadb_port);
        ui.add_space(12.0);
        render_port_pill(ui, i18n, "MySQL", config.mysql_port);
        ui.add_space(12.0);
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
            ui.horizontal(|ui| {
                ui.label(RichText::new(service.service_type.icon()).size(22.0));
                ui.heading(RichText::new(service.service_type.display_name()).size(14.0));
            });

            ui.add_space(8.0);

            // Status badge
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
                    ServiceStatus::Error(err) => {
                        ui.label(
                            RichText::new(format!("⚠ {}: {}", i18n.t("error"), err))
                                .color(Color32::from_rgb(239, 68, 68))
                                .strong(),
                        );
                    }
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(format!("{}:", i18n.t("port")));
                ui.label(RichText::new(service.port.to_string()).strong());
            });

            ui.horizontal(|ui| {
                ui.label(format!("{}:", i18n.t("version")));
                if let Some(ver) = &service.active_version {
                    ui.label(RichText::new(ver).monospace());
                } else {
                    ui.label(RichText::new(i18n.t("no_version_installed")).italics());
                }
            });

            ui.add_space(10.0);

            // Action buttons
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
