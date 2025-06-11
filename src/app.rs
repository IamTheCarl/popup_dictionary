use crate::parser::ParsedWord;
use eframe::{
    NativeOptions, egui,
    epaint::text::{FontInsert, InsertFontFamily},
};
use egui::{Color32, RichText};

pub fn run_app(words: &Vec<ParsedWord>) -> Result<(), eframe::Error> {
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
        Box::new(|cc| Ok(Box::new(MyApp::new(cc, words)))),
    )
}

struct MyApp {
    words: Vec<ParsedWord>,
    selected: usize,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>, words: &Vec<ParsedWord>) -> Self {
        // You can load initial state here if needed
        add_font(&cc.egui_ctx);
        Self {
            words: words.to_vec(),
            selected: 0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Sentence:");
            ui.horizontal_wrapped(|ui| {
                for (index, word) in self.words.iter().enumerate() {
                    let mut label_text = RichText::new(format!("{}", word)).size(20.0);
                    match word {
                        ParsedWord::Valid(_) => {
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
                        }
                        ParsedWord::Invalid(_) => {
                            ui.label(label_text.clone());
                        }
                    };
                }
            });
            ui.add_space(10.0);
            ui.heading("Definition:");

            // You can add more widgets here, for example, a button:
            if ui.button("Open in browser").clicked() {
                //self.text = "Button clicked!".to_owned();
            }
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
