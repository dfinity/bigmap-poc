use super::SearchIndexer;

#[test]
fn search_basic_test() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut s = SearchIndexer::new();

    for i in 0..100 as u8 {
        let key = format!("key-{}", i).into_bytes();
        let value = format!("some text before value-{} some text after", i);

        s.add_to_index(&key, &value);
    }

    for i in 0..100 as u8 {
        let key_expected = format!("key-{}", i).into_bytes();
        let search_string = format!("VALUE-{}", i);

        let search_result = s.search_keys_by_query(&search_string);

        assert_eq!(search_result[0], key_expected);
        assert_eq!(search_result.len(), 1);
    }
}

#[test]
fn search_by_key() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut s = SearchIndexer::new();

    for i in 0..100 as u8 {
        let key = format!("some text before key-{} some text after", i).into_bytes();
        let value = format!("some text before value-{} some text after", i);

        s.add_to_index(&key, &value);
    }

    for i in 0..100 as u8 {
        let key_expected = format!("some text before key-{} some text after", i).into_bytes();
        let search_string = format!("key-{}", i);

        let search_result = s.search_keys_by_query(&search_string);

        assert_eq!(search_result[0], key_expected);
        assert_eq!(search_result.len(), 1);
    }
}

#[test]
fn search_contain_all_terms() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut s = SearchIndexer::new();

    for i in 0..100 as u8 {
        let key = format!("key-{}", i).into_bytes();
        let value = format!("some text before value-{} some TERM-{} text after", i, i);

        s.add_to_index(&key, &value);
    }

    for i in 0..100 as u8 {
        let key_expected = format!("key-{}", i).into_bytes();
        let search_string = format!("some term-{} before value-{}", i, i);

        let search_result = s.search_keys_by_query(&search_string);

        assert_eq!(search_result[0], key_expected);
        assert_eq!(search_result.len(), 1);
    }
}

#[test]
fn search_with_stemming() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut s = SearchIndexer::new();

    let key = b"key-stem".to_vec();
    let value =
        "Stemming is funnier than a bummer says the sushi loving computer scientist".to_string();
    // Stemmed with https://snowballstem.org/demo.html to:
    // stem is funnier than a bummer say the sushi love comput scientist

    s.add_to_index(&key, &value);

    let key = b"key-wiki".to_vec();
    let value =
        "In linguistic morphology and information retrieval, stemming is the process of reducing inflected (or sometimes derived) words to their word stem, base or root form—generally a written word form.".to_string();
    // in linguist morpholog and inform retriev, stem is the process of reduc inflect (or sometim deriv) word to their word stem, base or root form—gener a written word form.

    s.add_to_index(&key, &value);

    let search_result = s.search_keys_by_query(&"Sushi".to_string());
    assert_eq!(search_result[0], b"key-stem".to_vec());
    assert_eq!(search_result.len(), 1);

    let search_result = s.search_keys_by_query(&"love".to_string());
    assert_eq!(search_result[0], b"key-stem".to_vec());
    assert_eq!(search_result.len(), 1);

    let search_result = s.search_keys_by_query(&"computing".to_string());
    assert_eq!(search_result[0], b"key-stem".to_vec());
    assert_eq!(search_result.len(), 1);

    let search_result = s.search_keys_by_query(&"Stemming".to_string());
    assert_eq!(search_result.len(), 2);
    let search_result = s.search_keys_by_query(&"stem".to_string());
    assert_eq!(search_result.len(), 2);
}
