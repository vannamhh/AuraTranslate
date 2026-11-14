//! Counts on the four real v4 files in `out/`. Run by hand after `--layer all`:
//! `cargo test --test reading_key_real_data -- --ignored --nocapture`.

use rusqlite::Connection;

const KEYLESS_ROWS_IN_CORE: i64 = 70;
const UNCLOSED_PAREN_ROWS: i64 = 0;

fn open(name: &str) -> Option<Connection> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("out")
        .join(name);
    path.is_file().then(|| Connection::open(path).unwrap())
}

#[test]
#[ignore = "needs the four built .db files in tools/dict-build/out, absent on CI"]
fn real_v4_files_have_the_declared_keyless_rows_and_every_gloss_lang() {
    for name in [
        "dict-core.db",
        "dict-thieu-chuu.db",
        "dict-vietphrase.db",
        "dict-tran-van-chanh.db",
    ] {
        let Some(conn) = open(name) else {
            eprintln!("skipped, {name} is not built");
            return;
        };
        let keyless: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM dict_entry WHERE reading IS NOT NULL AND reading <> '' AND reading_key IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let expected = if name == "dict-core.db" {
            KEYLESS_ROWS_IN_CORE
        } else {
            0
        };
        assert_eq!(keyless, expected, "{name}: rows with a reading and no key");
        let undeclared: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM dict_source s WHERE s.gloss_lang IS NULL
                   AND EXISTS (SELECT 1 FROM dict_sense d WHERE d.source_id = s.id)",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            undeclared, 0,
            "{name}: sources with senses and no gloss_lang"
        );
    }
    let core = open("dict-core.db").unwrap();
    let gloss_lang_of = |source: &str| -> String {
        core.query_row(
            "SELECT s.gloss_lang FROM dict_entry e JOIN dict_source s ON s.id = e.source_id
              WHERE e.headword = '你好' AND s.code = ?1 LIMIT 1",
            [source],
            |r| r.get(0),
        )
        .unwrap()
    };
    assert_eq!(gloss_lang_of("cvdict"), "vi");
    assert_eq!(gloss_lang_of("cc-cedict"), "en");
    let unclosed: i64 = core
        .query_row(
            "SELECT COUNT(*) FROM dict_entry WHERE (length(reading) - length(replace(reading, '(', '')))
                <> (length(reading) - length(replace(reading, ')', '')))",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(unclosed, UNCLOSED_PAREN_ROWS);
    let after_paren: i64 = core
        .query_row(
            "SELECT COUNT(*) FROM dict_entry WHERE reading LIKE '%)_%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        after_paren, 2,
        "readings with text after the closing paren (靠背兒椅子, 筷兒筒)"
    );
}
