use eframe::{
    NativeOptions, egui,
    epaint::text::{FontInsert, InsertFontFamily},
};
use egui::{Color32, Context, CornerRadius, Pos2, Rect, RichText};
use log::{error, warn};
use std::sync::{Arc, Mutex};

use crate::plugin::{Plugin, Plugins, Token};

pub const WINDOW_INIT_WIDTH: i16 = 450;
pub const WINDOW_INIT_HEIGHT: i16 = 450;
pub const APP_NAME: &str = "Popup Dictionary";

pub struct Config {
    pub initial_plugin: Option<String>,
    pub open_at_cursor: bool,
    pub wrapped: bool,
}

pub fn run_app(sentence: &str, config: Config) -> Result<(), eframe::Error> {
    #[cfg(feature = "hyprland-support")]
    let is_hyprland: bool = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();

    let mut init_pos: Option<Pos2> = None;
    let options: NativeOptions;
    if config.open_at_cursor {
        match crate::window_helper::get_optimal_init_pos(
            #[cfg(feature = "hyprland-support")]
            is_hyprland,
        ) {
            Ok(optimal_pos) => {
                init_pos = Some(optimal_pos);

                options = NativeOptions {
                    viewport: egui::ViewportBuilder::default()
                        .with_position(optimal_pos)
                        .with_inner_size([WINDOW_INIT_WIDTH as f32, WINDOW_INIT_HEIGHT as f32])
                        .with_min_inner_size([100.0, 100.0])
                        .with_title(APP_NAME),
                    ..Default::default()
                };
            }
            Err(e) => {
                warn!("Failed to get optimal init pos with error: {:?}", e);
                options = NativeOptions {
                    viewport: egui::ViewportBuilder::default()
                        .with_inner_size([WINDOW_INIT_WIDTH as f32, WINDOW_INIT_HEIGHT as f32])
                        .with_min_inner_size([100.0, 100.0])
                        .with_title(APP_NAME),
                    ..Default::default()
                };
            }
        }
    } else {
        options = NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([WINDOW_INIT_WIDTH as f32, WINDOW_INIT_HEIGHT as f32])
                .with_min_inner_size([100.0, 100.0])
                .with_title(APP_NAME),
            ..Default::default()
        };
    }

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            Ok(Box::new(MyApp::new(
                cc,
                config,
                init_pos,
                #[cfg(feature = "hyprland-support")]
                is_hyprland,
                sentence,
            )))
        }),
    )
}

enum PluginState {
    Initial,
    Loading,
    Ready(Box<dyn Plugin>),
}

pub struct MyApp {
    config: Config,
    init_pos: Option<Pos2>,
    #[cfg(feature = "hyprland-support")]
    is_hyprland: bool,
    sentence: String,
    selected_token_index: Option<usize>,
    plugin_state: Arc<Mutex<PluginState>>,
    available_plugins: Vec<Plugins>,
    active_plugin_index: usize,
}

impl MyApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        config: Config,
        init_pos: Option<Pos2>,
        #[cfg(feature = "hyprland-support")] is_hyprland: bool,
        sentence: &str,
    ) -> Self {
        Self::load_main_font(&cc.egui_ctx);

        let available_plugins: Vec<Plugins> = Plugins::all();
        println!("plugin: {:?}", config.initial_plugin);
        let init_plugin_idx: usize = if let Some(init_plugin) = &config.initial_plugin {
            available_plugins
                .iter()
                .position(|p| p.name() == init_plugin)
                .unwrap_or(0)
        } else {
            0
        };
        println!("plugin: {}", init_plugin_idx);

        let mut app = Self {
            config,
            init_pos,
            #[cfg(feature = "hyprland-support")]
            is_hyprland,
            sentence: sentence.to_string(),
            selected_token_index: None,
            plugin_state: Arc::new(Mutex::new(PluginState::Initial)),
            available_plugins,
            active_plugin_index: init_plugin_idx,
        };

        app.try_load_plugin(init_plugin_idx);

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

        self.selected_token_index = None;
        self.active_plugin_index = plugin_index;
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Some(init_pos) = self.init_pos {
            #[cfg(feature = "hyprland-support")]
            if self.is_hyprland {
                if let Err(e) =
                    crate::window_helper::move_window_hyprland(init_pos.x as i16, init_pos.y as i16)
                {
                    error!("{}", e);
                } else {
                    self.init_pos = None;
                }
            }

            #[cfg(not(feature = "wayland-support"))]
            if let Err(e) =
                crate::window_helper::move_window_x11(init_pos.x as i32, init_pos.y as i32)
            {
                error!("{}", e);
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

        egui::CentralPanel::default()
            .frame(main_frame)
            .show(ctx, |ui| {
                let available_height = ui.available_height();
                let mut header_height = 42.0;
                let footer_height = 42.0;

                match &(*self.plugin_state.lock().unwrap()) {
                    PluginState::Ready(plugin) => {
                        let tokens: &Vec<Token> = plugin.get_tokens();

                        if self.selected_token_index.is_none() {
                            let mut first_valid_idx: usize = 0;
                            let mut curr_idx: usize = 0;
                            while curr_idx < tokens.len() {
                                if tokens[curr_idx].is_valid() {
                                    first_valid_idx = curr_idx;
                                    break;
                                }
                                curr_idx += 1;
                            }
                            self.selected_token_index = Some(first_valid_idx);
                        }
                        let selected_token_idx: usize = self.selected_token_index.unwrap();

                        let token_header = egui::ScrollArea::horizontal()
                            .id_salt("token_header")
                            //.max_height(header_height)
                            .show(ui, |ui| {
                                if self.config.wrapped {
                                    ui.horizontal_wrapped(|ui| {
                                        if let Some(idx) =
                                            display_token_header(ui, tokens, selected_token_idx)
                                        {
                                            self.selected_token_index = Some(idx);
                                        }
                                    });
                                } else {
                                    ui.horizontal(|ui| {
                                        if let Some(idx) =
                                            display_token_header(ui, tokens, selected_token_idx)
                                        {
                                            self.selected_token_index = Some(idx);
                                        }
                                    });
                                }

                                ui.add_space(10.0);
                            });

                        header_height = token_header.inner_rect.height();

                        ui.separator();
                        let space_left = ui.available_height();
                        let center_height = space_left - footer_height;
                        println!("available: {}, header height: {}, footer height: {}, center height: {}, left: {}", available_height, header_height, footer_height, center_height, space_left);
                        egui::ScrollArea::vertical()
                            .id_salt("plugin_display_section")
                            .max_height(center_height)
                            .auto_shrink(false)
                            .show(ui, |ui| {
                                plugin.display_token(
                                    ctx,
                                    &main_frame,
                                    self,
                                    ui,
                                    &tokens[selected_token_idx],
                                );
                            });
                    }
                    _ => {
                        let center_height = available_height - footer_height + 2.0;
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), center_height),
                            egui::Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| {
                                ui.horizontal(|ui| {
                                    // horizontal centering by ms-eevee on github:
                                    //
                                    // We create a closure function containing our elements, as we will be calling it twice.
                                    // Any additional elements to be centered would go within this closure.
                                    let elements = |ui: &mut egui::Ui| {
                                        ui.spinner();
                                        ui.add(egui::Label::new(
                                            RichText::new("Loading Plugin...").size(20.0),
                                        ));
                                    };

                                    // Create a new child Ui with the invisible flag set so that the element does not actually
                                    // get shown on the GUI.
                                    // As a sidenote, we are taking advantage of the fact that new_child() does not allocate any of
                                    // the widget's space in the parent UI, so we are free to draw as much as we want without
                                    // advancing the parent's cursor.
                                    let mut hidden =
                                        ui.new_child(egui::UiBuilder::new().invisible());

                                    // Call our elements closure, passing in the invisible Ui child to be rendered.
                                    elements(&mut hidden);

                                    // We get the size of the rendered elements through min_rect() here as well.
                                    let diff: f32 = hidden.min_rect().width();

                                    // Add a space before rendering the element to the main screen.
                                    ui.add_space((ui.max_rect().width() / 2.) - (diff / 2.));
                                    // Finally, render the elements to the main UI.
                                    elements(ui);
                                });
                            },
                        );
                        //ctx.request_repaint();
                    }
                }


                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width(), footer_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                    // Calculate right-side bar width
                    let button_width: f32 = 18.0;
                    let button_spacing: f32 = ui.spacing().item_spacing.x;
                    let num_buttons: f32 = 3.0;
                    let fixed_area_width = (button_width * num_buttons as f32)
                        + (button_spacing * (num_buttons - 1.0))
                        + button_spacing * 2.0;

                    let available_width: f32 = ui.available_width() - fixed_area_width;
                    egui::ScrollArea::horizontal()
                        .id_salt("plugin_footer")
                        .max_height(footer_height)
                        .max_width(available_width)
                        .show(ui, |ui| {
                            ui.horizontal_centered(|ui| {
                                let mut clicked_idx: Option<usize> = None;
                                for (idx, active_plugin) in
                                    self.available_plugins.iter().enumerate()
                                {
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
                                    if self.active_plugin_index != idx {
                                        self.try_load_plugin(idx);
                                    }
                                }
                            });
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new(RichText::new("⚙").size(16.0)))
                            .clicked()
                        {
                            // Settings button
                        }
                        if ui
                            .add(egui::Button::new(RichText::new("📋").size(16.0)))
                            .clicked()
                        {
                            // Copy button
                        }
                        if ui
                            .add(egui::Button::new(RichText::new("⭐").size(16.0)))
                            .clicked()
                        {
                            if let PluginState::Ready(plugin) =
                                &(*self.plugin_state.lock().unwrap())
                            {
                                plugin.open(ctx);
                            }
                        }
                    });
                });
            });
    }
}

fn display_token_header(
    ui: &mut egui::Ui,
    tokens: &Vec<Token>,
    selected_token_idx: usize,
) -> Option<usize> {
    for (idx, token) in tokens.iter().enumerate() {
        let font_size: f32 = 20.0;
        let mut label_text: RichText = RichText::new(&token.input_word).size(font_size);
        if token.is_valid() {
            label_text = label_text.underline().color(Color32::GRAY);
            if idx == selected_token_idx {
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
            let (background_rect, _) = ui.allocate_exact_size(text_size, egui::Sense::hover());
            let label_rect: Rect = Rect::from_center_size(background_rect.center(), text_size);

            let response = ui
                .scope_builder(egui::UiBuilder::new().max_rect(label_rect), |ui| {
                    ui.label(label_text)
                })
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
                return Some(idx);
            }
        } else {
            ui.label(label_text);
        }
    }
    return None;
}
