use crate::components::{PortIndicator, ServiceCard};
use crate::i18n::I18n;
use amp_core::models::{AppConfig, ServiceInfo, ServiceType};
use egui::{Color32, RichText, Ui, Vec2};

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
    // Scroll area only activates when window height is smaller than content
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
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

            ui.add_space(16.0);

            // 100% Dynamic & Responsive Service Cards (flex-wrap: 3 -> 2 -> 1 card on resize)
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(16.0, 16.0);
                for service in services {
                    if let Some(action) = ServiceCard::new(service, i18n).show(ui) {
                        actions.push(action);
                    }
                }
            });

            ui.add_space(24.0);
            ui.separator();
            ui.add_space(12.0);

            // Quick shortcuts
            ui.heading(RichText::new(i18n.t("quick_actions")).size(16.0));
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(12.0, 8.0);
                if ui.button(format!("📁 {}", i18n.t("open_www"))).clicked() {
                    let _ = open::that(&config.root_www_dir);
                }
                if ui.button(format!("🌐 Localhost (: {})", config.apache_port)).clicked() {
                    let _ = open::that(format!("http://localhost:{}", config.apache_port));
                }
                if ui.button("🐬 phpMyAdmin (Apache)").clicked() {
                    let _ = open::that(format!("http://localhost:{}/phpmyadmin", config.apache_port));
                }
                if ui.button("🐘 phpinfo()").clicked() {
                    let _ = open::that(format!("http://localhost:{}/phpinfo.php", config.apache_port));
                }
            });

            ui.add_space(18.0);
            ui.separator();
            ui.add_space(12.0);

            // 100% Dynamic Monitored Ports Status (responsive wrap)
            ui.heading(RichText::new(i18n.t("ports_status")).size(16.0));
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(18.0, 10.0);
                for service in services {
                    PortIndicator::show(ui, i18n, service.service_type.display_name(), service.port);
                }
            });

            ui.add_space(16.0);
        });
}
