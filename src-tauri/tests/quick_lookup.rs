//! `quick_lookup` (AD-69): group shapes, pinyin and Vietnamese-to-Chinese ranking, schema
//! v3 files, and the NFR1 measurement on the real dictionaries.
//!
//! Fixture databases take their DDL from `tools/dict-build/src/schema.rs` read as text, so
//! a schema change there reaches every case here instead of drifting.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::dict::{QuickLookupResponse, quick_lookup};
use auratranslate_lib::core::dict::pinyin::pinyin_key;
use auratranslate_lib::core::dict::{DictLayers, QuickGroup, QuickGroupId, QuickStatus};

struct Entry {
    lang: &'static str,
    headword: &'static str,
    reading: Option<&'static str>,
    glosses: Vec<String>,
}

struct Source {
    code: &'static str,
    gloss_lang: &'static str,
    entries: Vec<Entry>,
}

fn entry(
    lang: &'static str,
    headword: &'static str,
    reading: Option<&'static str>,
    gloss: &str,
) -> Entry {
    Entry {
        lang,
        headword,
        reading,
        glosses: vec![gloss.to_owned()],
    }
}

fn schema_text() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tools/dict-build/src/schema.rs");
    fs::read_to_string(path)
        .expect("read dict-build schema.rs")
        .replace("\r\n", "\n")
}

fn ddl_of(name: &str) -> String {
    let text = schema_text();
    let head = format!("pub const {name}: &str = \"\\\n");
    let start = text
        .find(&head)
        .unwrap_or_else(|| panic!("no DDL const {name}"))
        + head.len();
    let end = start + text[start..].find("\";").expect("DDL terminator");
    text[start..end].replace("\\\"", "\"")
}

fn all_ddl(version: u32) -> Vec<String> {
    let mut out = Vec::new();
    for name in [
        "DICT_META_DDL",
        "DICT_SOURCE_DDL",
        "DICT_ENTRY_DDL",
        "DICT_SENSE_DDL",
        "DICT_EXAMPLE_DDL",
        "DICT_CITATION_DDL",
        "CHAR_IDX_DDL",
        "ENTRY_INDEXES_DDL",
        "ENTRY_FTS_DDL",
        "SENSE_FTS_DDL",
        "SENSE_FTS_ND_DDL",
    ] {
        let mut ddl = ddl_of(name);
        if version < 4 {
            ddl = ddl
                .replace(",\n  gloss_lang     TEXT", "")
                .replace(",\n  reading_key   TEXT", "")
                .replace(
                    "CREATE INDEX idx_entry_reading_key   ON dict_entry(reading_key);\n",
                    "",
                );
        }
        out.push(ddl);
    }
    out
}

fn temp_dir(tag: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "aura-quick-{tag}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn build_layer(dir: &Path, file: &str, layer: &str, version: u32, sources: &[Source]) {
    let conn = rusqlite::Connection::open(dir.join(file)).unwrap();
    for ddl in all_ddl(version) {
        conn.execute_batch(&ddl).unwrap();
    }
    conn.execute(
        "INSERT INTO dict_meta (key, value) VALUES ('schema_version', ?1), ('layer', ?2)",
        rusqlite::params![version.to_string(), layer],
    )
    .unwrap();
    conn.execute_batch(&format!("PRAGMA user_version = {version};"))
        .unwrap();
    for (index, source) in sources.iter().enumerate() {
        let source_id = index as i64 + 1;
        conn.execute(
            "INSERT INTO dict_source (id, code, display_name, license_kind, license_text, attribution, \
             source_version, source_url, lang) VALUES (?1, ?2, ?2, 'open', 't', 'a', 'v', 'u', 'zh')",
            rusqlite::params![source_id, source.code],
        )
        .unwrap();
        if version >= 4 {
            conn.execute(
                "UPDATE dict_source SET gloss_lang = ?1 WHERE id = ?2",
                rusqlite::params![source.gloss_lang, source_id],
            )
            .unwrap();
        }
        for entry in &source.entries {
            conn.execute(
                "INSERT INTO dict_entry (source_id, lang, headword, reading) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![source_id, entry.lang, entry.headword, entry.reading],
            )
            .unwrap();
            let entry_id = conn.last_insert_rowid();
            if version >= 4 {
                conn.execute(
                    "UPDATE dict_entry SET reading_key = ?1 WHERE id = ?2",
                    rusqlite::params![entry.reading.and_then(pinyin_key), entry_id],
                )
                .unwrap();
            }
            for (ord, gloss) in entry.glosses.iter().enumerate() {
                conn.execute(
                    "INSERT INTO dict_sense (entry_id, source_id, gloss, ord) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![entry_id, source_id, gloss, ord as i64],
                )
                .unwrap();
            }
        }
    }
    conn.execute_batch(
        "INSERT INTO entry_fts(entry_fts) VALUES('rebuild');
         INSERT INTO sense_fts(sense_fts) VALUES('rebuild');
         INSERT INTO sense_fts_nd(sense_fts_nd) VALUES('rebuild');",
    )
    .unwrap();
}

fn base_sources() -> Vec<Source> {
    vec![
        Source {
            code: "fx-vi",
            gloss_lang: "vi",
            entries: vec![
                entry("zh", "你好", Some("nǐ hǎo"), "xin chào"),
                entry("zh", "您好", Some("nín hǎo"), "xin chào ngài"),
                entry("zh", "你好吗", Some("nǐ hǎo ma"), "bạn khỏe không"),
                entry("zh", "吃", Some("chī"), "ăn"),
                entry("zh", "安", Some("ān"), "an"),
                entry("zh", "平安", Some("píng ān"), "bình an"),
                entry("zh", "女", Some("nǚ (nü³)"), "con gái"),
                entry("zh", "奴", Some("nú"), "nô lệ"),
            ],
        },
        Source {
            code: "fx-en",
            gloss_lang: "en",
            entries: vec![
                entry("zh", "你好", Some("ni3 hao3"), "hello; hi"),
                entry("zh", "嗨", Some("hāi"), "xin chào"),
            ],
        },
        Source {
            code: "fx-vie",
            gloss_lang: "vi",
            entries: vec![
                entry("en", "hello", None, "xin chào"),
                entry("en", "an", None, "một"),
                entry("en", "eat", None, "ăn"),
            ],
        },
    ]
}

fn open_base(tag: &str, version: u32, sources: &[Source]) -> (PathBuf, DictLayers) {
    let dir = temp_dir(tag);
    build_layer(&dir, "base.db", "base", version, sources);
    let layers = DictLayers::open(&dir);
    (dir, layers)
}

fn run(layers: &DictLayers, query: &str) -> QuickLookupResponse {
    quick_lookup(Some(layers), query, &BTreeSet::new(), &[])
}

fn group(response: &QuickLookupResponse, id: QuickGroupId) -> &QuickGroup {
    response
        .groups
        .iter()
        .find(|g| g.id == id)
        .unwrap_or_else(|| {
            panic!(
                "no group {id:?} in {:?}",
                response.groups.iter().map(|g| g.id).collect::<Vec<_>>()
            )
        })
}

fn headwords(group: &QuickGroup) -> Vec<&str> {
    group.rows.iter().map(|r| r.headword.as_str()).collect()
}

fn finish(dir: PathBuf, layers: DictLayers) {
    layers.close();
    drop(layers);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn pinyin_key_follows_the_shared_fixture_table() {
    let table = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../tools/dict-build/tests/fixtures/pinyin_key.tsv"),
    )
    .unwrap();
    let mut rows = 0;
    for line in table.lines() {
        let (input, expected) = line.split_once('\t').expect("tab-separated row");
        let want = (expected != "NULL").then(|| expected.to_owned());
        assert_eq!(pinyin_key(input), want, "input {input:?}");
        rows += 1;
    }
    assert!(rows >= 20, "fixture table was emptied, {rows} rows");
}

#[test]
fn a_han_query_answers_exactly_one_zh_vi_group_with_the_panel_rows() {
    let (dir, layers) = open_base("han", 4, &base_sources());
    let response = run(&layers, "你好");
    assert_eq!(response.groups.len(), 1);
    let only = &response.groups[0];
    assert_eq!(only.id, QuickGroupId::ChineseToVietnamese);
    assert_eq!(only.status, QuickStatus::Rows);
    assert_eq!(headwords(only), vec!["你好"]);
    assert_eq!(
        only.rows[0].source_count, 2,
        "two sources carry the headword"
    );
    assert_eq!(only.rows[0].gloss.as_deref(), Some("xin chào"));
    finish(dir, layers);
}

#[test]
fn a_latin_query_answers_three_groups_in_a_fixed_order() {
    let (dir, layers) = open_base("order", 4, &base_sources());
    for query in ["nihao", "xin chào", "an", "zzzz"] {
        let ids: Vec<QuickGroupId> = run(&layers, query).groups.iter().map(|g| g.id).collect();
        assert_eq!(
            ids,
            vec![
                QuickGroupId::Pinyin,
                QuickGroupId::VietnameseToChinese,
                QuickGroupId::EnglishToVietnamese
            ],
            "query {query:?}"
        );
    }
    finish(dir, layers);
}

#[test]
fn every_pinyin_spelling_of_one_phrase_gives_one_result_set() {
    let (dir, layers) = open_base("spellings", 4, &base_sources());
    let reference = headwords(group(&run(&layers, "nihao"), QuickGroupId::Pinyin))
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert_eq!(
        reference.first().map(String::as_str),
        Some("你好"),
        "exact key first"
    );
    assert!(
        reference.contains(&"你好吗".to_owned()),
        "prefix match after the exact one: {reference:?}"
    );
    let exact_at = reference.iter().position(|h| h == "你好").unwrap();
    let prefix_at = reference.iter().position(|h| h == "你好吗").unwrap();
    assert!(exact_at < prefix_at);
    for spelling in ["ni hao", "ni3hao3", "nǐhǎo", "NI3 HAO3"] {
        let got = headwords(group(&run(&layers, spelling), QuickGroupId::Pinyin))
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        assert_eq!(got, reference, "spelling {spelling:?}");
    }
    finish(dir, layers);
}

#[test]
fn a_pinyin_row_carries_the_vietnamese_source_and_gloss_not_the_english_one() {
    let (dir, layers) = open_base("pinyin-row", 4, &base_sources());
    let response = run(&layers, "nihao");
    let rows = &group(&response, QuickGroupId::Pinyin).rows;
    assert_eq!(rows[0].headword, "你好");
    assert_eq!(rows[0].source_code, "fx-vi");
    assert_eq!(rows[0].gloss.as_deref(), Some("xin chào"));
    assert_eq!(rows[0].reading.as_deref(), Some("nǐ hǎo"));
    finish(dir, layers);
}

#[test]
fn nv_finds_the_umlaut_word_through_all_three_spellings() {
    let (dir, layers) = open_base("nv", 4, &base_sources());
    for spelling in ["nv", "nu:", "nǚ"] {
        let found = headwords(group(&run(&layers, spelling), QuickGroupId::Pinyin)).contains(&"女");
        assert!(found, "spelling {spelling:?}");
    }
    finish(dir, layers);
}

#[test]
fn vietnamese_to_chinese_filters_both_the_entry_language_and_the_gloss_language() {
    let (dir, layers) = open_base("both-sides", 4, &base_sources());
    let response = run(&layers, "xin chào");
    let rows = &group(&response, QuickGroupId::VietnameseToChinese).rows;
    let found = headwords(group(&response, QuickGroupId::VietnameseToChinese));
    assert_eq!(found.first(), Some(&"你好"), "whole gloss first: {found:?}");
    assert!(
        found.contains(&"您好"),
        "gloss that starts with the query: {found:?}"
    );
    assert!(
        !found.contains(&"hello"),
        "an lang='en' entry with a Vietnamese gloss must not appear"
    );
    assert!(
        !found.contains(&"嗨"),
        "a zh entry whose gloss is English must not appear"
    );
    assert!(rows.iter().all(|r| r.source_code == "fx-vi"));
    finish(dir, layers);
}

#[test]
fn an_accented_query_stays_accented_and_a_plain_one_finds_both_with_the_plain_gloss_first() {
    let (dir, layers) = open_base("accent", 4, &base_sources());
    let accented_response = run(&layers, "ăn");
    let accented = headwords(group(&accented_response, QuickGroupId::VietnameseToChinese));
    assert_eq!(accented, vec!["吃"]);
    let plain_response = run(&layers, "an");
    let plain = headwords(group(&plain_response, QuickGroupId::VietnameseToChinese));
    assert!(plain.contains(&"吃") && plain.contains(&"安"), "{plain:?}");
    let first = plain.iter().position(|h| *h == "安").unwrap();
    let second = plain.iter().position(|h| *h == "吃").unwrap();
    assert!(
        first < second,
        "the gloss that equals the query unfolded goes first: {plain:?}"
    );
    finish(dir, layers);
}

#[test]
fn a_query_without_any_hit_is_named_empty_in_every_group() {
    let (dir, layers) = open_base("none", 4, &base_sources());
    let response = run(&layers, "qqqq");
    assert!(response.layers_loaded);
    for g in &response.groups {
        assert_eq!(g.status, QuickStatus::Empty, "{:?}", g.id);
        assert!(g.rows.is_empty() && g.error.is_none());
    }
    finish(dir, layers);
}

#[test]
fn a_v3_file_reports_outdated_for_pinyin_and_vietnamese_but_still_answers_english() {
    let (dir, layers) = open_base("v3", 3, &base_sources());
    let response = run(&layers, "hello");
    assert_eq!(
        group(&response, QuickGroupId::Pinyin).status,
        QuickStatus::OutdatedDictionary
    );
    assert_eq!(
        group(&response, QuickGroupId::VietnameseToChinese).status,
        QuickStatus::OutdatedDictionary
    );
    let english = group(&response, QuickGroupId::EnglishToVietnamese);
    assert_eq!(english.status, QuickStatus::Rows);
    assert_eq!(headwords(english), vec!["hello"]);
    assert!(response.skipped.is_empty(), "a v3 file still loads");
    let han = run(&layers, "你好");
    assert_eq!(
        han.groups[0].status,
        QuickStatus::Rows,
        "Chinese-to-Vietnamese still works on v3"
    );
    finish(dir, layers);
}

fn crowd(count: usize) -> Vec<Source> {
    const WORDS: [&str; 22] = [
        "甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸", "子", "丑", "寅", "卯", "辰",
        "巳", "午", "未", "申", "酉", "戌", "亥",
    ];
    vec![Source {
        code: "fx-crowd",
        gloss_lang: "vi",
        entries: WORDS[..count]
            .iter()
            .map(|word| entry("zh", word, Some("xx"), "đám đông"))
            .collect(),
    }]
}

#[test]
fn twenty_one_hits_give_twenty_rows_and_has_more_while_exactly_twenty_do_not() {
    let (dir, layers) = open_base("cut21", 4, &crowd(21));
    let response = run(&layers, "xx");
    let pinyin = group(&response, QuickGroupId::Pinyin);
    assert_eq!((pinyin.rows.len(), pinyin.has_more), (20, true));
    let gloss = group(&response, QuickGroupId::VietnameseToChinese);
    let _ = gloss;
    let by_gloss = run(&layers, "đám đông");
    let vi = group(&by_gloss, QuickGroupId::VietnameseToChinese);
    assert_eq!((vi.rows.len(), vi.has_more), (20, true));
    finish(dir, layers);

    let (dir, layers) = open_base("cut20", 4, &crowd(20));
    let response = run(&layers, "xx");
    let pinyin = group(&response, QuickGroupId::Pinyin);
    assert_eq!((pinyin.rows.len(), pinyin.has_more), (20, false));
    let vi = group(&run(&layers, "đám đông"), QuickGroupId::VietnameseToChinese).clone();
    assert_eq!((vi.rows.len(), vi.has_more), (20, false));
    finish(dir, layers);
}

#[test]
fn a_disabled_source_gives_no_row_in_the_pinyin_or_vietnamese_groups() {
    let (dir, layers) = open_base("disabled", 4, &base_sources());
    let disabled = BTreeSet::from(["fx-vi".to_owned()]);
    let response = quick_lookup(Some(&layers), "nihao", &disabled, &[]);
    let rows = &group(&response, QuickGroupId::Pinyin).rows;
    assert!(!rows.is_empty(), "fx-en still carries the reading");
    assert!(rows.iter().all(|r| r.source_code != "fx-vi"));
    let response = quick_lookup(Some(&layers), "xin chào", &disabled, &[]);
    assert!(
        group(&response, QuickGroupId::VietnameseToChinese)
            .rows
            .is_empty()
    );
    finish(dir, layers);
}

#[test]
fn removing_one_layer_file_leaves_the_other_groups_answering() {
    let dir = temp_dir("layers");
    build_layer(&dir, "base.db", "base", 4, &base_sources());
    build_layer(
        &dir,
        "extra.db",
        "extra",
        4,
        &[Source {
            code: "fx-extra",
            gloss_lang: "vi",
            entries: vec![entry("zh", "你们好", Some("nǐ men hǎo"), "chào các bạn")],
        }],
    );
    let both = DictLayers::open(&dir);
    assert!(headwords(group(&run(&both, "nimenhao"), QuickGroupId::Pinyin)).contains(&"你们好"));
    both.close();
    drop(both);
    fs::remove_file(dir.join("extra.db")).unwrap();
    let one = DictLayers::open(&dir);
    let response = run(&one, "nihao");
    assert_eq!(
        group(&response, QuickGroupId::Pinyin).status,
        QuickStatus::Rows
    );
    assert_eq!(
        group(&response, QuickGroupId::VietnameseToChinese).status,
        QuickStatus::Empty
    );
    finish(dir, one);
}

#[test]
fn no_layers_at_all_is_flagged_instead_of_looking_like_an_empty_answer() {
    let response = quick_lookup(None, "nihao", &BTreeSet::new(), &[]);
    assert!(!response.layers_loaded);
}

#[test]
fn a_layer_that_breaks_at_query_time_is_a_named_error_not_an_empty_group() {
    let (dir, layers) = open_base("closed", 4, &base_sources());
    layers.close();
    let response = run(&layers, "nihao");
    let pinyin = group(&response, QuickGroupId::Pinyin);
    assert_eq!(pinyin.status, QuickStatus::Error);
    assert!(pinyin.error.is_some());
    let vi = group(&response, QuickGroupId::VietnameseToChinese);
    assert_eq!(vi.status, QuickStatus::Error);
    finish(dir, layers);
}

#[test]
fn the_wire_names_of_groups_and_states_are_the_documented_identifiers() {
    let (dir, layers) = open_base("wire", 4, &base_sources());
    let value = serde_json::to_value(run(&layers, "nihao")).unwrap();
    let ids: Vec<&str> = value["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["pinyin", "vi_zh", "en_vi"]);
    assert_eq!(value["groups"][0]["status"], "rows");
    assert_eq!(value["route"], "en");
    let mut keys: Vec<&str> = value["groups"][0]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec!["error", "has_more", "id", "incomplete", "rows", "status"]
    );
    let mut row_keys: Vec<&str> = value["groups"][0]["rows"][0]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    row_keys.sort_unstable();
    assert_eq!(
        row_keys,
        vec![
            "gloss",
            "headword",
            "headword_simp",
            "reading",
            "source_code",
            "source_count",
            "source_name"
        ]
    );
    finish(dir, layers);
}

/// NFR1 on the real dictionaries: `AURA_QUICK_DICT_DIR` (default `tools/dict-build/out`).
/// `cargo test --release --test quick_lookup -- --ignored --nocapture bench_quick_lookup`
#[test]
#[ignore = "needs the four built v4 .db files; prints timings"]
fn bench_quick_lookup_on_the_real_dictionaries() {
    let dir = std::env::var("AURA_QUICK_DICT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tools/dict-build/out")
        });
    if !dir.is_dir() {
        eprintln!("skipped: {} is not a directory", dir.display());
        return;
    }
    let layers = DictLayers::open(&dir);
    assert_eq!(layers.layers().len(), 4, "skipped: {:?}", layers.skipped());
    let shapes: [&[&str]; 4] = [
        &[
            "你好",
            "中国",
            "学习",
            "山",
            "人民",
            "我们",
            "电脑",
            "女",
            "朋友",
            "喜欢",
            "国家",
            "银行",
            "吃饭",
            "大学生",
            "天气",
            "书",
            "时间",
            "工作",
            "中華",
            "謝謝",
        ],
        &[
            "nihao", "ni", "a", "zhongguo", "xue", "xiexie", "nv", "ren", "wo", "wo men", "shi",
            "peng you", "dian nao", "yin hang", "tian qi", "shu", "gong zuo", "zhong", "ma", "xi",
        ],
        &[
            "xin chào",
            "ăn",
            "người",
            "nước",
            "cảm ơn",
            "yêu",
            "học",
            "bạn bè",
            "tình yêu",
            "máy tính",
            "ngân hàng",
            "thời gian",
            "công việc",
            "trời",
            "đi",
            "một",
            "nhà",
            "lớn",
            "con",
            "cái",
        ],
        &[
            "an",
            "nguoi",
            "nuoc",
            "hello",
            "love",
            "ban",
            "thoi gian",
            "ngan hang",
            "the",
            "cong viec",
            "xin chao",
            "yeu",
            "hoc",
            "tinh yeu",
            "may tinh",
            "di",
            "mot",
            "nha",
            "con",
            "cai",
        ],
    ];
    let mut queries: Vec<&str> = shapes.iter().flat_map(|s| s.iter().copied()).collect();
    for _ in 0..1 {
        queries.extend(shapes.iter().flat_map(|s| s.iter().copied().take(30)));
    }
    let mut micros: Vec<(u128, &str)> = Vec::new();
    for query in &queries {
        let _ = run(&layers, query);
    }
    for _ in 0..3 {
        for query in &queries {
            let started = std::time::Instant::now();
            let response = run(&layers, query);
            micros.push((started.elapsed().as_micros(), *query));
            assert!(response.layers_loaded);
        }
    }
    if std::env::var("AURA_QUICK_BREAKDOWN").is_ok() {
        use auratranslate_lib::core::dict::{pinyin_group, vietnamese_to_chinese_group};
        let none = BTreeSet::new();
        for query in shapes.iter().flat_map(|s| s.iter().copied()) {
            let t = std::time::Instant::now();
            let _ = pinyin_group(&layers, query, &none);
            let a = t.elapsed().as_micros();
            let t = std::time::Instant::now();
            let _ = vietnamese_to_chinese_group(&layers, query, &none);
            let b = t.elapsed().as_micros();
            if a + b > 30_000 {
                eprintln!("  breakdown {query:?}: pinyin {a} us, vi_zh {b} us");
            }
        }
    }
    micros.sort_unstable();
    let pick = |p: f64| micros[((micros.len() as f64 * p).ceil() as usize).saturating_sub(1)];
    let (p50, p95, max) = (pick(0.50), pick(0.95), *micros.last().unwrap());
    eprintln!(
        "quick_lookup {} runs over {} distinct queries: p50 {} us, p95 {} us ({:?}), max {} us ({:?})",
        micros.len(),
        queries.len(),
        p50.0,
        p95.0,
        p95.1,
        max.0,
        max.1
    );
    for (us, q) in micros.iter().rev().take(8) {
        eprintln!("  slow: {us} us  {q:?}");
    }
    assert!(p95.0 < 100_000, "NFR1: p95 {} us", p95.0);
    layers.close();
}
