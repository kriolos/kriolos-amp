use crate::i18n::I18n;
use amp_core::service_manager::LogMessage;
use egui::{Color32, RichText, Ui};

pub fn render_logs(ui: &mut Ui, i18n: &I18n, logs: &[LogMessage], search_filter: &mut String) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.heading(RichText::new("📜 ").size(20.0));
        ui.heading(RichText::new(i18n.t("logs")).strong().size(20.0));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::TextEdit::singleline(search_filter).hint_text("Filtrar logs..."));
        });
    });

    ui.add_space(12.0);

    egui::Frame::canvas(ui.style())
        .fill(Color32::from_rgb(18, 18, 22))
        .inner_margin(10.0)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if logs.is_empty() {
                        ui.label(
                            RichText::new("Nenhum registo de log disponível no momento.")
                                .color(Color32::DARK_GRAY)
                                .italics(),
                        );
                    }

                    for entry in logs {
                        if !search_filter.is_empty()
                            && !entry
                                .message
                                .to_lowercase()
                                .contains(&search_filter.to_lowercase())
                        {
                            continue;
                        }

                        let text_color = if entry.is_error {
                            Color32::from_rgb(248, 113, 113)
                        } else {
                            Color32::from_rgb(209, 213, 219)
                        };

                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&entry.timestamp)
                                    .color(Color32::from_rgb(107, 114, 128))
                                    .monospace()
                                    .size(11.0),
                            );

                            ui.label(
                                RichText::new(format!("[{}]", entry.service_label))
                                    .color(Color32::from_rgb(96, 165, 250))
                                    .monospace()
                                    .size(11.0),
                            );

                            ui.label(
                                RichText::new(&entry.message)
                                    .color(text_color)
                                    .monospace()
                                    .size(12.0),
                            );
                        });
                    }
                });
        });
}
