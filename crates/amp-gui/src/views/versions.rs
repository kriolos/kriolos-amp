use crate::i18n::I18n;
use amp_core::models::{DownloadState, PackageItem, ServiceType, VersionManifest};
use egui::{Color32, ProgressBar, RichText, Ui};

pub fn render_versions(
    ui: &mut Ui,
    i18n: &I18n,
    manifest: Option<&VersionManifest>,
    installed_versions: &[(ServiceType, Vec<String>)],
    active_download: Option<&(String, DownloadState)>,
    downloads_to_start: &mut Vec<PackageItem>,
    refresh_requested: &mut bool,
) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.heading(RichText::new("📦 ").size(20.0));
        ui.heading(RichText::new(i18n.t("versions")).strong().size(20.0));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(format!("🔄 {}", i18n.t("refresh_catalog"))).clicked() {
                *refresh_requested = true;
            }
        });
    });

    ui.add_space(12.0);

    // Active download progress bar
    if let Some((pkg_name, state)) = active_download {
        egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(RichText::new(format!("⚡ {}", pkg_name)).strong());
                match state {
                    DownloadState::Downloading {
                        percent,
                        downloaded_bytes,
                        total_bytes,
                    } => {
                        let total_str = total_bytes
                            .map(|b| format!("{:.1} MB", b as f64 / 1_048_576.0))
                            .unwrap_or_else(|| "? MB".to_string());
                        let dl_str = format!("{:.1} MB", *downloaded_bytes as f64 / 1_048_576.0);

                        ui.add(ProgressBar::new(percent / 100.0).text(format!("{:.1}%", percent)));
                        ui.label(format!("{} ({} / {})", i18n.t("downloading"), dl_str, total_str));
                    }
                    DownloadState::Extracting => {
                        ui.label(
                            RichText::new(format!("📦 {}", i18n.t("extracting")))
                                .color(Color32::from_rgb(234, 179, 8)),
                        );
                    }
                    DownloadState::Completed => {
                        ui.label(
                            RichText::new(format!("✓ {}", i18n.t("download_complete")))
                                .color(Color32::from_rgb(34, 197, 94))
                                .strong(),
                        );
                    }
                    DownloadState::Failed(err) => {
                        ui.label(
                            RichText::new(format!("⚠ Erro: {}", err))
                                .color(Color32::from_rgb(239, 68, 68))
                                .strong(),
                        );
                    }
                    DownloadState::Idle => {}
                }
            });
        ui.add_space(16.0);
    }

    ui.separator();
    ui.add_space(8.0);
    ui.label(RichText::new(i18n.t("download_catalog")).strong());
    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        if let Some(man) = manifest {
            for pkg in &man.packages {
                let is_installed = installed_versions
                    .iter()
                    .find(|(st, _)| *st == pkg.service_type)
                    .map(|(_, vers)| vers.contains(&pkg.version))
                    .unwrap_or(false);

                egui::Frame::group(ui.style())
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let icon = match pkg.service_type {
                                ServiceType::Apache => "🌐",
                                ServiceType::Database => "🗄️",
                                ServiceType::Php => "🐘",
                                ServiceType::Tool => "🛠️",
                            };

                            ui.label(RichText::new(icon).size(18.0));
                            ui.label(RichText::new(&pkg.id).strong());
                            if !pkg.version.is_empty() {
                                ui.label(RichText::new(format!("v{}", pkg.version)).monospace());
                            }

                            if pkg.recommended {
                                ui.label(
                                    RichText::new("⭐ Recomendado")
                                        .color(Color32::from_rgb(234, 179, 8))
                                        .size(11.0),
                                );
                            }

                            ui.label(
                                RichText::new(format!("({})", pkg.platform))
                                    .color(Color32::GRAY)
                                    .size(11.0),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if pkg.external_link {
                                    if ui
                                        .button(format!("🌐 {}", i18n.t("open_in_browser")))
                                        .clicked()
                                    {
                                        let _ = open::that(&pkg.url);
                                    }
                                } else if is_installed {
                                    ui.label(
                                        RichText::new(format!("✓ {}", i18n.t("installed")))
                                            .color(Color32::from_rgb(34, 197, 94)),
                                    );
                                } else {
                                    if ui
                                        .button(format!("⬇ {}", i18n.t("download_and_install")))
                                        .clicked()
                                    {
                                        downloads_to_start.push(pkg.clone());
                                    }
                                }
                            });
                        });

                        if let Some(desc) = &pkg.description {
                            ui.add_space(2.0);
                            ui.label(RichText::new(desc).color(Color32::GRAY).size(12.0));
                        }
                    });
                ui.add_space(6.0);
            }
        } else {
            ui.label("Carregando catálogo de versões...");
        }
    });
}
