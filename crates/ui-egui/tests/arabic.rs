//! Arabic language support, RTL layout, cursor placement on the first word, and translation tests.

#[path = "common/mod.rs"]
mod common;

use egui::{Align, Event, FontId, text::LayoutJob};
use gridcraft_engine::Session;
use gridcraft_engine::core::CellRef;
use gridcraft_ui_egui::{SheetApp, bidi, grid::Geo, i18n::Language, theme};
use serde_json::json;

fn harness(session: Session) -> egui_kittest::Harness<'static, SheetApp> {
    common::harness_with(common::builder().with_size(egui::vec2(1200.0, 800.0)), common::app(session), 4)
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
    if theme::craft_fonts_loaded() {
        h.run_steps(2);
    }

    assert_eq!(h.state().ui.language, Language::Ar);
    assert_eq!(h.state().ui.language.tr("Home"), "الرئيسية");
    assert_eq!(h.state().ui.language.tr("File"), "ملف");
    assert_eq!(h.state().ui.language.tr("Sheet Right-to-Left"), "ورقة من اليمين لليسار");

    let _ = h.state_mut().run("app.language.english", json!({}));
    assert_eq!(h.state().ui.language, Language::En);
    let _ = h.state_mut().run("app.language.arabic", json!({}));
    assert_eq!(h.state().ui.language, Language::Ar);
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
    assert_eq!(bidi::text_align("مرحبا بالعالم"), Align::RIGHT);
    assert_eq!(bidi::text_align("Hello World"), Align::LEFT);

    // Logical character order is preserved so clicking the first word ("مرحبا") edits indices 0..5
    // rather than jumping to the end of the sentence ("التطبيق").
    let shaped = bidi::shape_arabic("جدول مبيعات");
    assert_eq!(shaped, "جدول مبيعات");
    assert!(shaped.starts_with("جدول"));

    let s_sent = bidi::shape_arabic("مرحبا بكم في التطبيق");
    assert_eq!(s_sent, "مرحبا بكم في التطبيق");
    assert_eq!(s_sent.chars().take(5).collect::<String>(), "مرحبا");
    assert_eq!(s_sent.chars().skip(13).collect::<String>(), "التطبيق");

    let single = bidi::shape_arabic("مرحبا");
    assert_eq!(single, "مرحبا");
}

#[test]
fn clicking_first_arabic_word_on_the_right_places_caret_in_first_word() {
    let ctx = egui::Context::default();
    ctx.set_fonts(theme::font_definitions_for_language(Language::Ar));
    let mut out = ctx.run_ui(egui::RawInput::default(), |_ui| {});
    out.textures_delta.clear();

    let font = FontId::new(14.0, egui::FontFamily::Proportional);
    let text = "مرحبا بكم في التطبيق";
    let has_arabic_font = text.chars().filter(|c| !c.is_whitespace()).all(|c| ctx.fonts_mut(|f| f.has_glyph(&font, c)));
    if !has_arabic_font {
        return;
    }

    let galley = ctx.fonts_mut(|f| {
        let mut job = LayoutJob::simple(bidi::shape_arabic(text), font, egui::Color32::BLACK, f32::INFINITY);
        job.halign = bidi::text_align(text);
        f.layout_job(job)
    });

    // With `job.halign = Align::RIGHT`, x = 0.0 is the right edge of the galley where the first
    // RTL word ("مرحبا", chars 0..5) is painted. Clicking slightly left of the right edge must
    // place the cursor inside the first word (index <= 5), never at the end of the sentence.
    let cursor = galley.cursor_from_pos(egui::vec2(-10.0, galley.rect.center().y));
    assert!(
        cursor.index <= 5,
        "clicking the first word on the right placed the caret at char {}, expected <= 5 (inside \"مرحبا\")",
        cursor.index
    );
}

#[test]
fn window_restore_refreshes_textures_and_repaints() {
    let mut h = harness(blank());

    // Simulate focus loss then restore
    h.input_mut().events.push(Event::WindowFocused(false));
    h.run_steps(2);

    h.input_mut().events.push(Event::WindowFocused(true));
    h.run_steps(2);

    // Call on_window_restored directly to verify it resets font state without panic
    let ctx = h.ctx.clone();
    h.state_mut().on_window_restored(&ctx);
    h.run_steps(2);
}
