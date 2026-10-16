//! `segment_version.translation_origin` (AD-55, migration step 39): column, CHECK, trigger, the
//! write at sign time, the "in use" version, restore, and the diff each history row carries.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::segment::{
    SegmentTargetEdit, TRANSLATION_ORIGIN_OTHER, TRANSLATION_ORIGIN_SELF, confirm_segment,
    read_segment_history, restore_segment_version, save_segment_targets, unconfirm_edited_segments,
};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::matching::DiffKind;
use auratranslate_lib::core::segment::translation_origin::{VersionText, version_matching_text};
use auratranslate_lib::core::store::{Store, StoreError, StoreSpec, Transaction};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-version-origin-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn work(tag: &str) -> (PathBuf, OpenWork, i64, i64) {
    let root = temp_dir(tag);
    let open = create_work_from_text(&root, "Xuat Xu", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");
    let (id, chapter_id): (i64, i64) = open
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT id, chapter_id FROM segment ORDER BY ord LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
        })
        .expect("doc segment dau");
    (root, open, id, chapter_id)
}

fn finish(root: PathBuf, open: OpenWork) {
    let dir = open.dir.clone();
    drop(open);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&root);
}

fn edit(id: i64, text: &str) -> SegmentTargetEdit {
    SegmentTargetEdit {
        id,
        target_text: text.to_owned(),
    }
}

/// Typed by hand, then signed: the next version is `self`.
fn sign_typed(open: &OpenWork, chapter_id: i64, id: i64, text: &str) {
    unconfirm_edited_segments(Some(open), chapter_id, &[edit(id, text)]).expect("ha trang thai");
    save_segment_targets(Some(open), chapter_id, &[edit(id, text)]).expect("lo ghi");
    confirm_segment(Some(open), id).expect("xac nhan");
}

/// Text put there by another mechanism (text and baseline in one write, AD-47), then signed
/// untouched: the next version carries `origin`.
fn sign_foreign(open: &OpenWork, id: i64, text: &str, origin: &str) {
    let (text, origin) = (text.to_owned(), origin.to_owned());
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET target_text = ?1, translation_origin = ?2, \
                 baseline_target_text = ?1, baseline_translation_origin = ?2, status = 'draft' \
                 WHERE id = ?3",
                (&text, &origin, id),
            )
        })
        .expect("dat van ban ngoai");
    confirm_segment(Some(open), id).expect("xac nhan");
}

fn segment_origin(open: &OpenWork, id: i64) -> String {
    open.store
        .read(move |conn| {
            conn.query_row(
                "SELECT translation_origin FROM segment WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
        })
        .expect("doc xuat xu segment")
}

fn null_origin_of(open: &OpenWork, version_id: i64) {
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment_version SET translation_origin = NULL WHERE id = ?1",
                [version_id],
            )
        })
        .expect("dat NULL cho hang co truoc buoc 39");
}

fn version_id_of(open: &OpenWork, id: i64, text: &str) -> i64 {
    read_segment_history(Some(open), id)
        .expect("doc lich su")
        .versions
        .iter()
        .find(|v| v.target_text == text)
        .unwrap_or_else(|| panic!("khong co phien ban `{text}`"))
        .id
}

fn origin_of_version(open: &OpenWork, version_id: i64) -> Option<String> {
    open.store
        .read(move |conn| {
            conn.query_row(
                "SELECT translation_origin FROM segment_version WHERE id = ?1",
                [version_id],
                |r| r.get(0),
            )
        })
        .expect("doc xuat xu phien ban")
}

fn insert_raw(open: &OpenWork, id: i64, origin: Option<&'static str>) -> Result<usize, StoreError> {
    open.store.write(move |tx: &Transaction<'_>| match origin {
        Some(o) => tx.execute(
            "INSERT INTO segment_version (segment_id, target_text, created_at, translation_origin) \
             VALUES (?1, 'x', 't', ?2)",
            (id, o),
        ),
        None => tx.execute(
            "INSERT INTO segment_version (segment_id, target_text, created_at) \
             VALUES (?1, 'x', 't')",
            [id],
        ),
    })
}

#[test]
fn the_trigger_aborts_an_insert_without_an_origin_and_the_check_refuses_empty_and_unknown_values() {
    let (root, open, id, _) = work("trigger");

    assert!(
        insert_raw(&open, id, None).is_err(),
        "NULL moi phai bi trigger chan"
    );
    assert!(
        insert_raw(&open, id, Some("")).is_err(),
        "'' khong thuoc tap"
    );
    assert!(
        insert_raw(&open, id, Some("ai")).is_err(),
        "gia tri la khong thuoc tap"
    );
    for ok in ["self", "other", "bilingual_import"] {
        assert!(
            insert_raw(&open, id, Some(ok)).is_ok(),
            "`{ok}` phai vao duoc"
        );
    }

    finish(root, open);
}

#[test]
fn opening_a_work_whose_segment_version_holds_an_origin_outside_the_set_is_refused() {
    let (root, open, id, _) = work("open-unknown");
    sign_typed(&open, 1, id, "Ban mot.");
    let dir = open.dir.clone();
    let indexed = auratranslate_lib::core::library::indexer::IndexedWork {
        work_id: open.meta.work_id.clone(),
        atproj_path: open.dir.clone(),
        name: open.meta.name.clone(),
        source_lang: open.meta.source_lang.clone(),
        genre: open.meta.genre.clone(),
        created_at: open.meta.created_at.clone(),
        updated_at: open.meta.updated_at.clone(),
        chapter_count: open.meta.chapter_count,
        status: open.meta.status.clone(),
        status_is_override: open.meta.status_is_override,
        chapter_done_count: open.meta.chapter_done_count,
    };
    drop(open);

    {
        let store = Store::open(StoreSpec::project(dir.join("project.db"))).expect("mo kho");
        store
            .write(|tx: &Transaction<'_>| {
                tx.execute_batch(
                    "PRAGMA ignore_check_constraints = ON; \
                     UPDATE segment_version SET translation_origin = 'la'; \
                     PRAGMA ignore_check_constraints = OFF;",
                )
            })
            .expect("dung fixture sua tay");
        store.close();
    }

    let err = auratranslate_lib::commands::project::open_work(&indexed.work_id, Some(&indexed))
        .expect_err("gia tri la o segment_version phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::StoreUnknownTranslationOrigin);
    assert_eq!(err.code(), "store.unknown_translation_origin");

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn signing_writes_the_same_origin_to_the_segment_and_to_the_new_version() {
    let (root, open, id, chapter_id) = work("sign");

    sign_typed(&open, chapter_id, id, "Toi go tay.");
    sign_foreign(&open, id, "Nguoi khac dich.", TRANSLATION_ORIGIN_OTHER);

    let history = read_segment_history(Some(&open), id).expect("doc lich su");
    let origins: Vec<(&str, Option<&str>)> = history
        .versions
        .iter()
        .map(|v| (v.target_text.as_str(), v.translation_origin.as_deref()))
        .collect();
    assert_eq!(
        origins,
        vec![
            ("Nguoi khac dich.", Some("other")),
            ("Toi go tay.", Some("self"))
        ]
    );
    assert_eq!(segment_origin(&open, id), TRANSLATION_ORIGIN_OTHER);

    finish(root, open);
}

#[test]
fn three_versions_one_unknown_one_mine_one_other_mark_exactly_the_row_in_use_and_diff_all() {
    let (root, open, id, chapter_id) = work("ticket");

    sign_typed(&open, chapter_id, id, "Ban cu truoc buoc ba muoi chin.");
    let old = version_id_of(&open, id, "Ban cu truoc buoc ba muoi chin.");
    null_origin_of(&open, old);
    sign_typed(&open, chapter_id, id, "Ban toi go tay.");
    sign_foreign(&open, id, "Ban de xuat cua AI.", TRANSLATION_ORIGIN_OTHER);

    let history = read_segment_history(Some(&open), id).expect("doc lich su");
    assert_eq!(history.versions.len(), 3);
    let in_use = history.versions[0].id;
    assert_eq!(history.current_version_id, Some(in_use));
    assert_eq!(
        history
            .versions
            .iter()
            .map(|v| v.translation_origin.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("other"), Some("self"), None]
    );
    assert!(
        history.versions[0]
            .spans
            .iter()
            .all(|s| s.kind == DiffKind::Equal),
        "hang dang dung so voi chinh no: toan equal"
    );
    for row in &history.versions[1..] {
        assert!(
            row.spans.iter().any(|s| s.kind != DiffKind::Equal),
            "hang khac van ban dang dung phai co span lech: {:?}",
            row.target_text
        );
    }

    finish(root, open);
}

#[test]
fn of_two_versions_with_the_same_text_only_the_newest_is_in_use() {
    let (root, open, id, chapter_id) = work("twins");

    sign_typed(&open, chapter_id, id, "Ban A.");
    sign_typed(&open, chapter_id, id, "Ban B.");
    sign_typed(&open, chapter_id, id, "Ban A.");

    let history = read_segment_history(Some(&open), id).expect("doc lich su");
    let newest_a = history.versions[0].id;
    assert_eq!(history.versions[0].target_text, "Ban A.");
    assert_eq!(history.versions[2].target_text, "Ban A.");
    assert_eq!(history.current_version_id, Some(newest_a));

    finish(root, open);
}

#[test]
fn a_draft_matching_no_version_has_no_row_in_use() {
    let (root, open, id, chapter_id) = work("draft");

    sign_typed(&open, chapter_id, id, "Ban da ky.");
    save_segment_targets(Some(&open), chapter_id, &[edit(id, "Ban dang soan.")]).expect("lo ghi");

    let history = read_segment_history(Some(&open), id).expect("doc lich su");
    assert_eq!(history.current_version_id, None);
    assert_eq!(history.versions.len(), 1);

    finish(root, open);
}

fn restore_then_sign(open: &OpenWork, id: i64, version_id: i64) {
    restore_segment_version(Some(open), id, version_id, true).expect("khoi phuc");
    confirm_segment(Some(open), id).expect("ky khong sua");
}

#[test]
fn restoring_then_signing_untouched_keeps_other_and_mine_and_turns_unknown_into_other() {
    let (root, open, id, chapter_id) = work("restore-origin");

    sign_typed(&open, chapter_id, id, "Ban cu truoc buoc ba muoi chin.");
    let unknown = version_id_of(&open, id, "Ban cu truoc buoc ba muoi chin.");
    null_origin_of(&open, unknown);
    sign_foreign(&open, id, "Ban nguoi khac.", TRANSLATION_ORIGIN_OTHER);
    sign_typed(&open, chapter_id, id, "Ban toi.");
    let other = version_id_of(&open, id, "Ban nguoi khac.");
    let mine = version_id_of(&open, id, "Ban toi.");

    restore_then_sign(&open, id, other);
    assert_eq!(segment_origin(&open, id), "other");

    restore_then_sign(&open, id, mine);
    assert_eq!(segment_origin(&open, id), TRANSLATION_ORIGIN_SELF);

    restore_then_sign(&open, id, unknown);
    assert_eq!(segment_origin(&open, id), TRANSLATION_ORIGIN_OTHER);
    let newest = read_segment_history(Some(&open), id)
        .expect("doc lich su")
        .versions[0]
        .id;
    assert_eq!(origin_of_version(&open, newest).as_deref(), Some("other"));

    finish(root, open);
}

#[test]
fn after_a_restore_the_row_in_use_and_the_diffs_follow_the_new_text() {
    let (root, open, id, chapter_id) = work("restore-reread");

    sign_typed(&open, chapter_id, id, "Ban mot.");
    sign_typed(&open, chapter_id, id, "Ban hai.");
    let first = version_id_of(&open, id, "Ban mot.");
    assert_eq!(
        read_segment_history(Some(&open), id)
            .expect("doc")
            .current_version_id,
        Some(version_id_of(&open, id, "Ban hai."))
    );

    restore_segment_version(Some(&open), id, first, true).expect("khoi phuc");
    let after = read_segment_history(Some(&open), id).expect("doc lai");
    assert_eq!(after.current_version_id, Some(first));
    let row = after
        .versions
        .iter()
        .find(|v| v.id == first)
        .expect("hang vua khoi phuc");
    assert!(row.spans.iter().all(|s| s.kind == DiffKind::Equal));

    finish(root, open);
}

#[test]
fn a_retired_segment_still_reads_with_its_source_text_labels_and_diff() {
    let (root, open, id, chapter_id) = work("retired");

    sign_typed(&open, chapter_id, id, "Ban mot.");
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET retired_at = '2026-10-16T00:00:00.000Z' WHERE id = ?1",
                [id],
            )
        })
        .expect("cho ve huu");

    let history = read_segment_history(Some(&open), id).expect("doc segment ve huu");
    assert_eq!(history.source_text, "一。");
    assert_eq!(history.current_version_id, Some(history.versions[0].id));
    assert_eq!(
        history.versions[0].translation_origin.as_deref(),
        Some("self")
    );

    finish(root, open);
}

#[test]
fn the_diff_rebuilds_the_version_text_and_the_current_text_and_the_wire_carries_no_baseline() {
    let (root, open, id, chapter_id) = work("diff-wire");

    sign_typed(&open, chapter_id, id, "The dragon roared loudly.");
    sign_typed(&open, chapter_id, id, "The dragon whispered softly.");

    let history = read_segment_history(Some(&open), id).expect("doc lich su");
    let value = serde_json::to_value(&history).expect("serialize");
    let old = &value["versions"][1];
    assert_eq!(old["target_text"], "The dragon roared loudly.");
    assert_eq!(old["translation_origin"], "self");

    let mut from_version = String::new();
    let mut from_current = String::new();
    for span in old["spans"].as_array().expect("spans") {
        let (kind, text) = (
            span["kind"].as_str().expect("kind"),
            span["text"].as_str().expect("text"),
        );
        match kind {
            "equal" => {
                from_version.push_str(text);
                from_current.push_str(text);
            }
            "delete" => from_version.push_str(text),
            "insert" => from_current.push_str(text),
            other => panic!("kind la: {other}"),
        }
    }
    assert_eq!(from_version.trim(), "The dragon roared loudly.");
    assert_eq!(from_current.trim(), "The dragon whispered softly.");
    assert!(value["current_version_id"].is_i64());
    assert_eq!(value["source_text"], "一。");

    assert!(
        !value.to_string().contains("baseline_"),
        "khong DTO nao mang khoa `baseline_*`"
    );

    finish(root, open);
}

#[test]
fn the_selector_takes_the_largest_id_after_trim_and_nfc_and_none_when_nothing_matches() {
    let versions = [
        VersionText {
            id: 3,
            target_text: "caf\u{e9}",
        },
        VersionText {
            id: 9,
            target_text: "  cafe\u{301} ",
        },
        VersionText {
            id: 12,
            target_text: "khac",
        },
        VersionText {
            id: 5,
            target_text: "cafe",
        },
    ];
    assert_eq!(version_matching_text(versions, "caf\u{e9}"), Some(9));
    assert_eq!(version_matching_text(versions, "cafe"), Some(5));
    assert_eq!(version_matching_text(versions, "khong co"), None);
    assert_eq!(version_matching_text([], "x"), None);
}
