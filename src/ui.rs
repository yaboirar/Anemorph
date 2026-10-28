use super::*;
use crate::i18n::tr;
mod views;
use egui::{
    Align2, FontId, Id, Layout, Pos2, Rect, Response, RichText, Sense, Stroke, StrokeKind, Ui,
    Vec2, ViewportCommand,
};

#[cfg(test)]
mod title_bar_tests {
    use super::*;

    fn drag_commands(start: Pos2) -> Vec<ViewportCommand> {
        let ctx = Context::default();
        let cc = eframe::CreationContext::_new_kittest(ctx.clone());
        let mut app = App::new(&cc, Some(std::env::temp_dir().join("anemorph-drag-test")));
        let mut frame = |events: Vec<egui::Event>| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000., 700.))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::TopBottomPanel::top("navigation")
                        .exact_height(52.)
                        .frame(egui::Frame::NONE)
                        .show(ctx, |ui| app.nav(ui));
                },
            )
        };
        let _ = frame(vec![]);
        let pressed = frame(vec![
            egui::Event::PointerMoved(start),
            egui::Event::PointerButton {
                pos: start,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]);
        let output = frame(vec![egui::Event::PointerMoved(start + Vec2::new(16., 0.))]);
        let mut commands = pressed.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .clone();
        commands.extend(
            output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .clone(),
        );
        commands
    }

    #[test]
    fn stationary_navigation_hover_finishes_animating() {
        for x in (410..=590).step_by(6) {
            let ctx = Context::default();
            let cc = eframe::CreationContext::_new_kittest(ctx.clone());
            let mut app = App::new(
                &cc,
                Some(std::env::temp_dir().join("anemorph-hover-settle")),
            );
            app.transition = Instant::now() - Duration::from_secs(1);
            let mut delay = Duration::ZERO;
            for frame in 0..90 {
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000., 700.))),
                        time: Some(frame as f64 / 60.),
                        events: if frame == 1 {
                            vec![egui::Event::PointerMoved(Pos2::new(x as f32, 26.))]
                        } else {
                            vec![]
                        },
                        ..Default::default()
                    },
                    |ctx| {
                        egui::TopBottomPanel::top("navigation")
                            .exact_height(52.)
                            .show(ctx, |ui| app.nav(ui));
                    },
                );
                delay = output.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
            }
            assert!(
                delay > Duration::from_millis(100),
                "hover at x={x} keeps repainting: {delay:?}"
            );
        }
    }
    #[test]
    fn title_bar_drags_on_both_sides_and_tabs() {
        for point in [
            Pos2::new(100., 20.),
            Pos2::new(750., 20.),
            Pos2::new(450., 25.),
        ] {
            assert!(
                drag_commands(point)
                    .iter()
                    .any(|c| matches!(c, ViewportCommand::StartDrag)),
                "{point:?}"
            );
        }
        assert!(
            !drag_commands(Pos2::new(974., 15.))
                .iter()
                .any(|c| matches!(c, ViewportCommand::StartDrag))
        );
    }
    #[test]
    fn brand_follows_window_size_and_ignores_interface_zoom() {
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(1080., 740.));
        let a = brand_rect(viewport, 1.);
        for zoom in [0.75, 1.5] {
            let b = brand_rect(
                Rect::from_min_size(Pos2::ZERO, viewport.size() / zoom),
                zoom,
            );
            assert!((a.width() - b.width() * zoom).abs() < 0.01);
            assert!((a.left() - b.left() * zoom).abs() < 0.01);
        }
        let small = brand_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(480., 360.)), 1.);
        assert!(small.width() < a.width());
        assert_eq!(DEFAULT_ACCENT, Color32::from_rgb(53, 150, 151));
    }
    #[test]
    fn menu_buttons_stay_adjacent_when_labels_expand() {
        let ctx = Context::default();
        let cc = eframe::CreationContext::_new_kittest(ctx.clone());
        let mut app = App::new(&cc, Some(std::env::temp_dir().join("anemorph-menu-test")));
        app.settings.animations = false;
        for step in 0..3 {
            let events = if step == 0 {
                vec![]
            } else {
                vec![egui::Event::PointerMoved(app.nav_rects[2].center())]
            };
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000., 700.))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::TopBottomPanel::top("navigation")
                        .exact_height(52.)
                        .frame(egui::Frame::NONE)
                        .show(ctx, |ui| app.nav(ui));
                },
            );
            assert!((app.nav_rects[1].left() - app.nav_rects[0].right() - 6.).abs() < 0.01);
            assert!((app.nav_rects[2].left() - app.nav_rects[1].right() - 6.).abs() < 0.01);
        }
        assert_eq!(app.nav_rects[2].width(), 36.);
        assert_eq!(app.nav_rects[0].width(), 36.);
        app.navigate(Page::Configuration, &ctx);
        assert_eq!(app.tab_target, 2.);
    }
    #[test]
    fn header_checkbox_toggles_all_files_without_changing_readiness() {
        fn render(ctx: &Context, app: &mut App, events: Vec<egui::Event>) {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(800., 500.))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.table(ui, app.palette()));
                },
            );
        }
        let ctx = Context::default();
        let cc = eframe::CreationContext::_new_kittest(ctx.clone());
        let mut app = App::new(&cc, Some(std::env::temp_dir().join("anemorph-header-test")));
        let root = std::env::temp_dir().join("anemorph-header-files");
        app.rows = engine::plan(
            &root,
            "Show",
            &[root.join("E01.mkv"), root.join("Movie.mkv")],
            &Settings::default(),
        )
        .unwrap();
        render(&ctx, &mut app, vec![]);
        let pos = ctx
            .read_response(Id::new("include_all_header"))
            .unwrap()
            .rect
            .center();
        for expected in [false, true] {
            for pressed in [true, false] {
                render(
                    &ctx,
                    &mut app,
                    vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Modifiers::NONE,
                        },
                    ],
                );
            }
            assert_eq!(app.rows[0].included, expected);
            assert_eq!(app.rows[1].included, expected);
        }
    }
    #[test]
    fn folder_placeholder_is_vertically_centered() {
        let ctx = Context::default();
        let cc = eframe::CreationContext::_new_kittest(ctx.clone());
        let app = App::new(&cc, Some(std::env::temp_dir().join("anemorph-field-test")));
        let rect = Rect::from_min_size(Pos2::new(20., 20.), Vec2::new(400., 40.));
        let mut text = String::new();
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(800., 500.))),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let _ = App::field(ui, rect, &mut text, "folder", "Browse", app.palette());
                });
            },
        );
        let center = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(t) if t.galley.job.text == "Paste a folder path" => {
                    Some(t.pos.y + t.galley.size().y / 2.)
                }
                _ => None,
            })
            .expect("placeholder should be drawn");
        assert!((center - rect.center().y).abs() <= 1., "center={center}");
    }
    #[test]
    fn brand_opens_an_about_popup_and_escape_closes_it() {
        let ctx = Context::default();
        let cc = eframe::CreationContext::_new_kittest(ctx.clone());
        let mut app = App::new(&cc, Some(std::env::temp_dir().join("anemorph-about-test")));
        fn render(ctx: &Context, app: &mut App, events: Vec<egui::Event>) {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000., 700.))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::TopBottomPanel::top("navigation")
                        .exact_height(52.)
                        .frame(egui::Frame::NONE)
                        .show(ctx, |ui| app.nav(ui));
                    egui::TopBottomPanel::bottom("footer")
                        .exact_height(60.)
                        .show(ctx, |ui| app.footer(ui));
                    app.about_dialog(ctx);
                },
            );
        }
        render(&ctx, &mut app, vec![]);
        let point = ctx
            .read_response(Id::new("about_brand"))
            .unwrap()
            .rect
            .center();
        for pressed in [true, false] {
            render(
                &ctx,
                &mut app,
                vec![
                    egui::Event::PointerMoved(point),
                    egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        assert!(app.about_open);
        assert!(app.page == Page::Home);
        render(&ctx, &mut app, vec![]);
        let close = ctx
            .read_response(Id::new("about_close"))
            .unwrap()
            .rect
            .center();
        for pressed in [true, false] {
            render(
                &ctx,
                &mut app,
                vec![
                    egui::Event::PointerMoved(close),
                    egui::Event::PointerButton {
                        pos: close,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        assert!(!app.about_open);
        app.about_open = true;
        render(&ctx, &mut app, vec![]);
        render(
            &ctx,
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
        assert!(!app.about_open);
    }
    #[test]
    fn right_click_marks_a_skipped_file_as_movie_without_renaming_it() {
        let ctx = Context::default();
        let cc = eframe::CreationContext::_new_kittest(ctx.clone());
        let root =
            std::env::temp_dir().join(format!("anemorph-context-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let file = root.join("Unmarked film.mp4");
        std::fs::write(&file, b"fixture").unwrap();
        let mut app = App::new(&cc, Some(root.join("app")));
        app.data = root.join("data");
        app.root = Some(root.clone());
        app.series = "Show".into();
        app.media_markers = std::sync::Arc::new(engine::MediaMarkers::new());
        app.files = std::sync::Arc::new(vec![file.clone()]);
        app.rebuild();
        assert!(!app.rows[0].ready);
        fn render(ctx: &Context, app: &mut App, events: Vec<egui::Event>) -> egui::FullOutput {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(900., 500.))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.table(ui, app.palette()));
                },
            )
        }
        let _ = render(&ctx, &mut app, vec![]);
        let row = ctx
            .read_response(Id::new(("file_row", &file)))
            .unwrap()
            .rect
            .center();
        for pressed in [true, false] {
            let _ = render(
                &ctx,
                &mut app,
                vec![
                    egui::Event::PointerMoved(row),
                    egui::Event::PointerButton {
                        pos: row,
                        button: egui::PointerButton::Secondary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        let menu = render(&ctx, &mut app, vec![]);
        let button = menu
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.job.text == "Mark as movie" => {
                    Some(t.pos + Vec2::new(10., t.galley.size().y / 2.))
                }
                _ => None,
            })
            .expect("right-click movie action");
        for pressed in [true, false] {
            let _ = render(
                &ctx,
                &mut app,
                vec![
                    egui::Event::PointerMoved(button),
                    egui::Event::PointerButton {
                        pos: button,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        assert_eq!(app.rows[0].kind, engine::MediaKind::Movie);
        assert_eq!(app.rows[0].output, "Unmarked film Movie.mp4");
        assert!(file.exists());
        assert!(!root.join("Unmarked film Movie.mp4").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn header_dividers_resize_columns_and_status_text_aligns() {
        let ctx = Context::default();
        let cc = eframe::CreationContext::_new_kittest(ctx.clone());
        let root =
            std::env::temp_dir().join(format!("anemorph-column-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let file = root.join("Show S01E19.mp4");
        std::fs::write(&file, b"fixture").unwrap();
        let mut app = App::new(&cc, Some(root.join("app")));
        app.data = root.join("data");
        app.root = Some(root.clone());
        app.series = "Show".into();
        app.files = std::sync::Arc::new(vec![file]);
        app.rebuild();
        fn render(
            ctx: &Context,
            app: &mut App,
            width: f32,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 500.))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.table(ui, app.palette()));
                },
            )
        }
        fn drag(ctx: &Context, app: &mut App, divider: usize, delta: f32) {
            let start = ctx
                .read_response(Id::new(("column_divider", divider)))
                .unwrap()
                .rect
                .center();
            let end = start + Vec2::new(delta, 0.);
            let _ = render(
                ctx,
                app,
                1000.,
                vec![
                    egui::Event::PointerMoved(start),
                    egui::Event::PointerButton {
                        pos: start,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
            let _ = render(ctx, app, 1000., vec![egui::Event::PointerMoved(end)]);
            let _ = render(
                ctx,
                app,
                1000.,
                vec![egui::Event::PointerButton {
                    pos: end,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                }],
            );
        }
        let _ = render(&ctx, &mut app, 1000., vec![]);
        let width = 984.;
        let before = FileColumns::new(
            width,
            app.settings.table_name_ratio,
            app.settings.table_status_width,
        );
        drag(&ctx, &mut app, 0, 80.);
        let after = FileColumns::new(
            width,
            app.settings.table_name_ratio,
            app.settings.table_status_width,
        );
        assert!((after.widths[0] - before.widths[0] - 80.).abs() < 0.5);
        assert!((after.widths[2] - before.widths[2]).abs() < 0.5);
        drag(&ctx, &mut app, 1, -40.);
        let resized = FileColumns::new(
            width,
            app.settings.table_name_ratio,
            app.settings.table_status_width,
        );
        assert!((resized.widths[2] - after.widths[2] - 40.).abs() < 0.5);
        assert!((resized.widths[0] - after.widths[0]).abs() < 0.5);
        let output = render(&ctx, &mut app, 1000., vec![]);
        let position = |text: &str| {
            output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(t) if t.galley.job.text == text => Some(t.pos.x),
                    _ => None,
                })
                .unwrap()
        };
        assert!((position("Status") - position("No change")).abs() < 0.1);
        assert!(!app.rows[0].included && !app.rows[0].highlighted);
        let saved = Settings::load(&app.data.join("settings.json"));
        assert!((saved.table_name_ratio - app.settings.table_name_ratio).abs() < 0.001);
        assert!((saved.table_status_width - app.settings.table_status_width).abs() < 0.1);
        let original_ratio = app.settings.table_name_ratio;
        let _ = render(&ctx, &mut app, 480., vec![]);
        assert_eq!(app.settings.table_name_ratio, original_ratio);
        for width in [0., 100., 480., 700., 1000., 1920.] {
            let columns = FileColumns::new(width, 0.95, 10000.);
            assert!(columns.widths.iter().all(|v| v.is_finite() && *v >= 0.));
            assert!((columns.widths.iter().sum::<f32>() - (width - 52.).max(0.)).abs() < 0.01);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn right_click_episode_action_opens_numbering_and_apply_updates_only_preview() {
        let ctx = Context::default();
        let cc = eframe::CreationContext::_new_kittest(ctx.clone());
        let root =
            std::env::temp_dir().join(format!("anemorph-episode-context-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let file = root.join("Unnumbered Movie clip.mp4");
        std::fs::write(&file, b"fixture").unwrap();
        let mut app = App::new(&cc, Some(root.join("app")));
        app.data = root.join("data");
        app.root = Some(root.clone());
        app.series = "Show".into();
        app.settings.auto_detect = false;
        app.media_markers = std::sync::Arc::new(engine::MediaMarkers::new());
        app.files = std::sync::Arc::new(vec![file.clone()]);
        app.rebuild();
        fn render(ctx: &Context, app: &mut App, events: Vec<egui::Event>) -> egui::FullOutput {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(900., 500.))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.table(ui, app.palette()));
                    app.episode_marker_dialog(ctx);
                },
            )
        }
        let _ = render(&ctx, &mut app, vec![]);
        let row = ctx
            .read_response(Id::new(("file_row", &file)))
            .unwrap()
            .rect
            .center();
        for pressed in [true, false] {
            let _ = render(
                &ctx,
                &mut app,
                vec![
                    egui::Event::PointerMoved(row),
                    egui::Event::PointerButton {
                        pos: row,
                        button: egui::PointerButton::Secondary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        let output = render(&ctx, &mut app, vec![]);
        let point = |output: &egui::FullOutput, text: &str| {
            output
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.job.text == text => {
                        Some(t.pos + Vec2::new(10., t.galley.size().y / 2.))
                    }
                    _ => None,
                })
                .unwrap()
        };
        let button = point(&output, "Mark as episode");
        for pressed in [true, false] {
            let _ = render(
                &ctx,
                &mut app,
                vec![
                    egui::Event::PointerMoved(button),
                    egui::Event::PointerButton {
                        pos: button,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        let dialog = app.episode_dialog.as_mut().unwrap();
        dialog.season = 2;
        dialog.first_episode = 23;
        let output = render(&ctx, &mut app, vec![]);
        let button = point(&output, "Apply");
        for pressed in [true, false] {
            let _ = render(
                &ctx,
                &mut app,
                vec![
                    egui::Event::PointerMoved(button),
                    egui::Event::PointerButton {
                        pos: button,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        assert!(app.episode_dialog.is_none());
        assert_eq!(app.rows[0].kind, engine::MediaKind::Episode);
        assert_eq!(app.rows[0].output, "Show S02E23.mp4");
        assert!(file.exists());
        assert!(!root.join("Show S02E23.mp4").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn table_actions_are_icons_on_the_right() {
        let ctx = Context::default();
        let cc = eframe::CreationContext::_new_kittest(ctx.clone());
        let mut app = App::new(
            &cc,
            Some(std::env::temp_dir().join("anemorph-actions-test")),
        );
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000., 700.))),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.home(ui));
            },
        );
        let filter = ctx
            .read_response(Id::new("container_filters"))
            .unwrap()
            .rect;
        let refresh = ctx.read_response(Id::new("refresh_files")).unwrap().rect;
        assert!(filter.right() > 950.);
        assert!((filter.left() - refresh.right() - 8.).abs() < 0.01);
        assert!(output.shapes.iter().all(|s| !matches!(&s.shape, egui::Shape::Text(t) if t.galley.job.text == "Filters" || t.galley.job.text == "Refresh")));
    }
}
#[derive(Clone, Copy)]
struct Palette {
    bg: Color32,
    title_bar: Color32,
    surface: Color32,
    input: Color32,
    text: Color32,
    muted: Color32,
    line: Color32,
    button: Color32,
    accent: Color32,
    selected: Color32,
}
impl App {
    fn palette(&self) -> Palette {
        let d = self.settings.dark_mode;
        let accent = self
            .settings
            .accent_hex
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(DEFAULT_ACCENT);
        let surface = Color32::from_gray(if d { 29 } else { 238 });
        Palette {
            bg: Color32::from_gray(if d { 20 } else { 247 }),
            title_bar: Color32::from_gray(if d { 32 } else { 238 }),
            surface,
            input: Color32::from_gray(if d { 24 } else { 255 }),
            text: Color32::from_gray(if d { 237 } else { 32 }),
            muted: Color32::from_gray(if d { 160 } else { 98 }),
            line: Color32::from_gray(if d { 54 } else { 211 }),
            button: Color32::from_gray(if d { 43 } else { 231 }),
            accent,
            selected: blend(accent, surface, 0.23),
        }
    }
    pub(super) fn theme(&self, ctx: &Context) {
        let p = self.palette();
        let mut v = if self.settings.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        v.panel_fill = p.bg;
        v.window_fill = p.surface;
        v.extreme_bg_color = p.input;
        v.faint_bg_color = p.surface;
        v.override_text_color = Some(p.text);
        v.selection.bg_fill = p.selected;
        v.selection.stroke = Stroke::new(1.0_f32, p.accent);
        v.widgets.noninteractive.bg_fill = p.surface;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.line);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.text);
        v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.text);
        v.widgets.inactive.bg_fill = p.button;
        v.widgets.inactive.weak_bg_fill = p.button;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.line);
        v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, p.text);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, blend(p.accent, p.line, 0.35));
        v.widgets.active.bg_stroke = Stroke::new(1.0_f32, p.accent);
        v.widgets.hovered.bg_fill = blend(p.text, p.button, 0.08);
        v.widgets.active.bg_fill = blend(p.text, p.button, 0.12);
        v.widgets.active.fg_stroke = Stroke::new(1.0_f32, p.text);
        v.widgets.inactive.corner_radius = 7.into();
        v.widgets.hovered.corner_radius = 7.into();
        v.widgets.active.corner_radius = 7.into();
        v.widgets.noninteractive.corner_radius = 7.into();
        v.widgets.open.corner_radius = 7.into();
        v.window_corner_radius = 9.into();
        v.menu_corner_radius = 7.into();
        ctx.set_visuals(v);
        let mut style = (*ctx.style()).clone();
        style.spacing.button_padding = Vec2::new(12., 8.);
        style.spacing.item_spacing = Vec2::new(10., 10.);
        style.animation_time = 0.12;
        ctx.set_style(style);
    }
    fn nav(&mut self, ui: &mut Ui) {
        self.paint_brand(ui);
        let p = self.palette();
        let rect = ui.max_rect();
        let pages = [
            ("Home", Page::Home),
            ("Settings", Page::Settings),
            ("Configuration", Page::Configuration),
        ];
        let pointer = ui.ctx().input(|i| i.pointer.hover_pos());
        let hover = pointer.and_then(|pos| self.nav_rects.iter().position(|r| r.contains(pos)));
        let font = FontId::proportional(13.);
        let mut amounts = [0.; 3];
        let mut widths = [36.; 3];
        for (i, (label, _)) in pages.iter().enumerate() {
            let label = tr(ui.ctx(), label);
            let expanded = hover == Some(i)
                || self
                    .smoke
                    .as_ref()
                    .is_some_and(|s| (i == 1 && s.stage == 3) || (i == 2 && s.stage == 5));
            let t = if self.settings.animations {
                ui.ctx()
                    .animate_bool_with_time(Id::new(("nav-label", i)), expanded, 0.18)
            } else {
                0.
            };
            amounts[i] = 1. - (1. - t).powi(3);
            let text_width = ui
                .painter()
                .layout_no_wrap(label.to_string(), font.clone(), p.text)
                .size()
                .x;
            widths[i] += (text_width + 8.) * amounts[i];
        }
        let nav = Rect::from_center_size(
            rect.center(),
            Vec2::new(widths.iter().sum::<f32>() + 12., 36.),
        );
        let mut x = nav.left();
        for (i, width) in widths.iter().enumerate() {
            self.nav_rects[i] =
                Rect::from_min_size(Pos2::new(x, nav.top()), Vec2::new(*width, 36.));
            x += width + 6.;
        }
        let progress = self.progress();
        let ease = 1. - (1. - progress).powi(3);
        self.tab_x = self.tab_from + (self.tab_target - self.tab_from) * ease;
        let from = self.tab_x.floor().clamp(0., 2.) as usize;
        let to = (from + 1).min(2);
        let t = self.tab_x - from as f32;
        let pill = Rect::from_min_max(
            self.nav_rects[from].min.lerp(self.nav_rects[to].min, t),
            self.nav_rects[from].max.lerp(self.nav_rects[to].max, t),
        );
        ui.painter().rect_filled(pill, 6, p.button);
        for (i, (label, page)) in pages.into_iter().enumerate() {
            let label = tr(ui.ctx(), label);
            let r = self.nav_rects[i];
            let response = ui.interact(r, Id::new(("navigation", i)), Sense::click_and_drag());
            if response.drag_started_by(egui::PointerButton::Primary) {
                self.start_window_drag(ui.ctx());
            }
            let ink = if self.page == page { p.text } else { p.muted };
            if response.hovered() && self.page != page {
                ui.painter().rect_filled(r, 6, blend(p.text, p.bg, 0.05));
            }
            nav_icon(ui, Pos2::new(r.left() + 18., r.center().y), page, ink);
            if amounts[i] > 0.001 {
                let clip = Rect::from_min_max(
                    Pos2::new(r.left() + 31., r.top()),
                    r.max - Vec2::new(10., 0.),
                );
                ui.painter()
                    .with_clip_rect(clip.intersect(ui.clip_rect()))
                    .text(
                        Pos2::new(r.left() + 32. - 4. * (1. - amounts[i]), r.center().y),
                        Align2::LEFT_CENTER,
                        label,
                        font.clone(),
                        ink.gamma_multiply(amounts[i]),
                    );
            }
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::SelectableLabel, true, label)
            });
            if response.clicked() {
                self.navigate(page, ui.ctx());
            }
        }
        let title = Rect::from_min_max(rect.min + Vec2::new(5., 5.), rect.max - Vec2::new(5., 0.));
        let controls = Rect::from_min_size(
            Pos2::new(rect.right() - 132., rect.top()),
            Vec2::new(132., 34.),
        );
        let drag = ui.ctx().input(|i| {
            i.pointer.interact_pos().is_some_and(|pos| {
                title.contains(pos)
                    && !controls.contains(pos)
                    && !self.nav_rects.iter().any(|r| r.contains(pos))
            })
        });
        if drag && !self.about_open && self.episode_dialog.is_none() {
            if ui.ctx().input(|i| {
                i.pointer
                    .button_double_clicked(egui::PointerButton::Primary)
            }) {
                let maximized = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::Maximized(!maximized));
            } else if ui.ctx().input(|i| i.pointer.primary_pressed()) {
                self.start_window_drag(ui.ctx());
            }
        }
        let maximized = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
        for i in 0..3 {
            let r = Rect::from_min_size(
                Pos2::new(rect.right() - 132. + i as f32 * 44., rect.top()),
                Vec2::new(44., 34.),
            );
            let response = ui.interact(r, Id::new(("window_control", i)), Sense::click());
            let hovered = response.hovered();
            if hovered {
                ui.painter().rect_filled(
                    r,
                    0,
                    if i == 2 {
                        Color32::from_rgb(196, 43, 49)
                    } else {
                        p.button
                    },
                );
            }
            let ink = if hovered && i == 2 {
                Color32::WHITE
            } else {
                p.text
            };
            let stroke = Stroke::new(1.2_f32, ink);
            let c = r.center();
            let icon = Rect::from_center_size(c, Vec2::splat(10.));
            match i {
                0 => {
                    ui.painter()
                        .line_segment([c - Vec2::new(5., 0.), c + Vec2::new(5., 0.)], stroke);
                }
                1 if maximized => {
                    let back = Rect::from_min_size(c + Vec2::new(-2., -5.), Vec2::splat(7.));
                    ui.painter()
                        .line_segment([back.left_top(), back.right_top()], stroke);
                    ui.painter()
                        .line_segment([back.right_top(), back.right_bottom()], stroke);
                    let front = Rect::from_min_size(c + Vec2::new(-5., -2.), Vec2::splat(7.));
                    ui.painter()
                        .rect_stroke(front, 0, stroke, StrokeKind::Inside);
                }
                1 => {
                    ui.painter()
                        .rect_stroke(icon, 0, stroke, StrokeKind::Inside);
                }
                _ => {
                    ui.painter()
                        .line_segment([icon.left_top(), icon.right_bottom()], stroke);
                    ui.painter()
                        .line_segment([icon.right_top(), icon.left_bottom()], stroke);
                }
            }
            let response = response.on_hover_text(match i {
                0 => "Minimize",
                1 if maximized => "Restore",
                1 => "Maximize",
                _ => "Close",
            });
            if response.clicked() {
                match i {
                    0 => ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true)),
                    1 => ui
                        .ctx()
                        .send_viewport_cmd(ViewportCommand::Maximized(!maximized)),
                    _ => ui.ctx().send_viewport_cmd(ViewportCommand::Close),
                }
            }
        }
        if progress < 1. {
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }
    }
    fn paint_brand(&self, ui: &Ui) {
        let zoom = ui.ctx().zoom_factor();
        let viewport = ui.ctx().input(|i| i.content_rect());
        ui.painter().image(
            self.logo.id(),
            brand_rect(viewport, zoom),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
            Color32::from_white_alpha(if self.settings.dark_mode { 6 } else { 8 }),
        );
    }
    fn brand_label(&mut self, ui: &mut Ui) {
        let p = self.palette();
        let rect = ui.max_rect();
        let zoom = ui.ctx().zoom_factor();
        let native_width = ui.ctx().content_rect().width() * zoom;
        let size = (native_width / 60.).clamp(14., 19.) / zoom;
        let brand_font = FontId::new(size, egui::FontFamily::Name("brand".into()));
        let brand_galley =
            ui.painter()
                .layout_no_wrap("Anemorph".into(), brand_font.clone(), p.text);
        let brand_size = brand_galley.size();
        let brand_rect = Rect::from_min_size(
            rect.left_top() + Vec2::new(8. / zoom, 4. / zoom),
            Vec2::new(brand_size.x + 8. / zoom, 36. / zoom),
        );
        let brand = ui.interact(brand_rect, Id::new("about_brand"), Sense::click());
        if brand.clicked() {
            self.about_open = true;
        }
        if brand.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        brand.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "About Anemorph")
        });
        let brand_hovered = brand.hovered() || self.smoke.as_ref().is_some_and(|s| s.stage == 1);
        let glow = if self.settings.animations {
            ui.ctx()
                .animate_bool_with_time(Id::new("brand_glow"), brand_hovered, 0.18)
        } else if brand_hovered {
            1.
        } else {
            0.
        };
        let glow = 1. - (1. - glow).powi(3);
        let origin = rect.left_top() + Vec2::new(10. / zoom, 15. / zoom - brand_size.y / 2.);
        // Reuse the same glyph layout for a small fixed halo. No blur surfaces
        // or idle timer: only hover transitions request additional frames.
        if glow > 0.001 {
            for (radius, opacity) in [(3.5, 0.025), (1.5, 0.065)] {
                for step in 0..8 {
                    let offset =
                        Vec2::angled(step as f32 * std::f32::consts::TAU / 8.) * (radius / zoom);
                    ui.painter().galley_with_override_text_color(
                        origin + offset,
                        brand_galley.clone(),
                        p.accent.gamma_multiply(glow * opacity),
                    );
                }
            }
        }
        ui.painter().galley_with_override_text_color(
            origin,
            brand_galley,
            blend(p.accent, p.text, glow),
        );
        ui.painter().text(
            rect.left_top() + Vec2::new(10. / zoom, 31. / zoom),
            Align2::LEFT_CENTER,
            concat!("v", env!("CARGO_PKG_VERSION")),
            FontId::proportional(10. / zoom),
            p.muted,
        );
    }
    fn footer(&mut self, ui: &mut Ui) {
        let p = self.palette();
        let rect = ui.max_rect();
        self.brand_label(ui);
        let (_, count, _) = self.row_counts();
        let y = rect.center().y - 18.;
        let rename = Rect::from_min_size(Pos2::new(rect.right() - 140., y), Vec2::new(136., 36.));
        let undo = rename.translate(Vec2::new(-146., 0.));
        let text = Rect::from_min_max(
            rect.min + Vec2::new(8., 10.),
            Pos2::new(undo.left() - 12., rect.bottom() - 8.),
        );
        if self.error && text.width() > 170. {
            paint_left(
                ui,
                Rect::from_min_max(text.min + Vec2::new(130., 0.), text.max),
                &self.status,
                Color32::from_rgb(242, 158, 152),
                12.,
            );
        }
        if ui
            .add_enabled_ui(!self.busy && self.undo_available, |ui| {
                ui.put(undo, egui::Button::new(tr(ui.ctx(), "Undo last batch")))
            })
            .inner
            .clicked()
        {
            self.execute(true, ui.ctx())
        }
        if ui
            .add_enabled_ui(!self.busy && !self.dirty && count > 0, |ui| {
                ui.put(
                    rename,
                    egui::Button::new(
                        RichText::new(
                            tr(ui.ctx(), "Rename {count} files")
                                .replace("{count}", &count.to_string()),
                        )
                        .color(on_color(p.accent)),
                    )
                    .fill(p.accent),
                )
            })
            .inner
            .clicked()
        {
            self.execute(false, ui.ctx())
        }
    }
    fn field(
        ui: &mut Ui,
        rect: Rect,
        text: &mut String,
        id: &str,
        kind: &str,
        p: Palette,
    ) -> (Response, bool) {
        ui.painter().rect_filled(rect, 7, p.input);
        ui.painter()
            .rect_stroke(rect, 7, Stroke::new(1.0_f32, p.line), StrokeKind::Inside);
        let bw = if kind == "Browse" {
            (ui.painter()
                .layout_no_wrap(
                    tr(ui.ctx(), "Browse").into(),
                    FontId::proportional(13.),
                    p.text,
                )
                .size()
                .x
                + 24.)
                .max(68.)
                .min(rect.width() * 0.45)
        } else {
            36.
        };
        let input = Rect::from_min_max(
            rect.min + Vec2::new(2., 1.),
            Pos2::new(rect.right() - bw, rect.bottom() - 1.),
        );
        let edit = ui.put(
            input,
            egui::TextEdit::singleline(text)
                .id(Id::new(id))
                .hint_text(
                    RichText::new(tr(
                        ui.ctx(),
                        if kind == "Browse" {
                            "Paste a folder path"
                        } else {
                            ""
                        },
                    ))
                    .color(p.muted),
                )
                .frame(false)
                .vertical_align(egui::Align::Center)
                .min_size(input.size())
                .margin(Vec2::new(10., 0.)),
        );
        let r = Rect::from_min_max(
            Pos2::new(rect.right() - bw + 2., rect.top() + 3.),
            rect.max - Vec2::new(3., 3.),
        );
        let button = if kind == "Browse" {
            ui.put(r, egui::Button::new(tr(ui.ctx(), "Browse")).frame(false))
        } else {
            let response = ui.interact(r, Id::new("refresh_series"), Sense::click());
            let c = r.center();
            draw_refresh(ui.painter(), c, 6., p.muted);
            response.on_hover_text(tr(ui.ctx(), "Restore detected series name"))
        };
        (edit, button.clicked())
    }
    fn home(&mut self, ui: &mut Ui) {
        let p = self.palette();
        let width = ui.available_width();
        let top = ui.cursor().min;
        let left = (width - 16.) * 0.6;
        let fr = Rect::from_min_size(top + Vec2::new(0., 24.), Vec2::new(left, 40.));
        let sr = Rect::from_min_size(
            top + Vec2::new(left + 16., 24.),
            Vec2::new(width - left - 16., 40.),
        );
        ui.painter().text(
            top,
            Align2::LEFT_TOP,
            tr(ui.ctx(), "Folder"),
            FontId::proportional(12.),
            p.muted,
        );
        ui.painter().text(
            top + Vec2::new(left + 16., 0.),
            Align2::LEFT_TOP,
            tr(ui.ctx(), "Series name"),
            FontId::proportional(12.),
            p.muted,
        );
        let (fe, browse) = Self::field(ui, fr, &mut self.folder, "folder", "Browse", p);
        let (se, refresh) = Self::field(ui, sr, &mut self.series, "series", "Refresh", p);
        if fe.has_focus() || se.has_focus() {
            self.table_focus = false
        }
        if fe.changed() {
            self.folder_edited(ui.ctx());
        }
        if browse && !self.busy {
            let mut dialog = rfd::FileDialog::new();
            if let Some(root) = &self.root {
                dialog = dialog.set_directory(root);
            }
            if let Some(path) = dialog.pick_folder() {
                self.open(path, true, ui.ctx())
            }
        }
        if (fe.has_focus() || fe.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.load_typed_folder(ui.ctx())
        }
        if se.changed() {
            self.series_suggested = false;
            self.dirty = true;
            self.plan_due = Some(Instant::now() + Duration::from_millis(280));
            ui.ctx().request_repaint_after(Duration::from_millis(280));
        }
        if refresh {
            self.refresh_series()
        } else if se.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.rebuild()
        }
        ui.advance_cursor_after_rect(Rect::from_min_size(top, Vec2::new(width, 74.)));
        let (_, _, selected) = self.row_counts();
        let (heading, _) = ui.allocate_exact_size(Vec2::new(width, 36.), Sense::hover());
        let filter_rect = Rect::from_center_size(
            Pos2::new(heading.right() - 16., heading.center().y),
            Vec2::splat(32.),
        );
        let refresh_rect = filter_rect.translate(Vec2::new(-40., 0.));
        ui.painter().text(
            heading.left_center(),
            Align2::LEFT_CENTER,
            tr(ui.ctx(), "Files"),
            FontId::proportional(16.),
            p.text,
        );
        let counts = format!(
            "{} {}  |  {} {}",
            self.rows.len(),
            tr(ui.ctx(), "files"),
            selected,
            tr(ui.ctx(), "selected")
        );
        ui.painter()
            .with_clip_rect(Rect::from_min_max(
                heading.min + Vec2::new(70., 0.),
                Pos2::new(refresh_rect.left() - 8., heading.bottom()),
            ))
            .text(
                Pos2::new(refresh_rect.left() - 12., heading.center().y),
                Align2::RIGHT_CENTER,
                counts,
                FontId::proportional(12.),
                p.muted,
            );
        let refresh = icon_button(
            ui,
            refresh_rect,
            "refresh_files",
            ActionIcon::Refresh,
            p,
            !self.busy && self.root.is_some(),
            false,
        );
        if refresh.clicked()
            && let Some(root) = self.root.clone()
        {
            self.open(root, false, ui.ctx());
        }
        let previous = self.settings.container_mask();
        let filtered = !self.settings.filter_extensions.is_empty();
        let filter = icon_button(
            ui,
            filter_rect,
            "container_filters",
            ActionIcon::Filter,
            p,
            !self.busy,
            filtered,
        );
        egui::Popup::menu(&filter)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .frame(
                egui::Frame::new()
                    .fill(p.surface)
                    .stroke(Stroke::new(1.0_f32, p.line))
                    .corner_radius(9)
                    .inner_margin(12.),
            )
            .show(|ui| {
                ui.set_width(232.);
                ui.label(RichText::new(tr(ui.ctx(), "Containers")).strong());
                ui.add_space(6.);
                egui::ScrollArea::vertical()
                    .max_height((ui.ctx().content_rect().height() * 0.5).min(230.))
                    .show(ui, |ui| {
                        container_filters(ui, &mut self.settings, &mut self.container_anchor, p);
                    });
            });
        if previous != self.settings.container_mask() {
            self.save_settings();
            if let Some(root) = self.root.clone() {
                self.open(root, false, ui.ctx());
            }
        }
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 8.;
            let enabled = !self.busy && !self.rows.is_empty();
            if ui
                .add_enabled(
                    enabled,
                    egui::Button::new(tr(ui.ctx(), "Include")).min_size(Vec2::new(72., 32.)),
                )
                .clicked()
            {
                self.mark(Some(true));
            }
            if ui
                .add_enabled(
                    enabled,
                    egui::Button::new(tr(ui.ctx(), "Exclude")).min_size(Vec2::new(72., 32.)),
                )
                .clicked()
            {
                self.mark(Some(false));
            }
            let label = if self.any_file_selected() {
                "Unselect all"
            } else {
                "Select all"
            };
            if ui
                .add_enabled(
                    enabled,
                    egui::Button::new(tr(ui.ctx(), label)).min_size(Vec2::new(88., 32.)),
                )
                .clicked()
            {
                self.toggle_all_files();
            }
        });
        ui.add_space(4.);
        self.table(ui, p);
    }
}
fn brand_rect(viewport: Rect, zoom: f32) -> Rect {
    let window = viewport.size() * zoom;
    let size = (window.x * 0.18).min(window.y * 0.28).clamp(80., 200.) / zoom;
    Rect::from_min_size(
        viewport.min + Vec2::new(-20., -12.) / zoom,
        Vec2::splat(size),
    )
}

#[derive(Clone, Copy)]
enum ActionIcon {
    Close,
    Filter,
    Refresh,
}

fn soft_glow(ui: &Ui, rect: Rect, color: Color32, strength: f32) {
    for (expand, alpha) in [(4., 0.025), (2., 0.06), (0., 0.16)] {
        ui.painter().rect_stroke(
            rect.expand(expand),
            7,
            Stroke::new(1.5_f32, color.gamma_multiply(strength * alpha)),
            StrokeKind::Outside,
        );
    }
}

fn icon_button(
    ui: &Ui,
    rect: Rect,
    id: &str,
    icon: ActionIcon,
    p: Palette,
    enabled: bool,
    active: bool,
) -> Response {
    let response = ui.interact(
        rect,
        Id::new(id),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if response.hovered() && enabled {
        ui.painter().rect_filled(rect, 7, p.button);
    }
    if active {
        soft_glow(ui, rect.shrink(2.), p.accent, 1.);
    }
    let color = if !enabled {
        p.muted.gamma_multiply(0.4)
    } else if active || response.hovered() {
        p.accent
    } else {
        p.muted
    };
    let stroke = Stroke::new(1.5_f32, color);
    let c = rect.center();
    match icon {
        ActionIcon::Close => {
            ui.painter()
                .line_segment([c + Vec2::new(-5., -5.), c + Vec2::new(5., 5.)], stroke);
            ui.painter()
                .line_segment([c + Vec2::new(5., -5.), c + Vec2::new(-5., 5.)], stroke);
        }
        ActionIcon::Filter => {
            ui.painter().add(egui::Shape::closed_line(
                vec![
                    c + Vec2::new(-8., -6.),
                    c + Vec2::new(8., -6.),
                    c + Vec2::new(2., 1.),
                    c + Vec2::new(2., 7.),
                    c + Vec2::new(-2., 5.),
                    c + Vec2::new(-2., 1.),
                ],
                stroke,
            ));
        }
        ActionIcon::Refresh => draw_refresh(ui.painter(), c, 7., color),
    }
    let label = match icon {
        ActionIcon::Close => "Close",
        ActionIcon::Filter => "Container filters",
        ActionIcon::Refresh => "Refresh files",
    };
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    response.on_hover_text(tr(ui.ctx(), label))
}

fn draw_refresh(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    let stroke = Stroke::new(1.5_f32, color);
    let end = 5.5_f32;
    let points: Vec<_> = (0..25)
        .map(|i| center + Vec2::angled(0.45 + i as f32 * (end - 0.45) / 24.) * radius)
        .collect();
    let tip = *points.last().unwrap();
    painter.add(egui::Shape::line(points, stroke));
    let tangent = Vec2::new(-end.sin(), end.cos());
    let normal = Vec2::new(-tangent.y, tangent.x);
    painter.add(egui::Shape::line(
        vec![
            tip - tangent * 3.5 + normal * 2.5,
            tip,
            tip - tangent * 3.5 - normal * 2.5,
        ],
        stroke,
    ));
}

fn uninstall_action(ui: &mut Ui, p: Palette, enabled: bool) -> Response {
    let color = if enabled {
        if p.bg.r() < 128 {
            Color32::from_rgb(225, 131, 131)
        } else {
            Color32::from_rgb(170, 55, 55)
        }
    } else {
        p.muted.gamma_multiply(0.45)
    };
    let text = ui.painter().layout_no_wrap(
        tr(ui.ctx(), "Uninstall").into(),
        FontId::proportional(13.),
        color,
    );
    let width = text.size().x + 44.;
    let (row, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 34.), Sense::hover());
    let label = ui.painter().layout_no_wrap(
        tr(ui.ctx(), "Uninstall Anemorph").into(),
        FontId::proportional(13.),
        p.text,
    );
    ui.painter().galley(
        Pos2::new(row.left(), row.center().y - label.size().y / 2.),
        label,
        p.text,
    );
    let rect = Rect::from_min_size(
        Pos2::new(row.right() - width, row.top()),
        Vec2::new(width, row.height()),
    );
    let response = ui.interact(
        rect,
        Id::new("uninstall_anemorph"),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let hovered = enabled && response.hovered();
    ui.painter().rect(
        rect,
        7,
        blend(color, p.surface, if hovered { 0.13 } else { 0.035 }),
        Stroke::new(
            1.0_f32,
            color.gamma_multiply(if hovered { 0.75 } else { 0.35 }),
        ),
        StrokeKind::Inside,
    );
    let c = rect.left_center() + Vec2::new(18., 0.);
    let stroke = Stroke::new(1.4_f32, color);
    ui.painter()
        .line_segment([c + Vec2::new(-6., -4.), c + Vec2::new(6., -4.)], stroke);
    ui.painter().add(egui::Shape::line(
        vec![
            c + Vec2::new(-4., -1.),
            c + Vec2::new(-3., 6.),
            c + Vec2::new(3., 6.),
            c + Vec2::new(4., -1.),
        ],
        stroke,
    ));
    ui.painter().add(egui::Shape::line(
        vec![
            c + Vec2::new(-2., -5.),
            c + Vec2::new(-2., -7.),
            c + Vec2::new(2., -7.),
            c + Vec2::new(2., -5.),
        ],
        stroke,
    ));
    ui.painter().galley(
        Pos2::new(rect.left() + 32., rect.center().y - text.size().y / 2.),
        text,
        color,
    );
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, "Uninstall Anemorph")
    });
    response
}

fn select_container(
    settings: &mut Settings,
    anchor: &mut Option<(usize, bool)>,
    index: usize,
    shift: bool,
) {
    if shift && let Some((start, selected)) = *anchor {
        for (extension, _) in &engine::VIDEO_CONTAINERS[start.min(index)..=start.max(index)] {
            if settings.container_visible(extension) {
                settings.set_container_selected(extension, selected);
            }
        }
    } else {
        let extension = engine::VIDEO_CONTAINERS[index].0;
        settings.toggle_container(extension);
        *anchor = Some((index, settings.container_selected(extension)));
    }
}
#[cfg(test)]
mod filter_range_tests {
    use super::*;
    #[test]
    fn ranges_add_and_remove_without_restricting_other_choices() {
        let mut settings = Settings::default();
        let mut anchor = None;
        select_container(&mut settings, &mut anchor, 0, false);
        select_container(&mut settings, &mut anchor, 2, true);
        assert_eq!(settings.container_mask(), 0b111);
        select_container(&mut settings, &mut anchor, 1, false);
        select_container(&mut settings, &mut anchor, 0, true);
        assert_eq!(settings.container_mask(), 0b100);
        settings.toggle_all_containers();
        assert_eq!(
            settings.filter_extensions.len(),
            settings.home_filter_options.len()
        );
        settings.toggle_all_containers();
        assert_eq!(settings.container_mask(), 0);
        assert!(settings.container_enabled("mov") && settings.container_enabled("mkv"));
    }
}

fn container_filters(
    ui: &mut Ui,
    settings: &mut Settings,
    anchor: &mut Option<(usize, bool)>,
    p: Palette,
) {
    if settings.home_filter_options.is_empty() {
        ui.label(RichText::new(tr(ui.ctx(), "No filter options enabled")).color(p.muted));
        return;
    }
    let all = !settings.home_filter_options.is_empty()
        && settings.filter_extensions.len() == settings.home_filter_options.len();
    if ui
        .add(egui::Button::new(if all { "Unselect all" } else { "Select all" }).frame(false))
        .clicked()
    {
        settings.toggle_all_containers();
        *anchor = None;
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(6., 4.);
        for (index, (extension, label)) in engine::VIDEO_CONTAINERS.iter().enumerate() {
            if !settings.container_visible(extension) {
                continue;
            }
            if container_choice(
                ui,
                label,
                settings.container_selected(extension),
                p,
                settings.animations,
            )
            .clicked()
            {
                select_container(settings, anchor, index, ui.input(|i| i.modifiers.shift));
            }
        }
    });
}
fn container_options(ui: &mut Ui, settings: &mut Settings, p: Palette) {
    let all = settings.home_filter_options.len() == engine::VIDEO_CONTAINERS.len();
    if ui
        .add(egui::Button::new(if all { "Unselect all" } else { "Select all" }).frame(false))
        .clicked()
    {
        settings.home_filter_options = if all {
            Vec::new()
        } else {
            engine::VIDEO_CONTAINERS
                .iter()
                .map(|(ext, _)| (*ext).to_owned())
                .collect()
        };
        settings
            .filter_extensions
            .retain(|ext| settings.home_filter_options.contains(ext));
    }
    ui.push_id("home_filter_visibility", |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(6., 4.);
            for (extension, label) in engine::VIDEO_CONTAINERS {
                if container_choice(
                    ui,
                    label,
                    settings.container_visible(extension),
                    p,
                    settings.animations,
                )
                .clicked()
                {
                    settings.toggle_container_visibility(extension);
                }
            }
        });
    });
}

fn container_choice(
    ui: &mut Ui,
    label: &str,
    selected: bool,
    p: Palette,
    animations: bool,
) -> Response {
    let text_width = ui
        .painter()
        .layout_no_wrap(label.into(), FontId::proportional(13.), p.text)
        .size()
        .x;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(text_width + 24., 30.), Sense::click());
    let t = if animations {
        ui.ctx()
            .animate_bool_with_time(ui.id().with(("container_glow", label)), selected, 0.14)
    } else if selected {
        1.
    } else {
        0.
    };
    let r = rect.shrink2(Vec2::new(1., 2.));
    ui.painter()
        .rect_filled(r, 6, blend(p.accent, p.input, 0.12 * t));
    if t > 0.001 {
        soft_glow(ui, r.shrink(1.), p.accent, t);
    }
    if response.hovered() {
        ui.painter().rect_stroke(
            r,
            6,
            Stroke::new(1.0_f32, p.muted.gamma_multiply(0.4)),
            StrokeKind::Inside,
        );
    }
    ui.painter().text(
        r.left_center() + Vec2::new(10., 0.),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.),
        blend(p.accent, p.text, t),
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, selected, label)
    });
    response
}

fn nav_icon(ui: &Ui, c: Pos2, page: Page, color: Color32) {
    let stroke = Stroke::new(1.5_f32, color);
    let painter = ui.painter();
    if page == Page::Home {
        painter.add(egui::Shape::line(
            vec![
                c + Vec2::new(-9., -1.),
                c + Vec2::new(0., -8.),
                c + Vec2::new(9., -1.),
            ],
            stroke,
        ));
        painter.add(egui::Shape::line(
            vec![
                c + Vec2::new(-6., -2.),
                c + Vec2::new(-6., 7.),
                c + Vec2::new(-2., 7.),
                c + Vec2::new(-2., 2.),
                c + Vec2::new(2., 2.),
                c + Vec2::new(2., 7.),
                c + Vec2::new(6., 7.),
                c + Vec2::new(6., -2.),
            ],
            stroke,
        ));
    } else if page == Page::Settings {
        let points = (0..32)
            .map(|n| {
                let radius = if n % 4 < 2 { 9. } else { 7. };
                c + Vec2::angled(n as f32 * std::f32::consts::TAU / 32.) * radius
            })
            .collect();
        painter.add(egui::Shape::closed_line(points, stroke));
        painter.circle_stroke(c, 3., stroke);
    } else {
        painter.circle_stroke(c + Vec2::new(-3., -5.), 3., stroke);
        painter.add(egui::Shape::line(
            vec![
                c + Vec2::new(-9., 7.),
                c + Vec2::new(-8., 3.),
                c + Vec2::new(-5., 1.),
                c + Vec2::new(-1., 1.),
            ],
            stroke,
        ));
        painter.add(egui::Shape::line(
            vec![
                c + Vec2::new(2., 8.),
                c + Vec2::new(7., 2.),
                c + Vec2::new(9., 1.),
                c + Vec2::new(9., -3.),
                c + Vec2::new(6., -1.),
                c + Vec2::new(4., -3.),
                c + Vec2::new(5., -6.),
                c + Vec2::new(2., -4.),
                c + Vec2::new(2., 0.),
                c + Vec2::new(-2., 6.),
            ],
            stroke,
        ));
    }
}

fn card(ui: &mut Ui, p: Palette, f: impl FnOnce(&mut Ui)) {
    ui.add_space(4.);
    egui::Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0_f32, p.line))
        .corner_radius(9)
        .inner_margin(16.)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            f(ui)
        });
    ui.add_space(18.);
}
fn toggle(ui: &mut Ui, label: &str, value: &mut bool, p: Palette) {
    let label = tr(ui.ctx(), label);
    let galley = ui.painter().layout(
        label.to_string(),
        FontId::proportional(13.),
        p.text,
        (ui.available_width() - 56.).max(80.),
    );
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 32_f32.max(galley.size().y + 8.)),
        Sense::click(),
    );
    ui.painter().galley(
        Pos2::new(rect.left(), rect.center().y - galley.size().y / 2.),
        galley,
        p.text,
    );
    let track = Rect::from_center_size(
        Pos2::new(rect.right() - 19., rect.center().y),
        Vec2::new(38., 22.),
    );
    ui.painter().rect_filled(
        track,
        11,
        if *value {
            p.accent
        } else {
            Color32::from_gray(90)
        },
    );
    ui.painter().circle_filled(
        Pos2::new(
            if *value {
                track.right() - 11.
            } else {
                track.left() + 11.
            },
            track.center().y,
        ),
        8.,
        if *value { on_color(p.accent) } else { p.text },
    );
    if response.clicked() {
        *value = !*value
    }
}

fn hsv_picker(ui: &mut Ui, color: &mut Color32, hue: &mut f32) -> bool {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(184., 112.), Sense::hover());
    let sv = Rect::from_min_size(rect.min, Vec2::new(160., 110.));
    let hr = Rect::from_min_size(rect.min + Vec2::new(170., 0.), Vec2::new(14., 110.));
    let mut hsv = egui::ecolor::Hsva::from(*color);
    if hsv.s < 0.0001 || hsv.v < 0.0001 {
        hsv.h = *hue
    } else {
        *hue = hsv.h
    }
    let original = *color;
    let a = ui.interact(sv, Id::new("saturation_value"), Sense::click_and_drag());
    let b = ui.interact(hr, Id::new("hue"), Sense::click_and_drag());
    if (a.dragged() || a.clicked()) && a.interact_pointer_pos().is_some() {
        let p = a.interact_pointer_pos().unwrap();
        hsv.s = ((p.x - sv.left()) / sv.width()).clamp(0., 1.);
        hsv.v = (1. - (p.y - sv.top()) / sv.height()).clamp(0., 1.);
    }
    if (b.dragged() || b.clicked()) && b.interact_pointer_pos().is_some() {
        let p = b.interact_pointer_pos().unwrap();
        hsv.h = ((p.y - hr.top()) / hr.height()).clamp(0., 0.99999);
        *hue = hsv.h;
    }
    *color = Color32::from(hsv);
    let mut mesh = egui::Mesh::default();
    let n = 16_u32;
    for y in 0..=n {
        for x in 0..=n {
            let s = x as f32 / n as f32;
            let v = 1. - y as f32 / n as f32;
            mesh.colored_vertex(
                sv.min + Vec2::new(s * sv.width(), (1. - v) * sv.height()),
                Color32::from(egui::ecolor::Hsva {
                    h: hsv.h,
                    s,
                    v,
                    a: 1.,
                }),
            );
        }
    }
    for y in 0..n {
        for x in 0..n {
            let i = y * (n + 1) + x;
            mesh.add_triangle(i, i + 1, i + n + 1);
            mesh.add_triangle(i + 1, i + n + 2, i + n + 1);
        }
    }
    ui.painter().add(egui::Shape::mesh(mesh));
    let mut strip = egui::Mesh::default();
    for i in 0..=6 {
        let h = i as f32 / 6.;
        let c = Color32::from(egui::ecolor::Hsva {
            h,
            s: 1.,
            v: 1.,
            a: 1.,
        });
        strip.colored_vertex(Pos2::new(hr.left(), hr.top() + h * hr.height()), c);
        strip.colored_vertex(Pos2::new(hr.right(), hr.top() + h * hr.height()), c);
    }
    for i in 0..6 {
        let j = i * 2;
        strip.add_triangle(j, j + 1, j + 2);
        strip.add_triangle(j + 1, j + 3, j + 2);
    }
    ui.painter().add(egui::Shape::mesh(strip));
    ui.painter().circle_stroke(
        sv.min + Vec2::new(hsv.s * sv.width(), (1. - hsv.v) * sv.height()),
        4.5,
        Stroke::new(1.8_f32, Color32::WHITE),
    );
    let yy = hr.top() + hsv.h * hr.height();
    ui.painter().rect_stroke(
        Rect::from_center_size(Pos2::new(hr.center().x, yy), Vec2::new(17., 5.)),
        1,
        Stroke::new(1.4_f32, Color32::WHITE),
        StrokeKind::Middle,
    );
    *color != original
}

#[derive(Clone, Copy)]
struct FileColumns {
    widths: [f32; 3],
    status: bool,
    minimum_name: f32,
}
impl FileColumns {
    fn new(width: f32, ratio: f32, status_width: f32) -> Self {
        let available = (width - 52.).max(0.);
        let status = width >= 700.;
        let minimum_name = (available * 0.25).min(120.);
        let status_width = if status {
            status_width.clamp(
                90_f32.min(available - 2. * minimum_name),
                (available - 2. * minimum_name).max(0.),
            )
        } else {
            0.
        };
        let names = (available - status_width).max(0.);
        let original = (names * ratio).clamp(minimum_name, names - minimum_name);
        Self {
            widths: [original, names - original, status_width],
            status,
            minimum_name,
        }
    }
    fn offsets(self) -> [f32; 4] {
        [
            42.,
            42. + self.widths[0],
            42. + self.widths[0] + self.widths[1],
            42. + self.widths.iter().sum::<f32>(),
        ]
    }
}
fn paint_left(ui: &Ui, rect: Rect, text: &str, color: Color32, size: f32) {
    let text = tr(ui.ctx(), text);
    ui.painter()
        .with_clip_rect(rect.intersect(ui.clip_rect()))
        .text(
            rect.left_center(),
            Align2::LEFT_CENTER,
            text,
            FontId::proportional(size),
            color,
        );
}

fn format_index(ui: &mut Ui, movie: bool) {
    ui.push_id(
        if movie {
            "movie_index"
        } else {
            "episode_index"
        },
        |ui| {
            ui.collapsing(tr(ui.ctx(), "Format index"), |ui| {
                egui::Grid::new("index")
                    .num_columns(2)
                    .spacing([18., 8.])
                    .show(ui, |ui| {
                        let items: &[(&str, &str)] = if movie {
                            &[
                                ("[Movie Name]", "Movie title from the filename"),
                                ("[Movie]", "Movie"),
                                ("[Space]", "One space"),
                            ]
                        } else {
                            &[
                                ("[S] / [Season]", "S / Season"),
                                ("[E] / [Episode]", "E / Episode"),
                                ("[SeaNum]", "Season number"),
                                ("[EpsNum]", "Episode number"),
                                ("[Num]", "Two-digit numbers"),
                                ("[FixNum]", "Numbers without leading zeros"),
                                ("[Space]", "One space"),
                            ]
                        };
                        for (index, meaning) in items {
                            ui.monospace(*index);
                            ui.label(tr(ui.ctx(), meaning));
                            ui.end_row();
                        }
                    });
            });
        },
    );
}
