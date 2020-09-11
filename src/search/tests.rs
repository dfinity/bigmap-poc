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

        let search_result = s.search_by_query(&search_string);

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

        let search_result = s.search_by_query(&search_string);

        assert_eq!(search_result[0], key_expected);
        assert_eq!(search_result.len(), 1);
    }
}
