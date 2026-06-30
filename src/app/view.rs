use super::{ActiveTab, ConnectionStatus, EnemyColor, RadarApp};
use crate::services::script_runner::LaserScript;
use crate::state::{LaserSnapshot, SdrSnapshot};
use crate::theme;
use crate::widgets::{LaserPanel, StatusPanels};

impl RadarApp {
    pub(super) fn show_mode_rail(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                if let Some(texture) = self.logo_texture.as_ref() {
                    ui.add(
                        egui::Image::from_texture(texture)
                            .fit_to_exact_size(egui::vec2(34.0, 34.0))
                            .corner_radius(egui::CornerRadius::same(255)),
                    );
                } else {
                    let (logo_rect, _) =
                        ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(logo_rect.center(), 17.0, theme::BLUE_SOFT);
                    ui.painter().text(
                        logo_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "R",
                        egui::FontId::proportional(16.0),
                        theme::BLUE,
                    );
                }

                ui.add_space(8.0);
                self.show_mode_button(ui, "◎", ActiveTab::Sdr, "SDR");
                ui.add_space(8.0);
                self.show_mode_button(ui, "◈", ActiveTab::Laser, "Laser");
                ui.add_space(8.0);
                self.show_mode_button(ui, "◉", ActiveTab::Radar, "Radar");
            });

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!("{} pkt", self.data_count))
                    .color(theme::text_muted())
                    .size(12.0),
            );
            ui.label(
                egui::RichText::new(format!("{}s", self.start_time.elapsed().as_secs()))
                    .color(theme::text_faint())
                    .size(12.0),
            );
            ui.add_space(ui.available_height().max(0.0));
        });
    }

    fn show_mode_button(&mut self, ui: &mut egui::Ui, title: &str, tab: ActiveTab, subtitle: &str) {
        let selected = self.active_tab == tab;
        let fill = if selected {
            theme::BLUE
        } else {
            theme::card_bg()
        };
        let stroke = if selected {
            egui::Stroke::NONE
        } else {
            egui::Stroke::new(1.0, theme::border())
        };
        let text_color = if selected {
            theme::text_on_dark()
        } else {
            theme::text()
        };
        let sub_color = if selected {
            theme::BLUE_SOFT
        } else {
            theme::text_faint()
        };

        let response = egui::Frame::new()
            .fill(fill)
            .stroke(stroke)
            .corner_radius(egui::CornerRadius::same(14))
            .inner_margin(egui::Margin::symmetric(8, 10))
            .show(ui, |ui| {
                ui.set_min_width(42.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(title).color(text_color).size(18.0));
                    ui.add_space(2.0);
                    ui.label(egui::RichText::new(subtitle).color(sub_color).size(9.0));
                });
            })
            .response
            .interact(egui::Sense::click());

        if response.clicked() {
            self.active_tab = tab;
        }
    }

    pub(super) fn show_radar_sidebar(
        &mut self,
        ui: &mut egui::Ui,
        radar_snapshot: Option<&SdrSnapshot>,
    ) {
        Self::white_card(ui, "连接", |ui| {
            Self::status_chip(
                ui,
                self.connection_status == ConnectionStatus::Connected,
                "Signal feed",
            );
            ui.add_space(12.0);
            egui::Grid::new("radar_conn_grid")
                .num_columns(2)
                .min_col_width(78.0)
                .spacing([12.0, 10.0])
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("IP")
                            .color(theme::text_muted())
                            .size(13.0),
                    );
                    ui.add(egui::TextEdit::singleline(&mut self.ip).desired_width(f32::INFINITY));
                    ui.end_row();
                    ui.label(
                        egui::RichText::new("Port")
                            .color(theme::text_muted())
                            .size(13.0),
                    );
                    ui.add(egui::TextEdit::singleline(&mut self.port).desired_width(f32::INFINITY));
                    ui.end_row();
                });
            ui.add_space(12.0);
            if ui
                .add_sized(
                    [ui.available_width(), 32.0],
                    egui::Button::new("Reconnect radar stream"),
                )
                .clicked()
            {
                self.reconnect();
            }
            ui.add_space(8.0);
            egui::Grid::new("radar_meta_grid")
                .num_columns(2)
                .min_col_width(78.0)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Packets")
                            .color(theme::text_faint())
                            .size(12.0),
                    );
                    ui.label(
                        egui::RichText::new(self.data_count.to_string())
                            .color(theme::text())
                            .size(12.0),
                    );
                    ui.end_row();
                    ui.label(
                        egui::RichText::new("Last live")
                            .color(theme::text_faint())
                            .size(12.0),
                    );
                    let age = self
                        .last_update
                        .map(|last| format!("{:.1}s", last.elapsed().as_secs_f32()))
                        .unwrap_or_else(|| "--".to_string());
                    ui.label(egui::RichText::new(age).color(theme::text()).size(12.0));
                    ui.end_row();
                });

            if let Some(err) = &self.error_message {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(err).color(theme::RED).size(12.0));
            }
        });

        ui.add_space(14.0);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                StatusPanels::new().show(ui, radar_snapshot.map(|snapshot| &snapshot.signal));
            });
    }

    pub(super) fn show_laser_sidebar(
        &mut self,
        ui: &mut egui::Ui,
        laser_snapshot: Option<&LaserSnapshot>,
    ) {
        let laser_online = laser_snapshot.is_some_and(|snapshot| snapshot.online);
        let laser_listening = self.laser_runtime.is_started();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                Self::white_card(ui, "数据源", |ui| {
                    Self::status_chip(ui, laser_listening, "Laser UDP Listening");
                    ui.add_space(8.0);
                    Self::status_chip(
                        ui,
                        laser_online,
                        if laser_online {
                            "Receiving"
                        } else {
                            "No recent packets"
                        },
                    );
                    ui.add_space(12.0);
                    egui::Grid::new("laser_conn_grid")
                        .num_columns(2)
                        .min_col_width(78.0)
                        .spacing([12.0, 10.0])
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Port")
                                    .color(theme::text_muted())
                                    .size(13.0),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut self.laser_port)
                                    .desired_width(f32::INFINITY),
                            );
                            ui.end_row();
                            ui.label(
                                egui::RichText::new("Camera")
                                    .color(theme::text_muted())
                                    .size(13.0),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut self.camera_device)
                                    .desired_width(f32::INFINITY),
                            );
                            ui.end_row();
                        });
                    ui.add_space(12.0);
                    if ui
                        .add_sized(
                            [ui.available_width(), 32.0],
                            egui::Button::new("Reconnect laser listener"),
                        )
                        .clicked()
                    {
                        self.reconnect_laser();
                    }
                });

                ui.add_space(14.0);
                Self::white_card(ui, "脚本控制", |ui| {
                    let running = self.process_control.is_running();
                    let daemon_ok = self.process_control.daemon_alive();
                    let active_label = self
                        .process_control
                        .active()
                        .map(|s| s.label())
                        .unwrap_or("Idle");

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("状态:")
                                .color(theme::text_muted())
                                .size(13.0),
                        );
                        Self::status_chip(ui, running, active_label);
                    });
                    if daemon_ok && !running {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("daemon 存活 (可通过流控制发送命令)")
                                .color(theme::text_faint())
                                .size(11.0),
                        );
                    }
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("敌方颜色:")
                                .color(theme::text_muted())
                                .size(13.0),
                        );
                        egui::ComboBox::from_id_salt("enemy_color")
                            .selected_text(self.enemy_color.label())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.enemy_color, EnemyColor::Red, "Red");
                                ui.selectable_value(
                                    &mut self.enemy_color,
                                    EnemyColor::Blue,
                                    "Blue",
                                );
                                ui.selectable_value(
                                    &mut self.enemy_color,
                                    EnemyColor::Auto,
                                    "Auto",
                                );
                            });
                    });
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.stream_on_start, "启动时推流");
                    ui.checkbox(&mut self.record_on_start, "启动时内录");
                    ui.add_space(6.0);
                    let scripts = [
                        [LaserScript::Competition, LaserScript::Preview],
                        [LaserScript::Stream, LaserScript::Record],
                    ];
                    ui.columns(2, |columns| {
                        for (row_index, row) in scripts.iter().enumerate() {
                            for (column, script) in columns.iter_mut().zip(row.iter()) {
                                let label = script.label();
                                if column
                                    .add_sized(
                                        [column.available_width(), 30.0],
                                        egui::Button::new(label),
                                    )
                                    .clicked()
                                {
                                    let result = if script.is_daemon() {
                                        let enemy_cmd = self.enemy_color.fifo_cmd().to_owned();
                                        let stream_cmd = if self.stream_on_start {
                                            "stream on"
                                        } else {
                                            "stream off"
                                        }
                                        .to_owned();
                                        let record_cmd = if self.record_on_start {
                                            "record on"
                                        } else {
                                            "record off"
                                        }
                                        .to_owned();
                                        self.process_control.start_script_with_daemon_config(
                                            *script,
                                            &self.camera_device,
                                            enemy_cmd,
                                            stream_cmd,
                                            record_cmd,
                                        )
                                    } else {
                                        self.process_control
                                            .start_script(*script, &self.camera_device)
                                    };

                                    if let Err(e) = result {
                                        log::error!("Failed to start {}: {}", label, e);
                                    }
                                }
                            }
                            if row_index + 1 < scripts.len() {
                                for column in &mut columns[..] {
                                    column.add_space(6.0);
                                }
                            }
                        }
                    });
                    if running {
                        ui.add_space(10.0);
                        if ui
                            .add_sized([ui.available_width(), 30.0], egui::Button::new("Stop"))
                            .clicked()
                        {
                            self.process_control.stop_script();
                        }
                    }
                });

                ui.add_space(14.0);
                Self::white_card(ui, "比赛进程", |ui| {
                    let sdr_ok = self.process_control.is_sdr_running();
                    let unity_ok = self.process_control.is_unity_running();
                    let start_all_pending = self.process_control.has_pending_start_all();

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("SDR:")
                                .color(theme::text_muted())
                                .size(13.0),
                        );
                        Self::status_chip(ui, sdr_ok, if sdr_ok { "Running" } else { "Idle" });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if sdr_ok {
                                if ui
                                    .add_sized([72.0, 24.0], egui::Button::new("Stop"))
                                    .clicked()
                                {
                                    self.process_control.stop_sdr();
                                }
                            } else if ui
                                .add_sized([72.0, 24.0], egui::Button::new("Start"))
                                .clicked()
                            {
                                if let Err(e) =
                                    self.process_control.start_sdr(self.enemy_color.sdr_arg())
                                {
                                    log::error!("Failed to start SDR: {}", e);
                                }
                            }
                        });
                    });
                    ui.add_space(2.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Radar:")
                                .color(theme::text_muted())
                                .size(13.0),
                        );
                        Self::status_chip(ui, unity_ok, if unity_ok { "Running" } else { "Idle" });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if unity_ok {
                                if ui
                                    .add_sized([72.0, 24.0], egui::Button::new("Stop"))
                                    .clicked()
                                {
                                    self.process_control.stop_unity();
                                }
                            } else if ui
                                .add_sized([72.0, 24.0], egui::Button::new("Start"))
                                .clicked()
                            {
                                if let Err(e) = self.process_control.start_unity() {
                                    log::error!("Failed to start Unity: {}", e);
                                }
                            }
                        });
                    });

                    ui.add_space(10.0);
                    if ui
                        .add_enabled(
                            !start_all_pending,
                            egui::Button::new(if start_all_pending {
                                "Starting..."
                            } else {
                                "Start All (SDR → Laser Competition)"
                            }),
                        )
                        .clicked()
                    {
                        let enemy_cmd = self.enemy_color.fifo_cmd().to_owned();
                        let stream_cmd = if self.stream_on_start {
                            "stream on"
                        } else {
                            "stream off"
                        }
                        .to_owned();
                        let record_cmd = if self.record_on_start {
                            "record on"
                        } else {
                            "record off"
                        }
                        .to_owned();
                        if let Err(e) = self.process_control.schedule_start_all(
                            self.enemy_color.sdr_arg(),
                            &self.camera_device,
                            enemy_cmd,
                            stream_cmd,
                            record_cmd,
                        ) {
                            log::error!("Start All failed: {}", e);
                        }
                    }

                    if sdr_ok || unity_ok || self.process_control.is_running() {
                        ui.add_space(6.0);
                        if ui
                            .add_sized([ui.available_width(), 30.0], egui::Button::new("Stop All"))
                            .clicked()
                        {
                            self.process_control.stop_all();
                        }
                    }
                });

                ui.add_space(14.0);
                Self::white_card(ui, "流控制", |ui| {
                    ui.columns(2, |columns| {
                        if columns[0]
                            .add_sized(
                                [columns[0].available_width(), 32.0],
                                egui::Button::new("Stream on"),
                            )
                            .clicked()
                        {
                            self.process_control.send_laser_command("stream on");
                        }
                        if columns[1]
                            .add_sized(
                                [columns[1].available_width(), 32.0],
                                egui::Button::new("Stream off"),
                            )
                            .clicked()
                        {
                            self.process_control.send_laser_command("stream off");
                        }
                    });
                });

                ui.add_space(14.0);
                LaserPanel::new().show_analysis_sidebar(
                    ui,
                    laser_snapshot.map(|snapshot| &snapshot.observation),
                );
            });
    }

    fn white_card(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::new()
            .fill(theme::card_bg())
            .stroke(egui::Stroke::new(1.0, theme::border()))
            .corner_radius(egui::CornerRadius::same(18))
            .shadow(egui::epaint::Shadow {
                offset: [0, 8],
                blur: 24,
                spread: 0,
                color: theme::shadow(),
            })
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(title).color(theme::text()).size(16.0));
                });
                ui.add_space(10.0);
                add_contents(ui);
            });
    }

    fn status_chip(ui: &mut egui::Ui, ok: bool, label: &str) {
        let fill = if ok {
            theme::success_bg()
        } else {
            theme::error_bg()
        };
        let text = if ok { theme::GREEN } else { theme::RED };
        egui::Frame::new()
            .fill(fill)
            .corner_radius(egui::CornerRadius::same(255))
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(format!("● {}", label))
                        .color(text)
                        .size(12.0),
                );
            });
    }
}
