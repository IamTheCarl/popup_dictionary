use egui::Color32;
use egui::Context;
use egui::RichText;
use egui::Ui;
use egui::containers::Frame;
use std::cell::RefCell;
use std::error::Error;

use crate::app;
use crate::app::MyApp;
use crate::plugin::Plugin;
use crate::plugin::Token;
use crate::plugins::jotoba_plugin::jotoba_tokenizer::Furigana;
use crate::plugins::jotoba_plugin::jotoba_tokenizer::JotobaTokenizer;

pub struct JotobaPlugin {
    tokens: Vec<Token>,
    jotoba_tokenizer: RefCell<JotobaTokenizer>, // TODO: REMOVE THIS REFCELL WHEN POSSIBLE
}

impl Plugin for JotobaPlugin {
    fn load_plugin(sentence: &str) -> Self {
        let mut jotoba_tokenizer: JotobaTokenizer = JotobaTokenizer::new();
        match jotoba_tokenizer.tokenize(sentence) {
            Ok(tokens) => Self {
                tokens,
                jotoba_tokenizer: RefCell::from(jotoba_tokenizer),
            },
            Err(e) => {
                // TODO: Add proper error handling.
                tracing::error!("Failed to tokenize input text with Jotoba due to error: {e}");
                panic!("{e}");
            }
        }
    }

    fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    fn display_token(&self, ctx: &Context, frame: &Frame, app: &MyApp, ui: &mut Ui, token: &Token) {
        if token.is_valid() {
            match self.jotoba_tokenizer.borrow_mut().get_response(token) {
                Ok(response) => {
                    /*
                    egui::TopBottomPanel::bottom("jotoba_footer")
                        .show_separator_line(false)
                        .frame(*frame)
                        .show(ctx, |ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(RichText::new("Open").size(20.0)).clicked() {
                                    ctx.open_url(egui::output::OpenUrl {
                                        url: format!(
                                            "https://jotoba.de/search/0/{}?l=en-US",
                                            self.get_sentence_string()
                                        ),
                                        new_tab: true,
                                    });
                                }
                            });
                        });
                    */
                    /*
                    egui::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, |ui| {*/
                    for word in &response.words {
                        if let Some(furigana) = &word.reading.furigana {
                            Self::display_furigana(ui, &furigana.furigana);
                        } else {
                            if let Some(kanji) = &word.reading.kanji {
                                tracing::warn!(
                                    "Kanji {} without furigana in Jotoba response.",
                                    kanji
                                );
                                ui.label(RichText::new(kanji).heading()); //.size(22.0));
                            } else {
                                ui.label(
                                    RichText::new(&word.reading.kana).heading(), //.size(22.0)
                                );
                            }
                        }

                        let mut count: u32 = 1;
                        for sense in &word.senses {
                            ui.label(
                                RichText::new(format!("{}. {}", count, sense.glosses.join(", ")))
                                    .small(),
                            );
                            count += 1;
                        }
                    }
                    /* });*/
                }
                Err(e) => tracing::debug!("Could not display token due to error: {e}"),
            };
        }
    }

    fn open(&self, ctx: &Context) {
        tracing::info!("Trying to open Jotoba website with input text.");

        match self.build_sanitized_url() {
            Ok(url) => {
                ctx.open_url(egui::OpenUrl::new_tab(url));
            }
            Err(e) => {
                tracing::warn!("Could not build Jotoba URL due to error: {}", e);
            }
        }
    }
}

impl JotobaPlugin {
    fn get_sentence_string(&self) -> String {
        self.tokens
            .iter()
            .map(|token| token.input_word.to_owned())
            .collect::<Vec<String>>()
            .join("")
    }

    fn build_sanitized_url(&self) -> Result<String, Box<dyn Error>> {
        let mut url =
            reqwest::Url::parse_with_params("https://jotoba.de/search/0/", &[("l", "en-US")])
                .map_err(|e| e.to_string())?;

        url.path_segments_mut()
            .map_err(|_| "URL cannot be a base")?
            .push(&self.get_sentence_string());

        Ok(url.to_string())
    }

    // TODO: Same exact function copied over from kihon_plugin. Perhaps unify?
    fn display_furigana(ui: &mut Ui, furigana_vec: &Vec<Furigana>) {
        let vertical_gap: f32 = 1.0;

        // calculate how wide (and tall) the entire string will be
        let mut total_width: f32 = 0.0;
        let mut max_height: f32 = 0.0;
        let mut galley_data = Vec::new();

        for furigana in furigana_vec {
            let main_galley = ui.fonts_mut(|f| {
                f.layout_no_wrap(
                    furigana.ruby.to_string(),
                    egui::FontId::proportional(app::BIG_TEXT_SIZE),
                    app::PRIMARY_TEXT_COLOR,
                )
            });

            let furigana_galley = if let Some(reading) = &furigana.rt {
                ui.fonts_mut(|f| {
                    f.layout_no_wrap(
                        reading.to_string(),
                        egui::FontId::proportional(app::TINY_TEXT_SIZE),
                        app::LIGHT_TEXT_COLOR,
                    )
                })
            } else {
                ui.fonts_mut(|f| {
                    f.layout_no_wrap(
                        "あ".to_string(), // invisible placeholder
                        egui::FontId::proportional(app::TINY_TEXT_SIZE),
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
                rect.top() + app::TINY_TEXT_SIZE + vertical_gap,
            );
            ui.painter()
                .galley(main_pos, main_galley, Color32::PLACEHOLDER);

            current_x += char_width;
        }
    }
}
