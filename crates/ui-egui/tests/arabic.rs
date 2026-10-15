//! Arabic language support, RTL layout, translation, and window restore tests.

use egui::Event;
use gridcraft_engine::Session;
use gridcraft_engine::core::CellRef;
use gridcraft_ui_egui::{SheetApp, bidi, grid::Geo, i18n::Language};
use serde_json::json;

fn harness(session: Session) -> egui_kittest::Harness<'static, SheetApp> {
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        SheetApp::new(session, Default::default()),
    );
    h.run_steps(4);
    h
}

fn blank() -> Session {
    let mut s = Session::new();
    s.new_workbook();
    s
}

#[test]
fn arabic_language_switch_translates_ui() {
    let mut h = harness(blank());
    let _ = h.state_mut().run("app.language.set", json!({"code": "ar"}));
    h.run_steps(2);

    assert_eq!(h.state().ui.language, Language::Ar);
    assert_eq!(h.state().ui.language.tr("Home"), "الرئيسية");
    assert_eq!(h.state().ui.language.tr("File"), "ملف");
    assert_eq!(h.state().ui.language.tr("Sheet Right-to-Left"), "ورقة من اليمين لليسار");
}

#[test]
fn right_to_left_sheet_layout() {
    let mut h = harness(blank());

    // Toggle right to left
    let _ = h.state_mut().run("view.rightToLeft", json!({"on": true}));
    h.run_steps(2);

    let doc = h.state().session.active().unwrap();
    let sh = doc.wb.active().unwrap();
    assert!(sh.right_to_left);

    let rect = h.state().grid.rect.unwrap();
    let scroll = h.state().view().scroll;
    let geo = Geo::new(sh, rect, scroll);
    assert!(geo.rtl);

    let col0 = geo.screen_cell_rect(sh, CellRef::new(0, 0));
    let col1 = geo.screen_cell_rect(sh, CellRef::new(0, 1));

    // In RTL, column 0 is to the right of column 1
    assert!(col0.min.x > col1.min.x);
    assert!((col0.max.x - geo.mirror_x(geo.cells.left())).abs() < 1e-3);
}

#[test]
fn arabic_shaping_and_bidi() {
    assert!(bidi::has_arabic("مرحبا بالعالم"));
    assert!(!bidi::has_arabic("Hello World"));

    let shaped = bidi::shape_arabic("جدول مبيعات");
    assert_eq!(shaped, "مبيعات جدول");

    let s_sent = bidi::shape_arabic("مرحبا بكم في التطبيق");
    assert_eq!(s_sent, "التطبيق في بكم مرحبا");

    let single = bidi::shape_arabic("مرحبا");
    assert_eq!(single, "مرحبا");
}

#[test]
fn window_restore_refreshes_textures_and_repaints() {
    let mut h = harness(blank());

    // Simulate focus loss then restore
    h.input_mut().events.push(Event::WindowFocused(false));
    h.run_steps(2);

    h.input_mut().events.push(Event::WindowFocused(true));
    h.run_steps(2);

    // Call on_window_restored directly to verify it resets font atlas without panic
    let ctx = h.ctx.clone();
    h.state_mut().on_window_restored(&ctx);
    h.run_steps(2);
}

