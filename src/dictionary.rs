use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled::Db;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Clone)]
pub struct Dictionary {
    db: Db,
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct DictionaryEntry {
    pub terms: Vec<DictionaryTerm>,
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct DictionaryTerm {
    pub frequency: Option<u32>,
    pub term: String,
    pub reading: String,
    pub meanings: Vec<String>,
}

// JMDict json
#[derive(Serialize, Deserialize)]
struct JMDict {
    tags: HashMap<String, String>,
    words: Vec<Word>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Word {
    kanji: Vec<Kanji>,
    kana: Vec<Kana>,
    sense: Vec<Sense>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Kanji {
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Kana {
    text: String,
    appliesToKanji: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Sense {
    appliesToKanji: Vec<String>,
    appliesToKana: Vec<String>,
    gloss: Vec<Gloss>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Gloss {
    text: String,
}
// ---

impl Dictionary {
    pub fn load_dictionary(path: &str) -> Result<Self, Box<dyn Error>> {
        let db: Db = sled::open(path)?;
        if !db.was_recovered() {
            Self::parse_jmdict_simplified(&db)?;
        }
        Ok(Self { db })
    }

    fn parse_jmdict_simplified(db: &Db) -> Result<(), Box<dyn Error>> {
        let frequency_map: HashMap<String, u32> = Self::parse_leeds_frequencies()?;

        let file: File = File::open("./src/dictionaries/jmdict-simplified/jmdict-eng-3.6.1.json")?;
        let jmdict: JMDict = serde_json::from_reader(BufReader::new(file))?;

        let wildcard: String = String::from("*");

        for word in &jmdict.words {
            if !word.kanji.is_empty() {
                for kanji in &word.kanji {
                    for kana in word.kana.iter().filter(|kana| {
                        kana.appliesToKanji.contains(&wildcard)
                            || kana.appliesToKanji.contains(&kanji.text)
                    }) {
                        let meanings: Vec<String> = word
                            .sense
                            .iter()
                            .filter(|sense| {
                                sense.appliesToKanji.contains(&wildcard)
                                    || sense.appliesToKanji.contains(&kanji.text)
                            })
                            .flat_map(|sense| sense.gloss.iter().map(|gloss| gloss.text.clone()))
                            .collect();

                        Self::insert_entry(
                            db,
                            &format!("term:{}", kanji.text),
                            &frequency_map.get(&kanji.text),
                            &kanji.text,
                            &kana.text,
                            &meanings,
                        )?;
                        Self::insert_entry(
                            db,
                            &format!("reading:{}", kana.text),
                            &frequency_map.get(&kanji.text),
                            &kanji.text,
                            &kana.text,
                            &meanings,
                        )?;
                    }
                }

                for kana in word
                    .kana
                    .iter()
                    .filter(|kana| kana.appliesToKanji.is_empty())
                {
                    let meanings: Vec<String> = word
                        .sense
                        .iter()
                        .filter(|sense| {
                            sense.appliesToKana.contains(&wildcard)
                                || sense.appliesToKana.contains(&kana.text)
                        })
                        .flat_map(|sense| sense.gloss.iter().map(|gloss| gloss.text.clone()))
                        .collect();

                    Self::insert_entry(
                        db,
                        &format!("reading:{}", kana.text),
                        &frequency_map.get(&kana.text),
                        "",
                        &kana.text,
                        &meanings,
                    )?;
                }
            } else {
                for kana in &word.kana {
                    let meanings: Vec<String> = word
                        .sense
                        .iter()
                        .filter(|sense| {
                            sense.appliesToKana.contains(&wildcard)
                                || sense.appliesToKana.contains(&kana.text)
                        })
                        .flat_map(|sense| sense.gloss.iter().map(|gloss| gloss.text.clone()))
                        .collect();

                    Self::insert_entry(
                        db,
                        &format!("reading:{}", kana.text),
                        &frequency_map.get(&kana.text),
                        "",
                        &kana.text,
                        &meanings,
                    )?;
                }
            }
        }

        db.flush()?;
        //println!("Loaded {} entries into dictionary", entries.len());

        Ok(())
    }

    fn parse_leeds_frequencies() -> Result<HashMap<String, u32>, Box<dyn Error>> {
        let mut frequency_map: HashMap<String, u32> = HashMap::new();
        let file: File = File::open("./src/dictionaries/leeds-corpus-frequency.txt")?;

        // note: prone to overflow?
        let mut line_num: u32 = 0;
        for line in BufReader::new(file).lines().map_while(Result::ok) {
            frequency_map.insert(line, line_num);
            line_num += 1;
        }

        Ok(frequency_map)
    }

    fn insert_entry(
        db: &Db,
        key: &str,
        frequency: &Option<&u32>,
        term: &str,
        reading: &str,
        meanings: &Vec<String>,
    ) -> Result<(), Box<dyn Error>> {
        let frequency: Option<u32> = match frequency {
            Some(freq_value) => Some(**freq_value),
            None => None,
        };
        let dictionary_term: DictionaryTerm = DictionaryTerm {
            frequency,
            term: term.to_string(),
            reading: reading.to_string(),
            meanings: meanings.to_vec(),
        };

        if let Some(serialized_entry) = db.get(key)? {
            let (mut dictionary_entry, _): (DictionaryEntry, usize) =
                bincode::decode_from_slice(&serialized_entry, bincode::config::standard())?;

            match frequency {
                Some(freq_value) => {
                    let insert_index: usize = dictionary_entry
                        .terms
                        .binary_search_by_key(&freq_value, |term| {
                            term.frequency.unwrap_or(u32::MAX)
                        })
                        .unwrap_or_else(|pos| pos);
                    dictionary_entry.terms.insert(insert_index, dictionary_term);
                }
                None => {
                    dictionary_entry.terms.push(dictionary_term);
                }
            };

            let serialized_entry: Vec<u8> =
                bincode::encode_to_vec(&dictionary_entry, bincode::config::standard())?;
            _ = db.insert(key, serialized_entry.as_slice())?;
        } else {
            let dictionary_entry = DictionaryEntry {
                terms: vec![dictionary_term],
            };
            let serialized_entry: Vec<u8> =
                bincode::encode_to_vec(&dictionary_entry, bincode::config::standard())?;

            _ = db.insert(key, serialized_entry.as_slice())?;
        }

        Ok(())
    }

    /*
    fn parse_jitendex(db: &Db) -> Result<(), Box<dyn Error>> {
        let mut num = 1;
        let mut current_path = format!(
            "{}{}{}",
            "./src/dictionaries/jitendex/term_bank_",
            num.to_string(),
            ".json"
        );
        while Path::new(&current_path).exists() {
            let file: File = File::open(&current_path)?;
            let entries: Vec<serde_json::Value> = serde_json::from_reader(BufReader::new(file))?;

            for entry_value in &entries {
                if let Some(entry_array) = entry_value.as_array() {
                    if entry_array.len() >= 6 {
                        // extract meanings
                        let mut meanings: Vec<String> = Vec::new();
                        if let Some(definitions) = entry_array[5].as_array() {
                            if !definitions.is_empty() {
                                meanings.extend(Self::jitendex_extract_meanings(&definitions[0]));
                            }
                        }
                        if !meanings.is_empty() {
                            let entry = DictionaryEntry {
                                term: entry_array[0].as_str().unwrap_or("").to_string(),
                                reading: entry_array[1].as_str().unwrap_or("").to_string(),
                                meanings: meanings,
                            };

                            let serialized_entry =
                                bincode::encode_to_vec(&entry, bincode::config::standard())?;

                            if !entry.term.is_empty() {
                                db.insert(
                                    format!("term:{}", entry.term),
                                    serialized_entry.as_slice(),
                                )?;
                            }

                            if !entry.reading.is_empty() && entry.reading != entry.term {
                                db.insert(
                                    format!("reading:{}", entry.reading),
                                    serialized_entry.as_slice(),
                                )?;
                            }
                        }
                    }
                }
            }

            db.flush()?;
            println!("Loaded {} entries into dictionary", entries.len());

            num = num + 1;
            current_path = format!(
                "{}{}{}",
                "./src/dictionaries/jitendex/term_bank_",
                num.to_string(),
                ".json"
            );
        }
        Ok(())
    }

    fn jitendex_extract_meanings(value: &Value) -> Vec<String> {
        let mut meanings = Vec::new();

        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    if key == "content" {
                        if let Value::String(s) = val {
                            meanings.push(s.clone());
                        } else {
                            meanings.extend(Self::jitendex_extract_meanings(val));
                        }
                    } else {
                        meanings.extend(Self::jitendex_extract_meanings(val));
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    meanings.extend(Self::jitendex_extract_meanings(item));
                }
            }
            _ => {}
        }
        meanings
    }*/

    pub fn lookup(&self, word: &str) -> Result<Option<DictionaryEntry>, Box<dyn Error>> {
        if let Some(serialized_entry) = self.db.get(format!("term:{}", word))? {
            let (entry, _): (DictionaryEntry, usize) =
                bincode::decode_from_slice(&serialized_entry, bincode::config::standard())
                    .expect(&format!("{:?}", &serialized_entry));
            return Ok(Some(entry));
        }
        if let Some(serialized_entry) = self.db.get(format!("reading:{}", word))? {
            let (entry, _): (DictionaryEntry, usize) =
                bincode::decode_from_slice(&serialized_entry, bincode::config::standard())
                    .expect("reading");
            return Ok(Some(entry));
        }
        Ok(None)
    }
}
