use curl::easy::Easy;
use curl::easy::List;
use egui::cache;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fmt;

use crate::plugin::Token;
use crate::plugin::Validity;

// Structure for caching
#[derive(Clone, Debug)]
enum CachedToken {
    Valid(ValidToken),
    Invalid(String),
}

#[derive(Clone, Debug)]
struct ValidToken {
    word: String,
    response: Response,
}

impl fmt::Display for CachedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CachedToken::Valid(cached_word) => write!(f, "{}", cached_word.word),
            CachedToken::Invalid(cached_word) => write!(f, "{}", cached_word),
        }
    }
}

// Jotoba API Response
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

pub struct JotobaTokenizer {
    token_cache: Vec<CachedToken>,
}

impl JotobaTokenizer {
    pub fn new() -> Self {
        return Self {
            token_cache: Vec::new(),
        };
    }

    pub fn tokenize(&mut self, sentence: &str) -> Result<Vec<Token>, Box<dyn Error>> {
        // temporarily limit sentence length for testing and to limit calls to jotoba api (spam)
        let sentence: String = sentence.chars().take(30).collect();
        //TODO: use jotoba search query completion api as pre-tokenizer. will vastly improve tokenization and likely reduce calls.

        let mut sentence: String = sentence.to_string();
        println!("{}", sentence);

        let mut token_cache: Vec<CachedToken> = Vec::new();
        let mut previous_word: String = String::new();
        while !sentence.is_empty() {
            let response: Response = self.query_jotoba(&sentence)?;
            //println!("{:?}", response);
            if response.words.len() > 0 {
                let mut removed: bool = false;

                for word in &response.words {
                    //println!("{:?}", word.reading.kanji);
                    if let Some(kanji) = &word.reading.kanji {
                        if let Some(remainder) = sentence.strip_prefix(kanji) {
                            let remainder_owned: String = remainder.to_string();
                            sentence.clear();
                            sentence.push_str(&remainder_owned);
                            removed = true;
                            token_cache.push(CachedToken::Valid(ValidToken {
                                word: kanji.clone(),
                                response: response.clone(),
                            }));
                            break;
                        }
                    }
                }

                if !removed {
                    //println!("{}", response.words[0].reading.kana);
                    if let Some(remainder) = sentence.strip_prefix(&response.words[0].reading.kana)
                    {
                        let remainder_owned: String = remainder.to_string();
                        sentence.clear();
                        sentence.push_str(&remainder_owned);
                        removed = true;
                        token_cache.push(CachedToken::Valid(ValidToken {
                            word: response.words[0].reading.kana.clone(),
                            response: response.clone(),
                        }));
                    } else {
                        if let Some(first_char) = sentence.chars().next() {
                            let char_len: usize = first_char.len_utf8();
                            let first_char: String = sentence.drain(0..char_len).collect();
                            previous_word.push_str(&first_char);
                            let words_len: usize = token_cache.len();
                            if words_len > 0 {
                                if let CachedToken::Valid(parsed_word) =
                                    token_cache.get_mut(words_len - 1).unwrap()
                                {
                                    if !parsed_word.word.is_empty() {
                                        token_cache.push(CachedToken::Valid(ValidToken {
                                            word: String::new(),
                                            response: response.clone(),
                                        }));
                                    }
                                } else {
                                    token_cache.push(CachedToken::Valid(ValidToken {
                                        word: String::new(),
                                        response: response.clone(),
                                    }));
                                }
                            } else {
                                token_cache.push(CachedToken::Valid(ValidToken {
                                    word: String::new(),
                                    response: response.clone(),
                                }));
                            }
                        } else {
                            return Err(Box::from("Input couldn't be parsed properly."));
                        }
                    }
                }
                if removed && !previous_word.is_empty() {
                    let words_len: usize = token_cache.len();
                    if let CachedToken::Valid(parsed_word) =
                        token_cache.get_mut(words_len - 2).unwrap()
                    {
                        parsed_word.word = previous_word.clone();
                    } else {
                        // This can occur when jotoba gives a response to a word but the input word itself is different.
                        // For example, when a typo happens: Input word = ユーザ but the correct spelling and jotoba response is ユーザー.
                        // This makes
                        return Err(Box::from("Logical error in previous_word."));
                    }
                    previous_word.clear();
                }
                //println!("{}", removed);
            } else {
                if let Some(first_char) = sentence.chars().next() {
                    let char_len: usize = first_char.len_utf8();
                    let first_char: String = sentence.drain(0..char_len).collect();
                    let words_len: usize = token_cache.len();
                    if words_len > 0 {
                        match token_cache.get_mut(words_len - 1).unwrap() {
                            CachedToken::Valid(last_token) => {
                                // this if prevents the problem from the comment above about e.g. typos
                                if last_token.word.is_empty() {
                                    previous_word.push_str(&first_char);
                                } else {
                                    token_cache.push(CachedToken::Invalid(first_char));
                                }
                            }
                            CachedToken::Invalid(parsed_word) => {
                                parsed_word.push_str(&first_char);
                            }
                        }
                    } else {
                        token_cache.push(CachedToken::Invalid(first_char));
                    }
                } else {
                    return Err(Box::from("No matching translation(s) found."));
                }
            }
        }
        println!("{:?}", token_cache);

        if token_cache.is_empty() {
            return Err(Box::from("No matching translation(s) found."));
        }

        let mut tokens: Vec<Token> = Vec::new();
        for cached_token in token_cache.iter() {
            match cached_token {
                CachedToken::Valid(valid_token) => {
                    let token = Token {
                        input_word: valid_token.word.to_string(),
                        deinflected_word: valid_token.word.to_string(),
                        conjugations: Vec::new(),
                        validity: Validity::VALID,
                    };
                    tokens.push(token);
                }
                CachedToken::Invalid(invalid_token) => {
                    let token = Token {
                        input_word: invalid_token.to_string(),
                        deinflected_word: invalid_token.to_string(),
                        conjugations: Vec::new(),
                        validity: Validity::INVALID,
                    };
                    tokens.push(token);
                }
            }
        }

        self.token_cache = token_cache;

        // words vector is now populated. query still contains the full sentence.
        // TODO: rework above code to also store wrong words/symbols in the words vector.
        // have it be a datatype that stores info on whether it's wrong or not.
        // if it's wrong, GUI can handle it that way.
        // if it's right, it has the Response stored, so GUI can use it on change to that word without having to re-request from server.

        Ok(tokens)
    }

    fn query_jotoba(&self, sentence: &String) -> Result<Response, Box<dyn Error>> {
        let mut easy = Easy::new();
        easy.url("https://jotoba.de/api/search/words")?;
        easy.post(true)?;
        let mut list = List::new();
        list.append("Content-Type: application/json")?;
        easy.http_headers(list)?;

        let mut buf = Vec::new();

        let request_string: String = format!(
            "{}{}{}",
            r#"{"query":""#, sentence, r#"","language":"English"}"#
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

    pub fn get_response(&mut self, token: &Token) -> Result<Response, Box<dyn Error>> {
        let cached_token = self
            .token_cache
            .iter()
            .find(|cached_token| match cached_token {
                CachedToken::Valid(valid_token) => valid_token.word == token.input_word,
                CachedToken::Invalid(_) => false,
            });

        match cached_token {
            Some(CachedToken::Valid(valid_token)) => Ok(valid_token.response.clone()),
            _ => {
                let response = self.query_jotoba(&token.input_word)?;
                if !response.words.is_empty() {
                    self.token_cache.push(CachedToken::Valid(ValidToken {
                        word: token.input_word.to_string(),
                        response: response.clone(),
                    }));
                    Ok(response)
                } else {
                    self.token_cache
                        .push(CachedToken::Invalid(token.input_word.to_string()));
                    Err(Box::from("No matching translation(s) found."))
                }
            }
        }
    }
}
