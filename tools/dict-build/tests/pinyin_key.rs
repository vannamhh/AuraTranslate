use dict_build::pinyin::pinyin_key;

#[test]
fn pinyin_key_follows_the_shared_fixture_table() {
    let table = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pinyin_key.tsv"),
    )
    .unwrap();
    let mut rows = 0;
    for line in table.lines() {
        let (input, expected) = line.split_once('\t').expect("tab-separated row");
        let want = if expected == "NULL" {
            None
        } else {
            Some(expected.to_string())
        };
        assert_eq!(pinyin_key(input), want, "input {input:?}");
        rows += 1;
    }
    assert!(
        rows >= 20,
        "fixture table must not be emptied, got {rows} rows"
    );
}
