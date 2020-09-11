// Full Text Search
// Input: (doc_id: Vec<u8>, document: String)
//
// Steps:
// - Split string into tokens,
// - Normalize tokens,
// - Get or assign a unique ID for each token,
// -

use roaring::RoaringBitmap;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use wyhash::WyHash;

pub type DetHashMap<K, V> = HashMap<K, V, BuildHasherDefault<WyHash>>;

// #[cfg(target_arch = "wasm32")]
// use ic_cdk::println;

use crate::Key;
// use std::convert::TryInto;

// Roaring Bitmaps only support 32-bit integers
type DocumentId = u32;
type Term = String;

#[derive(Default)]
struct TermData {
    frequency: usize,
    inverted_index: RoaringBitmap,
}

#[derive(Default)]
pub struct SearchIndexer {
    key_to_doc_id: DetHashMap<Key, DocumentId>,
    doc_id_to_key: DetHashMap<DocumentId, Key>,
    terms: DetHashMap<Term, TermData>,
}

impl SearchIndexer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_to_index(&mut self, key: &Key, document: &String) {
        // println!(
        //     "ingest_document: {} => {}",
        //     String::from_utf8_lossy(key),
        //     document
        // );
        let doc_id = match self.key_to_doc_id.get(key) {
            Some(doc_id) => *doc_id,
            None => {
                let doc_id = self.key_to_doc_id.len() as u32;
                self.key_to_doc_id.insert(key.clone(), doc_id);
                self.doc_id_to_key.insert(doc_id, key.clone());
                doc_id
            }
        };

        for term in document.split_whitespace() {
            let term = SearchIndexer::normalize_to_string(term);
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

    pub fn search_by_query(&self, query: &String) -> Vec<Key> {
        let mut result = Vec::new();

        let mut all_term_inverted_indexes = Vec::new();

        for term in query.split_whitespace() {
            let term = SearchIndexer::normalize_to_string(term);

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

    fn normalize_to_string(input: &str) -> String {
        String::from(input.to_lowercase())
    }
}

#[cfg(test)]
mod tests;
