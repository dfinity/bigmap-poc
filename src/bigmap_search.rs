use bigmap::{search::SearchIndexer, Key};
#[cfg(target_arch = "wasm32")]
use ic_cdk::println;
use ic_cdk::storage;
use ic_cdk_macros::*;

#[update]
fn add_to_search_index(key_doc: (Key, String)) {
    let search = storage::get_mut::<SearchIndexer>();

    let (key, document) = key_doc;
    if document.len() > 100 {
        println!(
            "BigMap Search Index: add key {} => document[0..100] {}",
            String::from_utf8_lossy(&key),
            &document[0..100]
        );
    } else {
        println!(
            "BigMap Search Index: add key {} => document {}",
            String::from_utf8_lossy(&key),
            document
        );
    }
    search.add_to_index(&key, &document);
}

#[update]
fn batch_add_to_search_index(doc_vec: Vec<(Key, String)>) -> u64 {
    let search = storage::get_mut::<SearchIndexer>();

    println!(
        "BigMap Search Index: batch_put_and_fts_index {} entries",
        doc_vec.len()
    );
    search.batch_add_to_index(&doc_vec)
}

#[update]
fn remove_from_search_index(key: Key) {
    let search = storage::get_mut::<SearchIndexer>();

    println!(
        "BigMap Search Index: remove key {}",
        String::from_utf8_lossy(&key)
    );
    search.remove_key(&key);
}

#[query]
fn search_keys_by_query(query: String) -> Vec<Key> {
    let search = storage::get::<SearchIndexer>();

    search.search_keys_by_query(&query)
}

#[query]
fn used_bytes() -> u64 {
    let search = storage::get::<SearchIndexer>();

    search.used_bytes() as u64
}

fn main() {}
