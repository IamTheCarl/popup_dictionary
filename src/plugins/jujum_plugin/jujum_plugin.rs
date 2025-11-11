use egui::Color32;
use egui::RichText;
use egui::Ui;
use std::path::PathBuf;

use crate::app::MyApp;
use crate::plugin::Plugin;
use crate::plugin::Token;
use crate::plugins::jujum_plugin::jmdict_dictionary::{
    Dictionary, DictionaryEntry, DictionaryTerm, Furigana,
};
use crate::plugins::jujum_plugin::jumandic_tokenizer::tokenize;

pub struct JujumPlugin {
    tokens: Vec<Token>,
    dictionary: Dictionary,
}

impl Plugin for JujumPlugin {
    fn load_plugin(sentence: &str) -> Self {
        println!("loading jmdict");
        let db_path: PathBuf = match dirs::config_dir() {
            Some(path) => path.join("popup_dictionary/db"),
            None => Err("No valid config path found in environment variables.").unwrap(),
        };
        let dictionary: Dictionary = Dictionary::load_dictionary(&db_path).unwrap();

        println!("tokenizing with jumandic");
        let tokens: Vec<Token> = tokenize(&sentence.to_string(), &dictionary).unwrap();

        Self { tokens, dictionary }
    }

    fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    fn display_token(
        &self,
        ctx: &egui::Context,
        frame: &egui::containers::Frame,
        app: &MyApp,
        ui: &mut Ui,
        token: &Token,
    ) {
        let forms_string: String = token
            .conjugations
            .iter()
            .map(|form| crate::plugins::jujum_plugin::jumandic_tokenizer::get_form(form))
            .collect::<Vec<&str>>()
            .join(", ");
        if forms_string != "*" {
            ui.scope(|ui| {
                ui.style_mut()
                    .visuals
                    .widgets
                    .noninteractive
                    .bg_stroke
                    .color = Color32::from_rgba_premultiplied(10, 10, 10, 10);
                ui.separator();
            });
            ui.label(
                RichText::new(format!("Forms: {}", forms_string))
                    .color(Color32::WHITE)
                    .size(14.0),
            );
        } else {
            ui.add_space(32.0);
        }
        ui.scope(|ui| {
            ui.style_mut()
                .visuals
                .widgets
                .noninteractive
                .bg_stroke
                .color = Color32::from_rgba_premultiplied(10, 10, 10, 10);
            ui.separator();
        });

        ui.style_mut().visuals.indent_has_left_vline = false;
        ui.style_mut().spacing.indent = 4.0;
        ui.indent("scroll_indent", |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    /*
                    Lookup in database in this order until exists:
                    1. base                                     -- first
                    2. surface
                    3. base minus last letter (e.g. 素敵な)
                    4. surface minus last letter                -- last
                    */
                    if let Some(dictionary_entry) = self
                        .dictionary
                        .lookup(&token.deinflected_word)
                        .expect(&format!(
                            "Error getting from database when looking up base: {}",
                            &token.deinflected_word
                        ))
                    {
                        self.display_terms_prioritized(ui, token, &dictionary_entry);
                    } else if let Some(dictionary_entry) =
                        self.dictionary.lookup(&token.input_word).expect(&format!(
                            "Error getting from database when looking up surface: {}",
                            &token.input_word
                        ))
                    {
                        self.display_terms_prioritized(ui, token, &dictionary_entry);
                    } else {
                        let mut base_minus_one: String = token.deinflected_word.clone();
                        _ = base_minus_one.pop();
                        if let Some(dictionary_entry) =
                            self.dictionary.lookup(&base_minus_one).expect(&format!(
                                "Error getting from database when looking up base-1: {}",
                                &base_minus_one
                            ))
                        {
                            self.display_terms_prioritized(ui, token, &dictionary_entry);
                        } else {
                            let mut surface_minus_one: String = token.input_word.clone();
                            _ = surface_minus_one.pop();
                            if let Some(dictionary_entry) =
                                self.dictionary.lookup(&surface_minus_one).expect(&format!(
                                    "Error getting from database when looking up surface-1: {}",
                                    &surface_minus_one
                                ))
                            {
                                self.display_terms_prioritized(ui, token, &dictionary_entry);
                            }
                        }
                    }

                    ui.add_space(40.0);
                });
        });
    }
}

impl JujumPlugin {
    fn display_terms_prioritized(&self, ui: &mut Ui, token: &Token, entry: &DictionaryEntry) {
        /*
        Display terms in this priority:
        1. no kanji, same as surface        -- first
        2. no kanji, same as base
        3. has kanji, same as surface
        4. has kanji, same as base
        5. rest                             -- last
        */

        let mut terms_to_display: Vec<DictionaryTerm> = entry.terms.clone();
        let mut filtered_terms: Vec<DictionaryTerm> = Vec::new();
        terms_to_display.retain_mut(|term| {
            if term.term.is_empty() && term.reading == token.input_word {
                filtered_terms.push(term.clone());
                false
            } else {
                true
            }
        });
        Self::display_terms(ui, &filtered_terms);
        filtered_terms.clear();
        terms_to_display.retain_mut(|term| {
            if term.term.is_empty() && term.reading == token.deinflected_word {
                filtered_terms.push(term.clone());
                false
            } else {
                true
            }
        });
        Self::display_terms(ui, &filtered_terms);
        filtered_terms.clear();
        terms_to_display.retain_mut(|term| {
            if !term.term.is_empty() && term.reading == token.input_word {
                filtered_terms.push(term.clone());
                false
            } else {
                true
            }
        });
        Self::display_terms(ui, &filtered_terms);
        filtered_terms.clear();
        terms_to_display.retain_mut(|term| {
            if !term.term.is_empty() && term.reading == token.deinflected_word {
                filtered_terms.push(term.clone());
                false
            } else {
                true
            }
        });
        Self::display_terms(ui, &filtered_terms);
        Self::display_terms(ui, &terms_to_display);
    }

    fn display_terms(ui: &mut Ui, terms: &Vec<DictionaryTerm>) {
        for dictionary_term in terms {
            if !dictionary_term.term.is_empty() {
                if let Some(furigana_vec) = &dictionary_term.furigana {
                    Self::display_furigana(ui, furigana_vec);
                } else {
                    let furigana: Vec<Furigana> = vec![Furigana {
                        ruby: dictionary_term.term.to_string(),
                        rt: Some(dictionary_term.reading.to_string()),
                    }];
                    Self::display_furigana(ui, &furigana);
                }
            } else {
                ui.label(
                    RichText::new(&dictionary_term.reading)
                        .size(22.0)
                        .color(Color32::WHITE),
                );
            }

            let mut count: u32 = 0;
            let mut last_tags: String = String::new();
            for meaning in &dictionary_term.meanings {
                let tags: String = meaning.tags.join("");
                if tags != last_tags {
                    last_tags = tags.clone();
                    if count > 0 {
                        ui.add_space(12.0);
                        count = 1;
                    }
                    ui.add_space(4.0);
                    Self::display_tags(ui, &meaning.tags);
                }
                if count == 0 {
                    count = 1;
                }

                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!("{}.", count))
                            .size(18.0)
                            .color(Color32::GRAY),
                    );
                    ui.label(
                        RichText::new(format!("{}", meaning.gloss.join(", ")))
                            .size(18.0)
                            .color(Color32::WHITE),
                    );
                });
                if meaning.info.len() > 0 {
                    ui.horizontal_top(|ui| {
                        ui.label(
                            RichText::new(format!("{}.", count))
                                .size(18.0)
                                .color(Color32::TRANSPARENT),
                        );
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(format!("{}", meaning.info.join("; ")))
                                    .size(12.0)
                                    .color(Color32::GRAY),
                            );
                        });
                    });
                }

                count += 1;
            }

            ui.add_space(10.0);

            let percent: f32 = 0.8;
            let width: f32 = ui.available_width() * percent;
            let margin: f32 = (ui.available_width() - width) / 2.0;

            ui.horizontal(|ui| {
                ui.add_space(margin);
                let rect: egui::Rect = ui.allocate_space(egui::vec2(width, 1.0)).1;
                ui.painter().line_segment(
                    [rect.left_center(), rect.right_center()],
                    egui::Stroke::new(1.0, Color32::from_rgba_premultiplied(20, 20, 20, 20)),
                );
            });

            ui.add_space(10.0);
        }
    }

    fn display_tags(ui: &mut Ui, tags: &Vec<String>) {
        ui.horizontal_wrapped(|ui| {
            for tag in tags {
                Self::display_tag(ui, tag);
            }
        });
    }

    fn display_tag(ui: &mut Ui, tag: &str) {
        let text_color: Color32 = Color32::WHITE;

        let text_galley = ui.fonts_mut(|f| {
            f.layout_no_wrap(
                tag.to_string(),
                egui::FontId::proportional(14.0),
                text_color,
            )
        });

        let padding = egui::Vec2::new(4.0, 0.0);
        let rect = egui::Rect::from_min_size(ui.cursor().min, text_galley.size() + (2.0 * padding));
        let response = ui
            .allocate_rect(rect, egui::Sense::hover())
            .on_hover_text(Dictionary::get_tag(tag));

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Help);
        }

        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(4),
            Color32::from_rgb(50, 50, 50),
        );

        ui.painter().galley(
            (rect.center() - text_galley.size() / 2.0) - egui::Vec2::new(0.0, 2.0),
            text_galley,
            text_color,
        );

        //ui.allocate_space(rect.size());
    }

    fn display_furigana(ui: &mut Ui, furigana_vec: &Vec<Furigana>) {
        let main_font_size: f32 = 22.0;
        let furigana_font_size: f32 = 14.0;
        let vertical_gap: f32 = 1.0;

        // calculate how wide (and tall) the entire string will be
        let mut total_width: f32 = 0.0;
        let mut max_height: f32 = 0.0;
        let mut galley_data = Vec::new();

        for furigana in furigana_vec {
            let main_galley = ui.fonts_mut(|f| {
                f.layout_no_wrap(
                    furigana.ruby.to_string(),
                    egui::FontId::proportional(main_font_size),
                    Color32::WHITE,
                )
            });

            let furigana_galley = if let Some(reading) = &furigana.rt {
                ui.fonts_mut(|f| {
                    f.layout_no_wrap(
                        reading.to_string(),
                        egui::FontId::proportional(furigana_font_size),
                        Color32::LIGHT_GRAY,
                    )
                })
            } else {
                ui.fonts_mut(|f| {
                    f.layout_no_wrap(
                        "あ".to_string(), // invisible placeholder
                        egui::FontId::proportional(furigana_font_size),
                        Color32::TRANSPARENT,
                    )
                })
            };

            let char_width: f32 = main_galley.size().x.max(furigana_galley.size().x);
            let char_height: f32 = main_galley.size().y + furigana_galley.size().y + vertical_gap;

            total_width += char_width;
            max_height = max_height.max(char_height);

            galley_data.push((main_galley, furigana_galley, char_width));
        }

        // then draw without gap between galleys
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(total_width, max_height),
            egui::Sense::empty(),
        );

        let mut current_x: f32 = rect.left();
        for (main_galley, furigana_galley, char_width) in galley_data {
            let furigana_pos = egui::Pos2::new(
                current_x + (char_width - furigana_galley.size().x) * 0.5,
                rect.top(),
            );
            ui.painter()
                .galley(furigana_pos, furigana_galley, Color32::PLACEHOLDER);

            let main_pos = egui::Pos2::new(
                current_x + (char_width - main_galley.size().x) * 0.5,
                rect.top() + furigana_font_size + vertical_gap,
            );
            ui.painter()
                .galley(main_pos, main_galley, Color32::PLACEHOLDER);

            current_x += char_width;
        }
    }
}
