use egui::accesskit::{Role, Toggled};
use egui::os::OperatingSystem;
use egui::Vec2;
use egui_kittest::{
    kittest::{NodeT, Queryable as _},
    Harness,
};
use reasypub::{t, Key, Locale, MainApp};

fn new_harness() -> Harness<'static, MainApp> {
    Harness::builder()
        .with_size(Vec2::new(1280.0, 720.0))
        .with_os(OperatingSystem::Windows)
        .build_eframe(|cc| MainApp::new(cc))
}

#[test]
fn gui_smoke_layout() {
    let locale = Locale::Zh;
    let tr = |key| t(locale, key);
    let harness = new_harness();

    harness.get_by_label("Reasypub");
    harness.get_by_label(tr(Key::Subtitle));
    harness.get_by_label(tr(Key::Sections));
    harness.get_by_label(tr(Key::QuickActions));
    harness.get_by_label(tr(Key::CoverPreview));
    harness.get_by_label(tr(Key::ExportSummary));
}

#[test]
fn gui_locale_switch_to_english() {
    let mut harness = new_harness();

    harness.get_by_label(t(Locale::Zh, Key::Sections));
    harness
        .get_by(|node| {
            node.role() == Role::ComboBox
                && node.value() == Some(Locale::Zh.label().to_string())
        })
        .click();
    harness.run();
    harness.get_by_label(Locale::En.label()).click();
    harness.run();
    harness.get_by_label(t(Locale::En, Key::Sections));
}

#[test]
fn gui_panel_navigation() {
    let locale = Locale::Zh;
    let tr = |key| t(locale, key);
    let mut harness = new_harness();

    let panels = [
        (Key::PanelChapters, Key::SplitMethod),
        (Key::PanelLayout, Key::LineHeight),
        (Key::PanelFonts, Key::FontSize),
        (Key::PanelPublishInfo, Key::Publisher),
        (Key::PanelCss, Key::CustomCss),
        (Key::PanelImages, Key::AddImage),
        (Key::PanelMisc, Key::OutputFolder),
    ];

    for (panel, expected) in panels {
        harness
            .get_by_role_and_label(Role::Button, tr(panel))
            .click();
        harness.run();
        harness.get_by_label(tr(expected));
    }
}

#[test]
fn gui_quick_actions_and_windows() {
    let locale = Locale::Zh;
    let tr = |key| t(locale, key);
    let mut harness = new_harness();

    // Conversion with empty text shows error modal.
    harness.get_by_label(tr(Key::Convert)).click();
    harness.run();
    harness.get_by_label(tr(Key::ConversionFailed));
    harness.get_by_label(tr(Key::Close)).click();
    harness.run();

    // Text editor window.
    let edit_txt = harness
        .get_all_by_label(tr(Key::EditTxt))
        .find(|node| node.accesskit_node().role() == Role::Button)
        .expect("Edit TXT button");
    edit_txt.click();
    harness.run();
    harness.get_by_label(tr(Key::TextEditor));

    // Chapter editor window.
    let chapter_editor = harness
        .get_all_by_label(tr(Key::ChapterEditor))
        .find(|node| node.accesskit_node().role() == Role::Button)
        .expect("Chapter Editor button");
    chapter_editor.click();
    harness.run();
    harness.get_by_label(tr(Key::AddChapter));

    // Checkbox states.
    let gallery = harness.get_by_label(tr(Key::IncludeGallery));
    assert_eq!(gallery.accesskit_node().toggled(), Some(Toggled::True));
    gallery.click();
    harness.run();
    let gallery = harness.get_by_label(tr(Key::IncludeGallery));
    assert_eq!(gallery.accesskit_node().toggled(), Some(Toggled::False));

    let toc = harness.get_by_label(tr(Key::InsertToc));
    assert_eq!(toc.accesskit_node().toggled(), Some(Toggled::True));
    toc.click();
    harness.run();
    let toc = harness.get_by_label(tr(Key::InsertToc));
    assert_eq!(toc.accesskit_node().toggled(), Some(Toggled::False));

    let chapter_edits = harness
        .get_all_by_label(tr(Key::UseChapterEdits))
        .next()
        .expect("Use chapter edits checkbox");
    assert_eq!(chapter_edits.accesskit_node().toggled(), Some(Toggled::False));
    chapter_edits.click();
    harness.run();
    let chapter_edits = harness
        .get_all_by_label(tr(Key::UseChapterEdits))
        .next()
        .expect("Use chapter edits checkbox");
    assert_eq!(chapter_edits.accesskit_node().toggled(), Some(Toggled::True));
}

#[test]
fn gui_snapshots() {
    let mut harness = new_harness();
    let tr = |key| t(Locale::Zh, key);

    harness.run();
    harness.snapshot("home_zh");

    harness
        .get_by_role_and_label(Role::Button, tr(Key::PanelCss))
        .click();
    harness.run();
    harness.snapshot("panel_css");

    let chapter_editor = harness
        .get_all_by_label(tr(Key::ChapterEditor))
        .find(|node| node.accesskit_node().role() == Role::Button)
        .expect("Chapter Editor button");
    chapter_editor.click();
    harness.run();
    harness.snapshot("chapter_editor");
}
