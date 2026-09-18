//! Mọi hàng của I/O Matrix của spec 4.7 (Xem prompt cuối cùng đã gửi) —
//! `commands::aiprompt::{assemble_and_record_prompt, read_last_assembled_prompt}` cộng hình
//! dạng WIRE (`AssembledPromptWire`/`InjectionLedgerWire`/…) mà Story 4.8 và màn hình soi
//! prompt cùng đọc.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO TỆP NÀY, KHÔNG THÊM VÀO `ai_rag_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! `ai_rag_contract.rs` canh HAI HÀM của `core::ai::rag` (`gather_glossary_context`/
//! `assemble_prompt`) — hàm THUẦN/TẠP tách rời, không `Store`/`OpenWork` thật, không bản ghi
//! phiên. Tệp này canh TẦNG LỆNH của Story 4.7: `commands::aiprompt::assemble_and_record_prompt`/
//! `read_last_assembled_prompt` — chỗ DUY NHẤT resolve bộ prompt hai tầng, đọc Chương THẬT,
//! chọn đúng segment, GHI/ĐỌC bản ghi phiên, và ánh xạ ra hình dạng WIRE (`serde::Serialize`)
//! mà webview thực sự nhận. Một ca ở `ai_rag_contract.rs` không hề chạm `AssembledPromptWire`
//! hay `LastAssembledPromptState` — hai lớp khác nhau, hai tệp khác nhau.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BA-TRẠNG-THÁI ĐỨNG TRƯỚC — đúng rationale của Phase 4 (spec, Tasks & Acceptance)
//! ─────────────────────────────────────────────────────────────────────────────
//! `NotAsked` khác `Asked` rỗng là lớp lỗi TRUNG TÂM của dự án (root `AGENTS.md`: rỗng im
//! lặng) và không gate nào canh nó qua TẦNG DÂY (wire) — `ai_rag_contract.rs` canh nó ở tầng
//! `core::ai::rag` THUẦN, nhưng một bản ánh xạ `From<GlossaryInjectionStatus>` cẩu thả (gộp
//! `NotAsked`/`Asked{injected: vec![]}` thành cùng một hình dạng trên dây) sẽ không bị bắt ở
//! đó — chỉ bị bắt Ở ĐÂY, nơi `AssembledPromptWire` thật được đọc lại. Vì vậy: cặp Marker
//! absent/Asked-nothing-matched, rồi TM never built, rồi cặp Nothing-recorded-yet/Empty-body
//! — đúng thứ tự BA cặp ba-trạng-thái mà Phase 4's task đòi đứng TRƯỚC, để chứng minh sự
//! phân biệt số nhiều đó sống sót qua đúng LỚP dây, không chỉ qua nội bộ Rust.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ SO SÁNH TRÊN TRƯỜNG CỤ THỂ, KHÔNG CHỈ "danh sách không rỗng"
//! ─────────────────────────────────────────────────────────────────────────────
//! Root `AGENTS.md`/`src-tauri/AGENTS.md`: "A case that passes on both the injected and the
//! suppressed list guards neither" và "an assert that holds on both branches guards
//! neither". Mọi ca dưới đây đối chiếu ĐÚNG `tier`/`start`/`end`/`translation` của TỪNG dòng —
//! không chỉ độ dài danh sách.
//!
//! Dựng fixture qua `create_work_from_text`/`add_manual_term`/`prompt_set_create` — đúng khuôn
//! `ai_rag_contract.rs`/`glossary_marks_contract.rs`/`segment_contract.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::aiprompt::{
    GlossaryTierWire, LastAssembledPromptState, PromptPieceKindWire, PromptPieceWire,
    assemble_and_record_prompt, clear_last_assembled_prompt_on_work_close,
    read_last_assembled_prompt,
};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::promptset::{PromptSetTierWire, prompt_set_create};
use auratranslate_lib::commands::segment::read_open_chapter_segments;
use auratranslate_lib::core::glossary::{Category, GlossaryTier, add_manual_term};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::promptset::PromptSetTier;
use auratranslate_lib::core::store::{Store, StoreSpec};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-ai-prompt-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn open_global(dir: &Path) -> Store {
    Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db")
}

/// Đúng khuôn `glossary_marks_contract.rs`/`segment_contract.rs` — `create_work_from_text` là
/// chỗ SẢN PHẨM DUY NHẤT dựng một `OpenWork` với `ScopeResolver::with_work(...)` thật.
fn open_work(root: &Path, tag: &str, lang: &str, text: &str) -> OpenWork {
    create_work_from_text(root, tag, lang, "", text.to_owned())
        .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"))
}

/// Bản ghi phiên RỖNG, mới — đúng khuôn `commands::aiprompt::LastAssembledPromptState::new(None)`
/// (`lib.rs::open_work_slot`), dựng tay ở đây vì test không chạy qua Tauri `app.manage`.
fn fresh_record() -> LastAssembledPromptState {
    std::sync::Mutex::new(None)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Cặp ba-trạng-thái #1 — Glossary: Marker absent (NotAsked) / Asked, nothing matched
// ═════════════════════════════════════════════════════════════════════════════════

/// Hàng "Marker absent" — thân KHÔNG mang `{{glossary_terms}}` ⇒ KHÔNG lượt gọi Glossary nào
/// chạy, và trên DÂY `kind` phải là `"not_asked"` với CẢ HAI trường payload `None` — không
/// một cờ `injected: Some(vec![])` giả lập "0 injected" thay cho "chưa hỏi".
#[test]
fn marker_absent_states_no_glossary_query_ran_and_never_claims_zero_injected() {
    let global_dir = temp_dir("marker-absent-global");
    let work_dir = temp_dir("marker-absent-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Marker Absent", "en", "A dragon roared.");

    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong"), "", Category::Other)
        .expect("them thuat ngu (khong duoc dung toi vi khong marker glossary trong than)");

    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "NoMarker", "{{source_segment}}")
        .expect("tao bo prompt");

    let segment_id = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments[0].id;
    let record = fresh_record();

    let wire = assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("NoMarker"), segment_id)
        .expect("lap rap khong duoc loi");

    assert_eq!(wire.ledger.glossary.kind, "not_asked");
    assert!(wire.ledger.glossary.injected.is_none(), "NotAsked khong duoc mang mot Some rong");
    assert!(wire.ledger.glossary.suppressed_by_pending_overlap.is_none());

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// Hàng "Asked, nothing matched" — thân MANG marker, câu KHÔNG chứa thuật ngữ đã chốt nào ⇒
/// `kind == "asked"` với `injected == Some(vec![])` — MỘT `Vec` RỖNG, khác hẳn `NotAsked` ở
/// ca trên (hai biến thể `enum`, không một `bool` cạnh một `Vec` rỗng).
#[test]
fn asked_but_nothing_matched_says_zero_injected_not_not_asked() {
    let global_dir = temp_dir("asked-empty-global");
    let work_dir = temp_dir("asked-empty-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Asked Empty", "en", "Nothing here.");

    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong"), "", Category::Other)
        .expect("them thuat ngu -- khong khop cau nay, dung y");

    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "AskedEmpty",
        "Terms: {{glossary_terms}}\n{{source_segment}}",
    )
    .expect("tao bo prompt");

    let segment_id = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments[0].id;
    let record = fresh_record();

    let wire = assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("AskedEmpty"), segment_id)
        .expect("lap rap khong duoc loi");

    assert_eq!(wire.ledger.glossary.kind, "asked");
    let injected = wire.ledger.glossary.injected.expect("Asked phai mang Some, du rong");
    assert!(injected.is_empty(), "khong thuat ngu nao khop cau nay: {injected:?}");
    let suppressed = wire
        .ledger
        .glossary
        .suppressed_by_pending_overlap
        .expect("Asked phai mang Some cho ca hai truong payload");
    assert!(suppressed.is_empty());

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ba-trạng-thái #2 — TM: never built (đến Epic 7, nhánh `Searched` không đường gọi nào của
// story này tạo được — xem doc-comment `commands/aiprompt.rs::wire::TmInjectionStatusWire`)
// ═════════════════════════════════════════════════════════════════════════════════

/// Bất kỳ lượt lắp ráp nào trong epic này ⇒ `kind == "not_built_yet"`, KHÔNG BAO GIỜ
/// `"searched"` với một lát cắt rỗng đọc như "0 similar sentences" — hai câu khác nhau hoàn
/// toàn ("TM chưa dựng" và "TM đã tra, không khớp gì").
#[test]
fn tm_is_never_built_yet_and_never_read_as_zero_similar_sentences() {
    let global_dir = temp_dir("tm-not-built-global");
    let work_dir = temp_dir("tm-not-built-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "TM Not Built", "en", "A quiet sentence.");

    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "TmProbe", "{{source_segment}}")
        .expect("tao bo prompt");

    let segment_id = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments[0].id;
    let record = fresh_record();

    let wire = assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("TmProbe"), segment_id)
        .expect("lap rap khong duoc loi");

    assert_eq!(wire.ledger.tm.kind, "not_built_yet");
    assert!(
        wire.ledger.tm.similar_segments.is_none(),
        "NotBuiltYet khong duoc mang mot Some rong -- do doc y het 'da tra, khong khop gi'"
    );

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ba-trạng-thái #3 — bản ghi phiên: Nothing recorded yet (None) / một bản ghi có `prompt` rỗng
// ═════════════════════════════════════════════════════════════════════════════════

/// Hàng "Nothing recorded yet" — `read_last_assembled_prompt` trên một bản ghi phiên chưa
/// từng qua `assemble_and_record_prompt` lần nào ⇒ `None`, KHÔNG một `IpcError` (I/O Matrix:
/// "this is a state, not an error").
#[test]
fn nothing_recorded_yet_this_session_is_a_state_not_an_error() {
    let record = fresh_record();
    let read_back = read_last_assembled_prompt(&record);
    assert!(read_back.is_none(), "chua lap lan nao trong phien nay phai doc thanh None");
}

/// Hàng "Empty body" — bộ prompt hiệu lực có `body == ""` ⇒ lượt lắp ráp vẫn THÀNH CÔNG
/// (không lỗi), bản ghi là `Some` mang `prompt == ""` — PHÂN BIỆT với ca trên (`None`): một
/// bản ghi RỖNG vẫn là "đã lắp", không phải "chưa lắp lần nào".
#[test]
fn an_empty_prompt_set_body_is_recorded_as_present_not_as_nothing_recorded() {
    let global_dir = temp_dir("empty-body-global");
    let work_dir = temp_dir("empty-body-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Empty Body", "en", "A sentence that is ignored.");

    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "EmptySet", "")
        .expect("tao bo prompt voi than RONG");

    let segment_id = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments[0].id;
    let record = fresh_record();

    let wire = assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("EmptySet"), segment_id)
        .expect("than RONG khong duoc la mot loi -- day la mot TRANG THAI, khong phai mot vi pham");

    assert_eq!(wire.prompt, "", "prompt cua ban ghi phai RONG, dung theo than bo prompt");
    // Cac phan khac cua ban ghi (dinh danh, ledger) van hien binh thuong -- khong bi "sup do"
    // theo `prompt` rong.
    assert_eq!(wire.segment_id, segment_id);
    assert_eq!(wire.prompt_set_name, "EmptySet");
    assert_eq!(wire.ledger.glossary.kind, "not_asked", "than rong khong mang {{glossary_terms}}");
    assert!(wire.ledger.source_segment_missing, "than rong khong mang {{source_segment}}");

    // Đọc lại qua `read_last_assembled_prompt` phải cho đúng CÙNG bản ghi -- `Some`, không
    // `None` -- phân biệt hẳn với ca "Nothing recorded yet" ở trên.
    let read_back = read_last_assembled_prompt(&record).expect("phai la Some -- da lap mot lan");
    assert_eq!(read_back.prompt, "");

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Happy path
// ═════════════════════════════════════════════════════════════════════════════════

/// Hai thuật ngữ đã chốt trong một câu; thân bộ prompt mang cả hai marker ⇒ bản ghi mang
/// prompt lắp đúng, ledger liệt HAI dòng tiêm với ĐỦ `translation`/`tier`/`start`/`end` — so
/// trên TRƯỜNG CỤ THỂ, không chỉ độ dài danh sách (root `AGENTS.md`).
#[test]
fn happy_path_records_two_confirmed_terms_with_their_translation_and_tier() {
    let global_dir = temp_dir("happy-global");
    let work_dir = temp_dir("happy-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Happy Path", "en", "A dragon guards the castle.");

    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong"), "", Category::Other)
        .expect("them dragon");
    add_manual_term(&global, None, GlossaryTier::Global, "castle", Some("lau dai"), "", Category::Other)
        .expect("them castle");

    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "Happy",
        "Terms: {{glossary_terms}}\nSentence: {{source_segment}}",
    )
    .expect("tao bo prompt");

    let segments = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments;
    let segment_id = segments[0].id;
    let sentence = segments[0].source_text.clone();
    assert_eq!(sentence, "A dragon guards the castle.", "tien de: van ban khong bi bien dang qua pipeline nhap");

    let record = fresh_record();
    let wire = assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("Happy"), segment_id)
        .expect("lap rap khong duoc loi");

    let dragon_start = sentence.find("dragon").expect("tien de: 'dragon' co trong cau");
    let dragon_end = dragon_start + "dragon".len();
    let castle_start = sentence.find("castle").expect("tien de: 'castle' co trong cau");
    let castle_end = castle_start + "castle".len();

    assert_eq!(
        wire.prompt,
        format!("Terms: \ndragon → rong\ncastle → lau dai\nSentence: {sentence}"),
        "chuoi prompt phai lap dung khoi hai cap + cau nguon, khong noi giua dong"
    );
    assert_eq!(wire.segment_id, segment_id);
    assert_eq!(wire.chapter_id, open.chapter_id);
    assert_eq!(wire.prompt_set_name, "Happy");
    assert_eq!(wire.prompt_set_tier, PromptSetTierWire::Global);

    assert_eq!(wire.ledger.glossary.kind, "asked");
    let injected = wire.ledger.glossary.injected.expect("Asked phai mang Some");
    assert_eq!(injected.len(), 2, "{injected:?}");

    assert_eq!(injected[0].source_term, "dragon");
    assert_eq!(injected[0].translation, "rong");
    assert_eq!(injected[0].start, dragon_start);
    assert_eq!(injected[0].end, dragon_end);
    assert_eq!(injected[0].tier, GlossaryTierWire::Global);

    assert_eq!(injected[1].source_term, "castle");
    assert_eq!(injected[1].translation, "lau dai");
    assert_eq!(injected[1].start, castle_start);
    assert_eq!(injected[1].end, castle_end);
    assert_eq!(injected[1].tier, GlossaryTierWire::Global);

    let suppressed = wire.ledger.glossary.suppressed_by_pending_overlap.expect("Asked phai mang Some");
    assert!(suppressed.is_empty());

    // 🔴 finding B1 (loop 1) -- guard THẬT của AC2/AC4: nối `.text` của TOÀN BỘ `pieces` theo
    // đúng thứ tự phải cho lại `wire.prompt` TỪNG BYTE. Đây là đối chứng spec đòi ("asserted in
    // ai_prompt_contract.rs") -- không phải một lời hứa trong doc-comment.
    let concatenated: String = wire.ledger.pieces.iter().map(|p| p.text.as_str()).collect();
    assert_eq!(
        concatenated, wire.prompt,
        "noi lai `pieces` phai cho dung `prompt` TUNG BYTE -- pieces={:?}",
        wire.ledger.pieces
    );
    // Và từng mảnh đúng nhãn của nó -- một assert chỉ so tổng nối lại có thể xanh dù nhãn sai
    // (vd. mảnh Glossary bị gắn nhãn Authored nhưng văn bản vẫn đúng vị trí). So CẢ nhãn LẪN
    // văn bản, từng mảnh một.
    //
    // 🔴 SỬA loop 2, finding P7 -- câu nguồn giờ mang nhãn RIÊNG `SourceSegment`, không còn gộp
    // vào `Authored` của đoạn "\nSentence: " đứng trước nó -- hai nhãn khác nhau không gộp mảnh
    // (xem `push_piece`), nên bốn mảnh thay vì ba.
    assert_eq!(wire.ledger.pieces.len(), 4, "{:?}", wire.ledger.pieces);
    assert_eq!(wire.ledger.pieces[0].kind, PromptPieceKindWire::Authored);
    assert_eq!(wire.ledger.pieces[0].text, "Terms: ");
    assert_eq!(wire.ledger.pieces[1].kind, PromptPieceKindWire::Glossary);
    assert_eq!(wire.ledger.pieces[1].text, "\ndragon → rong\ncastle → lau dai");
    assert_eq!(wire.ledger.pieces[2].kind, PromptPieceKindWire::Authored);
    assert_eq!(wire.ledger.pieces[2].text, "\nSentence: ");
    assert_eq!(wire.ledger.pieces[3].kind, PromptPieceKindWire::SourceSegment);
    assert_eq!(wire.ledger.pieces[3].text, sentence);

    // Bản ghi đọc lại phải khớp CHÍNH bản ghi vừa ghi -- "screen reads the record", không lắp
    // lại.
    let read_back = read_last_assembled_prompt(&record).expect("phai co ban ghi");
    assert_eq!(read_back.prompt, wire.prompt);
    assert_eq!(read_back.ledger.pieces, wire.ledger.pieces);

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Pending overlap
// ═════════════════════════════════════════════════════════════════════════════════

/// Một thuật ngữ CHỜ CHỐT che một thuật ngữ ĐÃ CHỐT khi hai bên chồng span (Decision 4, spec
/// 4.6) — mục đã chốt phải xuất hiện ở `suppressed_by_pending_overlap`, KHÔNG ở `injected`.
/// So trên trường cụ thể (`tier`/`start`/`end`/`translation`), đúng cảnh báo `AGENTS.md` về
/// một ca chỉ kiểm "danh sách không rỗng".
#[test]
fn a_pending_term_overlapping_a_confirmed_one_is_suppressed_not_injected() {
    let global_dir = temp_dir("pending-overlap-global");
    let work_dir = temp_dir("pending-overlap-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Pending Overlap", "en", "The dog walker arrived.");

    add_manual_term(&global, None, GlossaryTier::Global, "dog", Some("cho"), "", Category::Other)
        .expect("them dog da chot");
    add_manual_term(&global, None, GlossaryTier::Global, "dog walker", None, "", Category::Other)
        .expect("them dog walker cho chot");

    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "Overlap",
        "Terms: {{glossary_terms}}\n{{source_segment}}",
    )
    .expect("tao bo prompt");

    let segments = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments;
    let segment_id = segments[0].id;
    let sentence = segments[0].source_text.clone();
    assert_eq!(sentence, "The dog walker arrived.");

    let record = fresh_record();
    let wire = assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("Overlap"), segment_id)
        .expect("lap rap khong duoc loi");

    let dog_start = sentence.find("dog").expect("tien de: 'dog' co trong cau");
    let dog_end = dog_start + "dog".len();

    assert_eq!(wire.ledger.glossary.kind, "asked");
    let injected = wire.ledger.glossary.injected.expect("Asked phai mang Some");
    assert!(injected.is_empty(), "khong ben nao duoc chen: {injected:?}");

    let suppressed = wire.ledger.glossary.suppressed_by_pending_overlap.expect("Asked phai mang Some");
    assert_eq!(suppressed.len(), 1, "{suppressed:?}");
    assert_eq!(suppressed[0].source_term, "dog");
    assert_eq!(suppressed[0].translation, "cho");
    assert_eq!(suppressed[0].start, dog_start);
    assert_eq!(suppressed[0].end, dog_end);
    assert_eq!(suppressed[0].tier, GlossaryTierWire::Global);

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Sentence marker missing
// ═════════════════════════════════════════════════════════════════════════════════

/// Thân KHÔNG mang `{{source_segment}}` ⇒ lắp ráp vẫn THÀNH CÔNG (§Never: không hành vi từ
/// chối), và ledger phơi cờ CẢNH BÁO — không lỗi.
#[test]
fn a_body_missing_the_source_segment_marker_still_assembles_and_the_ledger_flags_it() {
    let global_dir = temp_dir("no-sentence-global");
    let work_dir = temp_dir("no-sentence-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "No Sentence Marker", "en", "This sentence is never used.");

    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "NoSentence", "Static text only.")
        .expect("tao bo prompt");

    let segment_id = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments[0].id;
    let record = fresh_record();

    let wire = assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("NoSentence"), segment_id)
        .expect("thieu {{source_segment}} khong duoc la mot loi");

    assert_eq!(wire.prompt, "Static text only.");
    assert!(wire.ledger.source_segment_missing, "than khong mang {{source_segment}} phai duoc gan co");

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Unknown marker
// ═════════════════════════════════════════════════════════════════════════════════

/// Marker lạ (`{{chapter_context}}`) đứng NGUYÊN VĂN trong prompt và được liệt tên trong
/// ledger — không bị coi là lỗi, không bị xoá.
#[test]
fn an_unknown_marker_stands_verbatim_and_is_named_in_the_ledger() {
    let global_dir = temp_dir("unknown-marker-global");
    let work_dir = temp_dir("unknown-marker-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Unknown Marker", "en", "Hello world.");

    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "UnknownMarker",
        "{{chapter_context}} {{source_segment}}",
    )
    .expect("tao bo prompt");

    let segments = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments;
    let segment_id = segments[0].id;
    let sentence = segments[0].source_text.clone();

    let record = fresh_record();
    let wire = assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("UnknownMarker"), segment_id)
        .expect("marker la khong duoc la mot loi");

    assert_eq!(wire.prompt, format!("{} {}", "{{chapter_context}}", sentence));
    assert_eq!(wire.ledger.unknown_markers, vec!["{{chapter_context}}".to_owned()]);

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// No set selected — hai nguyên nhân con, CÙNG một khoá
// ═════════════════════════════════════════════════════════════════════════════════

/// `prompt_set_name: None` (webview chưa chọn bộ nào) ⇒ `ai_prompt.no_set_selected`.
#[test]
fn no_prompt_set_name_sent_returns_no_set_selected() {
    let global_dir = temp_dir("no-name-sent-global");
    let work_dir = temp_dir("no-name-sent-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "No Name Sent", "en", "Anything.");
    let record = fresh_record();

    let err = assemble_and_record_prompt(Some(&global), Some(&open), &record, None, 1)
        .expect_err("None ten bo prompt phai la mot loi");

    assert_eq!(err.code(), "ai_prompt.no_set_selected");
    assert_eq!(err.message_key(), MessageKey::AiPromptNoSetSelected);

    // Khong lap rap gi ca -- ban ghi phai con RONG.
    assert!(read_last_assembled_prompt(&record).is_none());

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// Một tên gửi lên nhưng KHÔNG khớp bộ nào đã phân giải ⇒ CÙNG khoá `ai_prompt.no_set_selected`
/// (Phase 2's judgment call: một sự thật, một khoá) — dựng với `open: None` để chứng minh
/// đây thực sự là ca "không có bộ hiệu lực", không phải một ca "chưa mở Tác phẩm" trá hình
/// (thứ tự kiểm của `assemble_and_record_prompt`: bộ prompt hiệu lực TRƯỚC).
#[test]
fn an_unresolvable_prompt_set_name_also_returns_no_set_selected() {
    let global_dir = temp_dir("unresolvable-name-global");
    let global = open_global(&global_dir);
    let record = fresh_record();

    let err = assemble_and_record_prompt(Some(&global), None, &record, Some("khong-ton-tai"), 1)
        .expect_err("mot ten khong khop bo nao phai la mot loi");

    assert_eq!(err.code(), "ai_prompt.no_set_selected");
    assert_eq!(err.message_key(), MessageKey::AiPromptNoSetSelected);

    drop(global);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// No Work open / segment không có trong Chương — HAI KHOÁ RIÊNG (I/O Matrix: "the two causes
// are distinct keys")
// ═════════════════════════════════════════════════════════════════════════════════

/// `open: None`, dù bộ prompt hiệu lực đã có ⇒ `work.none_open` (tái dùng
/// `MessageKey::WorkNoneOpen`, không một khoá riêng cho story này).
#[test]
fn no_open_work_returns_work_none_open() {
    let global_dir = temp_dir("no-work-global");
    let global = open_global(&global_dir);
    // Tao bo prompt tren mot OpenWork TAM chi de resolve duoc ten -- roi bo qua no khi goi
    // ham thuc su voi `open: None`.
    let scratch_dir = temp_dir("no-work-scratch");
    let scratch = open_work(&scratch_dir, "Scratch", "en", "x");
    prompt_set_create(Some(&global), Some(&scratch), PromptSetTier::Global, "ValidSet", "{{source_segment}}")
        .expect("tao bo prompt Global (khong phu thuoc OpenWork nao)");
    drop(scratch);
    cleanup(&scratch_dir);

    let record = fresh_record();
    let err = assemble_and_record_prompt(Some(&global), None, &record, Some("ValidSet"), 1)
        .expect_err("khong Tac pham nao dang mo phai la mot loi");

    assert_eq!(err.code(), "work.none_open");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);

    drop(global);
    cleanup(&global_dir);
}

/// Tác phẩm ĐANG MỞ, Chương ĐỌC ĐƯỢC, nhưng `segment_id` không thuộc Chương đó ⇒
/// `ai_prompt.segment_not_in_chapter`, mang `segment_id`/`chapter_id` trong `params` — khoá
/// RIÊNG, khác hẳn `work.none_open`.
#[test]
fn a_segment_id_absent_from_the_open_chapter_returns_its_own_key() {
    let global_dir = temp_dir("bad-segment-global");
    let work_dir = temp_dir("bad-segment-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Bad Segment", "en", "Only one real sentence.");

    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "ValidSet", "{{source_segment}}")
        .expect("tao bo prompt");

    let real_segments = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments;
    let bogus_id = real_segments.iter().map(|s| s.id).max().unwrap_or(0) + 999_000;

    let record = fresh_record();
    let err = assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("ValidSet"), bogus_id)
        .expect_err("mot id la trong Chuong phai la mot loi");

    assert_eq!(err.code(), "ai_prompt.segment_not_in_chapter");
    assert_eq!(err.message_key(), MessageKey::AiPromptSegmentNotInChapter);
    assert_eq!(err.params().get("segment_id").map(String::as_str), Some(bogus_id.to_string().as_str()));
    assert_eq!(
        err.params().get("chapter_id").map(String::as_str),
        Some(open.chapter_id.to_string().as_str())
    );

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// Đối chứng trực tiếp cho "the two causes are distinct keys" — cả hai lỗi trên, đối chiếu
/// TRỰC TIẾP với NHAU trong một ca duy nhất: một cổng nào đó lỡ ánh xạ cả hai về CÙNG một
/// `code()`/`message_key()` sẽ bị bắt ngay đây, không chỉ ở hai ca riêng lẻ phía trên.
#[test]
fn no_work_open_and_segment_not_in_chapter_are_two_distinct_keys() {
    let global_dir = temp_dir("distinct-keys-global");
    let work_dir = temp_dir("distinct-keys-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Distinct Keys", "en", "Only one real sentence.");

    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "ValidSet", "{{source_segment}}")
        .expect("tao bo prompt");

    let record_a = fresh_record();
    let no_work_err = assemble_and_record_prompt(Some(&global), None, &record_a, Some("ValidSet"), 1)
        .expect_err("khong Tac pham dang mo phai la mot loi");

    let real_segments = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments;
    let bogus_id = real_segments.iter().map(|s| s.id).max().unwrap_or(0) + 999_000;
    let record_b = fresh_record();
    let bad_segment_err = assemble_and_record_prompt(Some(&global), Some(&open), &record_b, Some("ValidSet"), bogus_id)
        .expect_err("mot id la phai la mot loi");

    assert_ne!(
        no_work_err.code(),
        bad_segment_err.code(),
        "hai nguyen nhan khac han nhau khong duoc chia se mot ma"
    );
    assert_ne!(
        no_work_err.message_key(),
        bad_segment_err.message_key(),
        "hai nguyen nhan khac han nhau khong duoc chia se mot message_key"
    );

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Stale record — danh tính của bản ghi (thứ màn hình dùng để phát hiện "cũ")
// ═════════════════════════════════════════════════════════════════════════════════

/// Bản ghi mang ĐÚNG `segment_id` của lượt lắp vừa xảy ra, và lượt lắp SAU (một câu khác)
/// ghi ĐÈ bằng danh tính MỚI — đây là dữ kiện wire mà `aiPromptRecordIsStale` (frontend, hàm
/// thuần) đọc để so sánh với câu đang có tiêu điểm; I/O Matrix "Stale record" chính là hệ quả
/// màn hình của dữ kiện này.
#[test]
fn the_record_carries_the_segment_identity_it_was_built_from() {
    let global_dir = temp_dir("identity-global");
    let work_dir = temp_dir("identity-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Identity", "zh", "Cau mot。Cau hai。");

    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "IdentitySet", "{{source_segment}}")
        .expect("tao bo prompt");

    let segments = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments;
    assert_eq!(segments.len(), 2, "tien de: hai cau phai cho ra hai segment rieng biet: {segments:?}");
    let (id_a, id_b) = (segments[0].id, segments[1].id);
    assert_ne!(id_a, id_b, "hai segment phai mang hai id khac nhau");

    let record = fresh_record();

    let wire_a = assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("IdentitySet"), id_a)
        .expect("lap rap cau A khong duoc loi");
    assert_eq!(wire_a.segment_id, id_a);
    assert_eq!(
        read_last_assembled_prompt(&record).expect("phai co ban ghi").segment_id,
        id_a,
        "doc lai ngay sau lap A phai mang dung danh tinh A"
    );

    let wire_b = assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("IdentitySet"), id_b)
        .expect("lap rap cau B khong duoc loi");
    assert_eq!(wire_b.segment_id, id_b);
    assert_eq!(
        read_last_assembled_prompt(&record).expect("phai co ban ghi").segment_id,
        id_b,
        "lap B phai DE len ban ghi cua A -- ban ghi PHIEN chi giu MOT ban gan nhat"
    );
    assert_ne!(
        wire_a.segment_id, wire_b.segment_id,
        "hai lan lap lien tiep tren hai cau khac nhau phai cho hai danh tinh khac nhau -- day \
         la dieu kien man hinh CAN de phat hien mot ban ghi CU (Stale record)"
    );

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// SỬA 2026-09-18 (lượt rà soát build) — `clear_last_assembled_prompt_on_work_close`, chỗ nối
// KHÔNG có ca nào canh trước bản sửa (Blind Hunter bắt được: `lib.rs::close_open_work` viết
// tay ba dòng, KHÔNG `pub`, không cách nào một `tests/*.rs` gọi tới; sửa bằng cách trích ra
// một `pub fn` riêng, đúng khuôn `clear_pending_prompt_import_work_tier`/
// `clear_pending_import_for_tier` — ca dưới đây là ca đơn vị TRỰC TIẾP của hàm đó, cùng
// khuôn `glossary_import_dialog_contract.rs` đã áp cho hai hàm láng giềng).
// ═════════════════════════════════════════════════════════════════════════════════

/// Một bản ghi ĐÃ CÓ trong phiên (segment/chapter identity thật của Tác phẩm A) ⇒ sau khi
/// gọi hàm xoá (đúng thời điểm `close_open_work` gọi nó khi Tác phẩm đang mở ĐÓNG), bản ghi
/// phải về `None` — đúng lý do doc-comment ghi: id của Tác phẩm A không còn nghĩa gì cho một
/// Tác phẩm B mở sau, có thể trùng số ngẫu nhiên.
#[test]
fn the_record_is_cleared_when_the_open_work_closes() {
    let global_dir = temp_dir("clear-on-close-global");
    let work_dir = temp_dir("clear-on-close-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Clear On Close", "en", "A sentence to clear.");

    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "ClearSet", "{{source_segment}}")
        .expect("tao bo prompt");

    let segment_id = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments[0].id;
    let record = fresh_record();

    assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("ClearSet"), segment_id)
        .expect("lap rap khong duoc loi");
    assert!(
        read_last_assembled_prompt(&record).is_some(),
        "tien de: phai co ban ghi truoc khi goi ham xoa"
    );

    clear_last_assembled_prompt_on_work_close(&record);

    assert!(
        read_last_assembled_prompt(&record).is_none(),
        "sau khi dong Tac pham, ban ghi phai ve None -- id cua Tac pham vua dong khong con \
         nghia gi cho mot Tac pham khac co the mo sau, trung so ngau nhien"
    );

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// Chưa từng lắp lần nào ⇒ gọi hàm xoá vẫn an toàn, không panic, và đọc lại vẫn `None` —
/// đúng lớp ca biên mà chính hàm này (một `Mutex<Option<T>>` trần) đã phải xử lý cho MỌI
/// lượt gọi, không riêng lượt có bản ghi.
#[test]
fn clearing_an_already_empty_record_on_work_close_is_a_no_op() {
    let record: LastAssembledPromptState = fresh_record();
    assert!(read_last_assembled_prompt(&record).is_none(), "tien de: chua lap lan nao");

    clear_last_assembled_prompt_on_work_close(&record);

    assert!(read_last_assembled_prompt(&record).is_none(), "van phai la None, khong panic");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 4.7 loop 2, task 6-3 (finding P3) — không một ca nào trong toàn kho pin hình dạng
// serde THẬT của một wire kiểu 4.7 trước bản này. `#[serde(rename_all = "snake_case")]` bị
// gỡ khỏi `PromptPieceKindWire`/`GlossaryInjectionStatusWire`'s tag/`TmInjectionStatusWire`'s
// tag thì mọi ca ở trên vẫn xanh (chúng so trên GIÁ TRỊ RUST đã giải mã, `PromptPieceKindWire::
// Authored`, không so trên CHUỖI JSON THẬT) — đúng khuyết tật `ipc_contract.rs`'s doc-comment
// đầu tệp cảnh báo ("so trên serde_json::to_value(…), không có tầng biến đổi nào chen giữa").
// ═════════════════════════════════════════════════════════════════════════════════

/// Wire THẬT trả về từ [`assemble_and_record_prompt`] (không dựng tay) — pin đúng ba nhãn
/// `pieces` mà đường gọi thật này SẢN XUẤT ĐƯỢC (`"authored"`/`"glossary"`/`"source_segment"`),
/// cộng TÊN TRƯỜNG của mọi tầng trên dây, dưới dạng CHUỖI JSON thật -- không phải Rust-side
/// `PartialEq` trên `enum`/`struct` đã giải mã.
#[test]
fn the_wire_returned_by_assemble_and_record_prompt_serializes_with_the_exact_tag_strings_and_field_names()
 {
    let global_dir = temp_dir("serde-pin-global");
    let work_dir = temp_dir("serde-pin-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Serde Pin", "en", "A dragon guards the castle.");

    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong"), "", Category::Other)
        .expect("them dragon");

    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "SerdePin",
        "Terms: {{glossary_terms}}\nSentence: {{source_segment}}",
    )
    .expect("tao bo prompt");

    let segments = read_open_chapter_segments(Some(&open)).expect("nap chuong").segments;
    let segment_id = segments[0].id;

    let record = fresh_record();
    let wire =
        assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("SerdePin"), segment_id)
            .expect("lap rap khong duoc loi");

    let json = serde_json::to_value(&wire).expect("AssembledPromptWire phai serialize duoc");

    let top_keys: std::collections::BTreeSet<&str> =
        json.as_object().expect("wire la mot object").keys().map(String::as_str).collect();
    assert_eq!(
        top_keys,
        std::collections::BTreeSet::from([
            "prompt",
            "segment_id",
            "chapter_id",
            "prompt_set_name",
            "prompt_set_tier",
            "ledger",
        ]),
        "TEN TRUONG cap AssembledPromptWire tren day that -- mot truong bi doi ten/them/bot \
         khong duoc mot ca nao khac trong tep nay bat, vi tat ca so tren gia tri Rust da giai ma"
    );
    assert_eq!(json["prompt_set_tier"], serde_json::json!("global"));

    let ledger = &json["ledger"];
    let ledger_keys: std::collections::BTreeSet<&str> =
        ledger.as_object().expect("ledger la mot object").keys().map(String::as_str).collect();
    assert_eq!(
        ledger_keys,
        std::collections::BTreeSet::from([
            "glossary",
            "tm",
            "unknown_markers",
            "source_segment_missing",
            "pieces",
        ]),
    );

    assert_eq!(ledger["glossary"]["kind"], serde_json::json!("asked"));
    let injected = ledger["glossary"]["injected"].as_array().expect("injected la mang");
    assert_eq!(injected.len(), 1);
    let injected_keys: std::collections::BTreeSet<&str> =
        injected[0].as_object().expect("phan tu injected la mot object").keys().map(String::as_str).collect();
    assert_eq!(
        injected_keys,
        std::collections::BTreeSet::from(["source_term", "translation", "start", "end", "tier"]),
    );
    assert_eq!(injected[0]["source_term"], serde_json::json!("dragon"));
    assert_eq!(injected[0]["tier"], serde_json::json!("global"), "GlossaryTierWire::Global phai la chuoi \"global\", khong \"Global\"");

    // `tm` luôn `not_built_yet` ở lệnh gọi này (tham số `tm` của `assemble_prompt` luôn `None`)
    // -- pin TAG này cũng ở tầng JSON, không ở tầng `TmInjectionStatus::NotBuiltYet` đã so.
    assert_eq!(ledger["tm"], serde_json::json!({ "kind": "not_built_yet", "similar_segments": null }));

    // `pieces` -- ĐÚNG BA nhãn đường gọi này sản xuất được, dưới dạng CHUỖI JSON snake_case.
    let pieces = ledger["pieces"].as_array().expect("pieces la mot mang");
    let kinds: Vec<&str> = pieces.iter().map(|p| p["kind"].as_str().expect("kind la chuoi")).collect();
    assert_eq!(
        kinds,
        vec!["authored", "glossary", "authored", "source_segment"],
        "neu #[serde(rename_all = \"snake_case\")] bi go khoi PromptPieceKindWire, cac chuoi nay \
         tro thanh \"Authored\"/\"Glossary\"/\"SourceSegment\" -- webview's isPromptPieceKindWire \
         se tu choi TOAN BO ban ghi, xem finding P3"
    );
    let piece_keys: std::collections::BTreeSet<&str> =
        pieces[0].as_object().expect("mot piece la mot object").keys().map(String::as_str).collect();
    assert_eq!(piece_keys, std::collections::BTreeSet::from(["kind", "text"]));

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// [`PromptPieceKindWire::Tm`] -- KHÔNG một lệnh gọi thật nào trong story này sản xuất được một
/// mảnh mang nhãn này (xem `ai_rag_contract.rs`'s
/// `pieces_tag_the_source_sentence_with_its_own_kind_separate_from_authored_and_tm_never_produces_a_piece`),
/// nhưng kiểu WIRE vẫn phải serialize đúng chuỗi `"tm"` -- Epic 7 sẽ là caller thật đầu tiên, và
/// khi đó tệp này phải đã pin đúng hình dạng trước khi có dữ liệu thật để pin nó.
#[test]
fn prompt_piece_kind_wire_tm_variant_serializes_to_the_literal_string_tm() {
    let piece = PromptPieceWire { kind: PromptPieceKindWire::Tm, text: "khong toi tu prompt that".to_owned() };
    let json = serde_json::to_value(&piece).expect("PromptPieceWire phai serialize duoc");
    assert_eq!(json, serde_json::json!({ "kind": "tm", "text": "khong toi tu prompt that" }));
}
