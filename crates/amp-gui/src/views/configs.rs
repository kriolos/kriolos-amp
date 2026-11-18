use crate::i18n::I18n;
use egui::{Color32, RichText, Ui};

pub fn render_configs(ui: &mut Ui, i18n: &I18n) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.heading(RichText::new("⚙️ ").size(20.0));
        ui.heading(RichText::new(i18n.t("configs")).strong().size(20.0));
    });

    ui.add_space(16.0);

    ui.heading(RichText::new(i18n.t("php_extensions_title")).size(16.0));
    ui.add_space(8.0);

    egui::Grid::new("php_extensions_grid")
        .num_columns(4)
        .spacing([24.0, 12.0])
        .show(ui, |ui| {
            let mut ext_curl = true;
            let mut ext_pdo = true;
            let mut ext_mbstring = true;
            let mut ext_openssl = true;
            let mut ext_gd = true;
            let mut ext_zip = true;
            let mut ext_xdebug = false;
            let mut ext_intl = true;

            ui.checkbox(&mut ext_curl, "curl");
            ui.checkbox(&mut ext_pdo, "pdo_mysql");
            ui.checkbox(&mut ext_mbstring, "mbstring");
            ui.checkbox(&mut ext_openssl, "openssl");
            ui.end_row();

            ui.checkbox(&mut ext_gd, "gd");
            ui.checkbox(&mut ext_zip, "zip");
            ui.checkbox(&mut ext_xdebug, "xdebug");
            ui.checkbox(&mut ext_intl, "intl");
            ui.end_row();
        });

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(12.0);

    ui.heading(RichText::new(i18n.t("edit_configs_title")).size(16.0));
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button(i18n.t("edit_apache_conf")).clicked() {}
        if ui.button(i18n.t("edit_php_ini")).clicked() {}
        if ui.button(i18n.t("edit_mysql_ini")).clicked() {}
    });

    ui.add_space(16.0);
    ui.label(
        RichText::new(i18n.t("configs_info"))
            .color(Color32::GRAY)
            .italics(),
    );
}
