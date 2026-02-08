use egui::Vec2;
use egui::accesskit::{Role, Toggled};
use egui::os::OperatingSystem;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable as _},
};
use reasypub::{Key, Locale, MainApp, t};

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
    harness.get_by_label(tr(Key::LineHeight));
}

#[test]
fn gui_locale_switch_to_english() {
    let mut harness = new_harness();

    harness.get_by_label(t(Locale::Zh, Key::Sections));
    harness
        .get_by(|node| {
            node.role() == Role::ComboBox && node.value() == Some(Locale::Zh.label().to_string())
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
        (Key::PanelLayout, Key::LineHeight),
        (Key::PanelChapters, Key::SplitMethod),
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

    harness
        .get_by_role_and_label(Role::Button, tr(Key::PanelCss))
        .click();
    harness.run();
    assert!(
        harness
            .query_by_label(tr(Key::ChapterHeaderImage))
            .is_none()
    );

    harness
        .get_by_role_and_label(Role::Button, tr(Key::PanelImages))
        .click();
    harness.run();
    harness.get_by_label(tr(Key::ChapterHeaderImage));
    harness.get_by_label(tr(Key::ChapterHeaderFullBleed));
}

#[test]
fn gui_template_i18n_labels_present_in_both_locales() {
    let mut harness = new_harness();
    let zh_template = t(Locale::Zh, Key::Template);
    let zh_classic_desc = t(Locale::Zh, Key::StyleClassicDesc);
    let en_template = t(Locale::En, Key::Template);
    let en_classic_desc = t(Locale::En, Key::StyleClassicDesc);

    harness
        .get_by_role_and_label(Role::Button, t(Locale::Zh, Key::PanelLayout))
        .click();
    harness.run();
    harness.get_by_label(zh_template);
    harness.get_by_label(zh_classic_desc);

    harness
        .get_by(|node| {
            node.role() == Role::ComboBox && node.value() == Some(Locale::Zh.label().to_string())
        })
        .click();
    harness.run();
    harness.get_by_label(Locale::En.label()).click();
    harness.run();

    harness
        .get_by_role_and_label(Role::Button, t(Locale::En, Key::PanelLayout))
        .click();
    harness.run();
    harness.get_by_label(en_template);
    harness.get_by_label(en_classic_desc);
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
        .find(|node| node.accesskit_node().role() == Role::CheckBox)
        .expect("Use chapter edits checkbox");
    let initial_toggle = chapter_edits.accesskit_node().toggled();
    chapter_edits.click();
    harness.run();
    let chapter_edits = harness
        .get_all_by_label(tr(Key::UseChapterEdits))
        .find(|node| node.accesskit_node().role() == Role::CheckBox)
        .expect("Use chapter edits checkbox");
    assert_ne!(chapter_edits.accesskit_node().toggled(), initial_toggle);

    // Chapter editor window.
    let chapter_editor = harness
        .get_all_by_label(tr(Key::ChapterEditor))
        .find(|node| node.accesskit_node().role() == Role::Button)
        .expect("Chapter Editor button");
    chapter_editor.click();
    harness.run();
    harness.get_by_label(tr(Key::AddChapter));
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
