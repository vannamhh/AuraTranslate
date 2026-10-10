//! Khớp bản reviewer với segment (AD-52 mục 6-7): máy khớp lúc xác nhận nhập, người nối/bỏ qua/tách,
//! và các đường làm đổi segment hoặc Chương. Mọi tệp reviewer được dựng bằng hàm xuất thật.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::chapter::merge_chapter_into_previous;
use auratranslate_lib::commands::export::{
    PendingReviewerImportState, export_docx_two_column, export_text, reviewer_import_confirm, reviewer_import_preview,
};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::segment::{merge_segments, review_accept_change, split_segment};
use auratranslate_lib::core::export::{
    AlignmentError, AlignmentItem, ChapterAlignment, DecidedBy, ExportScope, ImageMode, MIN_PAIR_SIMILARITY, ReviewCopyError,
    TextFormat, join, read_alignment, read_review_copy, review_diff, skip, skip_change, unjoin,
};
use auratranslate_lib::core::matching::{DiffKind, DiffSpan, MatchLang, similarity_percent};
use auratranslate_lib::core::store::Transaction;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-align-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

type Segs = Vec<(&'static str, &'static str)>;

struct Work {
    open: OpenWork,
    chapters: Vec<i64>,
    root: PathBuf,
    out: PathBuf,
}

fn build(tag: &str, chapters: &[(Option<&'static str>, Segs)]) -> Work {
    let root = temp_dir(tag);
    let out = temp_dir(&format!("{tag}-out"));
    let open = create_work_from_text(&root, "Tac Pham", "zh", "", "Chuong mot.".to_owned()).expect("tao tac pham");
    open.store.write(|tx: &Transaction<'_>| tx.execute("DELETE FROM segment", [])).expect("xoa segment");
    let mut ids = Vec::new();
    for (i, (title, segs)) in chapters.iter().enumerate() {
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
        let title = title.map(str::to_owned);
        let segs = segs.clone();
        open.store
            .write(move |tx: &Transaction<'_>| {
                tx.execute("UPDATE chapter SET title = ?1 WHERE id = ?2", (title, id))?;
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

fn pending() -> PendingReviewerImportState {
    PendingReviewerImportState::new(None)
}

impl Work {
    fn docx(&self) -> PathBuf {
        let file = export_docx_two_column(Some(&self.open), &ExportScope::Work, ImageMode::File, None, &self.out).expect("xuat docx");
        PathBuf::from(file.path)
    }

    fn markdown(&self) -> PathBuf {
        let file = export_text(Some(&self.open), &ExportScope::Work, ImageMode::Link, TextFormat::Markdown, None, &self.out)
            .expect("xuat md");
        PathBuf::from(file.path)
    }

    fn import(&self, path: &Path) {
        let state = pending();
        reviewer_import_preview(Some(&self.open), &state, path).expect("xem truoc");
        reviewer_import_confirm(Some(&self.open), None, &state).expect("xac nhan");
    }

    fn alignment(&self, chapter: usize) -> ChapterAlignment {
        read_alignment(&self.open.store, self.chapters[chapter]).expect("doc alignment")
    }

    fn scalar(&self, sql: &'static str) -> i64 {
        self.open.store.read(move |conn| conn.query_row(sql, [], |r| r.get(0))).expect("dem")
    }

    fn dump(&self, tables: &'static [&'static str]) -> Vec<String> {
        self.open
            .store
            .read(move |conn| {
                let mut out = Vec::new();
                for table in tables {
                    let mut stmt = conn.prepare(&format!("SELECT * FROM {table} ORDER BY 1, 2"))?;
                    let columns = stmt.column_count();
                    let rows = stmt
                        .query_map([], |row| {
                            Ok((0..columns).map(|i| format!("{:?}", row.get_ref(i))).collect::<Vec<_>>().join("|"))
                        })?
                        .collect::<rusqlite::Result<Vec<_>>>()?;
                    out.push(format!("{table}: {}", rows.join(" ; ")));
                }
                Ok(out)
            })
            .expect("dump")
    }

    fn segment_ids(&self, chapter: usize) -> Vec<i64> {
        let id = self.chapters[chapter];
        self.open
            .store
            .read(move |conn| {
                conn.prepare("SELECT id FROM segment WHERE chapter_id = ?1 AND retired_at IS NULL ORDER BY ord")?
                    .query_map([id], |r| r.get(0))?
                    .collect::<rusqlite::Result<Vec<i64>>>()
            })
            .expect("segment")
    }

    fn finish(self) {
        let dir = self.open.dir.clone();
        drop(self.open);
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&self.root);
        let _ = fs::remove_dir_all(&self.out);
    }
}

const SEGMENT_TABLE: &[&str] = &["segment"];
const ALIGNMENT_TABLES: &[&str] = &["alignment_group", "alignment_member"];

fn three(targets: [&'static str; 3]) -> Vec<(Option<&'static str>, Segs)> {
    vec![(Some("Mo dau"), vec![("a1", targets[0]), ("a2", targets[1]), ("a3", targets[2])])]
}

fn mine_and_reviewer(tag: &str, reviewer_segs: Segs) -> (Work, Work) {
    let mine = build(&format!("{tag}-mine"), &three(["dich mot", "dich hai", "dich ba"]));
    let reviewer = build(&format!("{tag}-rev"), &[(Some("Mo dau"), reviewer_segs)]);
    (mine, reviewer)
}

fn pair_of(alignment: &ChapterAlignment, segment_id: i64) -> Option<(DecidedBy, Vec<i64>)> {
    alignment
        .groups
        .iter()
        .find(|g| g.segment_ids.contains(&segment_id))
        .map(|g| (g.decided_by, g.row_ids.clone()))
}

// ───────────────────────── measured threshold ─────────────────────────

#[test]
fn the_pair_threshold_sits_between_a_lightly_edited_sentence_and_a_neighbouring_one() {
    let original = "Toi di tau hoa ve phia Bac xa hang ngan dam den mot thanh pho moi";
    let edited = "Toi di tau hoa ve phia Bac xa hang ngan dam toi mot thanh pho moi";
    let neighbour = "Co ay ngoi ben cua so nhin tuyet roi xa hang ngan dam den ga cuoi";
    let right = similarity_percent(original, edited, MatchLang::En);
    let wrong = similarity_percent(neighbour, edited, MatchLang::En);
    assert!(right >= MIN_PAIR_SIMILARITY, "cap dung phai qua nguong: {right}");
    assert!(wrong < MIN_PAIR_SIMILARITY, "cap lech mot vi tri phai o duoi nguong: {wrong}");
    assert!(wrong >= 25, "cau lech phai du gan de ca nay khoa nguong chu khong chi khoa bang 0: {wrong}");
    assert_eq!(MIN_PAIR_SIMILARITY, 65, "doi nguong phai di cung mot so do moi");
    assert_eq!(
        similarity_percent("abcdefghijklmnopqrst", "abcdefghijklmUVWXYZQ", MatchLang::Zh),
        65,
        "ti le dung 13/20 phai ra 65, khong roi xuong 64"
    );
}

// ───────────────────────── .docx: the I/O matrix ─────────────────────────

#[test]
fn editing_only_the_right_cells_pairs_every_row_one_to_one_and_resolves_the_chapter() {
    let (mine, reviewer) = mine_and_reviewer("right", vec![("a1", "dich mot da sua"), ("a2", "dich hai"), ("a3", "dich ba them")]);
    let before = mine.dump(SEGMENT_TABLE);
    mine.import(&reviewer.docx());

    let alignment = mine.alignment(0);
    assert_eq!(alignment.groups.len(), 3);
    assert!(alignment.unmatched_row_ids.is_empty() && alignment.unmatched_segment_ids.is_empty());
    assert!(alignment.is_resolved);
    let segments = mine.segment_ids(0);
    for (segment, row) in segments.iter().zip(&alignment.rows) {
        assert_eq!(pair_of(&alignment, *segment), Some((DecidedBy::Machine, vec![row.id])));
    }
    assert_eq!(mine.dump(SEGMENT_TABLE), before, "segment phai nguyen ven tung byte");
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_dropped_row_and_a_foreign_row_are_listed_and_no_pair_is_shifted() {
    let (mine, reviewer) = mine_and_reviewer("foreign", vec![("a1", "dich mot"), ("a2", "dich hai"), ("zz", "chuyen khac han")]);
    mine.import(&reviewer.docx());

    let alignment = mine.alignment(0);
    let segments = mine.segment_ids(0);
    assert_eq!(alignment.unmatched_segment_ids, vec![segments[2]]);
    assert_eq!(alignment.unmatched_row_ids, vec![alignment.rows[2].id]);
    assert!(!alignment.is_resolved);
    assert_eq!(alignment.groups.len(), 2);
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_row_the_reviewer_deleted_leaves_only_its_segment_in_the_list() {
    let (mine, reviewer) = mine_and_reviewer("deleted", vec![("a1", "dich mot"), ("a3", "dich ba")]);
    mine.import(&reviewer.docx());

    let alignment = mine.alignment(0);
    assert_eq!(alignment.unmatched_segment_ids, vec![mine.segment_ids(0)[1]]);
    assert!(alignment.unmatched_row_ids.is_empty());
    mine.finish();
    reviewer.finish();
}

#[test]
fn an_unanchored_pair_below_the_threshold_is_listed_while_a_close_one_is_paired() {
    let mine = build("gap", &[(Some("Mo dau"), vec![("a1", "x"), ("a2", "Toi di tau hoa ve phia Bac xa hang ngan dam den mot thanh pho moi"), ("a3", "Toi di bo ve nha trong dem toi")])]);
    let reviewer = build(
        "gap-rev",
        &[(Some("Mo dau"), vec![("a1", "x"), ("b2", "Toi di tau hoa ve phia Bac xa hang ngan dam toi mot thanh pho moi"), ("b3", "Co ay ngoi ben cua so nhin tuyet roi xa hang ngan dam den ga cuoi")])],
    );
    mine.import(&reviewer.docx());

    let alignment = mine.alignment(0);
    let segments = mine.segment_ids(0);
    assert_eq!(pair_of(&alignment, segments[1]).map(|p| p.1.len()), Some(1), "cap giong nhau duoc ghep theo vi tri");
    assert_eq!(alignment.unmatched_segment_ids, vec![segments[2]]);
    assert_eq!(alignment.unmatched_row_ids, vec![alignment.rows[2].id]);
    mine.finish();
    reviewer.finish();
}

// ───────────────────────── .md ─────────────────────────

#[test]
fn two_markdown_paragraphs_merged_by_the_reviewer_are_listed_and_the_rest_still_pair() {
    let mine = build(
        "md",
        &[(Some("Mo dau"), vec![("a1", "doan mot"), ("a2", "doan hai"), ("a3", "doan ba"), ("a4", "doan bon")])],
    );
    let exported = fs::read_to_string(mine.markdown()).expect("doc md");
    assert!(exported.contains("doan hai\n\ndoan ba"), "{exported}");
    let edited = temp_dir("md-edit").join("reviewer.md");
    fs::write(&edited, exported.replace("doan hai\n\ndoan ba", "doan hai doan ba")).expect("ghi md");

    mine.import(&edited);

    let alignment = mine.alignment(0);
    let segments = mine.segment_ids(0);
    assert_eq!(alignment.rows.len(), 3);
    assert_eq!(alignment.groups.len(), 2);
    assert!(pair_of(&alignment, segments[0]).is_some() && pair_of(&alignment, segments[3]).is_some());
    assert_eq!(alignment.unmatched_segment_ids, vec![segments[1], segments[2]]);
    assert_eq!(alignment.unmatched_row_ids, vec![alignment.rows[1].id]);
    mine.finish();
}

type MdRow = (&'static str, &'static str, Option<&'static str>, bool);

fn markdown_work(tag: &str, rows: &[MdRow], image_after_ord: Option<i64>) -> Work {
    let mine = build(tag, &[(Some("Mo dau"), vec![])]);
    let chapter = mine.chapters[0];
    let rows = rows.to_vec();
    mine.open
        .store
        .write(move |tx: &Transaction<'_>| {
            for (n, (source, target, role, ends)) in rows.iter().enumerate() {
                tx.execute(
                    "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, target_text, \
                     is_target_paragraph_end, created_at, updated_at, status, is_omitted, role) \
                     VALUES (?1, ?2, ?3, 1, ?4, ?5, 't', 't', 'draft', 0, ?6)",
                    rusqlite::params![chapter, i64::try_from(n).unwrap_or(0) + 1, source, target, ends, role],
                )?;
            }
            if let Some(anchor) = image_after_ord {
                tx.execute(
                    "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, byte_len, \
                     content_type, created_at) VALUES (?1, 'a.jpg', 'https://x.test/a.jpg', ?2, 10, 'image/jpeg', 't')",
                    rusqlite::params![chapter, anchor],
                )?;
            }
            Ok(())
        })
        .expect("chen segment");
    mine
}

fn assert_unedited_markdown_resolves(mine: Work, expected_groups: usize) {
    mine.import(&mine.markdown());
    let alignment = mine.alignment(0);
    assert!(alignment.unmatched_row_ids.is_empty(), "{:?}", alignment.unmatched_row_ids);
    assert!(alignment.unmatched_segment_ids.is_empty(), "{:?}", alignment.unmatched_segment_ids);
    assert!(alignment.is_resolved);
    assert_eq!(alignment.groups.len(), expected_groups);
    assert!(alignment.groups.iter().all(|g| g.decided_by == DecidedBy::Machine));
    mine.finish();
}

#[test]
fn an_unedited_markdown_paragraph_made_of_several_segments_resolves_in_one_group() {
    let mine = markdown_work(
        "md-multi",
        &[("a1", "cau mot.", None, false), ("a2", "cau hai.", None, true), ("a3", "doan sau.", None, true)],
        None,
    );
    assert_unedited_markdown_resolves(mine, 2);
}

#[test]
fn an_unedited_markdown_segment_holding_a_newline_resolves_as_one_group_of_two_rows() {
    let mine = markdown_work("md-newline", &[("a1", "dong mot\ndong hai", None, true), ("a2", "doan sau.", None, true)], None);
    mine.import(&mine.markdown());
    let alignment = mine.alignment(0);
    assert_eq!(alignment.rows.len(), 3);
    assert!(alignment.is_resolved);
    assert!(alignment.groups.iter().any(|g| g.row_ids.len() == 2 && g.segment_ids.len() == 1));
    mine.finish();
}

#[test]
fn an_unedited_markdown_image_with_alt_and_caption_mid_paragraph_resolves_fully() {
    let mine = markdown_work(
        "md-image",
        &[
            ("a1", "doan mot.", None, false),
            ("alt", "mo ta anh", Some("alt"), false),
            ("cap", "chu thich anh", Some("caption"), false),
            ("a2", "doan hai.", None, true),
        ],
        Some(1),
    );
    let exported = fs::read_to_string(mine.markdown()).expect("doc md");
    assert!(exported.contains("![mo ta anh]") && exported.contains("*chu thich anh*"), "{exported}");
    assert_unedited_markdown_resolves(mine, 4);
}

#[test]
fn a_markdown_segment_shared_by_two_paragraphs_stays_unpaired_when_only_one_of_them_pairs() {
    let mine = markdown_work("md-partial", &[("a1", "dong mot\ndong hai", None, true), ("a2", "doan sau.", None, true)], None);
    let exported = fs::read_to_string(mine.markdown()).expect("doc md");
    let edited = temp_dir("md-partial-edit").join("reviewer.md");
    fs::write(&edited, exported.replace("dong hai", "mot y hoan toan khac")).expect("ghi md");
    mine.import(&edited);
    let alignment = mine.alignment(0);
    let first = mine.segment_ids(0)[0];
    assert!(alignment.unmatched_segment_ids.contains(&first));
    assert_eq!(alignment.unmatched_row_ids.len(), 2, "ca hai hang cua segment chung deu o lai trong danh sach");
    mine.finish();
}

// ───────────────────────── user decisions ─────────────────────────

fn foreign_import(tag: &str) -> Work {
    let (mine, reviewer) = mine_and_reviewer(tag, vec![("a1", "dich mot"), ("a2", "dich hai"), ("zz", "chuyen khac han")]);
    mine.import(&reviewer.docx());
    reviewer.finish();
    mine
}

#[test]
fn joining_makes_one_user_group_and_the_resolved_flag_follows_the_list() {
    let mine = foreign_import("join");
    let chapter = mine.chapters[0];
    let before = mine.alignment(0);
    let (segment, row) = (before.unmatched_segment_ids[0], before.unmatched_row_ids[0]);
    let segment_dump = mine.dump(SEGMENT_TABLE);

    join(&mine.open.store, chapter, &[segment], &[row]).expect("noi");

    let after = mine.alignment(0);
    assert_eq!(pair_of(&after, segment), Some((DecidedBy::User, vec![row])));
    assert!(after.is_resolved && after.unmatched_row_ids.is_empty());
    assert_eq!(mine.dump(SEGMENT_TABLE), segment_dump);
    mine.finish();
}

#[test]
fn a_join_naming_an_unknown_or_grouped_id_fails_with_a_typed_error_and_writes_nothing() {
    let mine = foreign_import("join-bad");
    let chapter = mine.chapters[0];
    let alignment = mine.alignment(0);
    let (free_segment, free_row) = (alignment.unmatched_segment_ids[0], alignment.unmatched_row_ids[0]);
    let grouped_segment = mine.segment_ids(0)[0];
    let grouped_row = alignment.rows[0].id;
    let before = mine.dump(ALIGNMENT_TABLES);

    let cases: Vec<(&str, Vec<i64>, Vec<i64>)> = vec![
        ("segment la", vec![9999], vec![free_row]),
        ("hang la", vec![free_segment], vec![9999]),
        ("segment da co nhom", vec![grouped_segment], vec![free_row]),
        ("hang da co nhom", vec![free_segment], vec![grouped_row]),
        ("thieu phia hang", vec![free_segment], vec![]),
        ("thieu phia segment", vec![], vec![free_row]),
        ("id lap", vec![free_segment, free_segment], vec![free_row]),
    ];
    for (label, segments, rows) in cases {
        let result = join(&mine.open.store, chapter, &segments, &rows);
        assert!(matches!(result, Err(AlignmentError::InvalidSelection)), "{label}");
        assert_eq!(mine.dump(ALIGNMENT_TABLES), before, "{label}: 0 ghi");
    }
    mine.finish();
}

#[test]
fn skipping_sets_one_item_aside_as_a_one_sided_group() {
    let mine = foreign_import("skip");
    let chapter = mine.chapters[0];
    let alignment = mine.alignment(0);
    let (segment, row) = (alignment.unmatched_segment_ids[0], alignment.unmatched_row_ids[0]);
    let segment_dump = mine.dump(SEGMENT_TABLE);

    skip(&mine.open.store, chapter, AlignmentItem::Segment(segment)).expect("bo qua segment");
    skip(&mine.open.store, chapter, AlignmentItem::Row(row)).expect("bo qua hang");

    let after = mine.alignment(0);
    assert_eq!(pair_of(&after, segment), Some((DecidedBy::User, vec![])));
    assert!(after.groups.iter().any(|g| g.row_ids == vec![row] && g.segment_ids.is_empty()));
    assert!(after.is_resolved);
    assert!(matches!(skip(&mine.open.store, chapter, AlignmentItem::Row(row)), Err(AlignmentError::InvalidSelection)));
    assert_eq!(mine.dump(SEGMENT_TABLE), segment_dump);
    mine.finish();
}

#[test]
fn unjoining_returns_every_member_to_the_list_and_a_foreign_group_id_is_refused() {
    let (mine, reviewer) = mine_and_reviewer("unjoin", vec![("a1", "dich mot"), ("a2", "dich hai"), ("a3", "dich ba")]);
    mine.import(&reviewer.docx());
    let chapter = mine.chapters[0];
    let alignment = mine.alignment(0);
    assert!(alignment.is_resolved);
    let group = alignment.groups[0].clone();
    let segment_dump = mine.dump(SEGMENT_TABLE);

    unjoin(&mine.open.store, chapter, group.id).expect("tach");

    let after = mine.alignment(0);
    assert_eq!(after.unmatched_segment_ids, group.segment_ids);
    assert_eq!(after.unmatched_row_ids, group.row_ids);
    assert!(!after.is_resolved);
    assert!(matches!(unjoin(&mine.open.store, chapter, group.id), Err(AlignmentError::InvalidSelection)));
    assert_eq!(mine.dump(SEGMENT_TABLE), segment_dump);
    mine.finish();
    reviewer.finish();
}

// ───────────────────────── the writer and the reader ─────────────────────────

#[test]
fn confirming_the_import_aligns_inside_the_same_write_before_anything_reads_it() {
    let (mine, reviewer) = mine_and_reviewer("confirm", vec![("a1", "dich mot"), ("a2", "dich hai"), ("a3", "dich ba")]);
    mine.import(&reviewer.docx());

    assert_eq!(mine.scalar("SELECT COUNT(*) FROM alignment_group"), 3);
    assert_eq!(mine.scalar("SELECT COUNT(*) FROM review_chapter WHERE aligned_at IS NULL"), 0);
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_copy_imported_before_alignment_existed_is_aligned_once_on_first_read_and_never_again() {
    let (mine, reviewer) = mine_and_reviewer("lazy", vec![("a1", "dich mot"), ("a2", "dich hai"), ("a3", "dich ba")]);
    mine.import(&reviewer.docx());
    mine.open
        .store
        .write(|tx: &Transaction<'_>| {
            tx.execute("DELETE FROM alignment_member", [])?;
            tx.execute("DELETE FROM alignment_group", [])?;
            tx.execute("UPDATE review_chapter SET aligned_at = NULL", [])
        })
        .expect("gia lap ban nhap cu");

    let first = mine.alignment(0);
    assert_eq!(first.groups.len(), 3, "khop may chay mot lan, khong tra danh sach rong thay cho chua khop");
    assert!(first.is_resolved);

    unjoin(&mine.open.store, mine.chapters[0], first.groups[0].id).expect("tach");
    assert_eq!(mine.alignment(0).groups.len(), 2, "lan doc sau khong khop lai");
    mine.finish();
    reviewer.finish();
}

#[test]
fn reading_without_a_copy_or_after_a_boundary_change_is_a_typed_error() {
    let mine = build("typed", &[(Some("Mo dau"), vec![("a1", "x"), ("a2", "y")]), (None, vec![("b1", "z")])]);
    let never = read_alignment(&mine.open.store, mine.chapters[0]);
    assert!(matches!(never, Err(AlignmentError::Copy(ReviewCopyError::NotImported))));
    mine.import(&mine.docx());

    let mut open = mine.open;
    let b = mine.chapters[1];
    merge_chapter_into_previous(Some(&mut open), b).expect("gop");
    let mine = Work { open, ..mine };

    let stale = read_alignment(&mine.open.store, mine.chapters[0]);
    assert!(matches!(stale, Err(AlignmentError::Copy(ReviewCopyError::Stale))));
    assert!(matches!(join(&mine.open.store, mine.chapters[0], &[1], &[1]), Err(AlignmentError::Copy(ReviewCopyError::Stale))));
    mine.finish();
}

#[test]
fn the_reader_exposes_review_row_ids_in_file_order() {
    let (mine, reviewer) = mine_and_reviewer("ids", vec![("a1", "dich mot"), ("a2", "dich hai"), ("a3", "dich ba")]);
    mine.import(&reviewer.docx());
    let copy = read_review_copy(&mine.open.store, mine.chapters[0]).expect("ban");
    let alignment = mine.alignment(0);
    assert_eq!(copy.row_ids, alignment.rows.iter().map(|r| r.id).collect::<Vec<_>>());
    mine.finish();
    reviewer.finish();
}

#[test]
fn importing_again_says_how_many_hand_made_groups_are_lost_and_starts_from_the_machine() {
    let (mine, reviewer) = mine_and_reviewer("again", vec![("a1", "dich mot"), ("a2", "dich hai"), ("zz", "chuyen khac han")]);
    let file = reviewer.docx();
    mine.import(&file);
    let alignment = mine.alignment(0);
    join(&mine.open.store, mine.chapters[0], &alignment.unmatched_segment_ids, &alignment.unmatched_row_ids).expect("noi");
    assert_eq!(mine.scalar("SELECT COUNT(*) FROM alignment_group WHERE decided_by = 'user'"), 1);

    let state = pending();
    let preview = reviewer_import_preview(Some(&mine.open), &state, &file).expect("xem truoc");
    assert_eq!(preview.chapters[0].replaces.as_ref().map(|r| r.user_group_count), Some(1));
    reviewer_import_confirm(Some(&mine.open), None, &state).expect("xac nhan");

    assert_eq!(mine.scalar("SELECT COUNT(*) FROM alignment_group WHERE decided_by = 'user'"), 0);
    assert!(!mine.alignment(0).is_resolved);
    assert_eq!(mine.scalar("SELECT COUNT(*) FROM alignment_group"), 2, "khong nhom cu nao con sot lai");
    mine.finish();
    reviewer.finish();
}

// ───────────────────────── paths that change segments or Chapters ─────────────────────────

fn live_members(work: &Work) -> i64 {
    work.scalar(
        "SELECT COUNT(*) FROM alignment_member m JOIN segment s ON s.id = m.segment_id WHERE s.retired_at IS NULL",
    )
}

fn member_count(work: &Work) -> i64 {
    work.scalar("SELECT COUNT(*) FROM alignment_member WHERE segment_id IS NOT NULL")
}

#[test]
fn merging_two_paired_segments_hands_the_group_to_the_new_one_and_frees_the_other_row() {
    let (mine, reviewer) = mine_and_reviewer("regroup-merge", vec![("a1", "dich mot"), ("a2", "dich hai"), ("a3", "dich ba")]);
    mine.import(&reviewer.docx());
    let segments = mine.segment_ids(0);
    let rows: Vec<i64> = mine.alignment(0).rows.iter().map(|r| r.id).collect();

    merge_segments(Some(&mine.open), segments[1]).expect("gop segment");

    assert_eq!(live_members(&mine), member_count(&mine), "khong thanh vien nao tro vao segment ve huu");
    let after = mine.alignment(0);
    let merged = mine.segment_ids(0)[0];
    assert_eq!(pair_of(&after, merged), Some((DecidedBy::Machine, vec![rows[0]])));
    assert_eq!(after.unmatched_row_ids, vec![rows[1]]);
    assert!(!after.is_resolved);
    mine.finish();
    reviewer.finish();
}

#[test]
fn splitting_a_paired_segment_puts_every_new_piece_in_the_same_group() {
    let (mine, reviewer) = mine_and_reviewer("regroup-split", vec![("a1", "dich mot"), ("a2", "dich hai"), ("a3", "dich ba")]);
    mine.import(&reviewer.docx());
    let segments = mine.segment_ids(0);

    split_segment(Some(&mine.open), segments[1], vec![1]).expect("tach segment");

    assert_eq!(live_members(&mine), member_count(&mine));
    let after = mine.alignment(0);
    let pieces = &mine.segment_ids(0)[1..3];
    let group_a = pair_of(&after, pieces[0]);
    assert!(group_a.is_some() && group_a == pair_of(&after, pieces[1]));
    assert!(after.is_resolved);
    mine.finish();
    reviewer.finish();
}

#[test]
fn merging_chapter_b_into_a_leaves_no_alignment_row_of_b() {
    let mine = build("merge-chapter", &[(Some("Mo dau"), vec![("a1", "x"), ("a2", "y")]), (None, vec![("b1", "z"), ("b2", "w")])]);
    mine.import(&mine.docx());
    let b = mine.chapters[1];
    let b_copy = read_review_copy(&mine.open.store, b).expect("ban cua B").id;
    assert!(mine.scalar("SELECT COUNT(*) FROM alignment_group") >= 4);

    let mut open = mine.open;
    merge_chapter_into_previous(Some(&mut open), b).expect("gop");
    let mine = Work { open, ..mine };

    let sql: &'static str = Box::leak(format!("SELECT COUNT(*) FROM alignment_group WHERE review_chapter_id = {b_copy}").into_boxed_str());
    assert_eq!(mine.scalar(sql), 0);
    assert_eq!(mine.scalar("SELECT COUNT(*) FROM alignment_member WHERE group_id NOT IN (SELECT id FROM alignment_group)"), 0);
    assert_eq!(mine.scalar("SELECT COUNT(*) FROM alignment_group"), 2, "nhom cua A con nguyen");
    mine.finish();
}

// ───────────────────────── review_diff ─────────────────────────

fn side(spans: &[DiffSpan], keep: DiffKind) -> String {
    spans.iter().filter(|s| s.kind == DiffKind::Equal || s.kind == keep).map(|s| s.text.as_str()).collect()
}

fn only_equal(spans: &[DiffSpan]) -> bool {
    spans.iter().all(|s| s.kind == DiffKind::Equal)
}

impl Work {
    fn diff(&self, chapter: usize) -> Vec<auratranslate_lib::core::export::GroupDiff> {
        review_diff(&self.open.store, self.chapters[chapter]).expect("review_diff")
    }
}

#[test]
fn a_changed_word_is_a_delete_on_my_side_and_an_insert_on_the_reviewers_and_nothing_else_is_marked() {
    let (mine, reviewer) = mine_and_reviewer("diff-word", vec![("a1", "dich mot"), ("a2", "dich bon"), ("a3", "dich ba")]);
    mine.import(&reviewer.docx());

    let diffs = mine.diff(0);
    assert_eq!(diffs.len(), 3);
    assert!(only_equal(&diffs[0].spans) && only_equal(&diffs[2].spans));
    let changed = &diffs[1].spans;
    assert!(changed.iter().any(|s| s.kind == DiffKind::Delete && s.text.contains("hai")), "{changed:?}");
    assert!(changed.iter().any(|s| s.kind == DiffKind::Insert && s.text.contains("bon")), "{changed:?}");
    assert_eq!(side(changed, DiffKind::Delete), "dich hai");
    assert_eq!(side(changed, DiffKind::Insert), "dich bon");
    mine.finish();
    reviewer.finish();
}

#[test]
fn identical_pairs_carry_no_delete_or_insert_and_the_diff_writes_nothing() {
    let (mine, reviewer) = mine_and_reviewer("diff-same", vec![("a1", "dich mot"), ("a2", "dich hai"), ("a3", "dich ba")]);
    mine.import(&reviewer.docx());
    let before = (mine.dump(SEGMENT_TABLE), mine.dump(ALIGNMENT_TABLES));

    let diffs = mine.diff(0);
    assert_eq!(diffs.len(), 3);
    assert!(diffs.iter().all(|d| only_equal(&d.spans)));
    assert_eq!((mine.dump(SEGMENT_TABLE), mine.dump(ALIGNMENT_TABLES)), before);
    mine.finish();
    reviewer.finish();
}

#[test]
fn canonically_equal_texts_in_different_unicode_forms_are_not_marked() {
    let mine = build("diff-nfc-mine", &[(Some("Mo dau"), vec![("a1", "d\u{1ecb}ch m\u{1ed9}t")])]);
    let reviewer = build("diff-nfc-rev", &[(Some("Mo dau"), vec![("a1", "di\u{323}ch mo\u{323}\u{302}t")])]);
    mine.import(&reviewer.docx());

    let diffs = mine.diff(0);
    assert_eq!(diffs.len(), 1);
    assert!(only_equal(&diffs[0].spans), "{:?}", diffs[0].spans);
    mine.finish();
    reviewer.finish();
}

#[test]
fn groups_come_back_in_translation_order_and_a_row_only_group_comes_last() {
    let (mine, reviewer) = mine_and_reviewer(
        "diff-order",
        vec![("a1", "dich mot"), ("a2", "dich hai"), ("a3", "dich ba"), ("zz", "dong them cua reviewer")],
    );
    mine.import(&reviewer.docx());
    let alignment = mine.alignment(0);
    let extra = alignment.rows[3].id;
    skip(&mine.open.store, mine.chapters[0], AlignmentItem::Row(extra)).expect("bo qua");

    let diffs = mine.diff(0);
    let segments = mine.segment_ids(0);
    let order: Vec<Vec<i64>> = diffs.iter().map(|d| d.segment_ids.clone()).collect();
    assert_eq!(order, vec![vec![segments[0]], vec![segments[1]], vec![segments[2]], vec![]]);
    assert_eq!(diffs[3].row_ids, vec![extra]);
    assert_eq!(diffs[3].decided_by, DecidedBy::User);
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_markdown_paragraph_of_several_segments_is_diffed_as_the_joined_text_and_the_sides_rebuild() {
    let mine = markdown_work(
        "diff-multi",
        &[("a1", "cau mot.", None, false), ("a2", "cau hai.", None, true), ("a3", "doan sau.", None, true)],
        None,
    );
    let exported = fs::read_to_string(mine.markdown()).expect("doc md");
    assert!(exported.contains("cau mot. cau hai."), "{exported}");
    let edited = temp_dir("diff-multi-edit").join("reviewer.md");
    fs::write(&edited, exported.replace("cau mot. cau hai.", "cau mot. cau ba.")).expect("ghi md");
    mine.import(&edited);

    let diffs = mine.diff(0);
    assert_eq!(diffs.len(), 2);
    let first = &diffs[0];
    assert_eq!(first.segment_ids.len(), 2);
    assert_eq!(side(&first.spans, DiffKind::Delete), "cau mot. cau hai.");
    assert_eq!(side(&first.spans, DiffKind::Insert), "cau mot. cau ba.");
    assert!(first.spans.iter().any(|s| s.kind != DiffKind::Equal));
    assert!(only_equal(&diffs[1].spans));
    mine.finish();
}

#[test]
fn review_diff_names_a_chapter_without_a_copy_instead_of_returning_an_empty_list() {
    let mine = build("diff-none", &three_chapter());
    assert!(matches!(
        review_diff(&mine.open.store, mine.chapters[0]),
        Err(AlignmentError::Copy(ReviewCopyError::NotImported))
    ));
    mine.finish();
}

fn three_chapter() -> Vec<(Option<&'static str>, Segs)> {
    three(["dich mot", "dich hai", "dich ba"])
}

const DECISION_TABLE: &[&str] = &["review_decision"];

fn one_changed(tag: &str) -> (Work, Work) {
    let (mine, reviewer) = mine_and_reviewer(tag, vec![("a1", "dich mot"), ("a2", "dich bon"), ("a3", "dich ba")]);
    mine.import(&reviewer.docx());
    (mine, reviewer)
}

fn changed_group(work: &Work) -> (i64, i64, i64) {
    let diffs = work.diff(0);
    let group = &diffs[1];
    (group.group_id, group.segment_ids[0], group.row_ids[0])
}

fn segment_row(work: &Work, id: i64) -> (String, String, String, String, String) {
    work.open
        .store
        .read(move |conn| {
            conn.query_row(
                "SELECT target_text, status, translation_origin, baseline_target_text, baseline_translation_origin \
                 FROM segment WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
        })
        .expect("doc segment")
}

fn give_copy(work: &Work, id: i64, text: &'static str) {
    work.open
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("INSERT INTO segment_version (segment_id, target_text, created_at) VALUES (?1, ?2, 't')", (id, text))
        })
        .expect("chen ban sao");
}

fn code_of<T: std::fmt::Debug>(result: Result<T, auratranslate_lib::core::i18n::IpcError>) -> String {
    result.expect_err("phai loi").code().to_owned()
}

#[test]
fn accepting_a_one_to_one_change_writes_the_reviewers_text_as_other_draft_and_adds_no_version() {
    let (mine, reviewer) = one_changed("accept");
    let (group, segment, row) = changed_group(&mine);
    give_copy(&mine, segment, "dich hai");
    let versions = mine.scalar("SELECT COUNT(*) FROM segment_version");

    let outcome = review_accept_change(Some(&mine.open), mine.chapters[0], group, "dich hai", false).expect("chap nhan");

    assert!(!outcome.needs_confirmation);
    assert_eq!((outcome.target_text.as_str(), outcome.translation_origin.as_str(), outcome.status.as_str()), ("dich bon", "other", "draft"));
    assert_eq!(
        segment_row(&mine, segment),
        ("dich bon".into(), "draft".into(), "other".into(), "dich bon".into(), "other".into())
    );
    assert_eq!(mine.scalar("SELECT COUNT(*) FROM segment_version"), versions, "khong tao SegmentVersion");
    let accepted: Vec<i64> = mine
        .open
        .store
        .read(|conn| {
            conn.prepare("SELECT review_row_id FROM review_decision WHERE decision = 'accepted'")?
                .query_map([], |r| r.get(0))?
                .collect::<rusqlite::Result<Vec<i64>>>()
        })
        .expect("doc quyet dinh");
    assert_eq!(accepted, vec![row]);
    let after = mine.diff(0);
    assert_eq!(after[1].group_id, group);
    assert_eq!(after[1].decision.map(|d| d.as_str()), Some("accepted"));
    assert!(only_equal(&after[1].spans));
    mine.finish();
    reviewer.finish();
}

#[test]
fn accepting_over_an_unsigned_draft_asks_first_and_writes_nothing_until_forced() {
    let (mine, reviewer) = one_changed("accept-ask");
    let (group, segment, _) = changed_group(&mine);
    let before = mine.dump(DECISION_TABLE);

    let asked = review_accept_change(Some(&mine.open), mine.chapters[0], group, "dich hai", false).expect("hoi lai");

    assert!(asked.needs_confirmation);
    assert_eq!(asked.target_text, "dich hai");
    assert_eq!(segment_row(&mine, segment).0, "dich hai");
    assert_eq!(segment_row(&mine, segment).2, "", "xuat xu khong doi");
    assert_eq!(mine.dump(DECISION_TABLE), before, "nhanh hoi lai khong ghi quyet dinh");

    let forced = review_accept_change(Some(&mine.open), mine.chapters[0], group, "dich hai", true).expect("ghi de");
    assert!(!forced.needs_confirmation);
    assert_eq!(segment_row(&mine, segment).0, "dich bon");
    mine.finish();
    reviewer.finish();
}

#[test]
fn accepting_an_empty_segment_needs_no_confirmation() {
    let (mine, reviewer) = one_changed("accept-empty");
    let (group, segment, _) = changed_group(&mine);
    mine.open
        .store
        .write(move |tx: &Transaction<'_>| tx.execute("UPDATE segment SET target_text = '' WHERE id = ?1", [segment]))
        .expect("xoa");
    let outcome = review_accept_change(Some(&mine.open), mine.chapters[0], group, "", false).expect("chap nhan");
    assert!(!outcome.needs_confirmation);
    assert_eq!(segment_row(&mine, segment).0, "dich bon");
    mine.finish();
    reviewer.finish();
}

#[test]
fn skipping_keeps_the_segment_and_the_decision_survives_a_fresh_read() {
    let (mine, reviewer) = one_changed("skip");
    let (group, segment, _) = changed_group(&mine);

    skip_change(&mine.open.store, mine.chapters[0], group).expect("bo qua");

    assert_eq!(segment_row(&mine, segment).0, "dich hai");
    let again = mine.diff(0);
    assert_eq!(again[1].decision.map(|d| d.as_str()), Some("skipped"));
    assert!(again[0].decision.is_none() && again[2].decision.is_none());
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_group_that_is_not_one_to_one_cannot_be_accepted_and_one_without_a_row_cannot_be_skipped() {
    let (mine, reviewer) = mine_and_reviewer("not-one", vec![("a1", "dich mot"), ("zz", "chuyen khac han"), ("a3", "dich ba")]);
    mine.import(&reviewer.docx());
    let alignment = mine.alignment(0);
    let lone = alignment.unmatched_segment_ids[0];
    let one_sided = skip(&mine.open.store, mine.chapters[0], AlignmentItem::Segment(lone)).expect("bo qua mot phia");
    let before = mine.scalar("SELECT COUNT(*) FROM segment WHERE status = 'confirmed'");

    assert_eq!(
        code_of(review_accept_change(Some(&mine.open), mine.chapters[0], one_sided, "dich hai", false)),
        "review.change_not_acceptable"
    );
    assert!(matches!(skip_change(&mine.open.store, mine.chapters[0], one_sided), Err(AlignmentError::InvalidSelection)));
    assert_eq!(mine.scalar("SELECT COUNT(*) FROM review_decision"), 0);
    assert_eq!(mine.scalar("SELECT COUNT(*) FROM segment WHERE status = 'confirmed'"), before);
    mine.finish();
    reviewer.finish();
}

#[test]
fn an_unchanged_pair_is_not_acceptable() {
    let (mine, reviewer) = one_changed("unchanged");
    let same = mine.diff(0)[0].group_id;
    assert_eq!(
        code_of(review_accept_change(Some(&mine.open), mine.chapters[0], same, "dich mot", false)),
        "review.change_not_acceptable"
    );
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_segment_edited_after_the_diff_is_refused_with_nothing_written() {
    let (mine, reviewer) = one_changed("edited");
    let (group, segment, _) = changed_group(&mine);
    give_copy(&mine, segment, "dich hai");
    let before = mine.dump(DECISION_TABLE);

    assert_eq!(
        code_of(review_accept_change(Some(&mine.open), mine.chapters[0], group, "dich khac", false)),
        "review.change_text_changed"
    );
    mine.open
        .store
        .write(move |tx: &Transaction<'_>| tx.execute("UPDATE segment SET target_text = 'da sua tay' WHERE id = ?1", [segment]))
        .expect("sua tay");
    assert_eq!(
        code_of(review_accept_change(Some(&mine.open), mine.chapters[0], group, "dich hai", false)),
        "review.change_text_changed",
        "diff cu khong con khop van ban hien tai"
    );
    assert_eq!(segment_row(&mine, segment).0, "da sua tay");
    assert_eq!(mine.dump(DECISION_TABLE), before);
    mine.finish();
    reviewer.finish();
}

#[test]
fn accepting_without_a_live_copy_is_the_typed_alignment_error() {
    let mine = build("accept-none", &three_chapter());
    assert_eq!(
        code_of(review_accept_change(Some(&mine.open), mine.chapters[0], 1, "dich mot", false)),
        "export.alignment_not_imported"
    );
    mine.finish();
}

#[test]
fn accepting_or_skipping_on_a_stale_copy_is_the_typed_error_and_writes_nothing() {
    let (mut mine, reviewer) = one_changed("stale-accept");
    let chapter = mine.chapters[0];
    let (group, segment, _) = changed_group(&mine);
    give_copy(&mine, segment, "dich hai");
    let split_at = mine.segment_ids(0)[1];
    auratranslate_lib::commands::chapter::split_chapter_at_segment(Some(&mut mine.open), split_at).expect("tach Chuong");
    let before = mine.dump(DECISION_TABLE);

    assert_eq!(code_of(review_accept_change(Some(&mine.open), chapter, group, "dich hai", false)), "export.alignment_stale");
    assert!(matches!(skip_change(&mine.open.store, chapter, group), Err(AlignmentError::Copy(ReviewCopyError::Stale))));
    assert_eq!(segment_row(&mine, segment).0, "dich hai");
    assert_eq!(mine.dump(DECISION_TABLE), before);
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_decision_dies_with_any_change_of_its_group() {
    let (mine, reviewer) = one_changed("decision-dies");
    let chapter = mine.chapters[0];
    let (group, _, _) = changed_group(&mine);
    skip_change(&mine.open.store, chapter, group).expect("bo qua");
    assert_eq!(mine.scalar("SELECT COUNT(*) FROM review_decision"), 1);
    unjoin(&mine.open.store, chapter, group).expect("tach");
    assert_eq!(mine.scalar("SELECT COUNT(*) FROM review_decision"), 0, "tach nhom xoa quyet dinh");

    let rows = mine.alignment(0).unmatched_row_ids;
    let segments = mine.alignment(0).unmatched_segment_ids;
    let joined = join(&mine.open.store, chapter, &segments, &rows).expect("noi lai");
    skip_change(&mine.open.store, chapter, joined).expect("bo qua nhom noi");
    assert_eq!(mine.scalar("SELECT COUNT(*) FROM review_decision"), 1);
    let middle = mine.segment_ids(0)[1];
    merge_segments(Some(&mine.open), middle).expect("gop segment");
    assert_eq!(mine.scalar("SELECT COUNT(*) FROM review_decision"), 0, "doi thanh vien segment xoa quyet dinh");
    mine.finish();
    reviewer.finish();
}

#[test]
fn merging_the_chapter_removes_its_decisions_with_the_reviewer_copy() {
    let two = vec![
        (Some("Mo dau"), vec![("a1", "dich mot"), ("a2", "dich hai"), ("a3", "dich ba")]),
        (Some("Hai"), vec![("b1", "b mot")]),
    ];
    let mut work = build("decision-merge2", &two);
    let rev = build("decision-merge2-rev", &[(Some("Mo dau"), vec![("a1", "dich mot"), ("a2", "dich bon"), ("a3", "dich ba")]), (Some("Hai"), vec![("b1", "b ba")])]);
    work.import(&rev.docx());
    let second = work.diff(1)[0].group_id;
    skip_change(&work.open.store, work.chapters[1], second).expect("bo qua");
    assert_eq!(work.scalar("SELECT COUNT(*) FROM review_decision"), 1);

    merge_chapter_into_previous(Some(&mut work.open), work.chapters[1]).expect("gop Chuong");

    assert_eq!(work.scalar("SELECT COUNT(*) FROM review_decision"), 0);
    work.finish();
    rev.finish();
}
