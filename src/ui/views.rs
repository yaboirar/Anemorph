use super::*;
impl App {
    pub(super) fn about_dialog(&mut self, ctx: &Context) {
        if !self.about_open {
            return;
        }
        let p = self.palette();
        let mut close = false;
        let response = egui::Modal::new(Id::new("about_anemorph"))
            .frame(egui::Frame::new().fill(p.surface).stroke(Stroke::new(1.0_f32, p.line)).corner_radius(12).inner_margin(20.))
            .show(ctx, |ui| {
                ui.set_width((ctx.input(|i| i.content_rect().width()) - 80.).clamp(220., 400.));
                ui.horizontal(|ui| {
                    ui.heading("About Anemorph");
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.), Sense::hover());
                        if icon_button(ui, rect, "about_close", ActionIcon::Close, p, true, false).clicked() { close = true; }
                    });
                });
                ui.add_space(12.);
                ui.label(format!("Version {APP_VERSION}"));
                ui.add_space(8.);
                ui.label("Anemorph is a local file renamer for anime and TV episodes.");
                ui.add_space(8.);
                ui.label("Detect season and episode numbers, choose your own naming format, and preview the filenames before applying a batch.");
                ui.add_space(8.);
                ui.label("Use it to tidy filenames in a media library. Container filters, file selection, collision checks, and Undo help manage each batch.");
                ui.add_space(8.);
                ui.label(RichText::new("Only filenames change. Your video contents stay intact.").color(p.muted));
            });
        if close || response.should_close() {
            self.about_open = false;
        }
    }
    fn color_editor(&mut self, ui: &mut Ui, p: Palette) {
        ui.horizontal_top(|ui| {
            if hsv_picker(ui, &mut self.draft_color, &mut self.picker_hue) {
                self.hex = hex(self.draft_color);
                self.color_message = "Unsaved color".into();
            }
            ui.add_space(8.);
            ui.vertical(|ui| {
                ui.label(RichText::new("Hex color").color(p.muted));
                let edit = ui.add(egui::TextEdit::singleline(&mut self.hex).desired_width(150.));
                if edit.changed() {
                    if let Some(c) = parse_hex(&self.hex) {
                        self.draft_color = c;
                        self.color_message = "Unsaved color".into()
                    } else {
                        self.color_message = "Enter a valid hex color.".into()
                    }
                }
                let valid = parse_hex(&self.hex);
                let saved = self
                    .settings
                    .accent_hex
                    .as_deref()
                    .and_then(parse_hex)
                    .unwrap_or(DEFAULT_ACCENT);
                let pending = valid.is_some_and(|c| c != saved);
                ui.horizontal(|ui| {
                    let button = egui::Button::new(RichText::new("Apply").color(if pending {
                        on_color(p.accent)
                    } else {
                        p.text
                    }))
                    .fill(if pending { p.accent } else { p.button })
                    .sense(if pending {
                        Sense::click()
                    } else {
                        Sense::hover()
                    });
                    if ui.add(button).clicked() && pending {
                        let previous = self.settings.accent_hex.clone();
                        self.settings.accent_hex = valid.map(hex);
                        match self.settings.save(&self.data.join("settings.json")) {
                            Ok(()) => {
                                self.color_message = "Saved".into();
                                self.theme(ui.ctx())
                            }
                            Err(e) => {
                                self.settings.accent_hex = previous;
                                self.color_message = e
                            }
                        }
                    }
                    if ui.button("Reset").clicked() {
                        self.draft_color = DEFAULT_ACCENT;
                        self.hex = hex(self.draft_color);
                        self.color_message = "Unsaved color".into();
                    }
                });
                ui.label(RichText::new(&self.color_message).size(11.).color(p.muted));
            });
        });
    }
    pub(super) fn table(&mut self, ui: &mut Ui, p: Palette) {
        let height = ui.available_height().max(0.);
        let width = ui.available_width();
        let status = width >= 700.;
        let col = ((width - 42. - if status { 116. } else { 0. }) / 2.).max(50.);
        let top = ui.cursor().min;
        let header = Rect::from_min_size(top, Vec2::new(width, 38.));
        ui.painter().rect_filled(
            Rect::from_min_size(top, Vec2::new(width, height)),
            7,
            p.input,
        );
        ui.painter().rect_filled(header, 5, p.surface);
        let (ready, included, _) = self.row_counts();
        let all = ready > 0 && ready == included;
        let some = included > 0;
        let check = Rect::from_center_size(
            Pos2::new(header.left() + 19., header.center().y),
            Vec2::splat(17.),
        );
        ui.painter().rect(
            check,
            4,
            if some { p.accent } else { p.input },
            Stroke::new(1.0_f32, if ready > 0 { p.muted } else { p.line }),
            StrokeKind::Inside,
        );
        if all {
            ui.painter().add(egui::Shape::line(
                vec![
                    check.min + Vec2::new(4., 8.),
                    check.min + Vec2::new(7., 11.),
                    check.min + Vec2::new(13., 5.),
                ],
                Stroke::new(1.7_f32, on_color(p.accent)),
            ));
        } else if some {
            ui.painter().line_segment(
                [
                    check.center() - Vec2::new(4., 0.),
                    check.center() + Vec2::new(4., 0.),
                ],
                Stroke::new(1.7_f32, on_color(p.accent)),
            );
        }
        let response = ui.interact(
            check.expand(5.),
            Id::new("include_all_header"),
            if ready > 0 {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        if response
            .on_hover_text(if all {
                "Exclude all files"
            } else {
                "Include all files"
            })
            .clicked()
        {
            self.counts.set(None);
            for row in self.rows.iter_mut().filter(|r| r.ready) {
                row.included = !all;
            }
        }
        for (label, x) in [
            ("Original filename", 42.),
            ("New filename", 42. + col),
            ("Status", 42. + col * 2.),
        ] {
            if label != "Status" || status {
                ui.painter().text(
                    top + Vec2::new(x + 10., 19.),
                    Align2::LEFT_CENTER,
                    label,
                    FontId::proportional(12.),
                    p.muted,
                );
            }
        }
        ui.advance_cursor_after_rect(header);
        let remaining = (height - 38.).max(0.);
        if self.rows.is_empty() {
            if remaining >= 45. {
                ui.painter().text(
                    top + Vec2::new(width / 2., 38. + remaining / 2.),
                    Align2::CENTER_CENTER,
                    if self.root.is_some() {
                        "No files match the container filters"
                    } else {
                        "Choose or drop a folder"
                    },
                    FontId::proportional(15.),
                    p.muted,
                );
            }
            ui.allocate_space(Vec2::new(width, remaining));
            return;
        }
        let row_height = if self.settings.compact { 28. } else { 38. };
        let mut action = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(remaining)
            .show_rows(ui, row_height, self.rows.len(), |ui, range| {
                self.max_visible_rows = self.max_visible_rows.max(range.len());
                for index in range {
                    let row = &self.rows[index];
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(width - 10., row_height), Sense::click());
                    if row.highlighted {
                        ui.painter().rect_filled(rect, 0, p.selected);
                    } else if response.hovered() {
                        ui.painter()
                            .rect_filled(rect, 0, blend(p.text, p.input, 0.04));
                    }
                    ui.painter().line_segment(
                        [rect.left_bottom(), rect.right_bottom()],
                        Stroke::new(1.0_f32, p.line),
                    );
                    let cb = Rect::from_center_size(
                        Pos2::new(rect.left() + 19., rect.center().y),
                        Vec2::splat(17.),
                    );
                    ui.painter().rect(
                        cb,
                        4,
                        if row.included { p.accent } else { p.input },
                        Stroke::new(1.0_f32, if row.ready { p.muted } else { p.line }),
                        StrokeKind::Inside,
                    );
                    if row.included {
                        let c = cb.min;
                        ui.painter().add(egui::Shape::line(
                            vec![
                                c + Vec2::new(4., 8.),
                                c + Vec2::new(7., 11.),
                                c + Vec2::new(13., 5.),
                            ],
                            Stroke::new(1.7_f32, on_color(p.accent)),
                        ));
                    }
                    let check =
                        ui.interact(cb.expand(5.), Id::new(("check", index)), Sense::click());
                    let original = Rect::from_min_size(
                        rect.min + Vec2::new(42., 3.),
                        Vec2::new(col - 10., row_height - 6.),
                    );
                    let output = original.translate(Vec2::new(col, 0.));
                    paint_left(ui, original, &row.original, p.text, 12.);
                    paint_left(ui, output, &row.output, p.text, 12.);
                    response.clone().on_hover_ui(|ui| {
                        ui.label(&row.original);
                        ui.label(&row.output);
                    });
                    if status {
                        let r = Rect::from_min_size(
                            rect.min + Vec2::new(42. + col * 2. + 5., 7.),
                            Vec2::new(105., row_height - 14.),
                        );
                        ui.put(
                            r,
                            egui::Label::new(RichText::new(row.status.as_ref()).size(11.).color(
                                if row.ready {
                                    blend(p.accent, p.text, 0.65)
                                } else {
                                    p.muted
                                },
                            ))
                            .halign(egui::Align::Min)
                            .truncate(),
                        );
                    }
                    if check.clicked() && row.ready {
                        action = Some((index, true, ui.input(|i| i.modifiers)))
                    } else if response.clicked() {
                        action = Some((index, false, ui.input(|i| i.modifiers)))
                    }
                }
            });
        if let Some((i, c, m)) = action {
            self.select(i, c, m)
        }
    }
    pub(super) fn settings_ui(&mut self, ui: &mut Ui) {
        let p = self.palette();
        let previous = self.settings.clone();
        ui.heading("Settings");
        ui.add_space(18.);
        ui.strong("Appearance");
        card(ui, p, |ui| {
            toggle(ui, "Dark mode", &mut self.settings.dark_mode, p);
            ui.separator();
            toggle(ui, "Compact file list", &mut self.settings.compact, p);
            ui.separator();
            toggle(ui, "Animations", &mut self.settings.animations, p);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Interface zoom");
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("+").clicked() {
                        self.settings.zoom_percent = (self.settings.zoom_percent + 10).min(150)
                    }
                    if ui
                        .button(format!("{}%", (self.effective_zoom * 100.).round()))
                        .on_hover_text("Reset zoom")
                        .clicked()
                    {
                        self.settings.zoom_percent = 100
                    }
                    if ui.button("−").clicked() {
                        self.settings.zoom_percent =
                            self.settings.zoom_percent.saturating_sub(10).max(75)
                    }
                });
            });
        });
        ui.strong("Highlight color");
        card(ui, p, |ui| self.color_editor(ui, p));
        self.apply_preferences(previous, ui.ctx());
    }
    pub(super) fn configuration_ui(&mut self, ui: &mut Ui) {
        let p = self.palette();
        let previous = self.settings.clone();
        ui.heading("Configuration");
        ui.add_space(18.);
        ui.strong("Episode format");
        card(ui, p, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.format_draft)
                    .desired_width(f32::INFINITY)
                    .min_size(Vec2::new(0., 36.))
                    .vertical_align(egui::Align::Center)
                    .margin(Vec2::new(10., 0.))
                    .id(Id::new("number_format"))
                    .hint_text(naming::DEFAULT_FORMAT),
            );
            if self
                .format_cache
                .as_ref()
                .is_none_or(|(text, _)| text != &self.format_draft)
            {
                self.format_cache = Some((
                    self.format_draft.clone(),
                    naming::Format::parse(&self.format_draft),
                ));
            }
            let format = &self.format_cache.as_ref().unwrap().1;
            match format {
                Ok(format) => {
                    let name = if self.series.trim().is_empty() {
                        "Anemorph"
                    } else {
                        self.series.trim()
                    };
                    ui.label(
                        RichText::new(format!("{name} {}.mkv", format.render(1, 9))).monospace(),
                    );
                }
                Err(error) => {
                    ui.label(RichText::new(error).color(Color32::from_rgb(242, 158, 152)));
                }
            }
            let pending = self.format_draft != self.settings.number_format && format.is_ok();
            ui.horizontal(|ui| {
                let apply = egui::Button::new(RichText::new("Apply").color(if pending {
                    on_color(p.accent)
                } else {
                    p.text
                }))
                .fill(if pending { p.accent } else { p.button })
                .sense(if pending {
                    Sense::click()
                } else {
                    Sense::hover()
                });
                if ui.add(apply).clicked() && pending {
                    self.settings.number_format = self.format_draft.clone();
                }
                if ui.button("Reset").clicked() {
                    self.format_draft = naming::DEFAULT_FORMAT.into();
                }
            });
            ui.collapsing("Format tokens", |ui| {
                egui::Grid::new("format_tokens").num_columns(2).spacing([18., 8.]).show(ui, |ui| {
                    for (token, example) in [
                        ("[S]", "S"), ("[Season]", "Season"),
                        ("[E]", "E"), ("[Episode]", "Episode"),
                        ("[SeaNum]", "Season number"), ("[EpsNum]", "Episode number"),
                        ("[Num]", "Two-digit numbers: 01, 09"),
                        ("[FixNum]", "Unpadded numbers: 1, 9"),
                        ("[Space]", "One space"),
                    ] { ui.monospace(token); ui.label(example); ui.end_row(); }
                });
                ui.label(RichText::new("Set [Num] or [FixNum] before number tokens. The style stays active until changed. Labels are literal text.").size(12.).color(p.muted));
            });
        });
        ui.strong("Naming");
        card(ui, p, |ui| {
            toggle(
                ui,
                "Detect season and episode numbers",
                &mut self.settings.auto_detect,
                p,
            );
            ui.separator();
            toggle(
                ui,
                "Fill series name from folder",
                &mut self.settings.auto_name,
                p,
            );
            ui.separator();
            toggle(
                ui,
                "Clear suggested name when folder is cleared",
                &mut self.settings.clear_series_on_empty,
                p,
            );
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Season when none is found");
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(egui::DragValue::new(&mut self.settings.default_season).range(0..=999));
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("First episode in manual mode");
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(egui::DragValue::new(&mut self.settings.first_episode).range(0..=9999));
                });
            });
        });
        ui.strong("Folders and files");
        card(ui, p, |ui| {
            toggle(ui, "Include subfolders", &mut self.settings.subfolders, p);
            ui.separator();
            ui.label("File types");
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(&mut self.settings.mkv, "MKV");
                ui.checkbox(&mut self.settings.mp4, "MP4");
                ui.checkbox(&mut self.settings.other_video, "AVI, M4V, WebM");
            });
            ui.separator();
            toggle(
                ui,
                "Reopen last folder on startup",
                &mut self.settings.remember_folder,
                p,
            );
        });
        if ui.button("Reset configuration").clicked() {
            let defaults = Settings::default();
            self.settings.auto_detect = defaults.auto_detect;
            self.settings.auto_name = defaults.auto_name;
            self.settings.clear_series_on_empty = defaults.clear_series_on_empty;
            self.settings.number_format = defaults.number_format.clone();
            self.format_draft = defaults.number_format;
            self.settings.default_season = defaults.default_season;
            self.settings.first_episode = defaults.first_episode;
            self.settings.subfolders = defaults.subfolders;
            self.settings.mkv = defaults.mkv;
            self.settings.mp4 = defaults.mp4;
            self.settings.other_video = defaults.other_video;
            self.settings.remember_folder = defaults.remember_folder;
        }
        self.apply_preferences(previous, ui.ctx());
    }
    fn apply_preferences(&mut self, previous: Settings, ctx: &Context) {
        if self.settings != previous {
            if self.settings.remember_folder {
                self.settings.last_folder = self.root.clone()
            }
            self.save_settings();
            self.theme(ctx);
            let filters = self.settings.subfolders != previous.subfolders
                || self.settings.mkv != previous.mkv
                || self.settings.mp4 != previous.mp4
                || self.settings.other_video != previous.other_video;
            if filters {
                if let Some(root) = self.root.clone() {
                    self.open(root, false, ctx)
                }
            } else if self.settings.auto_detect != previous.auto_detect
                || self.settings.default_season != previous.default_season
                || self.settings.first_episode != previous.first_episode
                || self.settings.number_format != previous.number_format
            {
                self.rebuild()
            }
        }
    }
    fn resize(&self, ctx: &Context) {
        let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else {
            return;
        };
        let r = ctx.input(|i| i.content_rect());
        // Caption buttons own these pixels, including the very top/right edge.
        if pos.y < r.top() + 34. && pos.x >= r.right() - 132. {
            return;
        }
        let l = pos.x < r.left() + 5.;
        let right = pos.x > r.right() - 5.;
        let t = pos.y < r.top() + 5.;
        let b = pos.y > r.bottom() - 5.;
        use egui::viewport::ResizeDirection::*;
        let direction = match (l, right, t, b) {
            (true, _, true, _) => Some(NorthWest),
            (true, _, _, true) => Some(SouthWest),
            (_, true, true, _) => Some(NorthEast),
            (_, true, _, true) => Some(SouthEast),
            (true, _, _, _) => Some(West),
            (_, true, _, _) => Some(East),
            (_, _, true, _) => Some(North),
            (_, _, _, true) => Some(South),
            _ => None,
        };
        if let Some(d) = direction {
            ctx.set_cursor_icon(match d {
                North | South => egui::CursorIcon::ResizeVertical,
                East | West => egui::CursorIcon::ResizeHorizontal,
                NorthEast | SouthWest => egui::CursorIcon::ResizeNeSw,
                _ => egui::CursorIcon::ResizeNwSe,
            });
            if ctx.input(|i| i.pointer.primary_pressed()) {
                ctx.send_viewport_cmd(ViewportCommand::BeginResize(d))
            }
        }
    }
    fn smoke_tick(&mut self, ctx: &Context) {
        let Some(smoke) = &mut self.smoke else { return };
        let dir = smoke.dir.clone();
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                let name = match smoke.snapshots {
                    0 => "home.png",
                    1 => "settings.png",
                    2 => "configuration.png",
                    3 => "minimum.png",
                    4 => "light.png",
                    5 => "about.png",
                    _ => "filters.png",
                };
                let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                let _ = image::save_buffer(
                    dir.join(name),
                    &bytes,
                    image.size[0] as u32,
                    image.size[1] as u32,
                    image::ColorType::Rgba8,
                );
                smoke.snapshots += 1;
            }
        }
        if smoke.stage == 17 {
            if smoke.last.elapsed() < Duration::from_millis(1800) {
                return;
            }
            let report = serde_json::json!({"language":"Rust","files":self.rows.len(),"screenshots":smoke.snapshots,"frames":self.frames,"max_visible_rows":self.max_visible_rows,"idle_repaints_1_8_seconds":self.frames-smoke.idle_start,"mean_ui_work_ms":self.ui_micros as f64/self.frames.max(1)as f64/1000.,"window":"Anemorph","maximized_restore_verified":smoke.native_restore_verified});
            let _ = std::fs::write(
                dir.join("smoke.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            );
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }
        if smoke.last.elapsed() < Duration::from_millis(500) {
            ctx.request_repaint_after(Duration::from_millis(100));
            return;
        }
        smoke.last = Instant::now();
        let stage = smoke.stage;
        match stage {
            0 => {
                let fixtures = dir.join("fixtures");
                let _ = std::fs::create_dir_all(&fixtures);
                for i in 1..=500 {
                    let _ =
                        std::fs::write(fixtures.join(format!("Show S02E{i:03}.mkv")), b"fixture");
                }
                self.open(fixtures, true, ctx)
            }
            1 => {
                if self.busy {
                    return;
                }
                self.series = "Anemorph".into();
                self.rebuild();
                ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default()));
            }
            2 => self.navigate(Page::Settings, ctx),
            3 => ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default())),
            4 => self.navigate(Page::Configuration, ctx),
            5 => ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default())),
            6 => {
                self.navigate(Page::Home, ctx);
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(480., 360.)));
            }
            7 => ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default())),
            8 => {
                self.settings.dark_mode = false;
                self.theme(ctx)
            }
            9 => ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default())),
            10 => self.about_open = true,
            11 => ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default())),
            12 => {
                self.about_open = false;
                if let Some(button) = ctx.read_response(Id::new("container_filters")) {
                    egui::Popup::open_id(ctx, egui::Popup::default_response_id(&button));
                }
            }
            13 => ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default())),
            14 => {
                if let Some(button) = ctx.read_response(Id::new("container_filters")) {
                    egui::Popup::close_id(ctx, egui::Popup::default_response_id(&button));
                }
                ctx.send_viewport_cmd(ViewportCommand::Maximized(true));
            }
            15 => {
                let verified = self
                    .native_window
                    .is_some_and(window_drag::verify_maximized_restore);
                self.smoke.as_mut().unwrap().native_restore_verified = verified;
            }
            16 => {
                self.smoke.as_mut().unwrap().idle_start = self.frames;
                let context = ctx.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(1850));
                    context.request_repaint();
                });
            }
            _ => {}
        }
        if let Some(s) = &mut self.smoke {
            s.stage += 1
        }
        if stage != 16 {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let measured = self.smoke.is_some().then(Instant::now);
        self.frames += 1;
        self.poll(ctx);
        if self.folder_due.is_some_and(|due| Instant::now() >= due) {
            self.load_typed_folder(ctx);
        }
        if self.plan_due.is_some_and(|due| Instant::now() >= due) {
            self.rebuild()
        }
        if self.busy && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose)
        }
        let base = ctx.input(|i| i.content_rect().size()) * ctx.zoom_factor();
        self.settings.window_width = base.x;
        self.settings.window_height = base.y;
        let fit = (base.x / 480.)
            .min(
                base.y
                    / if self.settings.auto_detect {
                        485.
                    } else {
                        555.
                    },
            )
            .max(0.5);
        let zoom = (self.settings.zoom_percent as f32 / 100.).min(fit);
        if (zoom - self.effective_zoom).abs() > 0.005 {
            self.effective_zoom = zoom;
            ctx.set_zoom_factor(zoom)
        }
        let dropped = ctx.input(|i| i.raw.dropped_files.iter().find_map(|f| f.path.clone()));
        if let Some(path) = dropped {
            let root = if path.is_dir() {
                path
            } else {
                path.parent().unwrap_or(Path::new("")).into()
            };
            self.open(root, true, ctx)
        }
        let editing_name =
            ctx.memory(|m| m.has_focus(Id::new("folder")) || m.has_focus(Id::new("series")));
        if self.page == Page::Home && self.table_focus && !editing_name && !self.about_open {
            if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, egui::Key::A)) {
                self.counts.set(None);
                for r in &mut self.rows {
                    r.highlighted = true
                }
            }
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Space)) {
                self.mark(None)
            }
        }
        let p = self.palette();
        egui::TopBottomPanel::top("navigation")
            .exact_height(52_f32.max(46. / ctx.zoom_factor()))
            .frame(egui::Frame::NONE.fill(p.bg))
            .show(ctx, |ui| self.nav(ui));
        if self.page == Page::Home {
            egui::TopBottomPanel::bottom("footer")
                .exact_height(60.)
                .frame(egui::Frame::NONE.fill(p.surface).inner_margin(8.))
                .show(ctx, |ui| self.footer(ui));
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(p.bg).inner_margin(20.))
            .show(ctx, |ui| {
                self.paint_brand(ui);
                let progress = self.progress();
                let ease = 1. - (1. - progress).powi(3);
                let bounds = ui.max_rect().translate(Vec2::new(
                    if self.page == Page::Home { -12. } else { 12. } * (1. - ease),
                    0.,
                ));
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(bounds)
                        .layout(Layout::top_down(egui::Align::Min)),
                );
                child.set_clip_rect(ui.clip_rect());
                child.set_opacity(0.3 + 0.7 * ease);
                if self.busy {
                    child.disable();
                }
                if self.page == Page::Home {
                    self.home(&mut child)
                } else {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(&mut child, |ui| {
                            let width = ui.available_width().min(700.);
                            let left = (ui.available_width() - width) / 2.;
                            let r = Rect::from_min_size(
                                ui.cursor().min + Vec2::new(left, 0.),
                                Vec2::new(width, ui.available_height()),
                            );
                            ui.scope_builder(egui::UiBuilder::new().max_rect(r), |ui| {
                                if self.page == Page::Settings {
                                    self.settings_ui(ui)
                                } else {
                                    self.configuration_ui(ui)
                                }
                            });
                        });
                }
            });
        if !self.about_open {
            self.resize(ctx);
        }
        self.about_dialog(ctx);
        self.smoke_tick(ctx);
        if let Some(start) = measured {
            self.ui_micros += start.elapsed().as_micros();
        }
    }
    fn on_exit(&mut self) {
        self.save_settings();
    }
}
