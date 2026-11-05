//! Hand-run measurement of the convention matcher against AD-67 rule 11: build at 2,000 rules,
//! one sentence, and a 9,850-sentence Chapter, then the whole `convention_scan_chapter` on a real
//! Work. It asserts the budgets and prints the numbers with the population they were taken on.
//!
//!     cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
//!       --test story_16_1_measure_convention -- --ignored --nocapture --test-threads=1

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use auratranslate_lib::commands::convention::{ConventionMatcherState, convention_scan_chapter};
use auratranslate_lib::commands::project::create_work_from_text;
use auratranslate_lib::core::convention::{Matcher, Rule, RuleTier, rule_key};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};

const RULES: usize = 2_000;
const SENTENCES: usize = 9_850;
const CHAPTER_RUNS: usize = 20;
const SENTENCE_RUNS: usize = 5;
const BUILD_RUNS: usize = 20;

const BUILD_BUDGET_MS: f64 = 20.0;
const SENTENCE_P95_BUDGET_MS: f64 = 1.0;
const CHAPTER_P95_BUDGET_MS: f64 = 250.0;

const SYLLABLES: [&str; 40] = [
    "bảo", "đảm", "an", "toàn", "người", "việc", "làm", "thật", "sự", "cần", "thiết", "kỷ", "luật",
    "quốc", "gia", "nghiệp", "vụ", "phát", "triển", "hệ", "thống", "chính", "xác", "nhanh",
    "chóng", "kết", "quả", "tốt", "đẹp", "mạnh", "mẽ", "yên", "tâm", "đời", "sống", "lớn", "nhỏ",
    "xa", "gần", "mới",
];

fn rules() -> Vec<Rule> {
    let mut state = 0x2545_f491_4f6c_dd1du64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        usize::try_from(state % 1_000_003).expect("fits")
    };
    let mut out = vec![Rule {
        key: rule_key("đảm bảo"),
        avoid_phrase: "đảm bảo".to_owned(),
        replacement_phrase: Some("bảo đảm".to_owned()),
        tier: RuleTier::Global,
    }];
    while out.len() < RULES {
        let words = 2 + next() % 3;
        let phrase: Vec<&str> = (0..words)
            .map(|_| SYLLABLES[next() % SYLLABLES.len()])
            .collect();
        let avoid = format!("{} #{}", phrase.join(" "), out.len());
        out.push(Rule {
            key: rule_key(&avoid),
            avoid_phrase: avoid,
            replacement_phrase: None,
            tier: RuleTier::Global,
        });
    }
    out
}

fn sentence(i: usize) -> String {
    let hit = if i % 20 < 1 { "Đảm bảo " } else { "" };
    format!(
        "{hit}Người dịch ngồi đọc lại chương số {i}, thấy mọi việc đã làm xong và chẳng còn điều gì phải lo, nhưng vẫn muốn soát thêm một lượt cho yên tâm."
    )
}

fn percentile(sorted: &[Duration], p: f64) -> Duration {
    let idx = ((sorted.len() as f64 * p).ceil() as usize).clamp(1, sorted.len()) - 1;
    sorted[idx]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

#[test]
#[ignore = "hand-run measurement; use the bench-release profile"]
fn measure_convention_matcher_and_chapter_scan() {
    let texts: Vec<String> = (0..SENTENCES).map(sentence).collect();
    let rule_list = rules();
    println!(
        "build: {}, population: {RULES} rules (one real), {SENTENCES} sentences of ~150 chars, 1 in 20 matching",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "bench-release"
        }
    );

    let mut builds: Vec<Duration> = (0..BUILD_RUNS)
        .map(|_| {
            let started = Instant::now();
            let matcher = Matcher::build(rule_list.clone());
            let took = started.elapsed();
            std::hint::black_box(matcher);
            took
        })
        .collect();
    builds.sort();
    println!(
        "matcher build: median {:.2} ms, p95 {:.2} ms",
        ms(builds[builds.len() / 2]),
        ms(percentile(&builds, 0.95))
    );

    let matcher = Matcher::build(rule_list);
    let mut per_sentence: Vec<Duration> = Vec::new();
    for _ in 0..SENTENCE_RUNS {
        for text in &texts {
            let started = Instant::now();
            std::hint::black_box(matcher.scan(text));
            per_sentence.push(started.elapsed());
        }
    }
    per_sentence.sort();
    let sentence_p95 = ms(percentile(&per_sentence, 0.95));
    println!(
        "one sentence: median {:.4} ms, p95 {sentence_p95:.4} ms, max {:.4} ms",
        ms(per_sentence[per_sentence.len() / 2]),
        ms(per_sentence[per_sentence.len() - 1])
    );

    let mut chapters: Vec<Duration> = (0..CHAPTER_RUNS)
        .map(|_| {
            let started = Instant::now();
            for text in &texts {
                std::hint::black_box(matcher.scan(text));
            }
            started.elapsed()
        })
        .collect();
    chapters.sort();
    let chapter_p95 = ms(percentile(&chapters, 0.95));
    println!(
        "matcher over the Chapter: median {:.1} ms, p95 {chapter_p95:.1} ms",
        ms(chapters[chapters.len() / 2])
    );

    assert!(
        ms(percentile(&builds, 0.95)) <= BUILD_BUDGET_MS,
        "build budget"
    );
    assert!(sentence_p95 <= SENTENCE_P95_BUDGET_MS, "sentence budget");
    assert!(chapter_p95 <= CHAPTER_P95_BUDGET_MS, "chapter budget");

    let dir = std::env::temp_dir().join(format!("auratranslate-story16-1-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch dir");
    let global =
        Store::open(StoreSpec::global(PathBuf::from(&dir).join("global.db"))).expect("global");
    let cache = ConventionMatcherState::default();
    global
        .write(|tx: &Transaction<'_>| {
            let mut stmt = tx.prepare(
                "INSERT INTO convention_rule (avoid_phrase, replacement_phrase, rule_key, enabled, created_at) \
                 VALUES (?1, ?2, ?3, 1, 't')",
            )?;
            for rule in rules() {
                stmt.execute((&rule.avoid_phrase, &rule.replacement_phrase, &rule.key))?;
            }
            Ok(())
        })
        .expect("seed rules");
    let open = create_work_from_text(&dir, "Tac Pham", "en", "", "One.".to_owned()).expect("work");
    let chapter_id = open.chapter_id;
    let seeded = texts.clone();
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("DELETE FROM segment", [])?;
            let mut stmt = tx.prepare(
                "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, target_text, \
                 is_target_paragraph_end, retired_at, created_at, updated_at, status, is_omitted) \
                 VALUES (?1, ?2, 'Source.', 1, ?3, 1, NULL, 't', 't', 'draft', 0)",
            )?;
            for (i, text) in seeded.iter().enumerate() {
                stmt.execute((chapter_id, i64::try_from(i).expect("ord"), text))?;
            }
            Ok(())
        })
        .expect("seed segments");
    let state = Mutex::new(Some(open));
    let cold = Instant::now();
    let first = convention_scan_chapter(Some(&global), Some(&state), &cache, 1).expect("scan");
    println!(
        "convention_scan_chapter, cold (builds the matcher, reads the Chapter, loads the Glossary): {:.1} ms, {} marked sentences",
        ms(cold.elapsed()),
        first.segments.len()
    );
    let mut wires: Vec<Duration> = (0..CHAPTER_RUNS)
        .map(|run| {
            let started = Instant::now();
            std::hint::black_box(
                convention_scan_chapter(
                    Some(&global),
                    Some(&state),
                    &cache,
                    u64::try_from(run).expect("run"),
                )
                .expect("scan"),
            );
            started.elapsed()
        })
        .collect();
    wires.sort();
    println!(
        "convention_scan_chapter, warm: median {:.1} ms, p95 {:.1} ms",
        ms(wires[wires.len() / 2]),
        ms(percentile(&wires, 0.95))
    );
    drop(state);
    drop(global);
    let _ = fs::remove_dir_all(&dir);
}
