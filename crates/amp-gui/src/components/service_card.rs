use crate::i18n::I18n;
use crate::views::dashboard::ServiceAction;
use amp_core::models::{ServiceInfo, ServiceStatus, VersionSource};
use egui::{Color32, RichText, Ui};

pub struct ServiceCard<'a> {
    pub info: &'a ServiceInfo,
    pub i18n: &'a I18n,
}

impl<'a> ServiceCard<'a> {
    pub fn new(info: &'a ServiceInfo, i18n: &'a I18n) -> Self {
        Self { info, i18n }
    }

    pub fn show(self, ui: &mut Ui) -> Option<ServiceAction> {
        let mut triggered_action = None;

        egui::Frame::group(ui.style())
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(240.0);

                    // Line 1: Header (Icon + Name)
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(self.info.service_type.icon()).size(18.0));
                        ui.heading(RichText::new(self.info.service_type.display_name()).size(14.0));
                    });

                    ui.add_space(8.0);

                    // Line 2: Status
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.i18n.t("status")));
                        match &self.info.status {
                            ServiceStatus::Running { pid, .. } => {
                                let text = if *pid > 0 {
                                    format!("● {} (PID: {})", self.i18n.t("online"), pid)
                                } else {
                                    format!("● {}", self.i18n.t("online"))
                                };
                                ui.label(
                                    RichText::new(text)
                                        .color(Color32::from_rgb(34, 197, 94))
                                        .strong(),
                                );
                            }
                            ServiceStatus::Starting => {
                                ui.label(
                                    RichText::new(format!("⏳ {}", self.i18n.t("starting")))
                                        .color(Color32::from_rgb(234, 179, 8))
                                        .strong(),
                                );
                            }
                            ServiceStatus::Stopped => {
                                ui.label(
                                    RichText::new(format!("○ {}", self.i18n.t("offline")))
                                        .color(Color32::from_rgb(156, 163, 175)),
                                );
                            }
                            ServiceStatus::Error(_) => {
                                ui.label(
                                    RichText::new(format!("⚠ {}", self.i18n.t("error")))
                                        .color(Color32::from_rgb(239, 68, 68))
                                        .strong(),
                                );
                            }
                        }
                    });

                    ui.add_space(2.0);

                    // Line 3: Port
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.i18n.t("port")));
                        ui.label(RichText::new(self.info.port.to_string()).strong());
                    });

                    ui.add_space(2.0);

                    // Line 4: Version (translated source tag)
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.i18n.t("version")));
                        if let Some(ver) = &self.info.active_version {
                            let source_tag = match self.info.version_source {
                                Some(VersionSource::Downloaded) => {
                                    format!(" ({})", self.i18n.t("source_downloaded"))
                                }
                                Some(VersionSource::System) => {
                                    format!(" ({})", self.i18n.t("source_system"))
                                }
                                None => String::new(),
                            };
                            ui.label(RichText::new(format!("{}{}", ver, source_tag)).monospace());
                        } else {
                            ui.label(RichText::new(self.i18n.t("no_version_installed")).italics());
                        }
                    });

                    ui.add_space(10.0);

                    // Line 5: Action buttons
                    ui.horizontal(|ui| {
                        let is_running = matches!(self.info.status, ServiceStatus::Running { .. });

                        if is_running {
                            if ui.button(format!("⏹ {}", self.i18n.t("stop"))).clicked() {
                                triggered_action = Some(ServiceAction::Stop(self.info.service_type));
                            }
                            if ui.button(format!("🔄 {}", self.i18n.t("restart"))).clicked() {
                                triggered_action = Some(ServiceAction::Restart(self.info.service_type));
                            }
                        } else {
                            if ui
                                .button(RichText::new(format!("▶ {}", self.i18n.t("start"))).strong())
                                .clicked()
                            {
                                triggered_action = Some(ServiceAction::Start(self.info.service_type));
                            }
                        }
                    });
                });
            });

        triggered_action
    }
}
