use crate::i18n::I18n;
use amp_core::models::AppConfig;
use egui::{Color32, RichText, Ui};

pub enum SettingsAction {
    Save,
    ResetDefaults,
}

pub fn render_settings(
    ui: &mut Ui,
    i18n: &mut I18n,
    config: &mut AppConfig,
    status_message: &Option<String>,
    actions: &mut Vec<SettingsAction>,
) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.heading(RichText::new("🔧 ").size(20.0));
        ui.heading(RichText::new(i18n.t("settings")).strong().size(20.0));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(format!("💾 {}", i18n.t("save_settings"))).clicked() {
                actions.push(SettingsAction::Save);
            }
            if ui.button(format!("🔄 {}", i18n.t("reset_defaults"))).clicked() {
                actions.push(SettingsAction::ResetDefaults);
            }
        });
    });

    if let Some(msg) = status_message {
        ui.add_space(6.0);
        ui.label(RichText::new(msg).color(Color32::from_rgb(34, 197, 94)).strong());
    }

    ui.add_space(14.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // --- 1. General & Preferences ---
        ui.heading(RichText::new("Geral & Aparência").size(16.0));
        ui.add_space(6.0);

        egui::Grid::new("general_settings_grid")
            .num_columns(2)
            .spacing([32.0, 12.0])
            .show(ui, |ui| {
                ui.label(format!("{}:", i18n.t("language")));
                let mut selected_lang = config.language.clone();
                egui::ComboBox::from_id_salt("settings_lang_select")
                    .selected_text(match selected_lang.as_str() {
                        "pt-PT" => "Português (pt-PT)",
                        _ => "English (en-US)",
                    })
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_value(&mut selected_lang, "pt-PT".to_string(), "Português (pt-PT)")
                            .clicked()
                        {
                            config.language = "pt-PT".to_string();
                            i18n.set_language("pt-PT");
                        }
                        if ui
                            .selectable_value(&mut selected_lang, "en-US".to_string(), "English (en-US)")
                            .clicked()
                        {
                            config.language = "en-US".to_string();
                            i18n.set_language("en-US");
                        }
                    });
                ui.end_row();

                ui.label(format!("{}:", i18n.t("theme")));
                egui::ComboBox::from_id_salt("settings_theme_select")
                    .selected_text(match config.theme.as_str() {
                        "light" => i18n.t("theme_light"),
                        _ => i18n.t("theme_dark"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut config.theme, "dark".to_string(), i18n.t("theme_dark"));
                        ui.selectable_value(&mut config.theme, "light".to_string(), i18n.t("theme_light"));
                    });
                ui.end_row();
            });

        ui.add_space(18.0);
        ui.separator();
        ui.add_space(10.0);

        // --- 2. Port Configuration ---
        ui.heading(RichText::new(i18n.t("ports_config")).size(16.0));
        ui.add_space(6.0);

        egui::Grid::new("ports_settings_grid")
            .num_columns(4)
            .spacing([24.0, 12.0])
            .show(ui, |ui| {
                ui.label("Apache HTTP:");
                ui.add(egui::DragValue::new(&mut config.apache_port).range(1..=65535));

                ui.label("Apache SSL (HTTPS):");
                ui.add(egui::DragValue::new(&mut config.apache_ssl_port).range(1..=65535));
                ui.end_row();

                ui.label("MySQL Database:");
                ui.add(egui::DragValue::new(&mut config.mysql_port).range(1..=65535));

                ui.label("MariaDB Database:");
                ui.add(egui::DragValue::new(&mut config.mariadb_port).range(1..=65535));
                ui.end_row();

                ui.label("PostgreSQL Database:");
                ui.add(egui::DragValue::new(&mut config.postgresql_port).range(1..=65535));

                ui.label("PHP-FPM:");
                ui.add(egui::DragValue::new(&mut config.php_fpm_port).range(1..=65535));
                ui.end_row();
            });

        ui.add_space(18.0);
        ui.separator();
        ui.add_space(10.0);

        // --- 3. Directory Paths ---
        ui.heading(RichText::new("Caminhos de Diretórios").size(16.0));
        ui.add_space(6.0);

        let mut www_str = config.root_www_dir.to_string_lossy().to_string();
        let mut bin_str = config.bin_dir.to_string_lossy().to_string();
        let mut logs_str = config.logs_dir.to_string_lossy().to_string();

        egui::Grid::new("paths_settings_grid")
            .num_columns(3)
            .spacing([18.0, 12.0])
            .show(ui, |ui| {
                ui.label(format!("{}:", i18n.t("root_dir")));
                if ui.add(egui::TextEdit::singleline(&mut www_str).desired_width(300.0)).changed() {
                    config.root_www_dir = www_str.clone().into();
                }
                if ui.button(i18n.t("browse")).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        config.root_www_dir = path;
                    }
                }
                ui.end_row();

                ui.label(format!("{}:", i18n.t("bin_dir")));
                if ui.add(egui::TextEdit::singleline(&mut bin_str).desired_width(300.0)).changed() {
                    config.bin_dir = bin_str.clone().into();
                }
                if ui.button(i18n.t("browse")).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        config.bin_dir = path;
                    }
                }
                ui.end_row();

                ui.label(format!("{}:", i18n.t("logs_dir")));
                if ui.add(egui::TextEdit::singleline(&mut logs_str).desired_width(300.0)).changed() {
                    config.logs_dir = logs_str.clone().into();
                }
                if ui.button(i18n.t("browse")).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        config.logs_dir = path;
                    }
                }
                ui.end_row();
            });

        ui.add_space(18.0);
        ui.separator();
        ui.add_space(10.0);

        // --- 4. Remote Catalog URL ---
        ui.heading(RichText::new(i18n.t("remote_manifest_url")).size(16.0));
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::singleline(&mut config.remote_manifest_url)
                .desired_width(550.0),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("💡 O catálogo remoto permite carregar novas versões de PHP, Apache e Bancos sem precisar recompilar ou atualizar o programa.")
                .color(Color32::GRAY)
                .size(12.0),
        );
    });
}
