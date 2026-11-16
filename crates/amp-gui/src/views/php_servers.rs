use crate::i18n::I18n;
use amp_core::models::PhpServerInstance;
use amp_core::PortChecker;
use egui::{Color32, RichText, Ui};

pub enum PhpServerAction {
    Start(String), // instance_id
    Stop(String),  // instance_id
    Add(PhpServerInstance),
    Remove(String),
}

pub fn render_php_servers(
    ui: &mut Ui,
    i18n: &I18n,
    instances: &[PhpServerInstance],
    is_running_checker: impl Fn(&str) -> bool,
    actions: &mut Vec<PhpServerAction>,
    new_server_name: &mut String,
    new_server_port: &mut u16,
    new_server_root: &mut String,
) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.heading(RichText::new("🐘 ").size(20.0));
        ui.heading(RichText::new(i18n.t("php_server_title")).strong().size(20.0));
    });

    ui.add_space(8.0);
    ui.label(
        RichText::new(i18n.t("embedded_php_info"))
            .color(Color32::from_rgb(148, 163, 184))
            .size(13.0),
    );

    ui.add_space(14.0);

    // Form to add a new PHP instance
    egui::Frame::group(ui.style())
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.label(RichText::new(i18n.t("add_php_server")).strong().size(15.0));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(format!("{}:", i18n.t("server_name")));
                ui.add(egui::TextEdit::singleline(new_server_name).hint_text("ex.: Site 2 / API"));

                ui.label(format!("{}:", i18n.t("port")));
                ui.add(egui::DragValue::new(new_server_port).range(1024..=65535));

                ui.label(format!("{}:", i18n.t("doc_root")));
                ui.add(egui::TextEdit::singleline(new_server_root).hint_text("ex.: www/api"));

                if ui.button("Criar Instância").clicked() {
                    let id = format!("php-inst-{}", new_server_port);
                    let instance = PhpServerInstance {
                        id,
                        name: if new_server_name.is_empty() {
                            format!("PHP Server (:{})", new_server_port)
                        } else {
                            new_server_name.clone()
                        },
                        host: "127.0.0.1".to_string(),
                        port: *new_server_port,
                        doc_root: if new_server_root.is_empty() {
                            "www".to_string()
                        } else {
                            new_server_root.clone()
                        },
                        php_version: None,
                        auto_start: false,
                    };
                    actions.push(PhpServerAction::Add(instance));
                    *new_server_port += 1;
                    new_server_name.clear();
                }
            });
        });

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(10.0);

    // List configured instances
    egui::ScrollArea::vertical().show(ui, |ui| {
        for inst in instances {
            let is_running = is_running_checker(&inst.id);
            let port_busy = PortChecker::is_port_in_use(inst.port);

            egui::Frame::group(ui.style())
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🐘").size(20.0));
                        ui.label(RichText::new(&inst.name).strong().size(16.0));
                        ui.label(RichText::new(format!("http://{}:{}", inst.host, inst.port)).monospace());

                        // Status Badge
                        if is_running {
                            ui.label(
                                RichText::new("● ATIVO")
                                    .color(Color32::from_rgb(34, 197, 94))
                                    .strong(),
                            );
                        } else if port_busy {
                            ui.label(
                                RichText::new("⚠ Porta Ocupada")
                                    .color(Color32::from_rgb(234, 179, 8))
                                    .size(11.0),
                            );
                        } else {
                            ui.label(
                                RichText::new("○ Parado")
                                    .color(Color32::GRAY),
                            );
                        }

                        // Action buttons
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑 Remover").clicked() {
                                actions.push(PhpServerAction::Remove(inst.id.clone()));
                            }

                            if ui.button("🌐 Abrir no Navegador").clicked() {
                                let _ = open::that(format!("http://{}:{}", inst.host, inst.port));
                            }

                            if is_running {
                                if ui.button("⏹ Parar").clicked() {
                                    actions.push(PhpServerAction::Stop(inst.id.clone()));
                                }
                            } else {
                                if ui.button(RichText::new("▶ Iniciar").strong()).clicked() {
                                    actions.push(PhpServerAction::Start(inst.id.clone()));
                                }
                            }
                        });
                    });

                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("Pasta Raiz: {} | Host: {}", inst.doc_root, inst.host))
                            .color(Color32::GRAY)
                            .size(12.0),
                    );
                });
            ui.add_space(8.0);
        }
    });
}
