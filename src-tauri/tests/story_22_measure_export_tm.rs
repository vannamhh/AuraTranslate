//! Hand-run measurements: `export_preview` on a 2,000-segment Chapter and `tm_export_counts` on a
//! 200,000-pair tier. Prints numbers only; no threshold assertion.
//!
//!     cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
//!       --test story_22_measure_export_tm -- --ignored --nocapture --test-threads=1

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use auratranslate_lib::commands::export::{export_content_of, export_preview};
use auratranslate_lib::commands::project::create_work_from_text;
use auratranslate_lib::commands::tm::tm_export_counts;
use auratranslate_lib::core::export::{ExportFormat, ExportOptions, ExportScope};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};

const CHAPTER_SEGMENTS: usize = 2_000;
const TM_PAIRS: usize = 200_000;
const PREVIEW_RUNS: usize = 200;
const COUNT_RUNS: usize = 7;

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-story22-{}-{tag}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn sentence(i: usize) -> String {
    format!("第{i}章的句子，主角在北凉城外遇见了一位旧友，两人并肩走过长街，谈起往事。")
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
fn measure_export_preview_on_a_2000_segment_chapter() {
    let root = scratch("preview");
    let open = create_work_from_text(&root, "Tac Pham", "zh", "", "Chuong mot.".to_owned())
        .expect("create Work");
    let chapter_id = open.chapter_id;
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("DELETE FROM segment", [])?;
            let mut stmt = tx.prepare(
                "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, target_text, \
                 is_target_paragraph_end, retired_at, created_at, updated_at, status, is_omitted) \
                 VALUES (?1, ?2, ?3, 1, ?4, 1, NULL, 't', 't', 'draft', 0)",
            )?;
            for i in 0..CHAPTER_SEGMENTS {
                stmt.execute(rusqlite::params![
                    chapter_id,
                    i64::try_from(i).unwrap_or(0),
                    sentence(i),
                    format!(
                        "Cau dich so {i}, nhan vat chinh gap lai mot nguoi ban cu ngoai thanh."
                    )
                ])?;
            }
            Ok(())
        })
        .expect("seed segments");
    let global = Store::open(StoreSpec::global(scratch("preview-g").join("global.db")))
        .expect("open global.db");
    let content = export_content_of(Some(&open), ExportOptions::default(), Some(&global))
        .expect("export content");

    for (name, format) in [
        ("plain", ExportFormat::Plain),
        ("markdown", ExportFormat::Markdown),
        ("docx_two_column", ExportFormat::DocxTwoColumn),
        ("docx_one_block", ExportFormat::DocxOneBlock),
    ] {
        let scope = ExportScope::Work;
        let run = || {
            export_preview(Some(&open), &scope, format, &content, None, Some(&global))
                .expect("preview")
        };
        let rows = run().rows.len();
        let mut times = Vec::with_capacity(PREVIEW_RUNS);
        for _ in 0..PREVIEW_RUNS {
            let t = Instant::now();
            let _ = run();
            times.push(t.elapsed());
        }
        times.sort();
        println!(
            "S22_EXPORT_PREVIEW\tformat={name}\tsegments={CHAPTER_SEGMENTS}\trows={rows}\truns={PREVIEW_RUNS}\tp50_ms={:.3}\tp95_ms={:.3}\tmax_ms={:.3}",
            ms(percentile(&times, 0.50)),
            ms(percentile(&times, 0.95)),
            ms(*times.last().expect("times"))
        );
    }
}

fn seed_tm(store: &Store, count: usize) {
    store
        .write(move |tx: &Transaction<'_>| {
            let mut stmt = tx.prepare(
                "INSERT INTO tm_unit (source_text, target_text, translation_origin, created_at) \
                 VALUES (?1, ?2, ?3, '2026-03-01T10:20:30.123Z')",
            )?;
            for i in 0..count {
                let origin = if i % 2 == 0 { "self" } else { "other" };
                stmt.execute((sentence(i), format!("Ban dich so {i}"), origin))?;
            }
            Ok(())
        })
        .expect("seed tm_unit");
}

#[test]
#[ignore = "hand-run measurement; use the bench-release profile"]
fn measure_tm_export_counts_on_200k_pairs() {
    let root = scratch("tm");
    let open = create_work_from_text(&root, "Tac Pham", "zh", "", "Chuong mot.".to_owned())
        .expect("create Work");
    let global =
        Store::open(StoreSpec::global(scratch("tm-g").join("global.db"))).expect("open global.db");
    seed_tm(&open.store, TM_PAIRS);
    seed_tm(&global, TM_PAIRS);

    for tier in ["work", "global"] {
        let counts = tm_export_counts(Some(&global), Some(&open), tier).expect("counts");
        let mut times = Vec::with_capacity(COUNT_RUNS);
        for _ in 0..COUNT_RUNS {
            let t = Instant::now();
            let _ = tm_export_counts(Some(&global), Some(&open), tier).expect("counts");
            times.push(t.elapsed());
        }
        times.sort();
        println!(
            "S22_TM_EXPORT_COUNTS\ttier={tier}\tpairs_per_tier={TM_PAIRS}\tall={}\tmine={}\truns={COUNT_RUNS}\tp50_ms={:.1}\tmax_ms={:.1}",
            counts.all,
            counts.mine,
            ms(percentile(&times, 0.50)),
            ms(*times.last().expect("times"))
        );
    }
}
