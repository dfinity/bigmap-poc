// Full Text Search
// Input: (doc_id: Vec<u8>, document: String)
//
// Steps:
// - Split string into tokens,
// - Normalize tokens,
// - Get or assign a unique ID for each token,
// -

use lazy_static::lazy_static;
use regex::Regex;
use roaring::RoaringBitmap;
use rust_stemmers::{Algorithm, Stemmer};
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;
use wyhash::WyHash;

pub type DetHashMap<K, V> = HashMap<K, V, BuildHasherDefault<WyHash>>;
pub type DetHashSet<K> = HashSet<K, BuildHasherDefault<WyHash>>;

// #[cfg(target_arch = "wasm32")]
// use ic_cdk::println;

use crate::Key;

// Roaring Bitmaps only support 32-bit integers
type DocumentId = u32;
type Term = String;

lazy_static! {
    static ref RE_NOT_ALPHANUM: Regex = Regex::new(r"\W").unwrap();
    static ref STOP_WORDS: HashSet<String> = include_str!("search/stop_words.txt")
        .split_whitespace()
        .into_iter()
        .map(String::from)
        .collect();
}

#[derive(Default)]
struct TermData {
    frequency: usize,
    inverted_index: RoaringBitmap,
}

pub struct SearchIndexer {
    key_to_doc_id: DetHashMap<Key, DocumentId>,
    doc_id_to_key: DetHashMap<DocumentId, Key>,
    terms: DetHashMap<Term, TermData>,
    stemmer: Stemmer,
}

impl Default for SearchIndexer {
    fn default() -> Self {
        Self {
            key_to_doc_id: DetHashMap::default(),
            doc_id_to_key: DetHashMap::default(),
            terms: DetHashMap::default(),
            stemmer: Stemmer::create(Algorithm::English),
        }
    }
}

impl SearchIndexer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_to_index(&mut self, key: &Key, doc: &String) {
        let doc_id = match self.key_to_doc_id.get(key) {
            Some(doc_id) => *doc_id,
            None => {
                let doc_id = self.key_to_doc_id.len() as u32;
                self.key_to_doc_id.insert(key.clone(), doc_id);
                self.doc_id_to_key.insert(doc_id, key.clone());
                doc_id
            }
        };

        let doc = format!("{} {}", String::from_utf8_lossy(key), doc);

        for term in RE_NOT_ALPHANUM.replace_all(&doc, " ").split_whitespace() {
            let term = self.normalize_to_string(term);
            if STOP_WORDS.contains(&term) {
                continue;
            }
            let term_data = match self.terms.get_mut(&term) {
                Some(t) => t,
                None => {
                    let d = TermData {
                        frequency: 0,
                        inverted_index: RoaringBitmap::new(),
                    };
                    self.terms.insert(term.clone(), d);
                    self.terms.get_mut(&term).unwrap()
                }
            };

            term_data.inverted_index.insert(doc_id);
            term_data.frequency += 1;
        }
    }

    pub fn batch_add_to_index(&mut self, doc_vec: &Vec<(Key, String)>) -> u64 {
        let result = doc_vec.len() as u64;
        for (key, doc) in doc_vec.into_iter() {
            self.add_to_index(key, doc);
        }
        result
    }

    pub fn search_keys_by_query(&self, query: &String) -> Vec<Key> {
        let mut result = Vec::new();

        let mut all_term_inverted_indexes = Vec::new();

        for term in RE_NOT_ALPHANUM.replace_all(query, " ").split_whitespace() {
            let term = self.normalize_to_string(term);
            if STOP_WORDS.contains(&term) {
                continue;
            }

            match self.terms.get(&term) {
                Some(term_data) => {
                    all_term_inverted_indexes.push(&term_data.inverted_index);
                }
                None => {}
            }
        }

        if !all_term_inverted_indexes.is_empty() {
            let mut result_bitmap = all_term_inverted_indexes[0].clone();

            for rb in all_term_inverted_indexes[1..].iter() {
                result_bitmap.intersect_with(rb)
            }

            for doc_id in result_bitmap {
                result.push(
                    self.doc_id_to_key
                        .get(&doc_id)
                        .expect("doc_id is expected to be valid")
                        .clone(),
                );
            }
        }

        result
    }

    pub fn remove_key(&mut self, key: &Key) {
        match self.key_to_doc_id.remove(key) {
            Some(doc_id) => {
                self.doc_id_to_key.remove(&doc_id);
                for term in self.terms.values_mut() {
                    term.inverted_index.remove(doc_id);
                }
            }
            None => {}
        }
    }

    pub fn used_bytes(&self) -> usize {
        std::mem::size_of_val(self)
    }

    fn normalize_to_string(&self, input: &str) -> String {
        String::from(self.stemmer.stem(&input.to_lowercase()))
    }
}

#[cfg(test)]
mod tests;
