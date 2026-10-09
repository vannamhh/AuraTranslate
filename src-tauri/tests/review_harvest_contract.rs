//! Thu hoạch thuật ngữ từ bản reviewer (FR54, AD-20, AD-52 mục 7). Bản reviewer dựng bằng hàm
//! xuất thật rồi sửa bằng mã: một Tác phẩm thứ hai cùng nguồn, khác bản dịch.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::chapter::merge_chapter_into_previous;
use auratranslate_lib::commands::export::{
    PendingReviewerImportState, ReviewerImportSummaryWire, export_docx_two_column, reviewer_import_confirm,
    reviewer_import_preview,
};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::export::{ExportScope, HarvestFinding, ImageMode, harvest_work, read_review_copy};
use auratranslate_lib::core::glossary::{
    Category, GlossaryTier, TermOrigin, add_manual_term, approve_candidate, confirmed_terms_for_injection, load_tier,
    pending_candidates, reject_candidate,
};
use auratranslate_lib::core::matching::MatchLang;
use auratranslate_lib::core::store::{PROJECT_MIGRATIONS, Store, StoreSpec, Transaction};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-harvest-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

type Pairs = Vec<(String, String)>;

fn pairs(rows: &[(&str, &str)]) -> Pairs {
    rows.iter().map(|&(s, t)| (s.to_owned(), t.to_owned())).collect()
}

struct Work {
    open: OpenWork,
    chapters: Vec<i64>,
    root: PathBuf,
    out: PathBuf,
}

fn build(tag: &str, chapters: &[Pairs]) -> Work {
    let root = temp_dir(tag);
    let out = temp_dir(&format!("{tag}-out"));
    let open = create_work_from_text(&root, "Tac Pham", "zh", "", "Chuong mot.".to_owned()).expect("tao tac pham");
    open.store.write(|tx: &Transaction<'_>| tx.execute("DELETE FROM segment", [])).expect("xoa segment");
    let mut ids = Vec::new();
    for (i, segs) in chapters.iter().enumerate() {
        let id = if i == 0 {
            open.chapter_id
        } else {
            let ord = i64::try_from(i).unwrap_or(0) + 1;
            open.store
                .write(move |tx: &Transaction<'_>| {
                    tx.execute(
                        "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                         VALUES (?1, NULL, 'x', 'not_started', 't', 't')",
                        (ord,),
                    )?;
                    Ok(tx.last_insert_rowid())
                })
                .expect("chen Chuong")
        };
        let segs = segs.clone();
        open.store
            .write(move |tx: &Transaction<'_>| {
                for (n, (source, target)) in segs.iter().enumerate() {
                    tx.execute(
                        "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, target_text, \
                         is_target_paragraph_end, created_at, updated_at, status, is_omitted) \
                         VALUES (?1, ?2, ?3, 1, ?4, 1, 't', 't', 'draft', 0)",
                        rusqlite::params![id, i64::try_from(n).unwrap_or(0) + 1, source, target],
                    )?;
                }
                Ok(())
            })
            .expect("chen segment");
        ids.push(id);
    }
    Work { open, chapters: ids, root, out }
}

impl Work {
    fn docx(&self) -> PathBuf {
        let file =
            export_docx_two_column(Some(&self.open), &ExportScope::Work, ImageMode::File, None, &self.out).expect("xuat docx");
        PathBuf::from(file.path)
    }

    fn import_with(&self, global: Option<&Store>, path: &Path) -> ReviewerImportSummaryWire {
        let state = PendingReviewerImportState::new(None);
        reviewer_import_preview(Some(&self.open), &state, path).expect("xem truoc");
        reviewer_import_confirm(Some(&self.open), global, &state).expect("xac nhan")
    }

    fn harvest(&self, global: &Store) -> Vec<HarvestFinding> {
        harvest_work(&self.open.scope, global, &self.open.store, "zh").expect("thu hoach")
    }

    fn rows(&self, sql: &'static str) -> Vec<String> {
        self.open
            .store
            .read(move |conn| {
                let mut stmt = conn.prepare(sql)?;
                let columns = stmt.column_count();
                let rows = stmt
                    .query_map([], |row| {
                        Ok((0..columns).map(|i| format!("{:?}", row.get_ref(i))).collect::<Vec<_>>().join("|"))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                Ok(rows)
            })
            .expect("doc")
    }
}

fn global_store(tag: &str) -> (Store, PathBuf) {
    let dir = temp_dir(tag);
    (Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db"), dir)
}

const BAC_LUONG: &str = "Bắc Lương vương";
const BAC_LUONG_SWAPPED: &str = "vương Bắc Lương";
const TAIL: [&str; 6] = ["đến rồi.", "đi rồi.", "cười.", "khóc.", "ngủ.", "ăn."];
const SOURCES: [&str; 6] = ["北凉王，来了。", "北凉王，走了。", "北凉王，笑了。", "北凉王，哭了。", "北凉王，睡了。", "北凉王，吃了。"];

fn lines(count: usize, changed: usize) -> (Pairs, Pairs) {
    let mut mine = Vec::new();
    let mut theirs = Vec::new();
    for i in 0..count {
        mine.push((SOURCES[i].to_owned(), format!("{BAC_LUONG} {}", TAIL[i])));
        let used = if i < changed { BAC_LUONG_SWAPPED } else { BAC_LUONG };
        theirs.push((SOURCES[i].to_owned(), format!("{used} {}", TAIL[i])));
    }
    (mine, theirs)
}

fn work_term(mine: &Work, global: &Store, source: &str, translation: &str) {
    add_manual_term(global, Some(&mine.open.store), GlossaryTier::Work, source, Some(translation), "", Category::Person)
        .expect("them thuat ngu Work");
}

fn global_term(mine: &Work, global: &Store, source: &str, translation: &str) {
    add_manual_term(global, Some(&mine.open.store), GlossaryTier::Global, source, Some(translation), "", Category::Person)
        .expect("them thuat ngu Global");
}

fn finding(source: &str, replaced: &str, proposed: &str, changed: i64, seen: i64) -> HarvestFinding {
    HarvestFinding {
        source_term: source.to_owned(),
        replaced_translation: replaced.to_owned(),
        proposed_translation: proposed.to_owned(),
        changed_count: changed,
        seen_count: seen,
    }
}

fn finish(works: &[&Work]) {
    for work in works {
        let _ = fs::remove_dir_all(&work.open.dir);
        let _ = fs::remove_dir_all(&work.root);
        let _ = fs::remove_dir_all(&work.out);
    }
}

fn setup(tag: &str, count: usize, changed: usize) -> (Work, Work, Store, PathBuf) {
    let (mine_rows, their_rows) = lines(count, changed);
    let mine = build(&format!("{tag}-mine"), &[mine_rows]);
    let reviewer = build(&format!("{tag}-rev"), &[their_rows]);
    let (global, dir) = global_store(tag);
    work_term(&mine, &global, "北凉王", BAC_LUONG);
    (mine, reviewer, global, dir)
}


#[test]
fn a_consistent_change_in_four_of_four_pairs_is_one_candidate_even_when_the_order_is_swapped() {
    let (mine, reviewer, global, dir) = setup("consistent", 4, 4);
    let summary = mine.import_with(Some(&global), &reviewer.docx());

    assert_eq!((summary.harvest_candidate_count, summary.harvest_error.as_ref()), (Some(1), None));
    assert_eq!(mine.harvest(&global), vec![finding("北凉王", BAC_LUONG, BAC_LUONG_SWAPPED, 4, 4)]);
    let candidates = pending_candidates(&mine.open.store).expect("doc hang cho");
    let detail = candidates[0].review_harvest.as_ref().expect("ung vien thu hoach");
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        (detail.replaced_translation.as_str(), detail.proposed_translation.as_str(), detail.changed_count, detail.seen_count),
        (BAC_LUONG, BAC_LUONG_SWAPPED, 4, 4)
    );
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_partial_change_reports_changed_over_seen() {
    let (mine, reviewer, global, dir) = setup("partial", 5, 3);
    mine.import_with(Some(&global), &reviewer.docx());

    assert_eq!(mine.harvest(&global), vec![finding("北凉王", BAC_LUONG, BAC_LUONG_SWAPPED, 3, 5)]);
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_reviewer_who_keeps_the_translation_gives_no_candidate() {
    let (mine, reviewer, global, dir) = setup("kept", 4, 0);
    let summary = mine.import_with(Some(&global), &reviewer.docx());

    assert_eq!(summary.harvest_candidate_count, Some(0));
    assert!(mine.harvest(&global).is_empty());
    assert!(pending_candidates(&mine.open.store).expect("doc").is_empty());
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_single_change_gives_no_candidate() {
    let (mine, reviewer, global, dir) = setup("single", 4, 1);
    mine.import_with(Some(&global), &reviewer.docx());

    assert!(mine.harvest(&global).is_empty());
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_rename_sharing_no_word_picks_the_shortest_new_phrase_not_the_one_glued_to_context() {
    let mine = build(
        "rename-mine",
        &[pairs(&[
            ("开门，快点。", "Hắn đả khai cửa lớn."),
            ("开门，慢点。", "Hắn đả khai cửa sổ."),
            ("开门，看看。", "Hắn đả khai cửa sau."),
        ])],
    );
    let reviewer = build(
        "rename-rev",
        &[pairs(&[
            ("开门，快点。", "Hắn mở cửa lớn."),
            ("开门，慢点。", "Hắn mở cửa sổ."),
            ("开门，看看。", "Hắn mở cửa sau."),
        ])],
    );
    let (global, dir) = global_store("rename");
    work_term(&mine, &global, "开门", "đả khai");
    work_term(&mine, &global, "长城", "Vạn Lý Trường Thành cổ xưa hùng vĩ");
    mine.import_with(Some(&global), &reviewer.docx());

    assert_eq!(mine.harvest(&global), vec![finding("开门", "đả khai", "mở", 3, 3)]);
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_rejected_pair_is_never_proposed_again_by_a_later_import() {
    let (mine, reviewer, global, dir) = setup("rejected", 4, 4);
    let file = reviewer.docx();
    mine.import_with(Some(&global), &file);
    let id = pending_candidates(&mine.open.store).expect("doc")[0].id;
    reject_candidate(&mine.open.store, id).expect("bo");

    let summary = mine.import_with(Some(&global), &file);

    assert_eq!(summary.harvest_candidate_count, Some(0));
    assert!(pending_candidates(&mine.open.store).expect("doc").is_empty());
    assert_eq!(mine.rows("SELECT id, resolution FROM glossary_candidate").len(), 1);
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_pending_pair_gets_its_counts_updated_when_another_chapter_is_imported() {
    let (a_mine, a_theirs) = lines(2, 2);
    let (b_mine, b_theirs) = lines(3, 2);
    let b_shift = |rows: Pairs| -> Pairs { rows.into_iter().map(|(s, t)| (s.replace('王', "王，王"), t)).collect() };
    let b_mine = b_shift(b_mine);
    let b_theirs = b_shift(b_theirs);
    let mine = build("two-mine", &[a_mine, b_mine]);
    let first = build("two-first", &[a_theirs]);
    let second = build("two-second", &[b_theirs]);
    let (global, dir) = global_store("two");
    work_term(&mine, &global, "北凉王", BAC_LUONG);

    mine.import_with(Some(&global), &first.docx());
    let after_first: Vec<_> = pending_candidates(&mine.open.store).expect("doc").into_iter().map(|c| c.review_harvest).collect();
    mine.import_with(Some(&global), &second.docx());
    let after_second = pending_candidates(&mine.open.store).expect("doc");

    assert_eq!(after_first.len(), 1);
    assert_eq!(after_first[0].as_ref().map(|d| (d.changed_count, d.seen_count)), Some((2, 2)));
    assert_eq!(after_second.len(), 1, "cap dang cho khong them hang");
    assert_eq!(after_second[0].review_harvest.as_ref().map(|d| (d.changed_count, d.seen_count)), Some((4, 5)));
    finish(&[&mine, &first, &second]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn approving_when_the_work_tier_has_the_term_updates_that_entry_to_the_proposal() {
    let (mine, reviewer, global, dir) = setup("approve-work", 4, 4);
    mine.import_with(Some(&global), &reviewer.docx());
    let id = pending_candidates(&mine.open.store).expect("doc")[0].id;

    approve_candidate(&mine.open.store, id, None, Category::Person).expect("duyet");

    let tier = load_tier(&mine.open.store).expect("doc tang Work");
    assert_eq!(tier.len(), 1, "cap nhat mot muc co san, khong them muc");
    let entry = &tier["北凉王"];
    assert_eq!(entry.translation.as_deref(), Some(BAC_LUONG_SWAPPED));
    assert_eq!(entry.term_origin, TermOrigin::ReviewHarvest);
    assert!(entry.is_confirmed());
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn approving_when_only_the_global_tier_has_the_term_adds_a_work_entry_that_shadows_it() {
    let mine_rows = lines(4, 4).0;
    let their_rows = lines(4, 4).1;
    let mine = build("approve-global-mine", &[mine_rows]);
    let reviewer = build("approve-global-rev", &[their_rows]);
    let (global, dir) = global_store("approve-global");
    global_term(&mine, &global, "北凉王", BAC_LUONG);
    mine.import_with(Some(&global), &reviewer.docx());
    let id = pending_candidates(&mine.open.store).expect("doc")[0].id;

    approve_candidate(&mine.open.store, id, None, Category::Person).expect("duyet");

    assert_eq!(load_tier(&global).expect("doc Global")["北凉王"].translation.as_deref(), Some(BAC_LUONG));
    let work_entry = &load_tier(&mine.open.store).expect("doc Work")["北凉王"];
    assert_eq!((work_entry.translation.as_deref(), work_entry.term_origin), (Some(BAC_LUONG_SWAPPED), TermOrigin::ReviewHarvest));
    let injected = confirmed_terms_for_injection(&mine.open.scope, &global, Some(&mine.open.store), "北凉王，来了。", MatchLang::Zh)
        .expect("tiem")
        .injected;
    assert_eq!(injected.len(), 1);
    assert_eq!((injected[0].tier, injected[0].translation.as_str()), (GlossaryTier::Work, BAC_LUONG_SWAPPED));
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_stale_copy_and_a_one_sided_group_contribute_nothing() {
    let (mine_rows, their_rows) = lines(4, 4);
    let mut extra = their_rows.clone();
    extra.push(("别处，无关。".to_owned(), format!("{BAC_LUONG_SWAPPED} thừa.")));
    let second_chapter = pairs(&[("二章，一。", "Chương hai.")]);
    let mine = build("stale-mine", &[mine_rows, second_chapter.clone()]);
    let reviewer = build("stale-rev", &[extra]);
    let (global, dir) = global_store("stale");
    work_term(&mine, &global, "北凉王", BAC_LUONG);
    mine.import_with(Some(&global), &reviewer.docx());
    assert_eq!(mine.harvest(&global).len(), 1, "hang them cua reviewer khong ghep duoc nen khong lam lech so dem");

    let mut mine = mine;
    merge_chapter_into_previous(Some(&mut mine.open), mine.chapters[1]).expect("gop");

    assert!(mine.harvest(&global).is_empty(), "ban cua Chuong loi thoi bi bo qua, khong loi");
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}


#[test]
fn harvesting_writes_no_glossary_entry_segment_or_reviewer_row() {
    let (mine, reviewer, global, dir) = setup("pure", 4, 4);
    let before_entries = (mine.rows("SELECT * FROM glossary_entry"), load_tier(&global).expect("doc").len());

    mine.import_with(Some(&global), &reviewer.docx());

    assert_eq!((mine.rows("SELECT * FROM glossary_entry"), load_tier(&global).expect("doc").len()), before_entries);
    let frozen = (mine.rows("SELECT * FROM segment"), mine.rows("SELECT * FROM review_row"), mine.rows("SELECT * FROM alignment_member"));
    let _ = mine.harvest(&global);
    assert_eq!(
        (mine.rows("SELECT * FROM segment"), mine.rows("SELECT * FROM review_row"), mine.rows("SELECT * FROM alignment_member")),
        frozen
    );
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn an_approved_term_reaches_the_injection_door_with_the_proposed_translation() {
    let (mine, reviewer, global, dir) = setup("rag", 4, 4);
    mine.import_with(Some(&global), &reviewer.docx());
    let id = pending_candidates(&mine.open.store).expect("doc")[0].id;
    approve_candidate(&mine.open.store, id, None, Category::Person).expect("duyet");

    let injected = confirmed_terms_for_injection(&mine.open.scope, &global, Some(&mine.open.store), "北凉王，来了。", MatchLang::Zh)
        .expect("tiem")
        .injected;

    assert_eq!(injected.iter().map(|t| (t.source_term.as_str(), t.translation.as_str())).collect::<Vec<_>>(), vec![("北凉王", BAC_LUONG_SWAPPED)]);
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_failed_harvest_keeps_the_import_and_reports_its_own_error() {
    let (mine, reviewer, global, dir) = setup("failed", 4, 4);
    mine.open.store.write(|tx: &Transaction<'_>| tx.execute_batch("DROP TABLE glossary_candidate")).expect("pha bang");

    let summary = mine.import_with(Some(&global), &reviewer.docx());

    assert_eq!(summary.harvest_candidate_count, None);
    assert_eq!(summary.harvest_error.as_ref().map(auratranslate_lib::core::i18n::IpcError::code), Some("export.harvest_failed"));
    assert_eq!(summary.chapter_count, 1);
    assert_eq!(read_review_copy(&mine.open.store, mine.chapters[0]).expect("ban reviewer van doc duoc").rows.len(), 4);
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_missing_global_store_is_a_harvest_error_not_a_silent_zero() {
    let (mine, reviewer, global, dir) = setup("noglobal", 4, 4);

    let summary = mine.import_with(None, &reviewer.docx());

    assert_eq!(summary.harvest_candidate_count, None);
    assert_eq!(summary.harvest_error.as_ref().map(auratranslate_lib::core::i18n::IpcError::code), Some("export.harvest_failed"));
    drop(global);
    finish(&[&mine, &reviewer]);
    let _ = fs::remove_dir_all(dir);
}


fn candidate_count(store: &Store) -> i64 {
    store.read(|conn| conn.query_row("SELECT COUNT(*) FROM glossary_candidate", [], |r| r.get(0))).expect("dem")
}

fn insert(store: &Store, sql: &'static str) -> bool {
    store.write(move |tx: &Transaction<'_>| Ok(tx.execute_batch(sql).is_ok())).expect("ghi")
}

#[test]
fn the_check_and_the_two_partial_indexes_hold_for_each_origin() {
    let dir = temp_dir("schema");
    let store = Store::open(StoreSpec::project(dir.join("project.db"))).expect("mo project.db");
    let harvest = |source: &str, proposed: &str| -> String {
        format!(
            "INSERT INTO glossary_candidate (source_term, candidate_origin, created_at, replaced_translation, \
             proposed_translation, changed_count, seen_count) VALUES ('{source}', 'review_harvest', 't', 'x', '{proposed}', 2, 3)"
        )
    };
    let leak = |sql: String| -> &'static str { Box::leak(sql.into_boxed_str()) };

    assert!(insert(&store, leak(harvest("A", "y1"))));
    assert!(insert(&store, leak(harvest("A", "y2"))), "mot thuat ngu duoc de xuat nhieu cach thay");
    assert!(!insert(&store, leak(harvest("A", "y1"))), "cap (S, Y) la duy nhat");
    assert!(insert(&store, "INSERT INTO glossary_candidate (source_term, candidate_origin, created_at) VALUES ('A', 'import_scan', 't')"));
    assert!(
        !insert(&store, "INSERT INTO glossary_candidate (source_term, candidate_origin, created_at) VALUES ('A', 'import_scan', 't')"),
        "quet nhap van mot hang cho moi thuat ngu"
    );
    assert!(
        !insert(&store, "INSERT INTO glossary_candidate (source_term, candidate_origin, created_at) VALUES ('B', 'review_harvest', 't')"),
        "ung vien thu hoach thieu bon cot"
    );
    assert!(
        !insert(
            &store,
            "INSERT INTO glossary_candidate (source_term, candidate_origin, created_at, proposed_translation) VALUES ('C', 'import_scan', 't', 'y')"
        ),
        "ung vien quet nhap khong duoc mang cot thu hoach"
    );
    assert_eq!(candidate_count(&store), 3);
    drop(store);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_database_at_step_31_migrates_and_keeps_its_import_scan_candidates() {
    let dir = temp_dir("migrate");
    let db = dir.join("project.db");
    let old = Store::open(StoreSpec { migrations: &PROJECT_MIGRATIONS[..PROJECT_MIGRATIONS.len() - 1], ..StoreSpec::project(db.clone()) })
        .expect("mo o buoc 31");
    assert_eq!(old.schema_version(), 31);
    assert!(insert(&old, "INSERT INTO glossary_candidate (source_term, candidate_origin, created_at) VALUES ('A', 'import_scan', 't')"));
    drop(old);

    let migrated = Store::open(StoreSpec::project(db)).expect("mo lai");

    assert_eq!(migrated.schema_version(), 32);
    let row: (String, Option<String>) = migrated
        .read(|conn| conn.query_row("SELECT source_term, proposed_translation FROM glossary_candidate", [], |r| Ok((r.get(0)?, r.get(1)?))))
        .expect("doc");
    assert_eq!(row, ("A".to_owned(), None));
    drop(migrated);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_pending_pair_the_recount_no_longer_finds_is_deleted_and_the_count_matches_the_queue() {
    let (mine, reviewer, global, dir) = setup("recount", 4, 4);
    let file = reviewer.docx();
    mine.import_with(Some(&global), &file);
    assert_eq!(pending_candidates(&mine.open.store).expect("doc").len(), 1);

    let (_, kept) = lines(4, 0);
    let unchanged = build("recount-unchanged", &[kept]);
    let summary = mine.import_with(Some(&global), &unchanged.docx());

    let pending = pending_candidates(&mine.open.store).expect("doc");
    assert!(pending.is_empty(), "cap khong con duoc dem lai phai bien mat");
    assert_eq!(summary.harvest_candidate_count, Some(i64::try_from(pending.len()).unwrap_or(-1)));
    finish(&[&mine, &reviewer, &unchanged]);
    let _ = fs::remove_dir_all(dir);
}
