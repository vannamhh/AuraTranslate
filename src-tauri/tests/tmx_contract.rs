//! TMX export and import of one TM tier (FR64, NFR9, AD-48): atomic export, two-phase import,
//! one transaction, identical pairs skipped, `x-aura-*` props round-trip.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::tm::{
    PendingTmxImport, PendingTmxImportState, clear_pending_tmx_import_for_work, tm_cancel_import, tm_confirm_global_import, tm_confirm_import,
    tm_export_tier, tm_export_tier_after_dialog, tm_open_import_preview,
};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};
use auratranslate_lib::core::tm::tmx::{MAX_TMX_BYTES, PlannedPair, TmxError, decode_tmx_bytes, parse_tmx};
use auratranslate_lib::core::tm::tmx_io::{read_tmx_file, write_tmx_file};
use auratranslate_lib::core::tm::PairOrigin;
use quick_xml::Reader;
use quick_xml::events::Event;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-tmx-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

struct Fixture {
    dir: PathBuf,
    global: Store,
    open: OpenWork,
    pending: PendingTmxImportState,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(Path::new(&self.dir));
    }
}

fn fixture(tag: &str) -> Fixture {
    fixture_with_lang(tag, "zh")
}

fn fixture_with_lang(tag: &str, lang: &str) -> Fixture {
    let dir = temp_dir(tag);
    let global = Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db");
    let open = create_work_from_text(&dir, tag, lang, "", "一。".to_owned()).expect("tao tac pham");
    Fixture { dir, global, open, pending: PendingTmxImportState::new(None) }
}

type Row = (String, String, String, String);

fn rows(store: &Store) -> Vec<Row> {
    store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT source_text, target_text, translation_origin, created_at FROM tm_unit ORDER BY id",
            )?;
            let rows = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .expect("doc tm_unit")
}

fn seed(store: &Store, data: &[(&str, &str, &str, &str)]) {
    let data: Vec<Row> =
        data.iter().map(|(a, b, c, d)| ((*a).into(), (*b).into(), (*c).into(), (*d).into())).collect();
    store
        .write(move |tx: &Transaction<'_>| {
            for (s, t, o, d) in &data {
                tx.execute(
                    "INSERT INTO tm_unit (source_text, target_text, translation_origin, created_at) VALUES (?1, ?2, ?3, ?4)",
                    (s, t, o, d),
                )?;
            }
            Ok(())
        })
        .expect("gieo");
}

fn export_to(f: &Fixture, tier: &str, name: &str) -> (PathBuf, String) {
    let path = f.dir.join(name);
    tm_export_tier(Some(&f.global), Some(&f.open), tier, &path).expect("xuat");
    let text = fs::read_to_string(&path).expect("doc tep xuat");
    (path, text)
}

fn import(f: &Fixture, tier: &str, text: &str, name: &str) -> Result<auratranslate_lib::commands::tm::TmxImportPreviewWire, auratranslate_lib::core::i18n::IpcError> {
    let path = f.dir.join(name);
    fs::write(&path, text).expect("ghi tep nhap");
    tm_open_import_preview(Some(&f.global), Some(&f.open), &f.pending, tier, &path)
}

fn confirm(f: &Fixture) -> Result<auratranslate_lib::commands::tm::TmxImportSummaryWire, auratranslate_lib::core::i18n::IpcError> {
    tm_confirm_import(Some(&f.global), Some(&f.open), &f.pending)
}

const D1: &str = "2026-03-01T10:20:30.123Z";
const D2: &str = "2026-04-02T01:02:03.456Z";

#[derive(Debug, Default)]
struct Walked {
    header: BTreeMap<String, String>,
    tus: Vec<WalkedTu>,
}

#[derive(Debug, Default)]
struct WalkedTu {
    creationdate: Option<String>,
    props: BTreeMap<String, String>,
    tuvs: Vec<(String, String)>,
}

/// An independent reader: a raw quick-xml event walk that shares nothing with `tmx.rs`.
fn walk(text: &str) -> Walked {
    let mut reader = Reader::from_str(text);
    let mut out = Walked::default();
    let mut tu: Option<WalkedTu> = None;
    let mut prop: Option<String> = None;
    let mut lang: Option<String> = None;
    let mut in_seg = false;
    let mut buf = String::new();
    loop {
        match reader.read_event().expect("tep xuat phai la XML hop le") {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let attr = |k: &str| {
                    e.attributes()
                        .flatten()
                        .find(|a| a.key.as_ref() == k.as_bytes())
                        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
                };
                match name.as_str() {
                    "header" => {
                        for a in e.attributes().flatten() {
                            out.header.insert(
                                String::from_utf8_lossy(a.key.as_ref()).into_owned(),
                                String::from_utf8_lossy(&a.value).into_owned(),
                            );
                        }
                    }
                    "tu" => tu = Some(WalkedTu { creationdate: attr("creationdate"), ..WalkedTu::default() }),
                    "prop" => {
                        prop = attr("type");
                        buf.clear();
                    }
                    "tuv" => lang = attr("xml:lang"),
                    "seg" => {
                        in_seg = true;
                        buf.clear();
                    }
                    _ => {}
                }
            }
            Event::Text(t) => buf.push_str(&t.decode().expect("van ban")),
            Event::GeneralRef(r) => {
                let name = r.decode().expect("tham chieu").into_owned();
                buf.push(match name.as_str() {
                    "amp" => '&',
                    "lt" => '<',
                    "gt" => '>',
                    "quot" => '"',
                    "#13" => '\r',
                    other => panic!("thuc the la {other}"),
                });
            }
            Event::End(e) => match e.name().as_ref() {
                b"prop" => {
                    if let (Some(t), Some(k)) = (tu.as_mut(), prop.take()) {
                        t.props.insert(k, buf.clone());
                    }
                }
                b"seg" if in_seg => {
                    in_seg = false;
                    if let (Some(t), Some(l)) = (tu.as_mut(), lang.take()) {
                        t.tuvs.push((l, buf.clone()));
                    }
                }
                b"tu" => out.tus.extend(tu.take()),
                _ => {}
            },
            _ => {}
        }
    }
    out
}

fn tmx_with(body: &str) -> String {
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<tmx version=\"1.4\"><header srclang=\"zh\"/><body>{body}</body></tmx>")
}

fn tu(zh: &str, vi: &str) -> String {
    format!("<tu><tuv xml:lang=\"zh\"><seg>{zh}</seg></tuv><tuv xml:lang=\"vi\"><seg>{vi}</seg></tuv></tu>")
}

#[test]
fn exporting_the_work_tier_writes_one_tu_per_distinct_pair_with_its_first_copy_and_required_header() {
    let f = fixture("export-work");
    seed(
        &f.open.store,
        &[("S", "A", "other", D2), ("S", "B", "other", D1), ("S", "A", "self", D1)],
    );

    let (_, text) = export_to(&f, "work", "work.tmx");
    let walked = walk(&text);

    for key in ["creationtool", "creationtoolversion", "segtype", "o-tmf", "adminlang", "srclang", "datatype"] {
        assert!(walked.header.contains_key(key), "header thieu {key}");
    }
    assert_eq!(walked.header["srclang"], "zh");
    assert_eq!(walked.header["creationtool"], "AuraTranslate");
    assert_eq!(walked.header["segtype"], "sentence");
    assert_eq!(walked.header["datatype"], "plaintext");
    assert_eq!(walked.tus.len(), 2, "mot tu cho moi cap (nguon, dich) khac nhau");
    for tu in &walked.tus {
        assert_eq!(tu.tuvs.len(), 2, "moi tu co du hai tuv");
        assert_eq!(tu.tuvs[0].0, "zh");
        assert_eq!(tu.tuvs[1].0, "vi");
    }
    let by_target: BTreeMap<&str, &WalkedTu> =
        walked.tus.iter().map(|t| (t.tuvs[1].1.as_str(), t)).collect();
    assert_eq!(by_target["A"].props["x-aura-origin"], "self", "ban dau theo thu tu AD-18: cua toi truoc");
    assert_eq!(by_target["A"].props["x-aura-created-at"], D1);
    assert_eq!(by_target["B"].props["x-aura-origin"], "other");
}

#[test]
fn exporting_the_global_tier_labels_each_pair_by_its_source_script_under_all_langs() {
    let f = fixture("export-global");
    seed(&f.global, &[("你好", "Xin chao", "self", D1), ("Hello", "Xin chao", "other", D1)]);

    let (_, text) = export_to(&f, "global", "global.tmx");
    let walked = walk(&text);

    assert_eq!(walked.header["srclang"], "*all*");
    let langs: BTreeMap<&str, &str> =
        walked.tus.iter().map(|t| (t.tuvs[0].1.as_str(), t.tuvs[0].0.as_str())).collect();
    assert_eq!(langs["你好"], "zh");
    assert_eq!(langs["Hello"], "en");
}

#[test]
fn export_dates_use_the_tmx_basic_format_beside_the_stored_value() {
    let f = fixture("export-date");
    seed(&f.open.store, &[("S", "A", "self", D1)]);

    let (_, text) = export_to(&f, "work", "d.tmx");

    assert_eq!(walk(&text).tus[0].creationdate.as_deref(), Some("20260301T102030Z"));
}

#[test]
fn a_round_trip_into_an_empty_tier_keeps_pairs_origins_and_dates() {
    let f = fixture("round-trip");
    seed(
        &f.open.store,
        &[("S", "A", "self", D1), ("S", "B", "bilingual_import", D2), ("T", "C", "other", D1), ("S", "A", "other", D2)],
    );
    let (_, text) = export_to(&f, "work", "rt.tmx");
    f.global.write(|_| Ok(())).expect("global san sang");

    let preview = import(&f, "global", &text, "rt-in.tmx").expect("xem truoc");
    let summary = confirm(&f).expect("xac nhan");

    assert_eq!(preview.new_count, 3);
    assert_eq!(summary.inserted, 3);
    let got = rows(&f.global);
    let pairs: Vec<(&str, &str)> = got.iter().map(|r| (r.0.as_str(), r.1.as_str())).collect();
    assert_eq!(pairs, vec![("S", "A"), ("S", "B"), ("T", "C")]);
    let origins: Vec<&str> = got.iter().map(|r| r.2.as_str()).collect();
    assert_eq!(origins, vec!["self", "bilingual_import", "other"], "moi cap mang xuat xu da xuat");
    let dates: Vec<&str> = got.iter().map(|r| r.3.as_str()).collect();
    assert_eq!(dates, vec![D1, D2, D1], "moi cap mang created_at da xuat");
}

#[test]
fn importing_the_same_file_twice_adds_nothing_the_second_time() {
    let f = fixture("reimport");
    let text = tmx_with(&(tu("一", "A") + &tu("二", "B")));

    import(&f, "work", &text, "a.tmx").expect("lan mot");
    assert_eq!(confirm(&f).expect("ghi").inserted, 2);
    let before = rows(&f.open.store);

    let second = import(&f, "work", &text, "a.tmx").expect("lan hai");
    let summary = confirm(&f).expect("xac nhan lan hai");

    assert_eq!((second.new_count, second.already_count), (0, 2), "moi cap deu da co");
    assert_eq!((summary.inserted, summary.already_count), (0, 2));
    assert_eq!(rows(&f.open.store), before);
}

#[test]
fn a_pair_repeated_inside_the_file_is_written_once_and_counted_as_already_there() {
    let f = fixture("dup-in-file");
    let text = tmx_with(&(tu("一", "A") + &tu("一", "A") + &tu("一", "B")));

    let preview = import(&f, "work", &text, "dup.tmx").expect("xem truoc");
    confirm(&f).expect("ghi");

    assert_eq!((preview.unit_count, preview.new_count, preview.already_count), (3, 2, 1));
    assert_eq!(rows(&f.open.store).len(), 2);
}

#[test]
fn a_foreign_tmx_without_props_imports_as_other_with_the_creation_date() {
    let f = fixture("foreign");
    let text = "<?xml version=\"1.0\"?><!DOCTYPE tmx SYSTEM \"tmx14.dtd\"><tmx version=\"1.4\"><header srclang=\"zh-CN\"/><body>\
        <tu creationdate=\"20200102T030405Z\"><tuv xml:lang=\"zh-CN\"><seg>你好</seg></tuv><tuv xml:lang=\"vi-VN\"><seg>Xin chao</seg></tuv></tu>\
        </body></tmx>";

    import(&f, "work", text, "foreign.tmx").expect("xem truoc");
    confirm(&f).expect("ghi");

    assert_eq!(
        rows(&f.open.store),
        vec![("你好".to_owned(), "Xin chao".to_owned(), "other".to_owned(), "2020-01-02T03:04:05.000Z".to_owned())]
    );
}

#[test]
fn a_unit_without_a_usable_date_is_stamped_now_in_the_stored_shape() {
    let f = fixture("date-now");
    let text = tmx_with(
        "<tu><prop type=\"x-aura-created-at\">hom qua</prop><tuv xml:lang=\"zh\"><seg>一</seg></tuv><tuv xml:lang=\"vi\"><seg>A</seg></tuv></tu>",
    );

    import(&f, "work", &text, "now.tmx").expect("xem truoc");
    confirm(&f).expect("ghi");

    let stored = &rows(&f.open.store)[0].3;
    assert_eq!(stored.len(), 24, "{stored}");
    assert!(stored.ends_with('Z') && stored.as_bytes()[19] == b'.', "{stored}");
}

#[test]
fn a_unit_missing_a_side_or_holding_a_blank_one_is_skipped_and_counted() {
    let f = fixture("skipped");
    let text = tmx_with(&format!(
        "<tu><tuv xml:lang=\"zh\"><seg>一</seg></tuv></tu>{}{}{}",
        tu("二", "   "),
        tu("  ", "B"),
        tu("三", "C")
    ));

    let preview = import(&f, "work", &text, "skip.tmx").expect("xem truoc");

    assert_eq!((preview.unit_count, preview.skipped_count, preview.new_count), (4, 3, 1));
}

#[test]
fn a_failed_preview_leaves_no_plan_and_a_file_in_the_wrong_language_names_the_works_language() {
    let f = fixture("no-usable");
    import(&f, "work", &tmx_with(&tu("一", "A")), "ok.tmx").expect("lan mot");

    let wrong = "<tmx version=\"1.4\"><header srclang=\"en\"/><body><tu><tuv xml:lang=\"en\"><seg>x</seg></tuv><tuv xml:lang=\"vi\"><seg>y</seg></tuv></tu></body></tmx>";
    let err = import(&f, "work", wrong, "bad.tmx").expect_err("khong co cap dung duoc");

    assert_eq!(err.code(), "tm.tmx_no_usable_pair");
    assert_eq!(err.params().get("source_lang").map(String::as_str), Some("zh"));
    assert_eq!(confirm(&f).expect_err("ke hoach cu da bi bo").code(), "tm.no_pending_import");

    let global_err = import(&f, "global", "<tmx><body><tu><tuv xml:lang=\"vi\"><seg>y</seg></tuv></tu></body></tmx>", "g.tmx")
        .expect_err("khong co cap");
    assert!(global_err.params().get("source_lang").is_none(), "tang Global khong co ngon ngu nguon");
}

#[test]
fn an_unreadable_file_also_drops_the_earlier_plan() {
    let f = fixture("missing-file");
    import(&f, "work", &tmx_with(&tu("一", "A")), "ok.tmx").expect("lan mot");

    let err = tm_open_import_preview(Some(&f.global), Some(&f.open), &f.pending, "work", &f.dir.join("khong-co.tmx"))
        .expect_err("khong doc duoc");

    assert_eq!(err.code(), "tm.tmx_read_failed");
    assert_eq!(confirm(&f).expect_err("khong con ke hoach").code(), "tm.no_pending_import");
}

#[test]
fn confirm_reports_the_previews_already_count_plus_pairs_that_raced_in() {
    let f = fixture("already-sum");
    seed(&f.open.store, &[("一", "A", "self", D1), ("二", "B", "self", D1)]);
    let text = tmx_with(&(tu("一", "A") + &tu("二", "B") + &tu("二", "B") + &tu("三", "C") + &tu("四", "D")));

    let preview = import(&f, "work", &text, "s.tmx").expect("xem truoc");
    seed(&f.open.store, &[("三", "C", "self", D1)]);
    let summary = confirm(&f).expect("ghi");

    assert_eq!((preview.new_count, preview.already_count), (2, 3));
    assert_eq!((summary.inserted, summary.already_count), (1, 4), "3 luc xem truoc + 1 xuat hien truoc khi ghi");
}

#[test]
fn broken_xml_names_the_line_and_writes_nothing() {
    let f = fixture("malformed");
    let text = format!("<?xml version=\"1.0\"?>\n<tmx version=\"1.4\">\n<header srclang=\"zh\"/>\n<body>\n{}\n<tu><tuv xml:lang=\"zh\"><seg>never closed</tuv></tu>\n</body></tmx>", tu("一", "A"));

    let err = import(&f, "work", &text, "broken.tmx").expect_err("hong");

    assert_eq!(err.code(), "tm.tmx_malformed");
    let line: usize = err.params().get("line").expect("tham so line").parse().expect("so");
    assert_eq!(line, 6);
    assert!(rows(&f.open.store).is_empty());
    assert_eq!(confirm(&f).expect_err("khong co ke hoach").code(), "tm.no_pending_import");
}

#[test]
fn an_unclosed_document_and_a_foreign_root_are_refused() {
    assert!(matches!(parse_tmx("<tmx><body><tu>"), Err(TmxError::Malformed { .. })));
    assert!(matches!(parse_tmx("<html><body/></html>"), Err(TmxError::NoBody)));
    assert!(matches!(parse_tmx("<tmx version=\"1.4\"><header/></tmx>"), Err(TmxError::NoBody)));
    assert!(matches!(parse_tmx("<tmx><body>&nope;</body></tmx>"), Err(TmxError::Malformed { .. })));
}

#[test]
fn cancel_drops_the_plan_and_a_new_preview_replaces_it() {
    let f = fixture("cancel");
    import(&f, "work", &tmx_with(&tu("一", "A")), "a.tmx").expect("xem truoc");

    tm_cancel_import(&f.pending);

    assert_eq!(confirm(&f).expect_err("da huy").code(), "tm.no_pending_import");
    assert!(rows(&f.open.store).is_empty());

    import(&f, "work", &tmx_with(&tu("一", "A")), "a.tmx").expect("lan mot");
    import(&f, "work", &tmx_with(&tu("二", "B")), "b.tmx").expect("lan hai thay lan mot");
    confirm(&f).expect("ghi");
    assert_eq!(rows(&f.open.store).iter().map(|r| r.1.as_str()).collect::<Vec<_>>(), vec!["B"]);
}

#[test]
fn closing_the_work_clears_a_work_plan_and_keeps_a_global_one() {
    let f = fixture("close-work");
    import(&f, "work", &tmx_with(&tu("一", "A")), "w.tmx").expect("xem truoc work");
    clear_pending_tmx_import_for_work(&f.pending);
    assert_eq!(confirm(&f).expect_err("work da dong").code(), "tm.no_pending_import");

    tm_open_import_preview(Some(&f.global), None, &f.pending, "global", &{
        let p = f.dir.join("g.tmx");
        fs::write(&p, tmx_with(&tu("一", "A"))).unwrap();
        p
    })
    .expect("xem truoc global khong can Tac pham");
    clear_pending_tmx_import_for_work(&f.pending);
    assert_eq!(tm_confirm_import(Some(&f.global), None, &f.pending).expect("global con").inserted, 1);
    assert!(rows(&f.open.store).is_empty());
}

#[test]
fn the_work_tier_needs_an_open_work_and_the_global_tier_does_not() {
    let f = fixture("no-work");
    let path = f.dir.join("n.tmx");
    fs::write(&path, tmx_with(&tu("一", "A"))).unwrap();

    let err = tm_open_import_preview(Some(&f.global), None, &f.pending, "work", &path).expect_err("khong co tac pham");
    assert_eq!(err.code(), "work.none_open");
    let err = tm_export_tier(Some(&f.global), None, "work", &f.dir.join("x.tmx")).expect_err("khong co tac pham");
    assert_eq!(err.code(), "work.none_open");
    assert!(!f.dir.join("x.tmx").exists());

    tm_open_import_preview(Some(&f.global), None, &f.pending, "global", &path).expect("global van nhap duoc");
    assert_eq!(tm_confirm_import(Some(&f.global), None, &f.pending).expect("ghi").inserted, 1);
    assert_eq!(rows(&f.global).len(), 1);
}

#[test]
fn a_global_preview_stores_no_work_id() {
    let f = fixture("global-work-id");
    import(&f, "global", &tmx_with(&tu("一", "A")), "g.tmx").expect("xem truoc");
    let plan = f.pending.lock().unwrap();
    assert_eq!(plan.as_ref().expect("ke hoach").work_id, None);
}

#[test]
fn an_unknown_tier_is_refused_before_any_file_is_touched() {
    let f = fixture("bad-tier");
    let err = tm_export_tier(Some(&f.global), Some(&f.open), "both", &f.dir.join("x.tmx")).expect_err("tang la");
    assert_eq!(err.code(), "tm.invalid_filter");
    assert!(!f.dir.join("x.tmx").exists());
}

#[test]
fn a_failing_row_rolls_the_whole_confirm_back_and_keeps_the_plan() {
    let f = fixture("atomic");
    seed(&f.open.store, &[("keep", "keep", "self", D1)]);
    let pair = |s: &str, t: &str| PlannedPair {
        source_text: s.to_owned(),
        target_text: t.to_owned(),
        translation_origin: PairOrigin::Other,
        created_at: None,
    };
    *f.pending.lock().unwrap() = Some(PendingTmxImport {
        tier: auratranslate_lib::core::tm::TmTier::Work,
        pairs: vec![pair("a", "A"), pair("b", ""), pair("c", "C")],
        work_id: Some(f.open.meta.work_id.clone()),
        already_count: 0,
    });

    let err = confirm(&f).expect_err("hang giua vi pham CHECK");

    assert!(!err.code().is_empty());
    assert_eq!(rows(&f.open.store).len(), 1, "khong hang nao cua lo duoc ghi");
    assert!(f.pending.lock().unwrap().is_some(), "ke hoach con de thu lai");
}

#[test]
fn confirm_skips_pairs_written_between_preview_and_confirm() {
    let f = fixture("race");
    import(&f, "work", &tmx_with(&(tu("一", "A") + &tu("二", "B"))), "r.tmx").expect("xem truoc");
    seed(&f.open.store, &[("一", "A", "self", D1)]);

    let summary = confirm(&f).expect("ghi");

    assert_eq!((summary.inserted, summary.already_count), (1, 1));
    assert_eq!(rows(&f.open.store).len(), 2);
}

#[test]
fn seg_text_is_stored_as_read_and_inline_elements_add_no_text() {
    let f = fixture("verbatim");
    let text = tmx_with(
        "<tu><tuv xml:lang=\"zh\"><seg>  一<ph x=\"1\">{1}</ph> <hi type=\"b\">二</hi>&amp;&lt;&#13;\n</seg></tuv>\
         <tuv xml:lang=\"vi\"><seg><![CDATA[ A <b> ]]><bpt i=\"1\">&lt;b&gt;</bpt>B<ept i=\"1\">&lt;/b&gt;</ept></seg></tuv></tu>",
    );

    import(&f, "work", &text, "v.tmx").expect("xem truoc");
    confirm(&f).expect("ghi");

    let got = &rows(&f.open.store)[0];
    assert_eq!(got.0, "  一 二&<\r\n");
    assert_eq!(got.1, " A <b> B");
}

#[test]
fn text_with_markup_characters_survives_export_then_import() {
    let f = fixture("escape");
    seed(&f.open.store, &[("a<b> & \"c\"\r\n", "x\ty  ", "self", D1)]);
    let (_, text) = export_to(&f, "work", "e.tmx");

    import(&f, "global", &text, "e-in.tmx").expect("xem truoc");
    confirm(&f).expect("ghi");

    let got = &rows(&f.global)[0];
    assert_eq!((got.0.as_str(), got.1.as_str()), ("a<b> & \"c\"\r\n", "x\ty  "));
}

#[test]
fn the_source_side_follows_the_works_language_or_the_header_srclang() {
    let f = fixture_with_lang("langs", "en");
    let both = "<tu><tuv xml:lang=\"zh\"><seg>ZH</seg></tuv><tuv xml:lang=\"en-US\"><seg>EN</seg></tuv><tuv xml:lang=\"vi\"><seg>V</seg></tuv></tu>";
    import(&f, "work", &tmx_with(both), "l.tmx").expect("work en");
    confirm(&f).expect("ghi");
    assert_eq!(rows(&f.open.store)[0].0, "EN", "Work en lay tuv en");

    let all = format!(
        "<tmx version=\"1.4\"><header srclang=\"*all*\"/><body>{both}</body></tmx>"
    );
    import(&f, "global", &all, "g.tmx").expect("global *all*");
    confirm(&f).expect("ghi");
    assert_eq!(rows(&f.global)[0].0, "ZH", "*all* lay tuv dau khong phai vi");
}

#[test]
fn utf8_with_bom_and_utf16_with_bom_are_read_and_anything_else_is_refused() {
    let xml = tmx_with(&tu("一", "A"));
    let mut utf8 = vec![0xEF, 0xBB, 0xBF];
    utf8.extend(xml.as_bytes());
    let mut le = vec![0xFF, 0xFE];
    le.extend(xml.encode_utf16().flat_map(u16::to_le_bytes));
    let mut be = vec![0xFE, 0xFF];
    be.extend(xml.encode_utf16().flat_map(u16::to_be_bytes));

    for bytes in [utf8, le, be, xml.clone().into_bytes()] {
        let text = decode_tmx_bytes(&bytes).expect("doc duoc");
        assert_eq!(parse_tmx(&text).expect("phan tich").units.len(), 1);
    }
    assert_eq!(decode_tmx_bytes(&[0xFF, 0x41, 0x42]), Err(TmxError::NotUtf8));
    assert_eq!(decode_tmx_bytes(&[0xFF, 0xFE, 0x41]), Err(TmxError::NotUtf8));
}

#[test]
fn a_file_over_the_cap_is_refused_without_being_loaded() {
    let f = fixture("too-large");
    let path = f.dir.join("big.tmx");
    {
        use std::io::{Seek, SeekFrom, Write as _};
        let mut file = fs::File::create(&path).unwrap();
        file.seek(SeekFrom::Start(MAX_TMX_BYTES + 1)).unwrap();
        file.write_all(b"x").unwrap();
    }

    let err = tm_open_import_preview(Some(&f.global), Some(&f.open), &f.pending, "work", &path).expect_err("qua tran");

    assert_eq!(err.code(), "tm.tmx_too_large");
    assert_eq!(read_tmx_file(&path), Err(TmxError::TooLarge { limit: MAX_TMX_BYTES }));
}

#[test]
fn a_cancelled_save_dialog_writes_no_file_and_a_picked_one_writes_atomically() {
    let f = fixture("dialog");
    seed(&f.open.store, &[("S", "A", "self", D1)]);
    let path = f.dir.join("out.tmx");

    assert_eq!(tm_export_tier_after_dialog(Some(&f.global), Some(&f.open), "work", None).unwrap(), None);
    assert!(!path.exists());

    let written = tm_export_tier_after_dialog(Some(&f.global), Some(&f.open), "work", Some(path.clone())).unwrap();
    assert_eq!(written.as_deref(), Some(path.display().to_string().as_str()));
    let leftovers: Vec<_> = fs::read_dir(&f.dir)
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "khong de tep .tmp");
}

#[test]
fn writing_to_a_missing_folder_is_a_write_error_and_leaves_nothing() {
    let dir = temp_dir("write-fail");
    let err = write_tmx_file(&dir.join("nope").join("x.tmx"), "x").expect_err("thu muc khong co");
    assert!(matches!(err, TmxError::WriteFailed { .. }));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_global_header_srclang_picks_the_matching_tuv_among_several_languages() {
    let f = fixture("global-en");
    let text = "<tmx version=\"1.4\"><header srclang=\"en\"/><body><tu>\
        <tuv xml:lang=\"zh\"><seg>ZH</seg></tuv><tuv xml:lang=\"en\"><seg>EN</seg></tuv><tuv xml:lang=\"vi\"><seg>V</seg></tuv>\
        </tu></body></tmx>";

    import(&f, "global", text, "ge.tmx").expect("xem truoc");
    confirm(&f).expect("ghi");

    assert_eq!(rows(&f.global)[0].0, "EN");
}

#[test]
fn export_keeps_the_newest_copy_of_a_same_side_duplicate_and_the_highest_id_on_equal_dates() {
    let f = fixture("export-order");
    seed(
        &f.open.store,
        &[("S", "A", "other", D2), ("S", "A", "other", D1), ("T", "B", "other", D1), ("T", "B", "other", D1)],
    );

    let (_, text) = export_to(&f, "work", "o.tmx");
    let walked = walk(&text);

    assert_eq!(walked.tus.len(), 2);
    let by_source: BTreeMap<&str, &WalkedTu> = walked.tus.iter().map(|t| (t.tuvs[0].1.as_str(), t)).collect();
    assert_eq!(by_source["S"].props["x-aura-created-at"], D2, "ban moi nhat du id thap hon");
    let ids_in_order: Vec<&str> = walked.tus.iter().map(|t| t.tuvs[0].1.as_str()).collect();
    assert_eq!(ids_in_order, vec!["S", "T"], "thu tu ra theo id cua ban duoc chon");
}

#[test]
fn export_picks_the_higher_id_when_dates_are_equal() {
    let f = fixture("export-tie");
    seed(&f.open.store, &[("S", "A", "other", D1), ("U", "X", "other", D1), ("S", "A", "other", D1)]);

    let (_, text) = export_to(&f, "work", "t.tmx");

    let order: Vec<String> = walk(&text).tus.into_iter().map(|t| t.tuvs[0].1.clone()).collect();
    assert_eq!(order, vec!["U", "S"], "tu duoc xuat theo id cua ban duoc chon: ban id 3 cua S dung sau U (id 2)");
}

#[test]
fn an_out_of_range_created_at_prop_falls_back_to_the_creation_date() {
    let f = fixture("bad-date");
    let text = tmx_with(
        "<tu creationdate=\"20200102T030405Z\"><prop type=\"x-aura-created-at\">2026-13-45T25:61:61.000Z</prop>\
         <tuv xml:lang=\"zh\"><seg>一</seg></tuv><tuv xml:lang=\"vi\"><seg>A</seg></tuv></tu>",
    );

    import(&f, "work", &text, "bd.tmx").expect("xem truoc");
    confirm(&f).expect("ghi");

    assert_eq!(rows(&f.open.store)[0].3, "2020-01-02T03:04:05.000Z");
}

/// Measurement harness, not a guard: `/usr/bin/time -l cargo test --release --test tmx_contract
/// -- --ignored --exact a_cap_sized_file_previews_and_confirms`.
#[test]
#[ignore = "measures peak memory on a ~250 MiB file; run by hand on a release build"]
fn a_cap_sized_file_previews_and_confirms() {
    use std::io::Write as _;
    let f = fixture("cap-sized");
    let path = f.dir.join("big.tmx");
    let mut units = 0usize;
    {
        let mut out = std::io::BufWriter::new(fs::File::create(&path).unwrap());
        out.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<tmx version=\"1.4\"><header srclang=\"zh\"/><body>").unwrap();
        let mut written = 0u64;
        while written < 250 * 1024 * 1024 {
            let unit = format!(
                "<tu creationdate=\"20200102T030405Z\"><prop type=\"x-aura-origin\">other</prop><tuv xml:lang=\"zh\"><seg>\u{4f60}\u{597d}\u{4e16}\u{754c} cau nguon so {units} voi mot doan van kha dai de tep dat can tran</seg></tuv><tuv xml:lang=\"vi\"><seg>Ban dich so {units} cung la mot doan van kha dai de tep dat can tran</seg></tuv></tu>\n"
            );
            written += unit.len() as u64;
            out.write_all(unit.as_bytes()).unwrap();
            units += 1;
        }
        out.write_all(b"</body></tmx>").unwrap();
    }
    let size = fs::metadata(&path).unwrap().len();
    let started = std::time::Instant::now();
    let preview = tm_open_import_preview(Some(&f.global), Some(&f.open), &f.pending, "work", &path).expect("xem truoc");
    let previewed = started.elapsed();
    let summary = confirm(&f).expect("ghi");
    eprintln!("bytes={size} units={units} preview={previewed:?} total={:?} inserted={}", started.elapsed(), summary.inserted);
    assert_eq!(preview.new_count, units);
}

#[test]
fn a_work_plan_confirmed_with_no_work_open_writes_nothing_and_drops_the_plan() {
    let f = fixture("plan-no-work-open");
    import(&f, "work", &tmx_with(&tu("一", "A")), "a.tmx").expect("xem truoc");

    let err = tm_confirm_import(Some(&f.global), None, &f.pending).expect_err("khong co Tac pham dang mo");

    assert_eq!(err.code(), "tm.no_pending_import");
    assert!(rows(&f.global).is_empty(), "khong ghi gi vao global.db");
    assert!(rows(&f.open.store).is_empty(), "khong ghi gi vao Tac pham");
    assert!(f.pending.lock().unwrap().is_none(), "ke hoach bi bo");
}

#[test]
fn a_work_plan_confirmed_with_another_work_open_writes_nothing_and_drops_the_plan() {
    let a = fixture("plan-for-a");
    let b = fixture("open-is-b");
    import(&a, "work", &tmx_with(&tu("一", "A")), "a.tmx").expect("xem truoc cho A");

    let err = tm_confirm_import(Some(&a.global), Some(&b.open), &a.pending).expect_err("Tac pham khac dang mo");

    assert_eq!(err.code(), "tm.no_pending_import");
    assert!(rows(&b.open.store).is_empty(), "khong ghi gi vao Tac pham B");
    assert!(rows(&a.open.store).is_empty());
    assert!(a.pending.lock().unwrap().is_none(), "ke hoach bi bo");
}

#[test]
fn the_global_confirm_leaves_a_plan_that_turned_into_a_work_plan_untouched() {
    let f = fixture("global-confirm-sees-work");
    import(&f, "work", &tmx_with(&tu("一", "A")), "w.tmx").expect("xem truoc Work");

    let outcome = tm_confirm_global_import(Some(&f.global), &f.pending).expect("khong loi");

    assert!(outcome.is_none(), "ke hoach Work khong phai viec cua nhanh Global");
    assert!(rows(&f.global).is_empty(), "khong ghi gi vao global.db");
    assert!(rows(&f.open.store).is_empty());
    assert!(f.pending.lock().unwrap().is_some(), "ke hoach con de nhanh Work chay");
}

#[test]
fn the_global_confirm_writes_a_global_plan_to_global_and_clears_it() {
    let f = fixture("global-confirm");
    let path = f.dir.join("g.tmx");
    fs::write(&path, tmx_with(&tu("一", "A"))).unwrap();
    tm_open_import_preview(Some(&f.global), None, &f.pending, "global", &path).expect("xem truoc Global");

    let summary = tm_confirm_global_import(Some(&f.global), &f.pending).expect("ghi").expect("nhanh Global");

    assert_eq!(summary.inserted, 1);
    assert_eq!(rows(&f.global).len(), 1);
    assert!(f.pending.lock().unwrap().is_none());
}
