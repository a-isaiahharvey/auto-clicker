use std::sync::{mpsc::Sender, Arc, Mutex};

use egui::{self, DragValue, Response, Vec2};

#[derive(Debug, Default, Clone, Copy)]
pub struct ClickInterval {
    pub hours: usize,
    pub minutes: usize,
    pub seconds: usize,
    pub milliseconds: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum MouseButton {
    #[default]
    Left,
    Middle,
    Right,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ClickType {
    #[default]
    Single,
    Double,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ClickOptions {
    pub mouse_button: MouseButton,
    pub click_type: ClickType,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum ClickPosition {
    #[default]
    CurrentCursorPosition,
    Custom {
        x: usize,
        y: usize,
    },
}

pub struct MainApp {
    click_interval: ClickInterval,
    tx_click_interval: Sender<ClickInterval>,
    click_options: ClickOptions,
    tx_click_options: Sender<ClickOptions>,
    click_position: ClickPosition,
    tx_click_position: Sender<ClickPosition>,
    is_running: Arc<Mutex<bool>>,
}

impl MainApp {
    pub fn new(
        is_running: Arc<Mutex<bool>>,
        tx_click_interval: Sender<ClickInterval>,
        tx_click_options: Sender<ClickOptions>,
        tx_click_position: Sender<ClickPosition>,
    ) -> Self {
        let click_interval = ClickInterval::default();
        let click_options = ClickOptions::default();
        let click_position = ClickPosition::default();

        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            click_interval,
            tx_click_interval,
            click_options,
            tx_click_options,
            click_position,
            tx_click_position,
            is_running,
        }
    }
}

impl MainApp {
    pub fn update(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.heading("Click Interval");
                ui.horizontal(|ui| {
                    if ui
                        .add(egui::DragValue::new(&mut self.click_interval.hours))
                        .changed()
                    {
                        self.tx_click_interval.send(self.click_interval).unwrap();
                    };
                    ui.label("Hours");
                    if ui
                        .add(egui::DragValue::new(&mut self.click_interval.minutes))
                        .changed()
                    {
                        self.tx_click_interval.send(self.click_interval).unwrap();
                    };
                    ui.label("Minutes");
                    if ui
                        .add(egui::DragValue::new(&mut self.click_interval.seconds))
                        .changed()
                    {
                        self.tx_click_interval.send(self.click_interval).unwrap();
                    };
                    ui.label("Seconds");
                    if ui
                        .add(egui::DragValue::new(&mut self.click_interval.milliseconds))
                        .changed()
                    {
                        self.tx_click_interval.send(self.click_interval).unwrap();
                    };
                    ui.label("Milliseconds");
                })
            });

            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.set_width(408.5);
                    ui.vertical(|ui| {
                        ui.heading("Click Options");
                        egui::ComboBox::from_label("Mouse Button")
                            .selected_text(format!("{:?}", self.click_options.mouse_button))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                if ui
                                    .selectable_value(
                                        &mut self.click_options.mouse_button,
                                        MouseButton::Left,
                                        "Left",
                                    )
                                    .changed()
                                {
                                    self.tx_click_options.send(self.click_options).unwrap();
                                };
                                if ui
                                    .selectable_value(
                                        &mut self.click_options.mouse_button,
                                        MouseButton::Middle,
                                        "Middle",
                                    )
                                    .changed()
                                {};
                                if ui
                                    .selectable_value(
                                        &mut self.click_options.mouse_button,
                                        MouseButton::Right,
                                        "Right",
                                    )
                                    .changed()
                                {
                                    self.tx_click_options.send(self.click_options).unwrap();
                                };
                            });

                        egui::ComboBox::from_label("Click Type")
                            .selected_text(format!("{:?}", self.click_options.click_type))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                ui.selectable_value(
                                    &mut self.click_options.click_type,
                                    ClickType::Single,
                                    "Single",
                                );
                                ui.selectable_value(
                                    &mut self.click_options.click_type,
                                    ClickType::Double,
                                    "Double",
                                );
                            });
                    });
                });
            });

            ui.group(|ui| {
                ui.set_width(408.5);
                ui.heading("Click Position");

                if ui
                    .radio_value(
                        &mut self.click_position,
                        ClickPosition::CurrentCursorPosition,
                        "Current Cursor Position",
                    )
                    .changed()
                {
                    self.tx_click_position.send(self.click_position).unwrap();
                };

                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.click_position,
                        ClickPosition::Custom { x: 0, y: 0 },
                        "",
                    );
                    if let ClickPosition::Custom { x, y } = &mut self.click_position.clone() {
                        ui.label("X: ");
                        if ui.add(egui::DragValue::new(x)).changed() {
                            self.click_position = ClickPosition::Custom { x: *x, y: *y };
                            self.tx_click_position.send(self.click_position).unwrap();
                        };
                        ui.label("Y: ");
                        if ui.add(DragValue::new(y)).changed() {
                            self.click_position = ClickPosition::Custom { x: *x, y: *y };
                            self.tx_click_position.send(self.click_position).unwrap();
                        };
                    } else {
                        ui.label("X: ");
                        ui.add(egui::DragValue::new(&mut 0));
                        ui.label("Y: ");
                        ui.add(DragValue::new(&mut 0));
                    }
                });
            });

            ui.horizontal(|ui| {
                if create_button(ui, "Start (F6)").clicked() {
                    if let Ok(is_running) = &mut self.is_running.lock() {
                        **is_running = true;
                    }
                }
                ui.add_space(52.5);

                if create_button(ui, "Stop (F7)").clicked() {
                    if let Ok(is_running) = &mut self.is_running.lock() {
                        **is_running = false;
                    }
                }
                ui.add_space(52.5);

                if create_button(ui, "Toggle (F8)").clicked() {
                    if let Ok(is_running) = &mut self.is_running.lock() {
                        **is_running = !**is_running;
                    }
                }
            });
        });
    }
}

fn create_button(ui: &mut egui::Ui, text: &str) -> Response {
    let mut button = egui::Button::new(text);
    button = button.min_size(Vec2 { x: 100.0, y: 40.0 });

    ui.add(button)
}
