use crate::i18n::I18n;
use amp_core::models::{
    DownloadState, OsFilter, PackageItem, PackageKind, ServiceType, VersionManifest,
};
use egui::{Color32, ProgressBar, RichText, Ui};

pub fn render_versions(
    ui: &mut Ui,
    i18n: &I18n,
    manifest: Option<&VersionManifest>,
    installed_versions: &[(ServiceType, Vec<String>)],
    active_download: Option<&(String, String, DownloadState)>,
    selected_service_filter: &mut Option<ServiceType>,
    selected_kind_filter: &mut Option<PackageKind>,
    selected_os_filter: &mut OsFilter,
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

    ui.add_space(10.0);

    // Active download progress bar
    if let Some((pkg_name, url, state)) = active_download {
        egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("⚡ {}", pkg_name)).strong());
                    ui.label(
                        RichText::new(url)
                            .monospace()
                            .size(10.5)
                            .color(Color32::from_rgb(156, 163, 175)),
                    );
                });
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
                            RichText::new(format!("⚠ {}: {}", i18n.t("error"), err))
                                .color(Color32::from_rgb(239, 68, 68))
                                .strong(),
                        );
                        ui.label(
                            RichText::new(format!("URL: {}", url))
                                .color(Color32::from_rgb(248, 113, 113))
                                .monospace()
                                .size(11.0),
                        );
                    }
                    DownloadState::Idle => {}
                }
            });
        ui.add_space(12.0);
    }

    // Filter Bar - Services
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{}:", i18n.t("service_label"))).strong());

        let service_options = [
            (None, i18n.t("filter_all")),
            (Some(ServiceType::Php), i18n.t("filter_php")),
            (Some(ServiceType::Apache), i18n.t("filter_apache")),
            (Some(ServiceType::Mariadb), i18n.t("filter_mariadb")),
            (Some(ServiceType::Mysql), i18n.t("filter_mysql")),
            (Some(ServiceType::Postgresql), i18n.t("filter_postgresql")),
            (Some(ServiceType::Tool), i18n.t("filter_tools")),
        ];

        for (opt, label) in service_options {
            let is_selected = *selected_service_filter == opt;
            if ui.selectable_label(is_selected, label).clicked() {
                *selected_service_filter = opt;
            }
        }
    });

    ui.add_space(6.0);

    // Filter Bar - Package Kind
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{}:", i18n.t("type_label"))).strong());

        let kind_options = [
            (None, i18n.t("filter_kind_all")),
            (Some(PackageKind::Binary), i18n.t("filter_kind_binary")),
            (Some(PackageKind::Source), i18n.t("filter_kind_source")),
        ];

        for (opt, label) in kind_options {
            let is_selected = *selected_kind_filter == opt;
            if ui.selectable_label(is_selected, label).clicked() {
                *selected_kind_filter = opt;
            }
        }
    });

    ui.add_space(6.0);

    // Filter Bar - Operating System
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{}:", i18n.t("os_label"))).strong());

        let os_options = [
            (OsFilter::All, i18n.t("filter_os_all")),
            (OsFilter::Current, i18n.t("filter_os_current")),
            (OsFilter::Linux, i18n.t("filter_os_linux")),
            (OsFilter::Windows, i18n.t("filter_os_windows")),
            (OsFilter::MacOs, i18n.t("filter_os_macos")),
        ];

        for (opt, label) in os_options {
            let is_selected = *selected_os_filter == opt;
            if ui.selectable_label(is_selected, label).clicked() {
                *selected_os_filter = opt;
            }
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        if let Some(man) = manifest {
            let mut shown_count = 0;

            for pkg in &man.packages {
                // Apply filters
                if let Some(service_filter) = selected_service_filter {
                    if pkg.service_type != *service_filter {
                        continue;
                    }
                }

                if let Some(kind_filter) = selected_kind_filter {
                    if pkg.package_kind != *kind_filter {
                        continue;
                    }
                }

                if !pkg.matches_os(*selected_os_filter) {
                    continue;
                }

                shown_count += 1;

                let is_downloaded = installed_versions
                    .iter()
                    .find(|(st, _)| *st == pkg.service_type)
                    .map(|(_, vers)| vers.contains(&pkg.version))
                    .unwrap_or(false);

                egui::Frame::group(ui.style())
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(pkg.service_type.icon()).size(18.0));
                            ui.label(RichText::new(&pkg.id).strong());

                            if !pkg.version.is_empty() {
                                ui.label(RichText::new(format!("v{}", pkg.version)).monospace());
                            }

                            // Translated Badge Kind
                            match pkg.package_kind {
                                PackageKind::Binary => {
                                    ui.label(
                                        RichText::new(i18n.t("badge_binary"))
                                            .color(Color32::from_rgb(34, 197, 94))
                                            .size(11.0)
                                            .strong(),
                                    );
                                }
                                PackageKind::Source => {
                                    ui.label(
                                        RichText::new(i18n.t("badge_source"))
                                            .color(Color32::from_rgb(249, 115, 22))
                                            .size(11.0),
                                    );
                                }
                            }

                            if pkg.recommended {
                                ui.label(
                                    RichText::new(format!("⭐ {}", i18n.t("recommended")))
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
                                } else if is_downloaded {
                                    if ui
                                        .button(format!("🔄 {}", i18n.t("redownload")))
                                        .clicked()
                                    {
                                        downloads_to_start.push(pkg.clone());
                                    }
                                    ui.label(
                                        RichText::new(format!("✓ {}", i18n.t("downloaded")))
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

            if shown_count == 0 {
                ui.add_space(20.0);
                ui.label(RichText::new(i18n.t("no_packages_found")).italics());
            }
        } else {
            ui.label(i18n.t("loading_catalog"));
        }
    });
}
