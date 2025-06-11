use curl::easy::Easy;
use curl::easy::List;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug)]
pub enum ParsedWord {
    Valid(String),
    Invalid(String),
}

impl fmt::Display for ParsedWord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            ParsedWord::Valid(parsed_word) => write!(f, "{}", parsed_word),
            ParsedWord::Invalid(parsed_word) => write!(f, "{}", parsed_word),
        }
    }
}

pub fn parse_words(query: &String) -> Result<Vec<ParsedWord>, Box<dyn Error>> {
    let mut sentence: String = query.clone();
    println!("{}", sentence);

    let mut words: Vec<ParsedWord> = Vec::new();
    while !sentence.is_empty() {
        let response: Response = query_words(&sentence)?;
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
                        words.push(ParsedWord::Valid(kanji.clone()));
                        break;
                    }
                }
            }

            if !removed {
                //println!("{}", response.words[0].reading.kana);
                if let Some(remainder) = sentence.strip_prefix(&response.words[0].reading.kana) {
                    let remainder_owned: String = remainder.to_string();
                    sentence.clear();
                    sentence.push_str(&remainder_owned);
                    //removed = true;
                    words.push(ParsedWord::Valid(response.words[0].reading.kana.clone()));
                } else {
                    if let Some(first_char) = sentence.chars().next() {
                        let char_len: usize = first_char.len_utf8();
                        let first_char: String = sentence.drain(0..char_len).collect();
                        let words_len: usize = words.len();
                        if words_len > 0 {
                            match words.get_mut(words_len - 1).unwrap() {
                                ParsedWord::Valid(_) => {
                                    words.push(ParsedWord::Invalid(first_char));
                                }
                                ParsedWord::Invalid(parsed_word) => {
                                    parsed_word.push_str(&first_char);
                                }
                            }
                        } else {
                            words.push(ParsedWord::Invalid(first_char));
                        }
                    } else {
                        return Err(Box::from("Input couldn't be parsed properly."));
                    }
                }
            }
            //println!("{}", removed);
        } else {
            if let Some(first_char) = sentence.chars().next() {
                let char_len: usize = first_char.len_utf8();
                let first_char: String = sentence.drain(0..char_len).collect();
                let words_len: usize = words.len();
                if words_len > 0 {
                    match words.get_mut(words_len - 1).unwrap() {
                        ParsedWord::Valid(_) => {
                            words.push(ParsedWord::Invalid(first_char));
                        }
                        ParsedWord::Invalid(parsed_word) => {
                            parsed_word.push_str(&first_char);
                        }
                    }
                } else {
                    words.push(ParsedWord::Invalid(first_char));
                }
            } else {
                return Err(Box::from("No matching translation(s) found."));
            }
        }
    }
    println!("{:?}", words);

    if words.is_empty() {
        return Err(Box::from("No matching translation(s) found."));
    }

    // words vector is now populated. query still contains the full sentence.
    // TODO: rework above code to also store wrong words/symbols in the words vector.
    // have it be a datatype that stores info on whether it's wrong or not.
    // if it's wrong, GUI can handle it that way.
    // if it's right, it has the Response stored, so GUI can use it on change to that word without having to re-request from server.

    Ok(words)
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    words: Vec<Word>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Word {
    reading: Reading,
    senses: Vec<Sense>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Reading {
    kana: String,
    #[serde(default)]
    kanji: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Sense {
    glosses: Vec<String>,
}

fn query_words(query: &String) -> Result<Response, Box<dyn Error>> {
    let mut easy = Easy::new();
    easy.url("https://jotoba.de/api/search/words")?;
    easy.post(true)?;
    let mut list = List::new();
    list.append("Content-Type: application/json")?;
    easy.http_headers(list)?;

    let mut buf = Vec::new();

    let request_string: String = format!(
        "{}{}{}",
        r#"{"query":""#, query, r#"","language":"English"}"#
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
