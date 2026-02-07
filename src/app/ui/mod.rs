mod central;
mod dialogs;
mod preview_panel;
mod side_nav;
mod top_panel;

use super::MainApp;

pub(super) fn top_panel(app: &mut MainApp, ctx: &egui::Context) {
    top_panel::top_panel(app, ctx);
}

pub(super) fn side_nav(app: &mut MainApp, ctx: &egui::Context) {
    side_nav::side_nav(app, ctx);
}

pub(super) fn preview_panel(app: &mut MainApp, ctx: &egui::Context) {
    preview_panel::preview_panel(app, ctx);
}

pub(super) fn central_panel(app: &mut MainApp, ctx: &egui::Context) {
    central::central_panel(app, ctx);
}

pub(super) fn dialogs(app: &mut MainApp, ctx: &egui::Context) {
    dialogs::dialogs(app, ctx);
}
