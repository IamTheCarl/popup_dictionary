use crate::plugin::Plugin;
use crate::plugin::Token;

pub struct JotobaPlugin {
    tokens: Vec<Token>,
}

impl Plugin for JotobaPlugin {
    fn load_plugin(sentence: &str) -> Self {
        Self {
            tokens: vec![Token {
                input_word: "placeholder".to_string(),
                deinflected_word: "placeholder".to_string(),
                conjugations: vec!["placeholder".to_string()],
                validity: crate::plugin::Validity::UNKNOWN,
            }],
        }
    }

    fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    fn display_token(&self, app: &crate::app::MyApp, ui: &mut egui::Ui, token: &Token) {}
}
