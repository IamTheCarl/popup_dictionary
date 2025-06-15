use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled::Db;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

#[derive(Clone)]
pub struct Dictionary {
    db: Db,
}

#[derive(bincode::Encode, bincode::Decode, Debug)]
pub struct DictionaryEntry {
    pub terms: Vec<DictionaryTerm>,
}

#[derive(bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct DictionaryTerm {
    pub frequency: Option<u32>,
    pub common: bool,
    pub term: String,
    pub reading: String,
    pub furigana: Option<Vec<Furigana>>,
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
    common: bool,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Kana {
    common: bool,
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

// jmdict-furigana json
#[derive(Serialize, Deserialize, Debug)]
struct JMDictFurigana {
    text: String,
    reading: String,
    furigana: Vec<Furigana>,
}

#[derive(Serialize, Deserialize, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct Furigana {
    pub ruby: String,
    pub rt: Option<String>,
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
        let furigana_map: HashMap<String, Vec<Furigana>> = Self::parse_jmdict_furigana()?;

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

                        let mut frequency = frequency_map.get(&kanji.text);
                        if frequency.is_none() {
                            frequency = frequency_map.get(&kana.text);
                        }
                        Self::insert_entry(
                            db,
                            &format!("term:{}", kanji.text),
                            &frequency,
                            &kanji.common,
                            &kanji.text,
                            &kana.text,
                            &furigana_map.get(&format!("{},{}", &kanji.text, &kana.text)),
                            &meanings,
                        )?;

                        let mut frequency = frequency_map.get(&kana.text);
                        if frequency.is_none() {
                            frequency = frequency_map.get(&kanji.text);
                        }
                        Self::insert_entry(
                            db,
                            &format!("reading:{}", kana.text),
                            &frequency,
                            &kana.common,
                            &kanji.text,
                            &kana.text,
                            &furigana_map.get(&format!("{},{}", &kanji.text, &kana.text)),
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
                        &kana.common,
                        "",
                        &kana.text,
                        &None,
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
                        &kana.common,
                        "",
                        &kana.text,
                        &None,
                        &meanings,
                    )?;
                }
            }
        }

        db.flush()?;

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

    fn parse_jmdict_furigana() -> Result<HashMap<String, Vec<Furigana>>, Box<dyn Error>> {
        let mut furigana_map: HashMap<String, Vec<Furigana>> = HashMap::new();

        let file: File = File::open("./src/dictionaries/jmdict-furigana.json")?;
        let json: Vec<JMDictFurigana> = serde_json::from_reader(BufReader::new(file))?;

        for jmdict_furigana in json {
            furigana_map.insert(
                format!("{},{}", jmdict_furigana.text, jmdict_furigana.reading),
                jmdict_furigana.furigana,
            );
        }

        Ok(furigana_map)
    }

    fn insert_entry(
        db: &Db,
        key: &str,
        frequency: &Option<&u32>,
        common: &bool,
        term: &str,
        reading: &str,
        furigana: &Option<&Vec<Furigana>>,
        meanings: &Vec<String>,
    ) -> Result<(), Box<dyn Error>> {
        let frequency: Option<u32> = match frequency {
            Some(freq_value) => Some(**freq_value),
            None => None,
        };
        let furigana: Option<Vec<Furigana>> = match furigana {
            Some(furigana_vec) => Some(furigana_vec.to_vec()),
            None => None,
        };
        let dictionary_term: DictionaryTerm = DictionaryTerm {
            frequency,
            common: *common,
            term: term.to_string(),
            reading: reading.to_string(),
            furigana,
            meanings: meanings.to_vec(),
        };
        if term == "私" {
            println!(
                "{},{},{},{}",
                term,
                reading,
                common,
                frequency.unwrap_or(u32::MAX)
            )
        }
        if let Some(serialized_entry) = db.get(key)? {
            let (mut dictionary_entry, _): (DictionaryEntry, usize) =
                bincode::decode_from_slice(&serialized_entry, bincode::config::standard())?;

            /*
            Sorting of terms in each entry:
            1. common, freq         -- first
            2. common, no freq
            3. uncommon, freq
            4. uncommon, no freq    -- last
            */
            //TODO: implement combining terms with the same meanings into one with "alternative readings"
            if *common {
                if let Some(frequency) = frequency {
                    let mut inserted: bool = false;
                    for (index, term) in dictionary_entry.terms.iter().enumerate() {
                        if !term.common || term.frequency.is_none() {
                            dictionary_entry
                                .terms
                                .insert(index, dictionary_term.clone());
                            inserted = true;
                            break;
                        }
                        if let Some(term_frequency) = term.frequency {
                            if term_frequency > frequency {
                                dictionary_entry
                                    .terms
                                    .insert(index, dictionary_term.clone());
                                inserted = true;
                                break;
                            }
                        }
                    }
                    if !inserted {
                        dictionary_entry.terms.push(dictionary_term.clone());
                    }
                } else {
                    let mut inserted = false;
                    for (index, term) in dictionary_entry.terms.iter().enumerate() {
                        if !term.common {
                            dictionary_entry
                                .terms
                                .insert(index, dictionary_term.clone());
                            inserted = true;
                            break;
                        }
                    }
                    if !inserted {
                        dictionary_entry.terms.push(dictionary_term.clone());
                    }
                }
            } else {
                if let Some(frequency) = frequency {
                    let mut inserted: bool = false;
                    for (index, term) in dictionary_entry.terms.iter().enumerate() {
                        if term.common {
                            continue;
                        }
                        if !term.common && term.frequency.is_none() {
                            dictionary_entry
                                .terms
                                .insert(index, dictionary_term.clone());
                            inserted = true;
                            break;
                        }
                        if let Some(term_frequency) = term.frequency {
                            if term_frequency > frequency {
                                dictionary_entry
                                    .terms
                                    .insert(index, dictionary_term.clone());
                                inserted = true;
                                break;
                            }
                        }
                    }
                    if !inserted {
                        dictionary_entry.terms.push(dictionary_term.clone());
                    }
                } else {
                    dictionary_entry.terms.push(dictionary_term.clone());
                }
            }

            if term == "私" {
                println!("{:?}", dictionary_entry);
            }

            let serialized_entry: Vec<u8> =
                bincode::encode_to_vec(&dictionary_entry, bincode::config::standard())?;
            _ = db.insert(key, serialized_entry.as_slice())?;
        } else {
            if term == "私" {
                println!("[]");
            }
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
