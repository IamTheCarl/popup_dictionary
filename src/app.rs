use crate::parser::ParsedWord;
use eframe::{NativeOptions, egui};

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
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>, words: &Vec<ParsedWord>) -> Self {
        // You can load initial state here if needed
        Self {
            words: words.to_vec(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Sentence:");
            //ui.label(&self.text);

            // You can add more widgets here, for example, a button:
            if ui.button("Open in browser").clicked() {
                //self.text = "Button clicked!".to_owned();
            }
        });
    }
}
