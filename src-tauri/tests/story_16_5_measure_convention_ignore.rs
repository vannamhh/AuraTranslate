//! Hand-run measurement of the "not an error" filter on the hot path of `convention_scan_chapter`
//! (AD-67 rules 9 and 11): 2,000 rules, a 9,850-sentence Chapter, 2,000 remembered places, of which
//! every matching sentence has its own. It prints the scan with an empty memory and with the full
//! one, asserts the full one stays inside the Chapter budget, and asserts that the filter really
//! dropped what was remembered, so a fast number cannot come from a filter that never ran.
//!
//!     cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
//!       --test story_16_5_measure_convention_ignore -- --ignored --nocapture --test-threads=1

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use auratranslate_lib::commands::convention::{ConventionMatcherState, convention_scan_chapter};
use auratranslate_lib::commands::project::create_work_from_text;
use auratranslate_lib::core::convention::{Rule, RuleTier, rule_key, signature_of};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};

const RULES: usize = 2_000;
const SENTENCES: usize = 9_850;
const REMEMBERED: usize = 2_000;
const CHAPTER_RUNS: usize = 20;
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

fn is_hit(i: usize) -> bool {
    i.is_multiple_of(20)
}

fn sentence(i: usize) -> String {
    if is_hit(i) {
        format!(
            "Mục {i} ghi rằng đảm bảo an toàn cho người dịch ngồi đọc lại chương số {i}, thấy mọi việc đã làm xong và chẳng còn điều gì phải lo."
        )
    } else {
        format!(
            "Người dịch ngồi đọc lại chương số {i}, thấy mọi việc đã làm xong và chẳng còn điều gì phải lo, nhưng vẫn muốn soát thêm một lượt cho yên tâm."
        )
    }
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
fn measure_convention_scan_with_remembered_places() {
    let texts: Vec<String> = (0..SENTENCES).map(sentence).collect();
    let hits: Vec<usize> = (0..SENTENCES).filter(|i| is_hit(*i)).collect();
    println!(
        "build: {}, population: {RULES} rules (one real), {SENTENCES} sentences, {} matching, {REMEMBERED} remembered places ({} real, {} synthetic)",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "bench-release"
        },
        hits.len(),
        hits.len(),
        REMEMBERED - hits.len()
    );

    let dir = std::env::temp_dir().join(format!("auratranslate-story16-5-{}", std::process::id()));
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

    let run = |generation_base: u64| -> (Vec<Duration>, u32, usize) {
        let mut last = (0, 0);
        let mut times: Vec<Duration> = (0..CHAPTER_RUNS)
            .map(|n| {
                let started = Instant::now();
                let wire = convention_scan_chapter(
                    Some(&global),
                    Some(&state),
                    &cache,
                    generation_base + u64::try_from(n).expect("run"),
                )
                .expect("scan");
                let took = started.elapsed();
                last = (wire.ignored, wire.segments.len());
                std::hint::black_box(&wire);
                took
            })
            .collect();
        times.sort();
        (times, last.0, last.1)
    };

    let _ = run(0);
    let (empty_times, empty_ignored, empty_marked) = run(100);
    println!(
        "convention_scan_chapter, empty memory: median {:.1} ms, p95 {:.1} ms, ignored {empty_ignored}, marked sentences {empty_marked}",
        ms(empty_times[empty_times.len() / 2]),
        ms(percentile(&empty_times, 0.95))
    );
    assert_eq!(empty_ignored, 0);
    assert_eq!(empty_marked, hits.len());

    let real: Vec<(String, String, String)> = hits
        .iter()
        .map(|i| {
            let text = &texts[*i];
            let start = u32::try_from(text.find("đảm bảo").expect("phrase")).expect("fits");
            let before = text[..usize::try_from(start).expect("fits")]
                .encode_utf16()
                .count();
            let start16 = u32::try_from(before).expect("fits");
            let signature =
                signature_of(text, start16, start16 + 7, &rule_key("đảm bảo")).expect("signature");
            (signature.phrase, signature.before, signature.after)
        })
        .collect();
    let synthetic = REMEMBERED - real.len();
    state
        .lock()
        .expect("lock")
        .as_ref()
        .expect("open")
        .store
        .write(move |tx: &Transaction<'_>| {
            let mut stmt = tx.prepare(
                "INSERT INTO convention_ignore (rule_key, avoid_phrase, context_before, context_after, created_at) \
                 VALUES (?1, ?2, ?3, ?4, 't')",
            )?;
            let key = rule_key("đảm bảo");
            for (phrase, before, after) in &real {
                stmt.execute((&key, phrase, before, after))?;
            }
            for n in 0..synthetic {
                let before = format!("một câu khác hẳn số {n} trước ");
                stmt.execute((&key, "đảm bảo", &before, " sau"))?;
            }
            Ok(())
        })
        .expect("seed remembered places");

    let (full_times, full_ignored, full_marked) = run(1_000);
    let full_p95 = ms(percentile(&full_times, 0.95));
    println!(
        "convention_scan_chapter, {REMEMBERED} remembered places on ONE rule key (dense): median {:.1} ms, p95 {full_p95:.1} ms, ignored {full_ignored}, marked sentences {full_marked}",
        ms(full_times[full_times.len() / 2]),
    );
    assert_eq!(
        usize::try_from(full_ignored).expect("fits"),
        hits.len(),
        "the filter must drop every remembered match"
    );
    assert_eq!(full_marked, 0);

    // Same number of remembered places, now spread over the other rules' keys. `signature_matches`
    // rejects a foreign key before it ever compares context, so this is the CHEAPER distribution —
    // it is here to show that the dense one above is the worst case, and that grouping the memory
    // by rule key would therefore buy nothing at this population.
    let spread_keys: Vec<String> = rules()
        .into_iter()
        .skip(1)
        .map(|r| r.key)
        .take(synthetic)
        .collect();
    state
        .lock()
        .expect("lock")
        .as_ref()
        .expect("open")
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "DELETE FROM convention_ignore WHERE avoid_phrase = 'đảm bảo' AND context_after = ' sau'",
                [],
            )?;
            let mut stmt = tx.prepare(
                "INSERT INTO convention_ignore (rule_key, avoid_phrase, context_before, context_after, created_at) \
                 VALUES (?1, ?2, ?3, ?4, 't')",
            )?;
            for (n, key) in spread_keys.iter().enumerate() {
                let before = format!("một câu khác hẳn số {n} trước ");
                stmt.execute((key, "đảm bảo", &before, " sau"))?;
            }
            Ok(())
        })
        .expect("respread remembered places");

    let (spread_times, spread_ignored, spread_marked) = run(2_000);
    let spread_p95 = ms(percentile(&spread_times, 0.95));
    println!(
        "convention_scan_chapter, {REMEMBERED} remembered places SPREAD over many rule keys: median {:.1} ms, p95 {spread_p95:.1} ms, ignored {spread_ignored}, marked sentences {spread_marked}",
        ms(spread_times[spread_times.len() / 2]),
    );
    assert_eq!(
        usize::try_from(spread_ignored).expect("fits"),
        hits.len(),
        "spreading the memory must silence exactly the same places"
    );
    assert_eq!(spread_marked, 0);
    assert!(
        spread_p95 <= CHAPTER_P95_BUDGET_MS,
        "chapter budget, spread memory"
    );
    assert!(full_p95 <= CHAPTER_P95_BUDGET_MS, "chapter budget");

    drop(state);
    drop(global);
    let _ = fs::remove_dir_all(&dir);
}
