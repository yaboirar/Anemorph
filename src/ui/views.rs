use super::*;
impl App {
    pub(super) fn about_dialog(&mut self, ctx: &Context) {
        if !self.about_open {
            return;
        }
        let p = self.palette();
        let mut close = false;
        let response = egui::Modal::new(Id::new("about_anemorph"))
            .frame(
                egui::Frame::new()
                    .fill(p.surface)
                    .stroke(Stroke::new(1.0_f32, p.line))
                    .corner_radius(12)
                    .inner_margin(20.),
            )
            .show(ctx, |ui| {
                ui.set_width((ctx.input(|i| i.content_rect().width()) - 80.).clamp(220., 400.));
                ui.spacing_mut().item_spacing.y = 6.;
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.;
                    ui.horizontal(|ui| {
                        ui.heading("About Anemorph");
                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::splat(22.), Sense::hover());
                            if icon_button(
                                ui,
                                rect,
                                "about_close",
                                ActionIcon::Close,
                                p,
                                true,
                                false,
                            )
                            .clicked()
                            {
                                close = true;
                            }
                        });
                    });
                    ui.label(
                        RichText::new(format!("Version {APP_VERSION}"))
                            .size(11.)
                            .color(p.muted),
                    );
                });
                ui.add_space(8.);
                ui.label("A free, open-source file renamer for anime, TV, and movies.");
                ui.add_space(8.);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.;
                    ui.label(RichText::new("Created by").color(p.muted));
                    ui.hyperlink_to("@yaboirar", "https://github.com/yaboirar");
                });
            });
        if close || response.should_close() {
            self.about_open = false;
        }
    }
    pub(super) fn episode_marker_dialog(&mut self, ctx: &Context) {
        let p = self.palette();
        let name = if self.series.trim().is_empty() {
            "Anemorph"
        } else {
            self.series.trim()
        };
        let Some(dialog) = self.episode_dialog.as_mut() else {
            return;
        };
        let mut apply = false;
        let mut close = false;
        let response = egui::Modal::new(Id::new("episode_marker_dialog"))
            .frame(
                egui::Frame::new()
                    .fill(p.surface)
                    .stroke(Stroke::new(1_f32, p.line))
                    .corner_radius(12)
                    .inner_margin(20.),
            )
            .show(ctx, |ui| {
                ui.set_width((ctx.input(|i| i.content_rect().width()) - 80.).clamp(240., 420.));
                ui.spacing_mut().item_spacing.y = 8.;
                ui.horizontal(|ui| {
                    ui.heading("Mark as episode");
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(22.), Sense::hover());
                        if icon_button(
                            ui,
                            rect,
                            "episode_marker_close",
                            ActionIcon::Close,
                            p,
                            true,
                            false,
                        )
                        .clicked()
                        {
                            close = true;
                        }
                    });
                });
                let first = dialog
                    .entries
                    .iter()
                    .find(|e| e.numbering.is_none())
                    .unwrap();
                let filename = first.path.file_name().unwrap_or_default().to_string_lossy();
                if dialog.entries.len() == 1 {
                    ui.add(
                        egui::Label::new(RichText::new(filename.as_ref()).color(p.muted))
                            .truncate(),
                    )
                    .on_hover_text(filename.as_ref());
                } else {
                    ui.label(
                        RichText::new(format!(
                            "{} files need episode numbers",
                            dialog.unknown_count
                        ))
                        .color(p.muted),
                    );
                }
                ui.add_space(4.);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Season");
                        ui.add(
                            egui::DragValue::new(&mut dialog.season)
                                .range(0..=999)
                                .min_decimals(0),
                        );
                    });
                    ui.add_space(18.);
                    ui.vertical(|ui| {
                        ui.label(if dialog.unknown_count > 1 {
                            "First episode"
                        } else {
                            "Episode"
                        });
                        ui.add(
                            egui::DragValue::new(&mut dialog.first_episode)
                                .range(0..=9999)
                                .min_decimals(0),
                        );
                    });
                });
                let ext = first.path.extension().unwrap_or_default().to_string_lossy();
                ui.label(
                    RichText::new(format!(
                        "{name} {}.{ext}",
                        dialog.format.render(dialog.season, dialog.first_episode)
                    ))
                    .monospace()
                    .size(12.)
                    .color(p.muted),
                );
                if dialog.entries.len() > dialog.unknown_count {
                    ui.label(
                        RichText::new("Existing episode numbers are kept.")
                            .size(12.)
                            .color(p.muted),
                    );
                }
                if let Some(error) = &dialog.error {
                    ui.label(RichText::new(error).color(Color32::from_rgb(242, 158, 152)));
                }
                ui.add_space(4.);
                if ui
                    .add(
                        egui::Button::new(RichText::new("Apply").color(on_color(p.accent)))
                            .fill(p.accent),
                    )
                    .clicked()
                {
                    apply = true;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    apply = true;
                }
            });
        if close || response.should_close() {
            self.episode_dialog = None;
        } else if apply {
            self.apply_episode_marker();
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
        let top = ui.cursor().min;
        let header = Rect::from_min_size(top, Vec2::new(width, 38.));
        ui.painter().rect_filled(
            Rect::from_min_size(top, Vec2::new(width, height)),
            7,
            p.input,
        );
        ui.painter().rect_filled(header, 5, p.surface);
        for divider in 0..2 {
            let columns = FileColumns::new(
                width,
                self.settings.table_name_ratio,
                self.settings.table_status_width,
            );
            if divider == 1 && !columns.status {
                continue;
            }
            let offsets = columns.offsets();
            let x = top.x + offsets[divider + 1];
            let grip = Rect::from_center_size(
                Pos2::new(x, header.center().y),
                Vec2::new(10., header.height() - 4.),
            );
            let response = ui.interact(
                grip,
                Id::new(("column_divider", divider)),
                Sense::click_and_drag(),
            );
            if response.hovered() || response.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            if response.drag_started() {
                let start = ui
                    .input(|i| i.pointer.press_origin())
                    .unwrap_or(grip.center());
                self.column_drag = Some((divider, x - start.x));
            }
            let reset = response.double_clicked();
            if (response.dragged() || reset)
                && let Some(pointer) = response.interact_pointer_pos()
            {
                let offset = self
                    .column_drag
                    .filter(|(i, _)| *i == divider)
                    .map_or(0., |(_, offset)| offset);
                let position = pointer.x + offset - top.x;
                if divider == 0 {
                    let names = columns.widths[0] + columns.widths[1];
                    let original = if reset {
                        names * 0.5
                    } else {
                        (position - 42.).clamp(columns.minimum_name, names - columns.minimum_name)
                    };
                    if names > 0. {
                        self.settings.table_name_ratio = (original / names).clamp(0.05, 0.95);
                    }
                } else {
                    let available = columns.widths.iter().sum::<f32>();
                    let maximum = (available - columns.widths[0] - columns.minimum_name).max(90.);
                    let status_width = if reset {
                        116_f32.clamp(90., maximum)
                    } else {
                        (available - (position - 42.)).clamp(90., maximum)
                    };
                    self.settings.table_status_width = status_width;
                    self.settings.table_name_ratio =
                        (columns.widths[0] / (available - status_width)).clamp(0.05, 0.95);
                }
            }
            if response.drag_stopped() || reset {
                self.column_drag = None;
                self.save_settings();
            }
            let current = FileColumns::new(
                width,
                self.settings.table_name_ratio,
                self.settings.table_status_width,
            );
            let boundary = top.x + current.offsets()[divider + 1];
            ui.painter().line_segment(
                [
                    Pos2::new(boundary, header.top() + 8.),
                    Pos2::new(boundary, header.bottom() - 8.),
                ],
                Stroke::new(1_f32, p.line),
            );
        }
        let columns = FileColumns::new(
            width,
            self.settings.table_name_ratio,
            self.settings.table_status_width,
        );
        let offsets = columns.offsets();
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
        for (i, label) in ["Original filename", "New filename", "Status"]
            .into_iter()
            .enumerate()
        {
            if i == 2 && !columns.status {
                continue;
            }
            let rect = Rect::from_min_max(
                Pos2::new(top.x + offsets[i] + 10., header.top()),
                Pos2::new(top.x + offsets[i + 1] - 10., header.bottom()),
            );
            paint_left(ui, rect, label, p.muted, 12.);
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
        let mut marker_action = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(remaining)
            .show_rows(ui, row_height, self.rows.len(), |ui, range| {
                self.max_visible_rows = self.max_visible_rows.max(range.len());
                for index in range {
                    let row = &self.rows[index];
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(width - 10., row_height), Sense::hover());
                    let response =
                        ui.interact(rect, Id::new(("file_row", &row.source)), Sense::click());
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
                    let original = Rect::from_min_max(
                        rect.min + Vec2::new(offsets[0] + 10., 3.),
                        Pos2::new(rect.left() + offsets[1] - 10., rect.bottom() - 3.),
                    );
                    let output = Rect::from_min_max(
                        rect.min + Vec2::new(offsets[1] + 10., 3.),
                        Pos2::new(rect.left() + offsets[2] - 10., rect.bottom() - 3.),
                    );
                    paint_left(ui, original, &row.original, p.text, 12.);
                    paint_left(ui, output, &row.output, p.text, 12.);
                    response.clone().on_hover_ui(|ui| {
                        ui.label(match row.kind {
                            engine::MediaKind::Movie => "Movie",
                            engine::MediaKind::Episode => "Episode",
                        });
                        ui.label(row.status.as_ref());
                        ui.label(&row.original);
                        ui.label(&row.output);
                    });
                    if columns.status {
                        let status_rect = Rect::from_min_max(
                            rect.min + Vec2::new(offsets[2] + 10., 3.),
                            Pos2::new(rect.left() + offsets[3] - 10., rect.bottom() - 3.),
                        );
                        paint_left(
                            ui,
                            status_rect,
                            row.status.as_ref(),
                            if row.ready {
                                blend(p.accent, p.text, 0.65)
                            } else {
                                p.muted
                            },
                            11.,
                        );
                    }
                    response.context_menu(|ui| {
                        if ui.button("Mark as movie").clicked() {
                            marker_action = Some((index, Some(engine::MediaKind::Movie)));
                            ui.close();
                        }
                        if ui.button("Mark as episode").clicked() {
                            marker_action = Some((index, Some(engine::MediaKind::Episode)));
                            ui.close();
                        }
                        if self
                            .media_markers
                            .contains_key(&engine::media_marker_key(&row.source))
                            && ui.button("Use automatic detection").clicked()
                        {
                            marker_action = Some((index, None));
                            ui.close();
                        }
                    });
                    if response.secondary_clicked() && !row.highlighted {
                        action = Some((index, false, Modifiers::NONE));
                    }
                    if check.clicked() && row.ready {
                        action = Some((index, true, ui.input(|i| i.modifiers)))
                    } else if response.clicked() {
                        action = Some((index, false, ui.input(|i| i.modifiers)))
                    }
                }
            });
        if let Some((i, c, m)) = action {
            self.select(i, c, m);
        }
        if let Some((index, kind)) = marker_action {
            self.set_media_marker(index, kind);
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
        ui.strong("Integration");
        self.integration.poll(
            self.uninstaller
                .as_ref()
                .and_then(|p| p.parent())
                .map(PathBuf::from),
            ui.ctx(),
        );
        card(ui, p, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(34.);
                ui.label("First right-click menu");
                let label = if self.integration.pending {
                    "Working..."
                } else if self.integration.enabled {
                    "Disable"
                } else {
                    "Enable"
                };
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let active = !self.integration.enabled && !self.integration.pending;
                    let button = egui::Button::new(RichText::new(label).color(if active {
                        p.accent
                    } else {
                        p.text
                    }))
                    .min_size(Vec2::new(94., 34.))
                    .fill(if active {
                        blend(p.accent, p.surface, 0.09)
                    } else {
                        p.button
                    })
                    .stroke(Stroke::new(
                        1.0_f32,
                        if active {
                            blend(p.accent, p.line, 0.45)
                        } else {
                            p.line
                        },
                    ));
                    if ui
                        .add_enabled(
                            self.uninstaller.is_some() && !self.integration.pending,
                            button,
                        )
                        .on_hover_text(if self.integration.enabled {
                            "Return to the classic Explorer menu."
                        } else {
                            "Enable Anemorph on Windows 11's first right-click menu."
                        })
                        .clicked()
                        && let Some(root) = self.uninstaller.as_ref().and_then(|p| p.parent())
                    {
                        self.integration
                            .start(root.to_path_buf(), Some(!self.integration.enabled));
                    }
                });
            });
            if !self.integration.enabled {
                ui.label(RichText::new("Windows may ask for administrator approval to trust Anemorph's signing certificate.").small().color(p.muted));
            }
            if !self.integration.message.is_empty() {
                ui.label(
                    RichText::new(&self.integration.message)
                        .small()
                        .color(p.muted),
                );
            }
        });
        let enabled = self.uninstaller.is_some()
            && !self.busy
            && self.preview.is_none()
            && !self.integration.pending;
        let mut uninstall = false;
        card(ui, p, |ui| {
            uninstall = uninstall_action(ui, p, enabled)
                .on_hover_text(if self.uninstaller.is_some() {
                    "Open the clean uninstaller. Your videos are kept."
                } else {
                    "Available after installing Anemorph with setup."
                })
                .clicked();
        });
        if uninstall && let Some(path) = &self.uninstaller {
            match uninstall::launch(path, std::process::id()) {
                Ok(()) => {
                    self.plan_due = None;
                    self.folder_due = None;
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }
                Err(error) => self.uninstall_error = error,
            }
        }
        if !self.uninstall_error.is_empty() {
            ui.label(RichText::new(&self.uninstall_error).color(p.muted));
        }
    }
    pub(super) fn configuration_ui(&mut self, ui: &mut Ui) {
        let p = self.palette();
        let previous = self.settings.clone();
        ui.heading("Configuration");
        ui.add_space(18.);
        ui.strong("Filename format");
        card(ui, p, |ui| {
            ui.label(RichText::new("Episodes").color(p.muted));
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
            let episode_valid = match &self.format_cache.as_ref().unwrap().1 {
                Ok(format) => {
                    let name = if self.series.trim().is_empty() {
                        "Anemorph"
                    } else {
                        self.series.trim()
                    };
                    ui.label(
                        RichText::new(format!("{name} {}.mkv", format.render(1, 9)))
                            .monospace()
                            .size(12.)
                            .color(p.muted),
                    );
                    true
                }
                Err(error) => {
                    ui.label(RichText::new(error).color(Color32::from_rgb(242, 158, 152)));
                    false
                }
            };
            ui.add_space(8.);
            ui.label(RichText::new("Movies").color(p.muted));
            ui.add(
                egui::TextEdit::singleline(&mut self.movie_format_draft)
                    .desired_width(f32::INFINITY)
                    .min_size(Vec2::new(0., 36.))
                    .vertical_align(egui::Align::Center)
                    .margin(Vec2::new(10., 0.))
                    .id(Id::new("movie_format"))
                    .hint_text(naming::DEFAULT_MOVIE_FORMAT),
            );
            if self
                .movie_format_cache
                .as_ref()
                .is_none_or(|(text, _)| text != &self.movie_format_draft)
            {
                self.movie_format_cache = Some((
                    self.movie_format_draft.clone(),
                    naming::MovieFormat::parse(&self.movie_format_draft),
                ));
            }
            let movie_valid = match &self.movie_format_cache.as_ref().unwrap().1 {
                Ok(format) => {
                    ui.label(
                        RichText::new(format!(
                            "{}.mkv",
                            format.render(if self.series.trim().is_empty() {
                                "Anemorph"
                            } else {
                                self.series.trim()
                            })
                        ))
                        .monospace()
                        .size(12.)
                        .color(p.muted),
                    );
                    true
                }
                Err(error) => {
                    ui.label(RichText::new(error).color(Color32::from_rgb(242, 158, 152)));
                    false
                }
            };
            ui.add_space(6.);
            let pending = (self.format_draft != self.settings.number_format
                || self.movie_format_draft != self.settings.movie_format)
                && episode_valid
                && movie_valid;
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
                    self.settings.movie_format = self.movie_format_draft.clone();
                }
                if ui.button("Reset").clicked() {
                    self.format_draft = naming::DEFAULT_FORMAT.into();
                    self.movie_format_draft = naming::DEFAULT_MOVIE_FORMAT.into();
                }
            });
            ui.collapsing("Format tokens", |ui| {
                egui::Grid::new("filename_tokens")
                    .num_columns(2)
                    .spacing([18., 8.])
                    .show(ui, |ui| {
                        for (token, meaning) in [
                            ("[S] / [Season]", "S / Season"),
                            ("[E] / [Episode]", "E / Episode"),
                            ("[SeaNum]", "Season number"),
                            ("[EpsNum]", "Episode number"),
                            ("[Num]", "Two-digit numbers"),
                            ("[FixNum]", "Numbers without leading zeros"),
                            ("[Movie Name]", "Movie title from the filename"),
                            ("[Movie]", "Movie"),
                            ("[Space]", "One space"),
                        ] {
                            ui.monospace(token);
                            ui.label(meaning);
                            ui.end_row();
                        }
                    });
            });
        });
        ui.strong("Naming");
        card(ui, p, |ui| {
            toggle(
                ui,
                "Detect numbers in unformatted filenames",
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
            toggle(
                ui,
                "Number unrecognized files sequentially",
                &mut self.settings.manual_numbering,
                p,
            );
            if self.settings.manual_numbering {
                ui.horizontal(|ui| {
                    ui.label("First episode for unrecognized files");
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.settings.first_episode).range(0..=9999),
                        );
                    });
                });
            }
        });
        ui.strong("Folders and files");
        card(ui, p, |ui| {
            toggle(ui, "Include subfolders", &mut self.settings.subfolders, p);
            ui.separator();
            ui.label("Home filter options");
            container_options(ui, &mut self.settings, p);
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
            self.settings.manual_numbering = defaults.manual_numbering;
            self.settings.auto_name = defaults.auto_name;
            self.settings.clear_series_on_empty = defaults.clear_series_on_empty;
            self.settings.number_format = defaults.number_format.clone();
            self.format_draft = defaults.number_format;
            self.settings.movie_format = defaults.movie_format.clone();
            self.movie_format_draft = defaults.movie_format;
            self.settings.default_season = defaults.default_season;
            self.settings.first_episode = defaults.first_episode;
            self.settings.subfolders = defaults.subfolders;
            self.settings.filter_extensions = defaults.filter_extensions;
            self.settings.home_filter_options = defaults.home_filter_options;
            self.container_anchor = None;
            self.settings.remember_folder = defaults.remember_folder;
        }
        self.apply_preferences(previous, ui.ctx());
    }
    fn apply_preferences(&mut self, previous: Settings, ctx: &Context) {
        if self.settings != previous {
            if self.settings.home_filter_options != previous.home_filter_options {
                self.container_anchor = None;
            }
            if self.settings.remember_folder {
                self.settings.last_folder = self.root.clone()
            }
            self.save_settings();
            self.theme(ctx);
            let filters = self.settings.subfolders != previous.subfolders
                || self.settings.filter_extensions != previous.filter_extensions;
            if filters {
                if let Some(root) = self.root.clone() {
                    self.open(root, false, ctx)
                }
            } else if self.settings.auto_detect != previous.auto_detect
                || self.settings.manual_numbering != previous.manual_numbering
                || self.settings.default_season != previous.default_season
                || self.settings.first_episode != previous.first_episode
                || self.settings.number_format != previous.number_format
                || self.settings.movie_format != previous.movie_format
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
                    6 => "filters.png",
                    _ => "episode-marker.png",
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
            smoke.idle_frames = Some(self.frames - smoke.idle_start);
            smoke.stage = 18;
            smoke.last = Instant::now();
            if let Some(index) = self.rows.iter().position(|row| {
                row.source
                    .file_name()
                    .is_some_and(|name| name == "Unnumbered Movie clip.mp4")
            }) {
                self.set_media_marker(index, Some(engine::MediaKind::Episode));
            }
            ctx.request_repaint_after(Duration::from_millis(100));
            return;
        }
        if smoke.stage == 19 {
            if smoke.snapshots < 8 && smoke.last.elapsed() < Duration::from_secs(3) {
                ctx.request_repaint_after(Duration::from_millis(100));
                return;
            }
            let report = serde_json::json!({"language":"Rust","files":self.rows.len(),"screenshots":smoke.snapshots,"frames":self.frames,"max_visible_rows":self.max_visible_rows,"idle_repaints_1_8_seconds":smoke.idle_frames.unwrap_or(self.frames-smoke.idle_start),"mean_ui_work_ms":self.ui_micros as f64/self.frames.max(1)as f64/1000.,"window":"Anemorph","maximized_restore_verified":smoke.native_restore_verified});
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
                let _ = std::fs::write(
                    fixtures.join("[Group] Anemorph Movie [1080p].MOV"),
                    b"movie fixture",
                );
                let _ = std::fs::write(fixtures.join("Unnumbered Movie clip.mp4"), b"fixture");
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
                self.settings.auto_detect = false;
                self.rebuild();
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
            18 => ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default())),
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
        if self.integration.pending {
            self.integration.poll(None, ctx);
        }
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
        let fit = (base.x / 480.).min(base.y / 485.).max(0.5);
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
        if self.page == Page::Home
            && self.table_focus
            && !editing_name
            && !self.about_open
            && self.episode_dialog.is_none()
        {
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
            .frame(egui::Frame::NONE.fill(p.title_bar))
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
        self.episode_marker_dialog(ctx);
        self.smoke_tick(ctx);
        if let Some(start) = measured {
            self.ui_micros += start.elapsed().as_micros();
        }
    }
    fn on_exit(&mut self) {
        self.save_settings();
    }
}
