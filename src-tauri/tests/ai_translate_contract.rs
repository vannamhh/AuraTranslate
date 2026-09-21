//! Mọi hàng của I/O Matrix của spec 4.8 (Dịch một segment với kết quả chảy dần) — Phase 4.
//!
//! Bốn lớp canh trong MỘT tệp, đúng những gì spec 4.8's Code Map/§Tests that move đặt tên cho
//! tệp này:
//! 1. [`core::ai::client::split_sse_frames`] — hàm THUẦN tách khung SSE, không socket nào
//!    (AD-45), driven với khung chia làm hai khối, một khung tới trong hai lượt đọc, khung
//!    `[DONE]`, và một luồng dừng mà không thấy `[DONE]`.
//! 2. `commands::aitranslate::prepare_translate_call` — hàm THUẦN, một ca cho MỖI hàng ma
//!    trận I/O có thể canh được ở tầng Rust (config chưa xong, khoá vắng/từ chối, segment bị
//!    cắt, Chương/Tác phẩm không mở) — hai hàng còn lại của bảng ("Promote while generating",
//!    "Caret moves during generation") là trạng thái **frontend-only** (không một trường Rust
//!    nào biết "đang generating" — xem doc-comment `AiTranslateOutcomeWire`), nên không có ca
//!    Rust ở đây; `tests/frontend/aiTranslate*.test.ts` (agent khác) sở hữu hai hàng đó.
//! 3. `commands::aitranslate::run_translate_call` — thay một [`TranslationProvider`] GIẢ vào
//!    seam Phase 1 đã khai (`ports::TranslationProvider`, "seam mà `tests/**` thay thật một
//!    provider giả" — spec 4.8 §Never) để canh "token tới đúng thứ tự qua MỘT `Channel`" và
//!    "huỷ giữa chừng: không khung nào gửi thêm" bằng phép ĐO (đếm những gì THẬT SỰ đã gửi),
//!    không bằng một cờ trạng thái.
//! 4. `OpenAiClientError → IpcError` (định nghĩa trong `commands::aitranslate`) và
//!    `core::ai::client::build_request_body` (Quyết định 6) — hai chỗ còn lại của story mà
//!    một lượt gọi mạng thật mới chạm tới bình thường, giờ testable thuần vì
//!    [`build_request_body`] tách khỏi `OpenAiChatClient::translate` đúng cho mục đích này
//!    (xem doc-comment của nó ở `core/ai/client.rs`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO KHÔNG MỘT `TcpListener`/CLIENT MẠNG THẬT NÀO Ở ĐÂY
//! ─────────────────────────────────────────────────────────────────────────────
//! AD-45 cấm một cổng nghe trong bộ test, và một nhị phân test tự mở cổng đã gây đỏ giả trên
//! máy này (LuLu) nhiều lần (root `AGENTS.md`). Đây chính xác là lý do `OpenAiChatClient`
//! (client thật, lái `reqwest` async) KHÔNG có một ca nào ở đây: mọi hành vi của nó tách được
//! thành các hàm THUẦN (`split_sse_frames`, `build_request_body`) hoặc một seam thay-được
//! (`TranslationProvider`, canh qua `run_translate_call` + [`FakeProvider`]) — đúng lý lẽ
//! Design Notes spec 4.8's "Why the SSE parser is a pure function".
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO KEYCHAIN MOCK, KHÔNG KEYCHAIN THẬT
//! ─────────────────────────────────────────────────────────────────────────────
//! Cùng khuôn `aiconfig_contract.rs`: `keychain::read()` gọi thẳng OS keychain qua crate
//! `keyring`, và `core/aiconfig/keychain.rs` không có tham số nào để tráo — thay bằng cách cài
//! `keyring_core::mock::Store` MỘT LẦN cho cả nhị phân (`install_mock_keychain_store_once`,
//! sao chép NGUYÊN VĂN từ `aiconfig_contract.rs` — hai hằng `KEYCHAIN_SERVICE`/
//! `KEYCHAIN_ACCOUNT` không `pub` ở nguồn, một crate test KHÁC không với tới được). Mọi ca
//! CHẠM/KHẲNG ĐỊNH trạng thái khoá giữ [`KEYCHAIN_KEY_TEST_LOCK`] xuyên suốt đời ca — kho chỉ
//! có ĐÚNG MỘT credential dùng chung, và `cargo test` chạy song song theo mặc định.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Once};

use auratranslate_lib::commands::aiconfig::{ai_config_delete_key, ai_config_save_key};
use auratranslate_lib::commands::aiprompt::{
    LastAssembledPromptState, assemble_and_record_prompt, mark_prompt_as_sent,
    read_last_assembled_prompt,
};
use auratranslate_lib::commands::aitranslate::{
    PrepareOutcome, PreparedTranslateCall, prepare_translate_call, run_translate_call,
};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::promptset::prompt_set_create;
use auratranslate_lib::commands::segment::{
    TRANSLATION_ORIGIN_OTHER, confirm_segment, promote_ai_translation, read_open_chapter_segments,
    set_segment_omitted,
};
use auratranslate_lib::core::ai::client::{
    OpenAiClientError, SseEventOutcome, build_request_body, enforce_sse_buffer_cap,
    interpret_sse_event, split_sse_frames,
};
use auratranslate_lib::core::aiconfig::{AiConfigField, AiConfigTier, write_field};
use auratranslate_lib::core::i18n::{IpcError, MessageKey};
use auratranslate_lib::core::promptset::PromptSetTier;
use auratranslate_lib::core::store::{Store, StoreSpec};
use auratranslate_lib::ports::translation_provider::{
    TranslateOutcome, TranslateRequest, TranslationProvider,
};

// ═════════════════════════════════════════════════════════════════════════════════
// Fixture chung — cùng khuôn `ai_prompt_contract.rs`/`aiconfig_contract.rs`
// ═════════════════════════════════════════════════════════════════════════════════

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-ai-translate-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &std::path::Path) {
    let _ = fs::remove_dir_all(dir);
}

fn open_global(dir: &std::path::Path) -> Store {
    Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db")
}

fn open_work(root: &std::path::Path, tag: &str, lang: &str, text: &str) -> OpenWork {
    create_work_from_text(root, tag, lang, "", text.to_owned())
        .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"))
}

fn fresh_record() -> LastAssembledPromptState {
    std::sync::Mutex::new(None)
}

fn first_segment_id(open: &OpenWork) -> i64 {
    read_open_chapter_segments(Some(open)).expect("nap chuong").segments[0].id
}

/// `PrepareOutcome` không (và không được) `derive(Debug)` -- nó mang [`PreparedTranslateCall`]
/// ở biến thể `Ready`, và kiểu đó cấm `Debug` vì giữ khoá API đã lộ (§Always spec 4.8). Vì vậy
/// `Result::expect_err` (đòi `T: Debug`) không gọi được thẳng trên `Result<PrepareOutcome,
/// IpcError>` -- hàm này khớp tay thay cho nó.
fn expect_prepare_err(result: Result<PrepareOutcome, IpcError>, msg: &str) -> IpcError {
    match result {
        Ok(_) => panic!("{msg}: nhan duoc Ok, khong phai Err"),
        Err(e) => e,
    }
}

/// Xuất xứ đang nằm trên đĩa của một segment — cùng khuôn
/// `segment_contract.rs::read_origin`.
fn read_origin(open: &OpenWork, id: i64) -> String {
    open.store
        .read(move |conn| {
            conn.query_row("SELECT translation_origin FROM segment WHERE id = ?1", [id], |r| r.get(0))
        })
        .expect("doc xuat xu that bai")
}

// ═════════════════════════════════════════════════════════════════════════════════
// Keychain mock — sao chép NGUYÊN VĂN khuôn của `aiconfig_contract.rs` (xem doc-comment đầu
// tệp §VÌ SAO KEYCHAIN MOCK cho lý do không tái dùng được qua `use`).
// ═════════════════════════════════════════════════════════════════════════════════

const KEYCHAIN_SERVICE: &str = "com.auratranslate.desktop";
const KEYCHAIN_ACCOUNT: &str = "ai_provider_api_key";

/// Mọi ca CHẠM/KHẲNG ĐỊNH trạng thái khoá API giữ khoá này xuyên suốt đời ca — kho mock là
/// TOÀN TIẾN TRÌNH và chỉ có ĐÚNG MỘT credential, `cargo test` chạy song song theo mặc định.
static KEYCHAIN_KEY_TEST_LOCK: Mutex<()> = Mutex::new(());

fn install_mock_keychain_store_once() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let real_store_status = keyring::Entry::store_status();
        assert!(
            real_store_status.is_ok(),
            "kho keychain THAT khoi tao that bai ({real_store_status:?}) -- moi ca sau day se \
             cham `NoDefaultStore` thay vi mock"
        );
        keyring_core::set_default_store(
            keyring_core::mock::Store::new().expect("keyring_core::mock::Store::new that bai"),
        );
    });
}

fn inject_one_shot_keychain_error() {
    install_mock_keychain_store_once();
    let entry = keyring_core::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .expect("keyring_core::Entry::new that bai -- mock store chua duoc cai");
    let mock: &keyring_core::mock::Cred = entry
        .as_any()
        .downcast_ref()
        .expect("entry khong phai keyring_core::mock::Cred -- mock chua duoc cai truoc do");
    mock.set_error(keyring_core::Error::Invalid(
        "mock".to_owned(),
        "keychain tu choi tra loi (spec 4.8, ca gia lap that bai)".to_owned(),
    ));
}

fn reset_key_to_not_configured() {
    install_mock_keychain_store_once();
    ai_config_delete_key(AiConfigTier::Global).expect("dat lai trang thai khoa that bai");
}

fn save_key(value: &str) {
    install_mock_keychain_store_once();
    ai_config_save_key(AiConfigTier::Global, value).expect("luu khoa API that bai");
}

// ═════════════════════════════════════════════════════════════════════════════════
// 1. core::ai::client::split_sse_frames — hàm THUẦN, không socket nào
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn split_sse_frames_splits_two_complete_events_on_a_blank_line() {
    let buf = b"data: hello\n\ndata: world\n\n";
    let (events, remaining) = split_sse_frames(buf);
    assert_eq!(events, vec!["hello".to_owned(), "world".to_owned()]);
    assert!(remaining.is_empty());
}

/// Ca đối chứng "một khung chia làm hai khối mạng" (spec 4.8, Phase 4 task) — chỗ gọi tích
/// luỹ phần dư của lượt trước vào khối mới rồi gọi lại hàm trên TOÀN BỘ bộ đệm đã gộp.
#[test]
fn split_sse_frames_reassembles_a_frame_split_across_two_chunks() {
    let chunk1 = b"data: hel".to_vec();
    let (events1, remaining1) = split_sse_frames(&chunk1);
    assert!(events1.is_empty(), "chua du mot su kien tron ven -- 0 su kien");
    assert_eq!(remaining1, chunk1, "phan con lai phai la TOAN BO khoi dau, chua mat byte nao");

    let mut buf2 = remaining1;
    buf2.extend_from_slice(b"lo\n\n");
    let (events2, remaining2) = split_sse_frames(&buf2);
    assert_eq!(
        events2,
        vec!["hello".to_owned()],
        "khung chia lam hai khoi mang van phai duoc nhan dung o lan doc ke tiep"
    );
    assert!(remaining2.is_empty());
}

#[test]
fn split_sse_frames_returns_the_done_terminator_as_an_ordinary_event_payload() {
    let buf = b"data: [DONE]\n\n";
    let (events, remaining) = split_sse_frames(buf);
    assert_eq!(
        events,
        vec!["[DONE]".to_owned()],
        "split_sse_frames tra payload NGUYEN VAN -- nhan dien `[DONE]` la viec cua cho goi \
         (OpenAiChatClient::translate), khong phai cua ham thuan nay"
    );
    assert!(remaining.is_empty());
}

#[test]
fn split_sse_frames_handles_crlf_event_delimiters() {
    let buf = b"data: hello\r\n\r\ndata: world\r\n\r\n";
    let (events, remaining) = split_sse_frames(buf);
    assert_eq!(events, vec!["hello".to_owned(), "world".to_owned()]);
    assert!(remaining.is_empty());
}

#[test]
fn split_sse_frames_joins_multiple_data_lines_within_one_event_with_a_newline() {
    let buf = b"data: line one\ndata: line two\n\n";
    let (events, remaining) = split_sse_frames(buf);
    assert_eq!(events, vec!["line one\nline two".to_owned()]);
    assert!(remaining.is_empty());
}

/// Mô phỏng đúng TRẠNG THÁI ngay trước một kết nối ĐÓNG mà không thấy `[DONE]` (I/O Matrix
/// "Stream ends without `[DONE]`") — không một sự kiện TRỌN VẸN nào, toàn bộ vẫn là phần dư.
/// Việc BIẾN trạng thái này thành `OpenAiClientError::StreamEndedWithoutDone` chỉ xảy ra bên
/// trong `OpenAiChatClient::translate` (đọc thấy `chunk() == None`), không testable ở đây mà
/// không phạm AD-45 — mapping của biến thể đó sang `IpcError` được canh riêng ở §4 dưới.
#[test]
fn split_sse_frames_leaves_an_incomplete_trailing_event_as_remaining_bytes() {
    let buf = b"data: partial answer";
    let (events, remaining) = split_sse_frames(buf);
    assert!(events.is_empty(), "chua co dau ngan cach dong trang -- 0 su kien TRON VEN");
    assert_eq!(remaining, buf, "toan bo buffer o lai cho lan doc ke tiep -- khong byte nao mat");
}

/// Đối chứng review — một payload `data:` RỖNG (nhịp giữ kết nối một số nhà cung cấp gửi giữa
/// các token thật) bị BỎ QUA, không kết thúc luồng và không đi qua `parse_chunk_content` (chuỗi
/// rỗng không phải JSON hợp lệ, trước bản sửa này biến thành `MalformedEvent` và huỷ ngang một
/// lượt gọi đã trả tiền). Cả hai hình dạng thật của một `data:` rỗng — `"data:"` và `"data: "` —
/// đều phải tách thành đúng MỘT sự kiện qua `split_sse_frames`, rồi `interpret_sse_event` bỏ
/// qua nó.
#[test]
fn an_empty_data_frame_is_ignored_rather_than_ending_the_stream() {
    for buf in [&b"data:\n\n"[..], &b"data: \n\n"[..]] {
        let (events, remaining) = split_sse_frames(buf);
        assert_eq!(events, vec![String::new()], "mot data: rong van la MOT su kien, khong bi nuot");
        assert!(remaining.is_empty());

        let outcome = interpret_sse_event(&events[0]).expect("payload rong khong duoc la mot Err");
        assert_eq!(
            outcome,
            SseEventOutcome::Ignore,
            "payload rong phai bi BO QUA, khong phai Done va khong phai mot loi MalformedEvent"
        );
    }
}

/// Ca ÂM cạnh ca trên — một khung CHỈ-CHÚ-THÍCH (`": ping"`, không mang dòng `data:` nào) không
/// được đi tới `interpret_sse_event` bằng đường khác: `split_sse_frames` không tạo sự kiện nào
/// cho nó (hành vi đã đúng từ trước, không đụng ở review này).
#[test]
fn a_comment_only_frame_produces_no_event_at_all() {
    let buf = b": ping\n\n";
    let (events, remaining) = split_sse_frames(buf);
    assert!(events.is_empty(), "khung chi-chu-thich khong duoc tao mot su kien nao");
    assert!(remaining.is_empty());
}

/// Đối chứng review — bộ đệm tích luỹ CHƯA-ĐỦ-một-sự-kiện bị từ chối một khi vượt
/// [`enforce_sse_buffer_cap`]'s trần, thay vì phình vô hạn khi một endpoint sai hình/ác ý không
/// bao giờ gửi dấu phân cách `\n\n`/`\r\n\r\n`.
#[test]
fn a_buffer_that_never_sees_a_boundary_is_rejected_once_it_crosses_the_cap() {
    let just_under = vec![b'x'; 1024 * 1024];
    assert!(enforce_sse_buffer_cap(&just_under).is_ok(), "dung tran -- chua duoc tu choi");

    let just_over = vec![b'x'; 1024 * 1024 + 1];
    let err = enforce_sse_buffer_cap(&just_over).expect_err("vuot tran phai la mot Err");
    assert_eq!(err, OpenAiClientError::BufferOverflow { size: just_over.len() });
}

// ═════════════════════════════════════════════════════════════════════════════════
// 2. commands::aitranslate::prepare_translate_call — hàm THUẦN, một ca / hàng ma trận
// ═════════════════════════════════════════════════════════════════════════════════

/// AC1 — request mang ĐÚNG chuỗi 4.7's producer đã ghi, từng byte.
#[test]
fn translate_a_segment_sends_the_string_the_producer_recorded_byte_for_byte() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-ac1-test-key");

    let global_dir = temp_dir("ac1-global");
    let work_dir = temp_dir("ac1-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "AC1", "en", "A dragon roared.");

    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "Plain", "{{source_segment}}")
        .expect("tao bo prompt");

    let segment_id = first_segment_id(&open);
    let record = fresh_record();

    let outcome = prepare_translate_call(Some(&global), Some(&open), &record, Some("Plain"), segment_id)
        .expect("prepare khong duoc loi");
    let prepared = match outcome {
        PrepareOutcome::Ready(p) => p,
        PrepareOutcome::NotConfigured => panic!("phai san sang, khong phai chua cau hinh"),
    };

    let recorded = read_last_assembled_prompt(&record).expect("phai co ban ghi sau prepare");
    assert_eq!(
        prepared.prompt, recorded.prompt,
        "AC1: request phai mang DUNG chuoi 4.7's producer da ghi, tung byte -- khong lap lai"
    );
    assert_eq!(prepared.endpoint, "https://api.example.invalid/v1/chat/completions");
    assert_eq!(prepared.model, "gpt-test");
    assert_eq!(prepared.api_key, "sk-ac1-test-key");

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// I/O Matrix "Provider not configured" — `endpoint`/`model` rỗng ⇒ `NotConfigured`, không
/// một `Err`/`IpcError` nào. Không cần keychain: nhánh này trả về TRƯỚC khi chạm khoá, nên
/// "zero network calls" là đúng CẤU TRÚC — `prepare_translate_call` không hề gọi tới
/// `TranslationProvider`, cổng đó chỉ được thấy sau `Ready`.
#[test]
fn provider_not_configured_when_endpoint_or_model_is_empty_runs_zero_network_calls() {
    let global_dir = temp_dir("not-configured-empty-global");
    let work_dir = temp_dir("not-configured-empty-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "NotConfiguredEmpty", "en", "A dragon roared.");
    let segment_id = first_segment_id(&open);
    let record = fresh_record();

    // Không trường nào được ghi -- `endpoint`/`model` đọc về rỗng ở CẢ HAI tầng.
    let outcome = prepare_translate_call(Some(&global), Some(&open), &record, Some("Plain"), segment_id)
        .expect("khong duoc la mot Err -- 'chua cau hinh' la mot TRANG THAI");
    assert!(matches!(outcome, PrepareOutcome::NotConfigured));
    assert!(
        read_last_assembled_prompt(&record).is_none(),
        "producer cua 4.7 KHONG duoc goi khi chua cau hinh -- khong ban ghi nao duoc tao"
    );

    // Nua ca thu hai -- endpoint co, model rong: cung mot trang thai, khong phai mot Err rieng.
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    let outcome2 = prepare_translate_call(Some(&global), Some(&open), &record, Some("Plain"), segment_id)
        .expect("khong duoc la mot Err");
    assert!(matches!(outcome2, PrepareOutcome::NotConfigured), "model van rong -- van chua cau hinh");

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// I/O Matrix "Provider not configured" — `key_configured == Some(false)` (endpoint/model đủ,
/// khoá chưa từng lưu) đọc CÙNG trạng thái `NotConfigured` như thiếu endpoint/model, không một
/// nhánh lỗi riêng.
#[test]
fn provider_not_configured_when_no_key_is_saved() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();

    let global_dir = temp_dir("not-configured-no-key-global");
    let work_dir = temp_dir("not-configured-no-key-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "NotConfiguredNoKey", "en", "A dragon roared.");
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");

    let segment_id = first_segment_id(&open);
    let record = fresh_record();
    let outcome = prepare_translate_call(Some(&global), Some(&open), &record, Some("Plain"), segment_id)
        .expect("khong duoc la mot Err");
    assert!(matches!(outcome, PrepareOutcome::NotConfigured));

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// I/O Matrix "Keychain refuses to answer" — trạng thái `error`, tái dùng ĐÚNG khoá
/// `AiConfigKeychainUnavailable` của Story 4.3, không một khoá thứ hai cho cùng sự thật.
#[test]
fn keychain_refusing_to_answer_surfaces_the_existing_ai_config_keychain_unavailable_key() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();

    let global_dir = temp_dir("keychain-unavailable-global");
    let work_dir = temp_dir("keychain-unavailable-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "KeychainUnavailable", "en", "A dragon roared.");
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");

    inject_one_shot_keychain_error();

    let segment_id = first_segment_id(&open);
    let record = fresh_record();
    let err = expect_prepare_err(
        prepare_translate_call(Some(&global), Some(&open), &record, Some("Plain"), segment_id),
        "keychain tu choi tra loi phai la mot Err",
    );

    assert_eq!(err.message_key(), MessageKey::AiConfigKeychainUnavailable);
    assert!(err.retryable(), "mot keychain bi khoa/tu choi quyen co the thanh cong o luot bam lai");
    assert!(
        err.params().values().all(|v| v != "sk-decoy" && !v.contains("Bearer")),
        "gia tri khoa KHONG BAO GIO duoc lot vao tham so loi"
    );

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// I/O Matrix "Segment cut from the translation" — từ chối TRƯỚC khi request được dựng
/// (`§Always` spec 4.8), named message key, `retryable: false`. Đối chứng "trước khi dựng"
/// bằng phép ĐO: không bản ghi nào của 4.7's producer được tạo.
#[test]
fn segment_cut_from_translation_is_refused_before_the_request_is_built() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-omitted-test-key");

    let global_dir = temp_dir("omitted-global");
    let work_dir = temp_dir("omitted-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Omitted", "en", "A dragon roared.");
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "Plain", "{{source_segment}}")
        .expect("tao bo prompt");

    let segment_id = first_segment_id(&open);
    set_segment_omitted(Some(&open), segment_id, true).expect("dat co cat bo that bai");

    let record = fresh_record();
    let err = expect_prepare_err(
        prepare_translate_call(Some(&global), Some(&open), &record, Some("Plain"), segment_id),
        "segment is_omitted phai bi tu choi",
    );

    assert_eq!(err.code(), "ai_translate.segment_omitted");
    assert_eq!(err.message_key(), MessageKey::AiTranslateSegmentOmitted);
    assert_eq!(err.params().get("segment_id"), Some(&segment_id.to_string()));
    assert!(!err.retryable());
    assert!(
        read_last_assembled_prompt(&record).is_none(),
        "TU CHOI TRUOC khi request duoc dung -- producer cua 4.7 khong duoc goi"
    );

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// I/O Matrix "No Work open" — tái dùng ĐÚNG khoá `WorkNoneOpen`. Không cần cấu hình/khoá gì
/// -- nhánh này trả về TRƯỚC khi phân giải cấu hình hai tầng.
#[test]
fn no_work_open_reuses_the_existing_work_none_open_key() {
    let global_dir = temp_dir("no-work-open-global");
    let global = open_global(&global_dir);
    let record = fresh_record();

    let err = expect_prepare_err(
        prepare_translate_call(Some(&global), None, &record, Some("Plain"), 1),
        "khong Tac pham nao dang mo phai la mot Err",
    );
    assert_eq!(err.code(), "work.none_open");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);
    assert!(!err.retryable());

    drop(global);
    cleanup(&global_dir);
}

/// I/O Matrix "segment not in chapter" — tái dùng ĐÚNG khoá `AiPromptSegmentNotInChapter`
/// (cùng hàm `commands::aiprompt::segment_not_in_chapter` mà 4.7's producer tự dùng).
#[test]
fn segment_not_in_chapter_reuses_the_existing_ai_prompt_segment_not_in_chapter_key() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-not-in-chapter-test-key");

    let global_dir = temp_dir("not-in-chapter-global");
    let work_dir = temp_dir("not-in-chapter-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "NotInChapter", "en", "A dragon roared.");
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "Plain", "{{source_segment}}")
        .expect("tao bo prompt");

    let real_id = first_segment_id(&open);
    let bogus_id = real_id + 999_999;
    let record = fresh_record();
    let err = expect_prepare_err(
        prepare_translate_call(Some(&global), Some(&open), &record, Some("Plain"), bogus_id),
        "mot id khong thuoc Chuong dang mo phai la mot Err",
    );

    assert_eq!(err.code(), "ai_prompt.segment_not_in_chapter");
    assert_eq!(err.message_key(), MessageKey::AiPromptSegmentNotInChapter);
    assert_eq!(err.params().get("segment_id"), Some(&bogus_id.to_string()));
    assert!(!err.retryable());

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// Quyết định 6 — một `temperature`/`max_tokens` CHƯA ĐẶT không phải lý do gọi provider "chưa
/// cấu hình". Ma trận I/O nêu tên đúng `endpoint`/`model`/khoá cho hàng đó và không gì khác.
#[test]
fn a_blank_temperature_and_max_tokens_does_not_yield_not_configured() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-decision6-test-key");

    let global_dir = temp_dir("decision6-global");
    let work_dir = temp_dir("decision6-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Decision6", "en", "A dragon roared.");
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    // Temperature/MaxTokens CÓ Ý không ghi gì -- đúng trạng thái "chưa đặt" thật, cùng
    // `aiConfigState.ts:69`'s giá trị khởi tạo rỗng.
    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "Plain", "{{source_segment}}")
        .expect("tao bo prompt");

    let segment_id = first_segment_id(&open);
    let record = fresh_record();
    let outcome = prepare_translate_call(Some(&global), Some(&open), &record, Some("Plain"), segment_id)
        .expect("prepare khong duoc loi");
    match outcome {
        PrepareOutcome::Ready(p) => {
            assert_eq!(p.temperature, None, "chua dat phai la None, khong mot so duc san thay ho");
            assert_eq!(p.max_tokens, None);
        }
        PrepareOutcome::NotConfigured => panic!(
            "Quyet dinh 6: mot temperature/max_tokens CHUA DAT khong duoc la ly do bao \
             'chua cau hinh' -- endpoint/model/khoa da du"
        ),
    }

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Quyết định 2 — hai producer, MỘT hàm ghi: assemble-only và translate phải viết một bản ghi
// GIỐNG HỆT nhau cho cùng đầu vào, chỉ khác ở các sự thật ĐÃ GỬI.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn assemble_only_and_translate_write_an_identical_record_differing_only_in_the_sent_facts() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-two-producer-test-key");

    let global_dir = temp_dir("two-producer-global");
    let work_dir = temp_dir("two-producer-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "TwoProducer", "en", "A dragon roared.");
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "Plain", "{{source_segment}}")
        .expect("tao bo prompt");
    let segment_id = first_segment_id(&open);

    // San pham 1 -- CHI lap rap ("Xem prompt", khong mang su that da GUI).
    let record_assemble_only = fresh_record();
    let wire_assemble_only = assemble_and_record_prompt(
        Some(&global),
        Some(&open),
        &record_assemble_only,
        Some("Plain"),
        segment_id,
    )
    .expect("assemble-only khong duoc loi");

    // San pham 2 -- duong DICH: `prepare_translate_call` GOI LAI dung mot producer do, roi mo
    // phong mot luot GUI THANH CONG bang `mark_prompt_as_sent` (dung viec `wire::
    // ai_translate_segment` that su lam SAU khi mot lan gui tra `Done` -- tang lenh Tauri
    // ngoai pham vi test nay, canh o §3 qua `run_translate_call` + `FakeProvider`).
    let record_translate = fresh_record();
    let prepared = match prepare_translate_call(
        Some(&global),
        Some(&open),
        &record_translate,
        Some("Plain"),
        segment_id,
    )
    .expect("prepare khong duoc loi")
    {
        PrepareOutcome::Ready(p) => p,
        PrepareOutcome::NotConfigured => panic!("phai san sang"),
    };
    mark_prompt_as_sent(
        &record_translate,
        segment_id,
        &prepared.prompt,
        "gpt-test",
        "2026-09-21T00:00:00.000Z",
    );
    let wire_translate = read_last_assembled_prompt(&record_translate).expect("phai co ban ghi");

    // Giống hệt -- CHỈ một hàm ghi record (Quyết định 2).
    assert_eq!(prepared.prompt, wire_assemble_only.prompt, "ca hai duong phai lap DUNG mot prompt");
    assert_eq!(wire_translate.prompt, wire_assemble_only.prompt);
    assert_eq!(wire_translate.segment_id, wire_assemble_only.segment_id);
    assert_eq!(wire_translate.chapter_id, wire_assemble_only.chapter_id);
    assert_eq!(wire_translate.prompt_set_name, wire_assemble_only.prompt_set_name);
    assert_eq!(
        serde_json::to_value(&wire_translate.ledger).expect("serialize ledger 1"),
        serde_json::to_value(&wire_assemble_only.ledger).expect("serialize ledger 2"),
        "ledger phai giong het nhau -- khong mot duong nao lap lai/thay bien"
    );

    // Khác biệt DUY NHẤT được phép: các sự thật ĐÃ GỬI.
    assert_eq!(wire_assemble_only.sent_at, None, "assemble-only KHONG BAO GIO la mot luot gui");
    assert_eq!(wire_assemble_only.sent_model, None);
    assert!(wire_translate.sent_at.is_some(), "duong dich, sau khi gui thanh cong, phai co sent_at");
    assert_eq!(wire_translate.sent_model.as_deref(), Some("gpt-test"));

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// **Rà soát — `mark_prompt_as_sent` phải so cả `prompt`, không chỉ `segment_id`.** Nút "Xem
/// prompt" (FR71) không bị khoá bởi trạng thái dịch: người dùng bấm dịch RỒI bấm lắp lại prompt
/// cho ĐÚNG segment đó (đổi bộ prompt hiệu lực) trước khi lượt dịch cũ trả lời được. Bản ghi bị
/// GHI ĐÈ bởi lượt lắp ráp MỚI (cùng `segment_id`, `prompt` khác) — khi lượt dịch CŨ sau đó
/// `Done`, dấu "đã gửi" không được đóng lên bản ghi MỚI mà người dùng chỉ mới lắp ráp để soi,
/// chưa từng gửi.
#[test]
fn a_record_replaced_mid_flight_by_a_newer_assemble_is_left_unstamped() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-mid-flight-test-key");

    let global_dir = temp_dir("mid-flight-global");
    let work_dir = temp_dir("mid-flight-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "MidFlight", "en", "A dragon roared.");
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "First", "{{source_segment}}")
        .expect("tao bo prompt thu nhat");
    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "Second",
        "Da doi bo: {{source_segment}}",
    )
    .expect("tao bo prompt thu hai");
    let segment_id = first_segment_id(&open);

    let record = fresh_record();

    // Lượt 1 -- bấm dịch: `prepare_translate_call` ghi bản ghi bằng bộ "First" và giữ ĐÚNG
    // chuỗi sắp gửi trong `prepared.prompt`.
    let prepared = match prepare_translate_call(Some(&global), Some(&open), &record, Some("First"), segment_id)
        .expect("prepare khong duoc loi")
    {
        PrepareOutcome::Ready(p) => p,
        PrepareOutcome::NotConfigured => panic!("phai san sang"),
    };
    let sent_prompt = prepared.prompt.clone();

    // Lượt 2 -- TRONG LÚC lượt dịch (lượt 1) còn treo, người dùng lắp lại prompt cho CÙNG
    // segment với bộ "Second" -- `assemble_and_record_prompt` GHI ĐÈ đúng bản ghi đó.
    let replaced =
        assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("Second"), segment_id)
            .expect("lap rap lai khong duoc loi");
    assert_ne!(replaced.prompt, sent_prompt, "fixture phai tao ra hai prompt KHAC nhau");

    // Lượt 1 cuối cùng trả `Done` -- chỗ gọi thật (`wire::ai_translate_segment`) truyền đúng
    // chuỗi ĐÃ GỬI của lượt 1, không đọc lại `record` để lấy chuỗi "hiện có".
    mark_prompt_as_sent(&record, segment_id, &sent_prompt, "gpt-test", "2026-09-21T00:00:00.000Z");

    let wire = read_last_assembled_prompt(&record).expect("phai co ban ghi");
    assert_eq!(
        wire.prompt, replaced.prompt,
        "ban ghi phai giu NGUYEN prompt cua lot lap rap MOI NHAT, khong bi luot stamp cham vao"
    );
    assert_eq!(
        wire.sent_at, None,
        "mot ban ghi da bi GHI DE boi mot lot lap rap moi hon khong duoc dong dau 'da gui' -- no \
         chua tung duoc gui, du cung segment_id voi lot da gui that"
    );
    assert_eq!(wire.sent_model, None);

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// 3. commands::aitranslate::run_translate_call + FakeProvider — seam mà tests/** thay được
// ═════════════════════════════════════════════════════════════════════════════════

/// Lỗi giả — không mang `TranslateRequest`/khoá API, cùng khuôn `OpenAiClientError` (xem
/// doc-comment đầu `core/ai/client.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct FakeProviderError(String);

impl std::fmt::Display for FakeProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fake_provider[{}]", self.0)
    }
}

impl std::error::Error for FakeProviderError {}

enum FakeFinish {
    Done,
    Err(FakeProviderError),
}

/// `TranslationProvider` GIẢ — thay vào seam Phase 1 đã khai (spec 4.8 §Never: "The port
/// trait is the seam tests substitute at"). Gửi các token đã cho theo THỨ TỰ, hỏi
/// `should_cancel` GIỮA MỖI token (đúng hợp đồng doc-comment `TranslationProvider::translate`),
/// và trả kết thúc đã cấu hình sẵn nếu không bị huỷ giữa chừng.
struct FakeProvider {
    tokens: Vec<&'static str>,
    finish: FakeFinish,
}

impl TranslationProvider for FakeProvider {
    type Error = FakeProviderError;

    async fn translate(
        &self,
        _request: TranslateRequest<'_>,
        on_token: &mut dyn FnMut(&str),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<TranslateOutcome, Self::Error> {
        for token in &self.tokens {
            if should_cancel() {
                return Ok(TranslateOutcome::Cancelled);
            }
            on_token(token);
        }
        if should_cancel() {
            return Ok(TranslateOutcome::Cancelled);
        }
        match &self.finish {
            FakeFinish::Done => Ok(TranslateOutcome::Done),
            FakeFinish::Err(e) => Err(e.clone()),
        }
    }
}

fn dummy_prepared() -> PreparedTranslateCall {
    PreparedTranslateCall {
        endpoint: "https://api.example.invalid/v1/chat/completions".to_owned(),
        model: "gpt-test".to_owned(),
        temperature: None,
        max_tokens: None,
        api_key: "sk-fake-provider-test".to_owned(),
        prompt: "Hello world.".to_owned(),
    }
}

/// Dựng một `Channel<String>` thật (`tauri::ipc::Channel::new`, không `Runtime`/`AppHandle` —
/// spec 4.8 §Code Map: "The Channel is testable") thu lại đúng chuỗi mà `channel.send(...)` đã
/// gửi, giải mã NGƯỢC khỏi `InvokeResponseBody::Json` (thứ `send()` thật sự tạo ra, chính là
/// điều sẽ chạy trên dây).
fn collecting_channel() -> (tauri::ipc::Channel<String>, Arc<Mutex<Vec<String>>>) {
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_in = Arc::clone(&received);
    let channel = tauri::ipc::Channel::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            let text: String = serde_json::from_str(&json).unwrap_or_default();
            received_in.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(text);
        }
        Ok(())
    });
    (channel, received)
}

/// I/O Matrix "Translate a segment" (nửa "tokens arrive ... render as they land") + AC2 —
/// token tới ĐÚNG THỨ TỰ, mỗi token một lần gửi `Channel`, trạng thái kết thúc `Done`.
#[test]
fn tokens_arrive_on_the_channel_in_order_as_they_land_and_the_call_ends_done() {
    let provider = FakeProvider { tokens: vec!["Xin ", "chào"], finish: FakeFinish::Done };
    let prepared = dummy_prepared();
    let (channel, received) = collecting_channel();
    let should_cancel = || false;

    let outcome =
        tauri::async_runtime::block_on(run_translate_call(&provider, &prepared, &channel, &should_cancel))
            .expect("khong duoc loi");

    assert_eq!(outcome, TranslateOutcome::Done);
    let got = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    assert_eq!(got, vec!["Xin ".to_owned(), "chào".to_owned()], "token phai toi DUNG THU TU");
}

/// I/O Matrix "Cancel mid-stream" — Đối chứng bằng phép ĐO, không bằng cờ: sau khi huỷ, KHÔNG
/// khung nào gửi thêm (§Verification spec 4.8: "assert that no further Channel message
/// arrives, not merely that the state changed").
#[test]
fn cancel_mid_stream_sends_no_further_channel_message_and_reports_cancelled() {
    let provider = FakeProvider { tokens: vec!["one", "two", "three"], finish: FakeFinish::Done };
    let prepared = dummy_prepared();
    let (channel, received) = collecting_channel();
    let calls = AtomicUsize::new(0);
    // Huỷ đúng SAU khi đã nhận một token đầu tiên -- "two"/"three" không được phép tới.
    let should_cancel = || calls.fetch_add(1, Ordering::SeqCst) >= 1;

    let outcome =
        tauri::async_runtime::block_on(run_translate_call(&provider, &prepared, &channel, &should_cancel))
            .expect("huy la Ok(Cancelled), khong phai mot Err");

    assert_eq!(outcome, TranslateOutcome::Cancelled);
    let got = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    assert_eq!(
        got,
        vec!["one".to_owned()],
        "huy giua chung -- KHONG token nao SAU thoi diem huy duoc gui, do bang so dem THAT"
    );
}

/// Một lỗi provider GIỮA CHỪNG (kết nối rớt/`[DONE]` không tới) không xoá ngược token đã
/// nhận — §Always spec 4.8: "A dropped stream ends the call; whatever tokens arrived stay on
/// screen." `run_translate_call` không tự thử lại (AD-22): lỗi truyền NGUYÊN VẸN ra ngoài.
#[test]
fn a_provider_error_after_partial_tokens_leaves_earlier_tokens_visible_and_propagates_the_error() {
    let provider = FakeProvider {
        tokens: vec!["partial "],
        finish: FakeFinish::Err(FakeProviderError("dropped".to_owned())),
    };
    let prepared = dummy_prepared();
    let (channel, received) = collecting_channel();
    let should_cancel = || false;

    let err =
        tauri::async_runtime::block_on(run_translate_call(&provider, &prepared, &channel, &should_cancel))
            .expect_err("mot loi provider phai truyen nguyen qua run_translate_call");

    assert_eq!(err, FakeProviderError("dropped".to_owned()));
    let got = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    assert_eq!(
        got,
        vec!["partial ".to_owned()],
        "token da nhan TRUOC loi phai o lai tren Channel -- khong bi xoa nguoc"
    );
}

/// AC2's nửa "grep tìm 0 `emit`/`listen`" — canh bằng chính văn bản nguồn của tầng lệnh, đúng
/// khuôn các gate quét-mã-nguồn khác của kho (`ai_boundary.rs`).
#[test]
fn the_command_file_carries_zero_emit_or_listen_calls_on_the_translate_path() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("commands").join("aitranslate.rs");
    let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("khong doc duoc {}: {e}", path.display()));
    assert!(
        !src.contains(".emit("),
        "AC2: `commands/aitranslate.rs` khong duoc goi `.emit(` -- AD-22 doi streaming di qua \
         MOT Channel, khong su kien roi"
    );
    assert!(
        !src.contains(".listen("),
        "AC2: `commands/aitranslate.rs` khong duoc goi `.listen(` -- cung ly do tren"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// 4a. OpenAiClientError → IpcError — I/O Matrix "non-2xx" / "Stream ends without [DONE]"
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix "Provider returns a non-2xx" — `IpcError` mang mã trạng thái, KHÔNG thử lại.
#[test]
fn provider_returns_a_non_2xx_maps_to_a_non_retryable_ipc_error_naming_the_status() {
    let err: IpcError = OpenAiClientError::NonSuccessStatus { status: 401 }.into();
    assert_eq!(err.code(), "ai_translate.provider_call_failed");
    assert_eq!(err.message_key(), MessageKey::AiTranslateProviderCallFailed);
    assert_eq!(err.params().get("status"), Some(&"401".to_owned()));
    assert!(!err.retryable(), "non-2xx khong duoc thu lai");
}

/// I/O Matrix "Stream ends without [DONE]" — `IpcError` retryable, KHÔNG tự động thử lại ở
/// bất kỳ tầng nào (đó là quyết định của chỗ gọi/người dùng bấm lại, không phải cơ chế tự
/// động — AD-22).
#[test]
fn stream_ended_without_done_maps_to_a_retryable_ipc_error() {
    let err: IpcError = OpenAiClientError::StreamEndedWithoutDone.into();
    assert_eq!(err.message_key(), MessageKey::AiTranslateProviderCallFailed);
    assert!(err.params().is_empty());
    assert!(err.retryable(), "rot ket noi GIUA CHUNG co the thu lai o luot bam lai");
}

/// Đối chứng ĐỦ SÁU biến thể — khoá cả bảng `match` của `impl From<OpenAiClientError> for
/// IpcError` lại một lần, để một biến thể mới thêm vào tương lai không lặng lẽ rơi vào nhánh
/// mặc định sai.
#[test]
fn every_openai_client_error_variant_maps_to_the_provider_call_failed_key_with_the_documented_retryable_flag()
 {
    let cases: [(OpenAiClientError, bool); 7] = [
        (OpenAiClientError::ClientBuildFailed { detail: "x".to_owned() }, false),
        (OpenAiClientError::RequestFailed { detail: "x".to_owned() }, true),
        (OpenAiClientError::NonSuccessStatus { status: 500 }, false),
        (OpenAiClientError::ReadFailed { detail: "x".to_owned() }, true),
        (OpenAiClientError::StreamEndedWithoutDone, true),
        (OpenAiClientError::MalformedEvent { detail: "x".to_owned() }, false),
        (OpenAiClientError::BufferOverflow { size: 2 * 1024 * 1024 }, false),
    ];
    for (variant, expected_retryable) in cases {
        let label = format!("{variant:?}");
        let err: IpcError = variant.into();
        assert_eq!(err.message_key(), MessageKey::AiTranslateProviderCallFailed, "{label}");
        assert_eq!(err.retryable(), expected_retryable, "{label} phai anh xa retryable={expected_retryable}");
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// 4b. Quyết định 6 — core::ai::client::build_request_body: `skip_serializing_if` là THẬT
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_request_body_omits_temperature_and_max_tokens_when_unset() {
    let request = TranslateRequest {
        endpoint: "https://api.example.invalid/v1/chat/completions",
        model: "gpt-test",
        temperature: None,
        max_tokens: None,
        api_key: "sk-test",
        prompt: "Hello.",
    };
    let json = serde_json::to_string(&build_request_body(&request)).expect("serialize");
    assert!(
        !json.contains("temperature"),
        "Quyet dinh 6: truong nay phai VANG MAT (khong ca 'null') khi None: {json}"
    );
    assert!(!json.contains("max_tokens"), "Quyet dinh 6: truong nay phai VANG MAT khi None: {json}");
    assert!(json.contains("\"stream\":true"), "{json}");
}

#[test]
fn the_request_body_includes_temperature_and_max_tokens_when_set() {
    let request = TranslateRequest {
        endpoint: "https://api.example.invalid/v1/chat/completions",
        model: "gpt-test",
        temperature: Some(0.7),
        max_tokens: Some(512),
        api_key: "sk-test",
        prompt: "Hello.",
    };
    let json = serde_json::to_string(&build_request_body(&request)).expect("serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse lai");
    assert_eq!(value["temperature"], serde_json::json!(0.7));
    assert_eq!(value["max_tokens"], serde_json::json!(512));
}

/// AD-14 tại điểm ra dây cuối cùng — `content` mang ĐÚNG `request.prompt`, không lắp lại.
#[test]
fn the_request_body_carries_the_prompt_verbatim_as_the_user_message_content() {
    let request = TranslateRequest {
        endpoint: "https://api.example.invalid/v1/chat/completions",
        model: "gpt-test",
        temperature: None,
        max_tokens: None,
        api_key: "sk-test",
        prompt: "Dịch câu này giúp tôi.",
    };
    let json = serde_json::to_string(&build_request_body(&request)).expect("serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse lai");
    assert_eq!(value["messages"][0]["content"], "Dịch câu này giúp tôi.");
    assert_eq!(value["messages"][0]["role"], "user");
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC4 — promote ghi CẢ `target_text` LẪN `translation_origin = other` trong MỘT thao tác;
// xác nhận SAU đó không sửa một chữ giữ nguyên `other`.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn promote_writes_target_text_and_origin_other_in_one_operation_and_confirm_without_an_edit_keeps_it()
 {
    let work_dir = temp_dir("promote-ac4-work");
    let open = open_work(&work_dir, "PromoteAC4", "en", "A dragon roared.");
    let segment_id = first_segment_id(&open);

    let outcome = promote_ai_translation(Some(&open), segment_id, "Con rồng gầm.")
        .expect("promote khong duoc loi");
    assert_eq!(outcome.target_text, "Con rồng gầm.");
    assert_eq!(outcome.translation_origin, TRANSLATION_ORIGIN_OTHER);
    assert_eq!(
        read_origin(&open, segment_id),
        TRANSLATION_ORIGIN_OTHER,
        "AD-47① -- ca hai nua (van ban va xuat xu) phai cung o TRONG mot cau UPDATE"
    );

    // Mốc = văn bản VỪA promote (đúng bởi AD-47①(a): baseline được RESET về text vừa ghi) --
    // xác nhận KHÔNG sửa một chữ phải GIỮ NGUYÊN 'other', không rơi về 'self'. Đếm bằng cách
    // GỠ dòng ghi `translation_origin` khỏi `promote_ai_translation` sẽ làm chính ca này đỏ,
    // đúng counter-check AC4 đòi ("removing the origin write makes a named case go red").
    confirm_segment(Some(&open), segment_id, "Con rồng gầm.").expect("confirm khong duoc loi");
    assert_eq!(
        read_origin(&open, segment_id),
        TRANSLATION_ORIGIN_OTHER,
        "AC4: xac nhan khong sua mot chu phai GIU NGUYEN 'other'"
    );

    drop(open);
    cleanup(&work_dir);
}
