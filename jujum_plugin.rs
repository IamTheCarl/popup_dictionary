use crate::plugin::Plugin;
use crate::plugin::Token;

pub struct JujumPlugin {
    data: Vec<Token>,
}

impl Plugin for JujumPlugin {
    fn load_plugin() -> Self {
        Self { data: vec![] }
    }

    fn get_data(&self) -> &Vec<Token> {
        &self.data
    }
}
