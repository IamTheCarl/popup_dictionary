use std::{
    env,
    sync::{Arc, Mutex},
};

/*
use crate::{
    dictionary::{Dictionary, DictionaryEntry, DictionaryTerm, Furigana},
    tokenizer::ParsedWord,
};*/
use eframe::{
    NativeOptions, egui,
    epaint::text::{FontInsert, InsertFontFamily},
};
use egui::{Color32, CornerRadius, Pos2, RichText};
use enigo::{Enigo, Mouse};

use std::error::Error;

#[cfg(feature = "hyprland-support")]
use hyprland::prelude::*;
#[cfg(feature = "hyprland-support")]
use hyprland::{
    dispatch::{self, DispatchType},
    shared::HyprDataActive,
};

use crate::plugin::{Plugin, Plugins, Token};

fn get_optimal_window_pos(
    window_width: f32,
    window_height: f32,
    is_hyprland: bool,
) -> Result<Pos2, Box<dyn Error>> {
    let mut cursor_pos: Pos2 = Pos2::ZERO;
    let mut display_size: Pos2 = Pos2::ZERO;
    let mut cursor_found: bool = false;
    let mut display_found: bool = false;
    'outer: {
        #[cfg(feature = "hyprland-support")]
        // try hyprland
        if is_hyprland {
            if let Ok(pos) = hyprland::data::CursorPosition::get() {
                cursor_pos = Pos2::new(pos.x as f32, pos.y as f32);
                cursor_found = true;
            }
            if let Ok(monitor) = hyprland::data::Monitor::get_active() {
                display_size = Pos2::new(
                    (monitor.width as i32 + monitor.x) as f32,
                    (monitor.height as i32 + monitor.y) as f32,
                );
                display_found = true;
            }
            if cursor_found && display_found {
                break 'outer;
            }
        }
        println!("{} | {}", cursor_pos, display_size);

        // try x11/wayland/windows/macos
        // this can report wrong values, so make sure not to overwrite previous good values using cursor_found and display_found
        let enigo = Enigo::new(&enigo::Settings::default()).unwrap();
        if !cursor_found {
            if let Ok((x, y)) = enigo.location() {
                cursor_pos = Pos2::new(x as f32, y as f32);
                cursor_found = true;
            }
        }
        if !display_found {
            if let Ok((x, y)) = enigo.main_display() {
                display_size = Pos2::new(x as f32, y as f32);
                display_found = true;
            }
        }

        println!("{} | {}", cursor_pos, display_size);
        if cursor_found && display_found {
            break 'outer;
        }

        // try xwayland workaround
        let mut settings = enigo::Settings::default();

        settings.wayland_display = Some(":0".to_string());
        let enigo = Enigo::new(&settings).unwrap();
        if !cursor_found {
            if let Ok((x, y)) = enigo.location() {
                cursor_pos = Pos2::new(x as f32, y as f32);
                cursor_found = true;
            }
        }
        if !display_found {
            if let Ok((x, y)) = enigo.main_display() {
                display_size = Pos2::new(x as f32, y as f32);
                display_found = true;
            }
        }
        println!("{} | {}", cursor_pos, display_size);
    }

    println!("{} | {}", cursor_pos, display_size);
    if cursor_found && display_found {
        if display_size.x >= cursor_pos.x && display_size.y >= cursor_pos.y {
            let mut window_x = cursor_pos.x;
            let mut window_y = cursor_pos.y;

            if window_x + window_width > display_size.x {
                window_x -= window_width;
            }

            if window_y + window_height > display_size.y {
                window_y -= window_height;
            }

            return Ok(Pos2::new(window_x, window_y));
        } else {
            return Err(Box::from(
                "Couldn't find valid cursor position and/or display size.",
            ));
        }
    } else {
        return Err(Box::from(
            "Couldn't find valid cursor position and/or display size.",
        ));
    }
}

pub fn run_app(sentence: &str) -> Result<(), eframe::Error> {
    let mut start_pos: Option<Pos2> = None;

    let is_hyprland: bool = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();

    let options;
    if let Ok(optimal_pos) = get_optimal_window_pos(450.0, 450.0, is_hyprland) {
        start_pos = Some(optimal_pos);

        println!("start_pos: {}, {}", optimal_pos.x, optimal_pos.y);
        options = NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_position(optimal_pos)
                .with_inner_size([450.0, 450.0]) // Initial window size
                .with_min_inner_size([450.0, 450.0]) // Minimum window size
                .with_title("Popup Dictionary"), // Window title
            ..Default::default()
        };
    } else {
        println!("mouse not found");
        options = NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([450.0, 450.0]) // Initial window size
                .with_min_inner_size([450.0, 450.0]) // Minimum window size
                .with_title("Popup Dictionary"), // Window title
            ..Default::default()
        };
    }

    // Configure native window options

    // Run the eframe application
    eframe::run_native(
        "Popup Dictionary", // The name of your application
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc, start_pos, is_hyprland, sentence)))),
    )
}

enum PluginState {
    Loading,
    Ready(Box<dyn Plugin>),
}

pub struct MyApp {
    start_pos: Option<Pos2>,
    is_hyprland: bool,
    //words: Vec<ParsedWord>,
    sentence: String,
    selected_word_index: usize,
    //dictionary: Dictionary,
    plugin_state: Arc<Mutex<PluginState>>,
    available_plugins: Vec<Plugins>,
    active_plugin_index: usize,
}

impl MyApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        start_pos: Option<Pos2>,
        is_hyprland: bool,
        sentence: &str,
    ) -> Self {
        // You can load initial state here if needed
        Self::load_main_font(&cc.egui_ctx);

        let app = Self {
            start_pos,
            is_hyprland,
            //words: words.to_vec(),
            sentence: sentence.to_string(),
            selected_word_index: 0,
            //dictionary: dictionary.clone(),
            plugin_state: Arc::new(Mutex::new(PluginState::Loading)),
            available_plugins: Plugins::all(),
            active_plugin_index: 0,
        };

        app.load_active_plugin();

        app
    }

    fn load_main_font(ctx: &egui::Context) {
        ctx.add_font(FontInsert::new(
            "NotoSansCJKJP",
            egui::FontData::from_static(include_bytes!("./fonts/popup_font.ttc")), // Currently: Noto Sans CJK JP
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

    fn load_active_plugin(&self) {
        let state_clone = self.plugin_state.clone();
        *state_clone.lock().unwrap() = PluginState::Loading;

        let active_plugin = self.available_plugins[self.active_plugin_index];
        let plugin_sentence = String::from(&self.sentence);
        std::thread::spawn(move || {
            let plugin = active_plugin.generate(&plugin_sentence);
            *state_clone.lock().unwrap() = PluginState::Ready(plugin);
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(start_pos) = self.start_pos {
            if self.is_hyprland {
                #[cfg(feature = "hyprland-support")]
                {
                    let window_id = dispatch::WindowIdentifier::Title("Popup Dictionary");
                    if hyprland::dispatch::Dispatch::call(DispatchType::ResizeWindowPixel(
                        dispatch::Position::Exact(450, 450),
                        window_id.to_owned(),
                    ))
                    .is_ok()
                        && hyprland::dispatch::Dispatch::call(DispatchType::MoveWindowPixel(
                            hyprland::dispatch::Position::Exact(
                                start_pos.x as i16,
                                start_pos.y as i16,
                            ),
                            window_id,
                        ))
                        .is_ok()
                    {
                        self.start_pos = None;
                    }
                }
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(start_pos));
                self.start_pos = None;
            }
        }

        let custom_frame = egui::containers::Frame {
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

        egui::TopBottomPanel::bottom("footer")
            .min_height(40.0)
            .frame(custom_frame)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    for (i, active_plugin) in self.available_plugins.iter().enumerate() {
                        if ui
                            .add(egui::Button::selectable(
                                self.active_plugin_index == i,
                                RichText::new(active_plugin.name()).size(20.0),
                            ))
                            .clicked()
                        {
                            self.selected_word_index = 0;
                            self.active_plugin_index = i;
                            self.load_active_plugin();
                        }
                    }

                    /*
                    if ui
                        .button(RichText::new("Open in browser").size(20.0))
                        .clicked()
                    {
                        ctx.open_url(egui::output::OpenUrl {
                            url: format!(
                                "https://jotoba.de/search/0/{}?l=en-US",
                                get_sentence_string(&self.words)
                            ),
                            new_tab: true,
                        });
                    }*/
                })
            });

        egui::CentralPanel::default()
            .frame(custom_frame)
            .show(ctx, |ui| match &*self.plugin_state.lock().unwrap() {
                PluginState::Loading => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() / 2.0 - 20.0);
                        ui.spinner();
                        ui.add(egui::Label::new(
                            RichText::new("Loading Plugin...").size(20.0),
                        ));
                    });
                    ctx.request_repaint();
                }
                PluginState::Ready(plugin) => {
                    let tokens: &Vec<Token> = plugin.get_tokens();
                    ui.horizontal_top(|ui| {
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            ui.set_min_height(42.0);

                            for (index, word) in tokens.iter().enumerate() {
                                let font_size: f32 = 20.0;
                                let mut label_text: RichText =
                                    RichText::new(format!("{}", word.input_word)).size(font_size);
                                if word.is_valid() {
                                    label_text = label_text.underline().color(Color32::GRAY);
                                    if index == self.selected_word_index {
                                        label_text = label_text.color(Color32::WHITE);
                                    }

                                    let text_size: egui::Vec2 = {
                                        let temp_galley = ui.fonts_mut(|f| {
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
                                    let label_rect = egui::Rect::from_center_size(
                                        background_rect.center(),
                                        text_size,
                                    );

                                    let response = ui
                                        .scope_builder(
                                            egui::UiBuilder::new().max_rect(label_rect),
                                            |ui| ui.label(label_text),
                                        )
                                        .inner;
                                    if response.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    if response.hovered() {
                                        ui.painter().rect_filled(
                                            background_rect,
                                            egui::CornerRadius::same(4),
                                            Color32::from_rgba_premultiplied(40, 40, 40, 40),
                                        );
                                    }
                                    if response.clicked() {
                                        self.selected_word_index = index;
                                    }
                                } else {
                                    ui.label(label_text.clone());
                                }
                            }
                        });
                    });

                    ui.add_space(10.0);

                    plugin.display_token(
                        ctx,
                        &custom_frame,
                        self,
                        ui,
                        &tokens[self.selected_word_index],
                    );
                }
            });
    }
}

/*
fn get_sentence_string(words: &Vec<ParsedWord>) -> String {
    let mut sentence: String = String::new();
    for word in words {
        sentence.push_str(&format!("{}", word.surface));
    }
    sentence
}
*/
