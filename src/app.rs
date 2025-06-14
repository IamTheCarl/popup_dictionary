use crate::{dictionary::Dictionary, tokenizer::ParsedWord};
use eframe::{
    NativeOptions, egui,
    epaint::text::{FontInsert, InsertFontFamily},
};
use egui::{Color32, RichText};

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
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Sentence:");
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
                    if let Some(dictionary_entry) = self
                        .dictionary
                        .lookup(&self.words[self.selected].base)
                        .expect(&format!(
                            "Could not find {}",
                            &self.words[self.selected].base
                        ))
                    {
                        for dictionary_term in &dictionary_entry.terms {
                            if !dictionary_term.term.is_empty() {
                                ui.label(
                                    RichText::new(&dictionary_term.term)
                                        .size(22.0)
                                        .color(Color32::WHITE),
                                );
                            }
                            if !dictionary_term.reading.is_empty() {
                                ui.label(
                                    RichText::new(&dictionary_term.reading)
                                        .size(22.0)
                                        .color(Color32::WHITE),
                                );
                            }
                            let mut count: u32 = 1;
                            for meaning in &dictionary_term.meanings {
                                ui.label(
                                    RichText::new(format!("{}. {}", count, meaning)).size(18.0),
                                );
                                count += 1;
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
