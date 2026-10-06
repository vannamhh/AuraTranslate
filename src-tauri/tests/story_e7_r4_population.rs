//! Population builder for the OpenWorkState hold measurement in `e7-r4-ban-do/`: one Work with a 300-segment
//! Chapter plus 100,000 `tm_unit` pairs in the Work tier and 100,000 in Global.
//!
//! Ignored on purpose: it writes into a scratch bench HOME and takes seconds. Run by hand:
//!
//!     AURA_E7_R4_HOME=/tmp/auratranslate-nfr-bench-XXXXXX/home \
//!       cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
//!       --test story_e7_r4_population -- --ignored --nocapture

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use auratranslate_lib::commands::project::create_work_from_text;
use auratranslate_lib::core::library::indexer::Indexer;
use auratranslate_lib::core::store::{Store, StoreSpec};

const SCRATCH_MARKER: &str = "auratranslate-nfr-bench-";
const TM_PAIRS_PER_TIER: usize = 100_000;
const CHAPTER_SEGMENTS: usize = 300;
const CHAPTER_EXACT_HITS: usize = 150;
const WORK_TIER_SEED_BASE: u64 = 0;
const GLOBAL_TIER_SEED_BASE: u64 = 1_000_000;
const CHAPTER_SEED_BASE: u64 = 2_000_000;
const MIN_CHARS: usize = 30;
const MAX_CHARS: usize = 60;

/// Deterministic unique CJK sentence of 30-60 chars ending in `。`. The first four ideographs
/// encode `seed` in base 2000, so two seeds can never collide; the tail is an LCG walk.
fn cjk_sentence(seed: u64) -> String {
    const FIRST: u32 = 0x4E00;
    const SPAN: u64 = 2000;
    let total_chars = MIN_CHARS + (seed % (MAX_CHARS - MIN_CHARS + 1) as u64) as usize;
    let mut out = String::new();
    let mut rest = seed;
    for _ in 0..4 {
        out.push(char::from_u32(FIRST + (rest % SPAN) as u32).expect("ideograph"));
        rest /= SPAN;
    }
    let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    while out.chars().count() < total_chars - 1 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let offset = ((state >> 33) % 20_000) as u32;
        out.push(char::from_u32(FIRST + offset).expect("ideograph"));
    }
    out.push('。');
    out
}

fn bench_home() -> PathBuf {
    let raw = std::env::var_os("AURA_E7_R4_HOME").expect("AURA_E7_R4_HOME must name the scratch bench HOME");
    let home = PathBuf::from(raw);
    assert!(home.is_absolute(), "AURA_E7_R4_HOME must be absolute: {}", home.display());
    assert!(
        home.to_string_lossy().contains(SCRATCH_MARKER),
        "AURA_E7_R4_HOME must contain {SCRATCH_MARKER}: {}",
        home.display()
    );
    home
}

fn seed_tier(store: &Store, seed_base: u64, count: usize, tag: &'static str, hit_seeds: Vec<u64>) {
    store
        .write(move |tx| {
            let mut stmt = tx.prepare(
                "INSERT INTO tm_unit (source_text, target_text, translation_origin, created_at) \
                 VALUES (?1, ?2, 'self', '2026-03-01T10:20:30.123Z')",
            )?;
            for i in 0..count as u64 {
                stmt.execute((cjk_sentence(seed_base + i), format!("Ban dich {tag} {i}")))?;
            }
            for seed in &hit_seeds {
                stmt.execute((cjk_sentence(*seed), format!("Khop chinh xac {tag} {seed}")))?;
            }
            Ok(())
        })
        .expect("seed tm_unit");
}

fn count_tm(store: &Store) -> i64 {
    store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM tm_unit", [], |row| row.get(0)))
        .expect("count tm_unit")
}

fn count_tm_matching(store: &Store, sources: Vec<String>) -> usize {
    store
        .read(move |conn| {
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM tm_unit WHERE source_text = ?1")?;
            let mut hits = 0usize;
            for source in &sources {
                if stmt.query_row([source], |row| row.get::<_, i64>(0))? > 0 {
                    hits += 1;
                }
            }
            Ok(hits)
        })
        .expect("count exact hits")
}

#[test]
#[ignore = "R-4 hand-run builder: writes a 300-segment Work and 200k TM pairs into a scratch bench HOME"]
fn builds_the_e7_r4_population_in_the_scratch_bench_home() {
    let home = bench_home();
    let documents_root = home.join("Documents").join("AuraTranslate");
    let appdata = home.join("Library").join("Application Support").join("com.auratranslate.desktop");
    let work_name = std::env::var("AURA_E7_R4_WORK_NAME").unwrap_or_else(|_| "E7 R4 Work".to_owned());
    let global_path = appdata.join("global.db");
    assert!(!global_path.exists(), "global.db must not exist yet: {}", global_path.display());
    if documents_root.exists() {
        assert!(
            fs::read_dir(&documents_root).expect("read library root").next().is_none(),
            "library root must be empty: {}",
            documents_root.display()
        );
    }
    fs::create_dir_all(&documents_root).expect("create library root");
    fs::create_dir_all(&appdata).expect("create appdata");

    let chapter_sources: Vec<String> =
        (0..CHAPTER_SEGMENTS as u64).map(|i| cjk_sentence(CHAPTER_SEED_BASE + i)).collect();
    let distinct: BTreeSet<&String> = chapter_sources.iter().collect();
    assert_eq!(distinct.len(), CHAPTER_SEGMENTS, "chapter sources must be distinct");

    let opened = create_work_from_text(&documents_root, &work_name, "zh", "", chapter_sources.concat())
        .unwrap_or_else(|e| panic!("create_work_from_text failed: {e:?}"));

    let (chapters, live, draft, empty_target): (i64, i64, i64, i64) = opened
        .store
        .read(|conn| {
            Ok((
                conn.query_row("SELECT COUNT(*) FROM chapter", [], |r| r.get(0))?,
                conn.query_row("SELECT COUNT(*) FROM segment WHERE retired_at IS NULL", [], |r| r.get(0))?,
                conn.query_row(
                    "SELECT COUNT(*) FROM segment WHERE retired_at IS NULL AND status = 'draft'",
                    [],
                    |r| r.get(0),
                )?,
                conn.query_row(
                    "SELECT COUNT(*) FROM segment WHERE retired_at IS NULL AND trim(target_text) = ''",
                    [],
                    |r| r.get(0),
                )?,
            ))
        })
        .expect("read chapter shape");
    assert_eq!(chapters, 1, "Work must have exactly one Chapter");
    assert_eq!(live, CHAPTER_SEGMENTS as i64, "Chapter must hold {CHAPTER_SEGMENTS} live segments");
    assert_eq!(draft, CHAPTER_SEGMENTS as i64, "every segment must be draft");
    assert_eq!(empty_target, CHAPTER_SEGMENTS as i64, "every target must start empty");
    let stored_sources: BTreeSet<String> = opened
        .store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT source_text FROM segment WHERE retired_at IS NULL")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            rows.collect::<auratranslate_lib::core::store::SqlResult<BTreeSet<_>>>()
        })
        .expect("read sources");
    assert_eq!(
        stored_sources,
        chapter_sources.iter().cloned().collect::<BTreeSet<_>>(),
        "stored segment sources must equal the generated sentences"
    );

    let work_hits: Vec<u64> = (0..CHAPTER_EXACT_HITS as u64 / 2).map(|i| CHAPTER_SEED_BASE + i).collect();
    let global_hits: Vec<u64> = (CHAPTER_EXACT_HITS as u64 / 2..CHAPTER_EXACT_HITS as u64)
        .map(|i| CHAPTER_SEED_BASE + i)
        .collect();
    seed_tier(&opened.store, WORK_TIER_SEED_BASE, TM_PAIRS_PER_TIER - work_hits.len(), "work", work_hits);
    let global = Store::open(StoreSpec::global(global_path)).expect("open global.db");
    seed_tier(&global, GLOBAL_TIER_SEED_BASE, TM_PAIRS_PER_TIER - global_hits.len(), "global", global_hits);

    let work_pairs = count_tm(&opened.store);
    let global_pairs = count_tm(&global);
    assert_eq!(work_pairs, TM_PAIRS_PER_TIER as i64, "Work tier population");
    assert_eq!(global_pairs, TM_PAIRS_PER_TIER as i64, "Global tier population");

    let exact_hits = count_tm_matching(&opened.store, chapter_sources.clone())
        + count_tm_matching(&global, chapter_sources.clone());
    assert_eq!(exact_hits, CHAPTER_EXACT_HITS, "chapter sources with an exact TM hit");

    let all_tm_sources_in_range = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT MIN(length(source_text)), MAX(length(source_text)) FROM tm_unit",
                [],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
            )
        })
        .expect("tm source lengths");
    assert!(
        all_tm_sources_in_range.0 >= MIN_CHARS as i64 && all_tm_sources_in_range.1 <= MAX_CHARS as i64,
        "TM source lengths out of range: {all_tm_sources_in_range:?}"
    );

    let indexer = Indexer::open(appdata.join("library-index.db")).expect("open library index");
    indexer.rebuild(&documents_root, Some(&global)).expect("rebuild library index");
    let indexed = indexer
        .list_works(auratranslate_lib::core::library::indexer::WorkQuery::default())
        .expect("list indexed works");
    assert_eq!(indexed.works.len(), 1, "library index must hold exactly one Work");

    println!(
        "E7_R4_POPULATION\twork={work_name:?}\tchapters={chapters}\tsegments={live}\tdraft_segments={draft}\tdistinct_sources={}\twork_tm_pairs={work_pairs}\tglobal_tm_pairs={global_pairs}\tchapter_exact_hits={exact_hits}\ttm_source_chars={}..{}\tverdict=matches_declaration",
        distinct.len(),
        all_tm_sources_in_range.0,
        all_tm_sources_in_range.1
    );
}
