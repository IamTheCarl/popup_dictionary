use serde_json::Value;
use sled::Db;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Clone)]
pub struct Dictionary {
    db: Db,
}

#[derive(bincode::Encode, bincode::Decode)]
pub struct DictionaryEntry {
    pub term: String,
    pub reading: String,
    pub meanings: Vec<String>,
}

impl Dictionary {
    pub fn load_dictionary(path: &str) -> Result<Self, Box<dyn Error>> {
        let db: Db = sled::open(path)?;
        if !db.was_recovered() {
            Self::parse_jitendex(&db)?;
        }
        Ok(Self { db })
    }

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
                                meanings.extend(Self::extract_meanings(&definitions[0]));
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

    fn extract_meanings(value: &Value) -> Vec<String> {
        let mut meanings = Vec::new();

        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    if key == "content" {
                        if let Value::String(s) = val {
                            meanings.push(s.clone());
                        } else {
                            meanings.extend(Self::extract_meanings(val));
                        }
                    } else {
                        meanings.extend(Self::extract_meanings(val));
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    meanings.extend(Self::extract_meanings(item));
                }
            }
            _ => {}
        }
        meanings
    }

    pub fn lookup(&self, word: &str) -> Result<Option<DictionaryEntry>, Box<dyn Error>> {
        if let Some(serialized_entry) = self.db.get(format!("term:{}", word))? {
            let (entry, _): (DictionaryEntry, usize) =
                bincode::decode_from_slice(&serialized_entry, bincode::config::standard())?;
            return Ok(Some(entry));
        }
        if let Some(serialized_entry) = self.db.get(format!("reading:{}", word))? {
            let (entry, _): (DictionaryEntry, usize) =
                bincode::decode_from_slice(&serialized_entry, bincode::config::standard())?;
            return Ok(Some(entry));
        }
        Ok(None)
    }
}
