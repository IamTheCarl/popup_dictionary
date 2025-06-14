use curl::easy::{Easy, List};
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use vibrato::dictionary::LexType;
use vibrato::{Dictionary, Tokenizer};

pub fn tokenize(query: &String) -> Result<Vec<ParsedWord>, Box<dyn Error>> {
    let file: File = File::open("src/dictionaries/system.dic")?;
    let reader: BufReader<File> = BufReader::new(file);
    let dict: Dictionary = Dictionary::read(reader)?;

    let tokenizer: Tokenizer = Tokenizer::new(dict);
    let mut worker = tokenizer.new_worker();

    worker.reset_sentence(query);
    worker.tokenize();

    let mut words: Vec<ParsedWord> = Vec::new();
    for token in worker.token_iter() {
        let validity: Validity = match token.lex_type() {
            LexType::Unknown => Validity::INVALID,
            _ => {
                if token.feature().starts_with("特殊") {
                    Validity::INVALID
                } else {
                    Validity::UNKNOWN
                }
            }
        };

        words.push(ParsedWord {
            surface: token.surface().to_string(),
            base: token
                .feature()
                .split(",")
                .nth(4)
                .unwrap_or(token.surface())
                .to_string(),
            response: None,
            valid_word: validity,
        });
    }
    Ok(words)
}

#[derive(Clone)]
pub struct ParsedWord {
    pub surface: String,
    pub base: String,
    response: Option<Response>,
    valid_word: Validity,
}

impl ParsedWord {
    pub fn get_response(&mut self) -> Option<&Response> {
        match self.valid_word {
            Validity::VALID => self.response.as_ref(),
            Validity::INVALID => None,
            Validity::UNKNOWN => {
                if let Ok(response) = self.fetch_word() {
                    if response.words.is_empty() {
                        self.response = None;
                        self.valid_word = Validity::INVALID;
                    } else {
                        self.response = Some(response);
                        self.valid_word = Validity::VALID;
                    }
                } else {
                    self.response = None;
                    self.valid_word = Validity::INVALID;
                }
                self.response.as_ref()
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        match self.valid_word {
            Validity::INVALID => false,
            _ => true,
        }
    }

    fn fetch_word(&self) -> Result<Response, Box<dyn Error>> {
        let mut easy = Easy::new();
        easy.url("https://jotoba.de/api/search/words")?;
        easy.post(true)?;
        let mut list = List::new();
        list.append("Content-Type: application/json")?;
        easy.http_headers(list)?;

        let mut buf = Vec::new();

        let request_string: String = format!(
            "{}{}{}",
            r#"{"query":""#, self.surface, r#"","language":"English"}"#
        );
        let request: &[u8] = request_string.as_bytes();
        easy.post_fields_copy(request)?;

        {
            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                buf.extend_from_slice(data);
                Ok(data.len())
            })?;
            transfer.perform()?;
        }

        let json: Response =
            serde_json::from_str(String::from_utf8(buf.to_vec()).unwrap().as_str()).unwrap();

        Ok(json)
    }
}

#[derive(Clone)]
enum Validity {
    VALID,
    INVALID,
    UNKNOWN,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Response {
    pub words: Vec<Word>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Word {
    pub reading: Reading,
    pub senses: Vec<Sense>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Reading {
    pub kana: String,
    #[serde(default)]
    pub kanji: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sense {
    pub glosses: Vec<String>,
}
