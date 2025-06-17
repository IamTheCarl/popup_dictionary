use crate::{
    dictionary::{Dictionary, DictionaryEntry, DictionaryTerm, Furigana},
    tokenizer::ParsedWord,
};
use eframe::{
    NativeOptions, egui,
    epaint::text::{FontInsert, InsertFontFamily},
};
use egui::{Color32, RichText, Ui};

pub fn run_app(words: &Vec<ParsedWord>, dictionary: &Dictionary) -> Result<(), eframe::Error> {
    // Configure native window options
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0]) // Initial window size
            .with_min_inner_size([320.0, 240.0]) // Minimum window size
            .with_title("Popup Dictionary"), // Window title
        ..Default::default()
    };

    // Run the eframe application
    eframe::run_native(
        "Popup Dictionary", // The name of your application
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc, words, dictionary)))),
    )
}

struct MyApp {
    words: Vec<ParsedWord>,
    selected: usize,
    dictionary: Dictionary,
}

impl MyApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        words: &Vec<ParsedWord>,
        dictionary: &Dictionary,
    ) -> Self {
        // You can load initial state here if needed
        add_font(&cc.egui_ctx);
        Self {
            words: words.to_vec(),
            selected: 0,
            dictionary: dictionary.clone(),
        }
    }

    fn display_terms_prioritized(&self, ui: &mut Ui, entry: &DictionaryEntry) {
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
            if term.term.is_empty() && term.reading == self.words[self.selected].surface {
                filtered_terms.push(term.clone());
                false
            } else {
                true
            }
        });
        Self::display_terms(ui, &filtered_terms);
        filtered_terms.clear();
        terms_to_display.retain_mut(|term| {
            if term.term.is_empty() && term.reading == self.words[self.selected].base {
                filtered_terms.push(term.clone());
                false
            } else {
                true
            }
        });
        Self::display_terms(ui, &filtered_terms);
        filtered_terms.clear();
        terms_to_display.retain_mut(|term| {
            if !term.term.is_empty() && term.reading == self.words[self.selected].surface {
                filtered_terms.push(term.clone());
                false
            } else {
                true
            }
        });
        Self::display_terms(ui, &filtered_terms);
        filtered_terms.clear();
        terms_to_display.retain_mut(|term| {
            if !term.term.is_empty() && term.reading == self.words[self.selected].base {
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
            ui.horizontal(|ui| {
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

                /*
                if let Some(frequency) = dictionary_term.frequency {
                    ui.label(
                        RichText::new(format!("freq:{}", frequency))
                            .size(12.0)
                            .color(Color32::WHITE),
                    );
                }*/
            });

            let mut count: u32 = 1;
            let mut last_tags: String = String::new();
            for meaning in &dictionary_term.meanings {
                let mut tags: String = meaning.tags.join(", ");
                if tags == last_tags {
                    tags = String::from("-");
                } else {
                    last_tags = tags.clone();
                    if count > 1 {
                        ui.add_space(16.0);
                    }
                }

                ui.columns(2, |columns| {
                    columns[0].set_height(20.0);
                    columns[0].horizontal_wrapped(|ui| {
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

                    columns[1].set_height(20.0);
                    columns[1].with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.add_space(20.0);
                                ui.label(
                                    RichText::new(format!("{}", tags))
                                        .size(14.0)
                                        .color(Color32::GRAY),
                                );
                            });
                        },
                    );
                });
                if meaning.info.len() > 0 {
                    ui.horizontal_wrapped(|ui| {
                        ui.horizontal_top(|ui| {
                            ui.label(
                                RichText::new(format!("{}.", count))
                                    .size(18.0)
                                    .color(Color32::TRANSPARENT),
                            );
                            ui.label(
                                RichText::new(format!("{}", meaning.info.join(", ")))
                                    .size(12.0)
                                    .color(Color32::GRAY),
                            );
                        });
                    });
                }

                count += 1;
            }

            ui.add_space(10.0);
            ui.scope(|ui| {
                ui.style_mut()
                    .visuals
                    .widgets
                    .noninteractive
                    .bg_stroke
                    .color = Color32::from_rgba_premultiplied(20, 20, 20, 20);
                ui.separator();
            });
            ui.add_space(10.0);
        }
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
            let main_galley = ui.fonts(|f| {
                f.layout_no_wrap(
                    furigana.ruby.to_string(),
                    egui::FontId::proportional(main_font_size),
                    Color32::WHITE,
                )
            });

            let furigana_galley = if let Some(reading) = &furigana.rt {
                ui.fonts(|f| {
                    f.layout_no_wrap(
                        reading.to_string(),
                        egui::FontId::proportional(furigana_font_size),
                        Color32::LIGHT_GRAY,
                    )
                })
            } else {
                ui.fonts(|f| {
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.heading("Search:");
            ui.horizontal_wrapped(|ui| {
                for (index, word) in self.words.iter().enumerate() {
                    let mut label_text = RichText::new(format!("{}", word.surface)).size(20.0);
                    if word.is_valid() {
                        label_text = label_text.underline();
                        if index == self.selected {
                            label_text = label_text.color(Color32::WHITE);
                        }
                        let label = ui
                            .label(label_text.clone())
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        if label.hovered() {
                            label.clone().highlight();
                        }
                        if label.clicked() {
                            self.selected = index;
                        }
                    } else {
                        ui.label(label_text.clone());
                    }
                }
            });

            ui.add_space(10.0);
            ui.heading("Definition:");

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
                        .lookup(&self.words[self.selected].base)
                        .expect(&format!(
                            "Error getting from database when looking up base: {}",
                            &self.words[self.selected].base
                        ))
                    {
                        self.display_terms_prioritized(ui, &dictionary_entry);
                    } else if let Some(dictionary_entry) = self
                        .dictionary
                        .lookup(&self.words[self.selected].surface)
                        .expect(&format!(
                            "Error getting from database when looking up surface: {}",
                            &self.words[self.selected].surface
                        ))
                    {
                        self.display_terms_prioritized(ui, &dictionary_entry);
                    } else {
                        let mut base_minus_one: String = self.words[self.selected].base.clone();
                        _ = base_minus_one.pop();
                        if let Some(dictionary_entry) =
                            self.dictionary.lookup(&base_minus_one).expect(&format!(
                                "Error getting from database when looking up base-1: {}",
                                &base_minus_one
                            ))
                        {
                            self.display_terms_prioritized(ui, &dictionary_entry);
                        } else {
                            let mut surface_minus_one: String =
                                self.words[self.selected].surface.clone();
                            _ = surface_minus_one.pop();
                            if let Some(dictionary_entry) =
                                self.dictionary.lookup(&surface_minus_one).expect(&format!(
                                    "Error getting from database when looking up surface-1: {}",
                                    &surface_minus_one
                                ))
                            {
                                self.display_terms_prioritized(ui, &dictionary_entry);
                            }
                        }
                    }

                    ui.add_space(40.0);
                });
        });
        egui::TopBottomPanel::bottom("footer")
            .min_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
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
                    }
                })
            });
    }
}

fn add_font(ctx: &egui::Context) {
    ctx.add_font(FontInsert::new(
        "NotoSansCJKJP",
        egui::FontData::from_static(include_bytes!("./fonts/popup_font.ttc")),
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

fn get_sentence_string(words: &Vec<ParsedWord>) -> String {
    let mut sentence: String = String::new();
    for word in words {
        sentence.push_str(&format!("{}", word.surface));
    }
    sentence
}
