use egui::Color32;
use egui::RichText;
use egui::Ui;
use std::cell::RefCell;

use crate::app::MyApp;
use crate::plugin::Plugin;
use crate::plugin::Token;
use crate::plugins::jotoba_plugin::jotoba_tokenizer::JotobaTokenizer;

pub struct JotobaPlugin {
    tokens: Vec<Token>,
    jotoba_tokenizer: RefCell<JotobaTokenizer>,
}

impl Plugin for JotobaPlugin {
    fn load_plugin(sentence: &str) -> Self {
        let mut jotoba_tokenizer = JotobaTokenizer::new();
        let tokens = jotoba_tokenizer.tokenize(sentence).unwrap();
        Self {
            tokens,
            jotoba_tokenizer: RefCell::from(jotoba_tokenizer),
        }
    }

    fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    fn display_token(&self, app: &MyApp, ui: &mut Ui, token: &Token) {
        if token.is_valid() {
            if let Ok(response) = self.jotoba_tokenizer.borrow_mut().get_response(token) {
                for word in &response.words {
                    if let Some(kanji) = &word.reading.kanji {
                        ui.label(RichText::new(kanji).size(22.0).color(Color32::WHITE));
                    } else {
                        ui.label(
                            RichText::new(&word.reading.kana)
                                .size(22.0)
                                .color(Color32::WHITE),
                        );
                    }
                    let mut count: u32 = 1;
                    for sense in &word.senses {
                        ui.label(
                            RichText::new(format!("{}. {}", count, sense.glosses.join(", ")))
                                .size(18.0),
                        );
                        count += 1;
                    }
                }
            }
        }
    }
}
