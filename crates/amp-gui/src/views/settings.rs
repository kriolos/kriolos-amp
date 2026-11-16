use crate::i18n::I18n;
use egui::{RichText, Ui};

pub fn render_settings(
    ui: &mut Ui,
    i18n: &mut I18n,
    current_theme: &mut String,
    root_dir: &mut String,
) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.heading(RichText::new("🔧 ").size(20.0));
        ui.heading(RichText::new(i18n.t("settings")).strong().size(20.0));
    });

    ui.add_space(16.0);

    egui::Grid::new("settings_grid")
        .num_columns(2)
        .spacing([32.0, 16.0])
        .show(ui, |ui| {
            // Language selector
            ui.label(format!("{}:", i18n.t("language")));
            let mut selected_lang = i18n.current_language().to_string();
            egui::ComboBox::from_id_salt("lang_select")
                .selected_text(match selected_lang.as_str() {
                    "pt-PT" => "Português (pt-PT)",
                    _ => "English (en-US)",
                })
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(&mut selected_lang, "pt-PT".to_string(), "Português (pt-PT)")
                        .clicked()
                    {
                        i18n.set_language("pt-PT");
                    }
                    if ui
                        .selectable_value(&mut selected_lang, "en-US".to_string(), "English (en-US)")
                        .clicked()
                    {
                        i18n.set_language("en-US");
                    }
                });
            ui.end_row();

            // Theme selector
            ui.label(format!("{}:", i18n.t("theme")));
            egui::ComboBox::from_id_salt("theme_select")
                .selected_text(match current_theme.as_str() {
                    "light" => i18n.t("theme_light"),
                    _ => i18n.t("theme_dark"),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(current_theme, "dark".to_string(), i18n.t("theme_dark"));
                    ui.selectable_value(current_theme, "light".to_string(), i18n.t("theme_light"));
                });
            ui.end_row();

            // Root www dir
            ui.label(format!("{}:", i18n.t("root_dir")));
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(root_dir).desired_width(260.0));
                if ui.button(i18n.t("browse")).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        *root_dir = path.to_string_lossy().to_string();
                    }
                }
            });
            ui.end_row();
        });
}
