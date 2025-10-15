use bincode::config::Endianness;
use curl::easy::{Easy, List};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use vibrato::dictionary::LexType;
use vibrato::{Dictionary, Tokenizer};

const CONJ_FORMS: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "*" => "*",
};

pub fn get_form(form: &str) -> &str {
    match CONJ_FORMS.get(form) {
        Some(description) => description,
        None => form,
    }
}

pub fn tokenize(
    query: &String,
    dictionary: &crate::dictionary::Dictionary,
) -> Result<Vec<ParsedWord>, Box<dyn Error>> {
    let system_dic_path: PathBuf = match dirs::config_dir() {
        Some(path) => path.join("popup_dictionary/dicts/system.dic"),
        None => Err("No valid config path found in environment variables.")?,
    };
    let system_dic: File = File::open(system_dic_path)?;
    let reader: BufReader<File> = BufReader::new(system_dic);
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
                } else if token.feature().starts_with("助詞") {
                    Validity::VALID
                } else {
                    Validity::UNKNOWN
                }
            }
        };
        println!("{:?}", token.feature());
        let conjform: String = token.feature().split(",").nth(3).unwrap_or("*").to_string();
        println!("{:?}", conjform);
        words.push(ParsedWord {
            surface: token.surface().to_string(),
            base: token
                .feature()
                .split(",")
                .nth(4)
                .unwrap_or(token.surface())
                .to_string(),
            forms: [conjform].to_vec(),
            response: None,
            valid_word: validity,
        });
    }

    words = improve_tokens(&mut words, dictionary);

    Ok(words)
}

fn improve_tokens(
    words: &mut Vec<ParsedWord>,
    dictionary: &crate::dictionary::Dictionary,
) -> Vec<ParsedWord> {
    let mut new_words: Vec<ParsedWord> = Vec::new();
    let mut start_idx: usize = 0;

    while start_idx < words.len() {
        let mut found_match: bool = false;

        let is_not_particle: bool = match words[start_idx].valid_word {
            Validity::VALID => false,
            _ => true,
        };
        if is_not_particle {
            let word_len = if words.len() > 37 { 37 } else { words.len() }; // 37 characters is the biggest word in the dictionary

            for end_idx in (start_idx + 1..=word_len).rev() {
                let base: String = words[start_idx..end_idx]
                    .iter()
                    .map(|w| w.base.as_str())
                    .collect::<String>();
                let surface: String = words[start_idx..end_idx]
                    .iter()
                    .map(|w| w.surface.as_str())
                    .collect::<String>();

                if let Some(_) = dictionary.lookup(&surface).expect(&format!(
                    "Error getting from database when looking up base: {}",
                    surface
                )) {
                    found_match = true;
                } else if let Some(_) = dictionary.lookup(&base).expect(&format!(
                    "Error getting from database when looking up base: {}",
                    base
                )) {
                    found_match = true;
                }
                if found_match {
                    let mut seen: HashSet<String> = HashSet::new();
                    let combined_forms: Vec<String> = words[start_idx..end_idx]
                        .iter()
                        .flat_map(|word| &word.forms)
                        .filter(|form| seen.insert(form.to_string()))
                        .cloned()
                        .collect();
                    println!(
                        "{:?},{:?},{:?},{:?}",
                        surface,
                        base,
                        combined_forms,
                        words[start_idx..end_idx].to_vec()
                    );
                    new_words.push(ParsedWord {
                        surface,
                        base,
                        forms: combined_forms,
                        response: None,
                        valid_word: Validity::UNKNOWN,
                    });
                    start_idx = end_idx;
                    break;
                }
            }
        }

        if !found_match {
            new_words.push(words[start_idx].clone());
            start_idx += 1;
        }
    }

    new_words
}

#[derive(Clone, Debug)]
pub struct ParsedWord {
    pub surface: String,    // term as input by user
    pub base: String,       // deinflected surface as given by tokenizer
    pub forms: Vec<String>, // conjforms
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

#[derive(Clone, Debug)]
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
