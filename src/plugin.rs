use egui::Ui;

use crate::app::MyApp;

pub trait Plugin: Send + 'static {
    fn load_plugin(sentence: &str) -> Self
    where
        Self: Sized;
    fn get_tokens(&self) -> &Vec<Token>;
    fn display_token(&self, app: &MyApp, ui: &mut Ui, token: &Token);
}

#[derive(Clone, Copy, PartialEq)]
pub enum Plugins {
    Jujum,
    Jotoba,
}

impl Plugins {
    pub fn all() -> Vec<Self> {
        vec![Plugins::Jujum, Plugins::Jotoba]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Plugins::Jujum => "jmdict+jumandic",
            Plugins::Jotoba => "jotoba",
        }
    }

    pub fn generate(&self, sentence: &str) -> Box<dyn Plugin> {
        match self {
            Plugins::Jujum => Box::new(
                crate::plugins::jujum_plugin::jujum_plugin::JujumPlugin::load_plugin(sentence),
            ),
            Plugins::Jotoba => Box::new(crate::plugins::jotoba_plugin::JotobaPlugin::load_plugin(
                sentence,
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Validity {
    VALID,
    INVALID,
    UNKNOWN,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub input_word: String,        // term as input by user (surface)
    pub deinflected_word: String,  // deinflected surface as given by tokenizer (base)
    pub conjugations: Vec<String>, // conjforms
    pub validity: Validity,
}

impl Token {
    pub fn is_valid(&self) -> bool {
        // UNKNOWN words might be valid, so they shouldn't count as invalid
        match self.validity {
            Validity::INVALID => false,
            _ => true,
        }
    }
}
