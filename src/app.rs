use eframe::{
    NativeOptions, egui,
    epaint::text::{FontInsert, InsertFontFamily},
};
use egui::{Color32, Context, CornerRadius, Pos2, Rect, RichText};
use enigo::{Enigo, Mouse, Settings};
use log::warn;
use std::error::Error;
use std::sync::{Arc, Mutex};

#[cfg(feature = "hyprland-support")]
use hyprland::prelude::*;

use crate::plugin::{Plugin, Plugins, Token};

const WINDOW_INIT_WIDTH: f32 = 450.0;
const WINDOW_INIT_HEIGHT: f32 = 450.0;
const APP_NAME: &str = "Popup Dictionary";

pub fn run_app(sentence: &str) -> Result<(), eframe::Error> {
    #[cfg(feature = "hyprland-support")]
    let is_hyprland: bool = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();

    let mut init_pos: Option<Pos2> = None;
    let options: NativeOptions;
    match get_optimal_init_pos(
        #[cfg(feature = "hyprland-support")]
        is_hyprland,
    ) {
        Ok(optimal_pos) => {
            init_pos = Some(optimal_pos);

            options = NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_position(optimal_pos)
                    .with_inner_size([WINDOW_INIT_WIDTH, WINDOW_INIT_HEIGHT])
                    .with_min_inner_size([100.0, 100.0])
                    .with_title(APP_NAME),
                ..Default::default()
            };
        }
        Err(e) => {
            warn!("Failed to get optimal init pos with error: {:?}", e);
            options = NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([WINDOW_INIT_WIDTH, WINDOW_INIT_HEIGHT])
                    .with_min_inner_size([100.0, 100.0])
                    .with_title(APP_NAME),
                ..Default::default()
            };
        }
    }

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            Ok(Box::new(MyApp::new(
                cc,
                init_pos,
                #[cfg(feature = "hyprland-support")]
                is_hyprland,
                sentence,
            )))
        }),
    )
}

fn get_optimal_init_pos(
    #[cfg(feature = "hyprland-support")] is_hyprland: bool,
) -> Result<Pos2, Box<dyn Error>> {
    let mut cursor_pos: Option<Pos2> = None;
    let mut display_size: Option<Pos2> = None;
    'outer: {
        #[cfg(feature = "hyprland-support")]
        if is_hyprland {
            use hyprland::data::{CursorPosition, Monitor};

            if let Ok(pos) = CursorPosition::get() {
                cursor_pos = Some(Pos2::new(pos.x as f32, pos.y as f32));
            }
            if let Ok(monitor) = Monitor::get_active() {
                display_size = Some(Pos2::new(
                    (monitor.width as i32 + monitor.x) as f32,
                    (monitor.height as i32 + monitor.y) as f32,
                ));
            }

            if cursor_pos.is_some() && display_size.is_some() {
                break 'outer;
            }
        }

        // try x11/wayland/windows/macos
        // this can report wrong values, so making sure not to overwrite previous good values
        let enigo: Enigo = Enigo::new(&Settings::default())?;
        if cursor_pos.is_none() {
            if let Ok((x, y)) = enigo.location() {
                cursor_pos = Some(Pos2::new(x as f32, y as f32));
            }
        }
        if !display_size.is_none() {
            if let Ok((x, y)) = enigo.main_display() {
                display_size = Some(Pos2::new(x as f32, y as f32));
            }
        }

        if cursor_pos.is_some() && display_size.is_some() {
            break 'outer;
        }

        #[cfg(feature = "wayland-support")]
        {
            // try xwayland workaround
            let mut settings: Settings = Settings::default();
            settings.wayland_display = Some(":0".to_string());
            let enigo: Enigo = Enigo::new(&settings)?;
            if cursor_pos.is_none() {
                if let Ok((x, y)) = enigo.location() {
                    cursor_pos = Some(Pos2::new(x as f32, y as f32));
                }
            }
            if !display_size.is_none() {
                if let Ok((x, y)) = enigo.main_display() {
                    display_size = Some(Pos2::new(x as f32, y as f32));
                }
            }
        }
    }

    if let Some(cursor_pos) = cursor_pos
        && let Some(display_size) = display_size
    {
        if display_size.x >= cursor_pos.x && display_size.y >= cursor_pos.y {
            let mut window_x: f32 = cursor_pos.x;
            let mut window_y: f32 = cursor_pos.y;

            if window_x + WINDOW_INIT_WIDTH > display_size.x {
                window_x -= WINDOW_INIT_WIDTH;
            }

            if window_y + WINDOW_INIT_HEIGHT > display_size.y {
                window_y -= WINDOW_INIT_HEIGHT;
            }

            return Ok(Pos2::new(window_x, window_y));
        } else {
            return Err(Box::from(format!(
                "Cursor position ({}, {}) outside display bounds ({}, {}).",
                cursor_pos.x, cursor_pos.y, display_size.x, display_size.y
            )));
        }
    } else {
        return Err(Box::from(
            "No valid cursor position and/or display size found.",
        ));
    }
}

enum PluginState {
    Initial,
    Loading,
    Ready(Box<dyn Plugin>),
}

pub struct MyApp {
    init_pos: Option<Pos2>,
    #[cfg(feature = "hyprland-support")]
    is_hyprland: bool,
    sentence: String,
    selected_word_index: usize,
    plugin_state: Arc<Mutex<PluginState>>,
    available_plugins: Vec<Plugins>,
    active_plugin_index: usize,
}

impl MyApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        init_pos: Option<Pos2>,
        #[cfg(feature = "hyprland-support")] is_hyprland: bool,
        sentence: &str,
    ) -> Self {
        Self::load_main_font(&cc.egui_ctx);

        let mut app = Self {
            init_pos,
            #[cfg(feature = "hyprland-support")]
            is_hyprland,
            sentence: sentence.to_string(),
            selected_word_index: 0,
            plugin_state: Arc::new(Mutex::new(PluginState::Initial)),
            available_plugins: Plugins::all(),
            active_plugin_index: 0,
        };

        app.try_load_plugin(0);

        app
    }

    fn load_main_font(ctx: &Context) {
        ctx.add_font(FontInsert::new(
            "NotoSansCJKJP",
            #[cfg(not(target_os = "windows"))]
            egui::FontData::from_static(include_bytes!("./fonts/popup_font.ttc")), // Currently: Noto Sans CJK JP
            #[cfg(target_os = "windows")]
            egui::FontData::from_static(include_bytes!(".\\fonts\\popup_font.ttc")), // Currently: Noto Sans CJK JP
            vec![
                InsertFontFamily {
                    family: egui::FontFamily::Proportional,
                    priority: egui::epaint::text::FontPriority::Highest,
                },
                InsertFontFamily {
                    family: egui::FontFamily::Monospace,
                    priority: egui::epaint::text::FontPriority::Lowest,
                },
            ],
        ));
    }

    fn try_load_plugin(&mut self, plugin_index: usize) {
        if plugin_index >= self.available_plugins.len() {
            return;
        }

        let state_clone: Arc<Mutex<PluginState>> = Arc::clone(&self.plugin_state);
        {
            let mut state = state_clone.lock().unwrap();
            match *state {
                PluginState::Loading => {
                    return;
                }
                PluginState::Ready(_) => {
                    if self.active_plugin_index == plugin_index {
                        return;
                    }
                    *state = PluginState::Loading;
                }
                _ => {
                    *state = PluginState::Loading;
                }
            }
        }

        let active_plugin: Plugins = self.available_plugins[plugin_index];
        let plugin_sentence: String = self.sentence.to_owned();
        std::thread::spawn(move || {
            // TODO: Implement error handling, logging?
            let plugin: Box<dyn Plugin> = active_plugin.generate(&plugin_sentence);
            *state_clone.lock().unwrap() = PluginState::Ready(plugin);
        });

        self.selected_word_index = 0;
        self.active_plugin_index = plugin_index;
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Some(init_pos) = self.init_pos {
            #[cfg(feature = "hyprland-support")]
            if self.is_hyprland {
                {
                    use hyprland::dispatch::{Dispatch, DispatchType, Position, WindowIdentifier};

                    let window_id: WindowIdentifier<'_> = WindowIdentifier::Title(APP_NAME);
                    if Dispatch::call(DispatchType::ResizeWindowPixel(
                        Position::Exact(WINDOW_INIT_WIDTH as i16, WINDOW_INIT_HEIGHT as i16),
                        window_id.to_owned(),
                    ))
                    .is_ok()
                        && Dispatch::call(DispatchType::MoveWindowPixel(
                            Position::Exact(init_pos.x as i16, init_pos.y as i16),
                            window_id,
                        ))
                        .is_ok()
                    {
                        self.init_pos = None;
                    }
                }
            }

            #[cfg(not(feature = "hyprland-support"))]
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(init_pos));
                self.init_pos = None;
            }
        }

        let main_frame = egui::containers::Frame {
            corner_radius: CornerRadius::ZERO,
            fill: Color32::from_rgb(27, 28, 29),
            inner_margin: egui::Margin {
                left: 2,
                right: 2,
                top: 2,
                bottom: 2,
            },
            outer_margin: egui::Margin {
                left: 2,
                right: 2,
                top: 2,
                bottom: 2,
            },
            shadow: egui::Shadow::NONE,
            stroke: egui::Stroke::NONE,
        };

        egui::TopBottomPanel::bottom("plugins_footer")
            .min_height(40.0)
            .frame(main_frame)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let mut clicked_idx: Option<usize> = None;
                    for (idx, active_plugin) in self.available_plugins.iter().enumerate() {
                        if ui
                            .add(egui::Button::selectable(
                                self.active_plugin_index == idx,
                                RichText::new(active_plugin.name()).size(20.0),
                            ))
                            .clicked()
                        {
                            clicked_idx = Some(idx);
                        }
                    }
                    if let Some(idx) = clicked_idx {
                        self.try_load_plugin(idx);
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(main_frame)
            .show(ctx, |ui| match &(*self.plugin_state.lock().unwrap()) {
                PluginState::Ready(plugin) => {
                    let tokens: &Vec<Token> = plugin.get_tokens();
                    ui.horizontal_top(|ui| {
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            ui.set_min_height(42.0);

                            for (idx, token) in tokens.iter().enumerate() {
                                let font_size: f32 = 20.0;
                                let mut label_text: RichText =
                                    RichText::new(&token.input_word).size(font_size);
                                if token.is_valid() {
                                    label_text = label_text.underline().color(Color32::GRAY);
                                    if idx == self.selected_word_index {
                                        label_text = label_text.color(Color32::WHITE);
                                    }

                                    let text_size: egui::Vec2 = {
                                        let temp_galley: Arc<egui::Galley> = ui.fonts_mut(|f| {
                                            f.layout_no_wrap(
                                                label_text.text().to_string(),
                                                egui::FontId::proportional(font_size),
                                                Color32::PLACEHOLDER,
                                            )
                                        });
                                        temp_galley.size()
                                    };
                                    let (background_rect, _) =
                                        ui.allocate_exact_size(text_size, egui::Sense::hover());
                                    let label_rect: Rect =
                                        Rect::from_center_size(background_rect.center(), text_size);

                                    let response = ui
                                        .scope_builder(
                                            egui::UiBuilder::new().max_rect(label_rect),
                                            |ui| ui.label(label_text),
                                        )
                                        .inner;
                                    if response.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        ui.painter().rect_filled(
                                            background_rect,
                                            CornerRadius::same(4),
                                            Color32::from_rgba_premultiplied(40, 40, 40, 40),
                                        );
                                    }
                                    if response.clicked() {
                                        self.selected_word_index = idx;
                                    }
                                } else {
                                    ui.label(label_text);
                                }
                            }
                        });
                    });

                    ui.add_space(10.0);

                    plugin.display_token(
                        ctx,
                        &main_frame,
                        self,
                        ui,
                        &tokens[self.selected_word_index],
                    );
                }
                _ => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() / 2.0 - 20.0);
                        ui.spinner();
                        ui.add(egui::Label::new(
                            RichText::new("Loading Plugin...").size(20.0),
                        ));
                    });
                    ctx.request_repaint();
                }
            });
    }
}
