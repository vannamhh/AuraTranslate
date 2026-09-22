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
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Once};

use auratranslate_lib::commands::aiconfig::{ai_config_delete_key, ai_config_save_key};
use auratranslate_lib::commands::aiprompt::{
    LastAssembledPromptState, assemble_and_record_prompt, mark_prompt_as_sent,
    read_last_assembled_prompt,
};
use auratranslate_lib::commands::aitranslate::{
    AiTranslateBatchEventWire, AiTranslateBatchOutcome, PrepareBatchOutcome, PrepareOutcome,
    PreparedBatchItem, PreparedTranslateCall, batch_stopped_error, prepare_batch_call,
    prepare_translate_call, run_batch_call, run_translate_call,
};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::promptset::prompt_set_create;
use auratranslate_lib::commands::segment::{
    TRANSLATION_ORIGIN_OTHER, confirm_segment, promote_ai_translation, read_open_chapter_segments,
    set_segment_omitted,
};
use auratranslate_lib::core::ai::client::{
    ChunkUsage, OpenAiClientError, SseEventOutcome, build_request_body, enforce_sse_buffer_cap,
    interpret_sse_event, split_sse_frames,
};
use auratranslate_lib::core::aiconfig::{AiConfigField, AiConfigTier, write_field};
use auratranslate_lib::core::i18n::{IpcError, MessageKey};
use auratranslate_lib::core::promptset::PromptSetTier;
use auratranslate_lib::core::store::{Store, StoreSpec};
use auratranslate_lib::ports::translation_provider::{
    TranslateOutcome, TranslateRequest, TranslateUsage, TranslationProvider,
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

/// Cùng lý do [`expect_prepare_err`] một dòng ngay trên -- `PrepareBatchOutcome::Ready` mang
/// `PreparedBatchItem::ToTranslate { prepared: PreparedTranslateCall, .. }`, và kiểu đó cũng
/// cấm `Debug`.
fn expect_prepare_batch_err(result: Result<PrepareBatchOutcome, IpcError>, msg: &str) -> IpcError {
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
// 1b. Story 4.11 -- request mang `stream_options`, và khung usage cuối cùng không còn rơi
// vào `SseEventOutcome::Ignore` (bug được nêu tên nguyên văn ở §Code Map spec 4.11:
// "a chunk with choices: [] takes exactly that path today, so adding the struct field alone
// changes nothing observable" -- ca dưới đây là counter-check trực tiếp cho đúng câu đó).
// ═════════════════════════════════════════════════════════════════════════════════

/// Mọi request phải mang `stream_options: { include_usage: true }` -- đây là cách DUY NHẤT
/// provider biết phải gửi một khung `usage` trước `[DONE]` (§Design Notes spec 4.11). Không
/// `Option`/`skip_serializing_if`: trường này LUÔN có mặt, không như `temperature`/`max_tokens`.
#[test]
fn the_request_body_always_asks_for_usage_via_stream_options() {
    let request = TranslateRequest {
        endpoint: "https://api.example.invalid/v1/chat/completions",
        model: "gpt-test",
        temperature: None,
        max_tokens: None,
        api_key: "sk-test",
        prompt: "Hello.",
    };
    let json = serde_json::to_string(&build_request_body(&request)).expect("serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse lai");
    assert_eq!(value["stream_options"]["include_usage"], serde_json::json!(true));
}

/// 🔴 Counter-check trực tiếp cho khuyết tật nêu ở §Code Map spec 4.11 — một khung `usage`
/// mang `choices: []` (hình dạng THẬT của khung usage cuối của một provider tương thích
/// OpenAI, `stream_options.include_usage == true`) phải cho ra [`SseEventOutcome::Usage`],
/// KHÔNG [`SseEventOutcome::Ignore`]. Trước bản sửa của story này, `ChatCompletionsChunk`
/// không đọc trường `usage` nên đúng khung này rơi vào `Ignore` một cách im lặng — gỡ nhánh
/// `if let Some(usage) = chunk.usage { ... }` khỏi `interpret_sse_event` (`core/ai/client.rs`)
/// làm ĐÚNG ca này đỏ (đối chứng bằng tay lúc dựng story, xem báo cáo triển khai).
#[test]
fn a_usage_only_final_chunk_with_empty_choices_is_read_as_usage_not_ignored() {
    let usage_chunk = r#"{"id":"x","object":"chat.completion.chunk","choices":[],"usage":{"prompt_tokens":100,"completion_tokens":312,"total_tokens":412}}"#;
    let outcome = interpret_sse_event(usage_chunk).expect("khung usage hop le khong duoc la Err");
    assert_eq!(
        outcome,
        SseEventOutcome::Usage(ChunkUsage {
            prompt_tokens: Some(100),
            completion_tokens: Some(312),
            total_tokens: Some(412),
        }),
        "khung usage voi choices RONG phai doc thanh Usage, khong phai Ignore"
    );
}

/// 🔴 Rà soát coordinator — một `usage` CÓ MẶT trong JSON nhưng không báo được một số token
/// nào (`{"usage":{}}`, ba trường đều vắng mặt) phải đọc như KHÔNG có khung usage nào tới —
/// KHÔNG một `Usage(ChunkUsage { None, None, None })` mà tầng trên hiểu nhầm thành "đã có usage,
/// mọi số đều 0" (frozen §Always: "never `0`"). Đối chứng bằng cách rơi xuống ĐÚNG nhánh
/// `Ignore` mà một chunk không mang `usage` lẫn `delta.content` sẽ rơi vào.
#[test]
fn a_usage_object_present_but_reporting_no_token_counts_at_all_is_read_as_no_usage() {
    let empty_usage_chunk = r#"{"id":"x","choices":[],"usage":{}}"#;
    let outcome = interpret_sse_event(empty_usage_chunk).expect("khong duoc la Err");
    assert_eq!(
        outcome,
        SseEventOutcome::Ignore,
        "usage rong (khong mot truong nao) phai doc nhu KHONG co usage, khong phai Usage voi so 0"
    );
}

/// 🔴 Rà soát coordinator — một `usage` báo MỘT PHẦN (chỉ `total_tokens`, thiếu
/// `prompt_tokens`/`completion_tokens`) VẪN là một `Usage` thật (số token đó có nghĩa, hiện
/// được) — khác ca `{}` ngay trên. `interpret_sse_event` giữ nguyên hai trường vắng mặt là
/// `None`, không đúc `0` thay — `ChunkUsage::into_translate_usage` (đối chứng riêng ngay dưới)
/// mới là nơi quyết định `cost_usd` có được tính hay không từ hình dạng THIẾU này.
#[test]
fn a_usage_frame_reporting_only_total_tokens_is_read_as_usage_with_the_other_two_fields_none() {
    let partial_usage_chunk = r#"{"id":"x","choices":[],"usage":{"total_tokens":412}}"#;
    let outcome = interpret_sse_event(partial_usage_chunk).expect("khong duoc la Err");
    assert_eq!(
        outcome,
        SseEventOutcome::Usage(ChunkUsage { prompt_tokens: None, completion_tokens: None, total_tokens: Some(412) }),
        "usage MOT PHAN van la Usage that -- total_tokens=412 co nghia, hai truong con lai None"
    );
}

/// 🔴 Rà soát coordinator — `ChunkUsage::into_translate_usage` không được đúc một chi phí BỊA
/// từ hai trường VẮNG MẶT. Cùng khung `{"total_tokens":412}` ở ca ngay trên: token count 412
/// phải lên màn hình (thật), nhưng `cost_usd` PHẢI là `None` (không phải một `Some(0.0)` giả —
/// công thức giá cần CẢ HAI `prompt_tokens`/`completion_tokens`, và ở đây cả hai đều không
/// biết, không phải bằng `0`).
#[test]
fn into_translate_usage_does_not_fabricate_a_cost_when_the_prompt_and_completion_split_is_unknown() {
    let partial = ChunkUsage { prompt_tokens: None, completion_tokens: None, total_tokens: Some(412) };
    let usage = partial.into_translate_usage("claude-sonnet-5");
    assert_eq!(usage.total_tokens, 412, "so token THAT phai len man hinh du thieu chi tiet vao/ra");
    assert_eq!(
        usage.cost_usd, None,
        "khong du CA HAI prompt_tokens/completion_tokens thi KHONG duoc tinh gia -- None, khong phai Some(0.0) gia"
    );
}

/// Ca ĐỐI CHỨNG DƯƠNG cạnh ca trên — khi CẢ HAI `prompt_tokens`/`completion_tokens` đều có
/// mặt (dù `total_tokens` vắng mặt, một hình dạng hợp lệ khác), giá VẪN được tính, và
/// `total_tokens` rơi về đúng tổng hai chiều đã biết.
#[test]
fn into_translate_usage_computes_a_cost_and_falls_back_to_the_sum_when_total_tokens_is_absent_but_both_parts_are_known() {
    let usage = ChunkUsage { prompt_tokens: Some(100), completion_tokens: Some(312), total_tokens: None }
        .into_translate_usage("claude-sonnet-5");
    assert_eq!(usage.total_tokens, 412, "total_tokens vang mat -- roi ve dung tong hai chieu da biet");
    assert!(usage.cost_usd.is_some(), "ca hai chieu deu biet -- gia PHAI duoc tinh");
}

/// 🔴 Rà soát coordinator — GHIM đúng lời doc-comment `SseEventOutcome::Usage` khẳng định:
/// một khung mang CẢ `usage` LẪN `delta.content` thì `usage` THẮNG, nội dung bị bỏ — trước ca
/// này, mệnh đề đó chỉ đứng trong doc-comment, không ai đối chứng được nó THẬT SỰ đúng hay chỉ
/// tình cờ đúng vì chưa ai gieo đúng hình dạng JSON hiếm này.
#[test]
fn a_frame_carrying_both_usage_and_delta_content_reads_as_usage_and_discards_the_content() {
    let both = r#"{"choices":[{"delta":{"content":"Xin"}}],"usage":{"total_tokens":412}}"#;
    let outcome = interpret_sse_event(both).expect("khong duoc la Err");
    assert_eq!(
        outcome,
        SseEventOutcome::Usage(ChunkUsage { prompt_tokens: None, completion_tokens: None, total_tokens: Some(412) }),
        "usage phai THANG khi mot khung mang ca hai -- dung nhu doc-comment SseEventOutcome::Usage khang dinh"
    );
}

/// Một khung PHIÊN BẢN giữa dòng (không usage, `delta.content` có mặt) vẫn phải đọc như
/// `Token` như trước — bản sửa thêm trường `usage` không được đổi hành vi của khung KHÔNG
/// mang `usage` (ca ÂM cạnh ca trên).
#[test]
fn a_token_chunk_without_usage_still_reads_as_token() {
    let token_chunk = r#"{"choices":[{"delta":{"content":"Xin"}}]}"#;
    let outcome = interpret_sse_event(token_chunk).expect("khung token hop le khong duoc la Err");
    assert_eq!(outcome, SseEventOutcome::Token("Xin".to_owned()));
}

/// `core::ai::pricing::estimate_cost_usd` — mô hình CÓ hàng trong bảng giá trả `Some`, tính
/// đúng công thức (giá/triệu token × số token / 1_000_000, cộng hai chiều vào/ra).
#[test]
fn estimate_cost_usd_computes_a_price_for_a_seeded_model_id() {
    let cost = auratranslate_lib::core::ai::pricing::estimate_cost_usd("claude-sonnet-5", 100, 312)
        .expect("claude-sonnet-5 phai co hang trong bang gia");
    let expected = (100.0 / 1_000_000.0 * 2.0) + (312.0 / 1_000_000.0 * 10.0);
    assert!((cost - expected).abs() < 1e-12, "cong thuc gia phai dung: got={cost}, expected={expected}");
}

/// I/O Matrix spec 4.11 "Model id collides with a table row" — TÀI LIỆU HOÁ một giới hạn ĐÃ
/// CHẤP NHẬN, không phải một lỗi cần vá: bảng giá khoá THEO TÊN model id, không theo bất kỳ
/// dấu hiệu cục bộ/đám mây nào (`core/ai/pricing.rs`'s doc-comment — FR66 cố ý không cho một
/// bộ phân biệt như vậy). Một mô hình cục bộ (Ollama/LM Studio) hay một proxy tự đặt tên trùng
/// MỘT id đã niêm yết trong bảng SẼ bị tính tiền y hệt bản đám mây thật.
///
/// 🔴 **SỬA (rà soát coordinator)** — bản trước gọi `estimate_cost_usd` HAI LẦN với ĐÚNG cùng
/// tham số rồi khẳng định hai kết quả bằng nhau: đúng với BẤT KỲ hàm thuần nào, không ca nào
/// gieo được để nó đỏ. Ca này thay bằng hai LƯỢT GỌI THẬT KHÁC NHAU (số token khác nhau, đóng
/// vai "cuộc gọi đám mây thật" và "cuộc gọi từ một mô hình cục bộ/proxy trùng tên") rồi đối
/// chiếu CẢ HAI với công thức giá CÔNG KHAI ($2/$10 mỗi triệu token, `PRICE_TABLE`) tính độc
/// lập ở đây — nếu `estimate_cost_usd` từng học thêm một cách phân biệt nguồn gọi (đọc một cờ
/// ẩn, một biến môi trường, …) khiến MỘT trong hai lượt lệch khỏi công thức công khai, ca này
/// đỏ đúng ở đó.
#[test]
fn a_local_or_proxied_model_answering_to_a_seeded_cloud_id_is_priced_as_if_it_were_the_real_cloud_model_an_accepted_limitation()
 {
    const INPUT_USD_PER_MILLION: f64 = 2.0;
    const OUTPUT_USD_PER_MILLION: f64 = 10.0;
    fn expected_cost(prompt_tokens: f64, completion_tokens: f64) -> f64 {
        (prompt_tokens / 1_000_000.0 * INPUT_USD_PER_MILLION)
            + (completion_tokens / 1_000_000.0 * OUTPUT_USD_PER_MILLION)
    }

    // "claude-sonnet-5" o day dong hai vai KHAC NHAU that su, khong phai mot loi goi lap lai:
    // mot lot voi so token mo phong mot cuoc goi dam may that, mot lot voi so token KHAC han mo
    // phong mot may cuc bo (hoac mot proxy) tra loi dung TEN nay -- I/O Matrix mo ta dung tinh
    // huong nay, vi ham chi biet doc TEN, khong biet cuoc goi da di dau.
    let real_cloud_call =
        auratranslate_lib::core::ai::pricing::estimate_cost_usd("claude-sonnet-5", 100, 312)
            .expect("id nay co hang trong bang");
    let same_id_from_a_local_or_proxied_caller =
        auratranslate_lib::core::ai::pricing::estimate_cost_usd("claude-sonnet-5", 9, 4)
            .expect("ham khong co cach nao phan biet duoc hai loi goi nay -- van cung mot hang");

    assert!(
        (real_cloud_call - expected_cost(100.0, 312.0)).abs() < 1e-12,
        "gia cua 'cuoc goi dam may that' phai khop CONG THUC CONG KHAI"
    );
    assert!(
        (same_id_from_a_local_or_proxied_caller - expected_cost(9.0, 4.0)).abs() < 1e-12,
        "gioi han da CHAP NHAN: mot loi goi KHAC (so token khac) nhung CUNG id van bi tinh tien \
         theo DUNG cong thuc cong khai -- khong co bo do nguon goi nao lam no lech di"
    );
}

/// I/O Matrix spec 4.11 "Usage arrives, model not in table" — một id CỤC BỘ (đúng id
/// `aiconfig_contract.rs` đã dùng cho mô hình Ollama, `llama3`) không có hàng trong bảng ⇒
/// `None`, đây LÀ quy tắc "mô hình cục bộ" (§Always spec 4.11), không một lỗi.
#[test]
fn estimate_cost_usd_returns_none_for_a_model_id_absent_from_the_table() {
    assert_eq!(auratranslate_lib::core::ai::pricing::estimate_cost_usd("llama3", 100, 312), None);
    assert_eq!(auratranslate_lib::core::ai::pricing::estimate_cost_usd("qwen2.5:14b", 100, 312), None);
}

/// AC: "every row states the date its prices were taken, so a stale row is visible rather
/// than silently believed" — `PRICES_AS_OF` phải là một chuỗi có mặt (không rỗng), và bảng
/// giá phải có ít nhất một hàng để hằng số đó có nghĩa.
#[test]
fn the_price_table_states_the_date_its_prices_were_taken() {
    assert!(!auratranslate_lib::core::ai::pricing::PRICES_AS_OF.is_empty());
    assert!(
        auratranslate_lib::core::ai::pricing::estimate_cost_usd("claude-sonnet-5", 1, 1).is_some(),
        "phai co it nhat mot hang de PRICES_AS_OF co nghia"
    );
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
    DoneWithUsage(TranslateUsage),
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
            FakeFinish::Done => Ok(TranslateOutcome::Done(None)),
            FakeFinish::DoneWithUsage(u) => Ok(TranslateOutcome::Done(Some(*u))),
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

    assert_eq!(outcome, TranslateOutcome::Done(None));
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

/// I/O Matrix "Error with zero tokens received" (nhánh ĐƠN) — §Coverage spec 4.10 nêu tên lỗ
/// này: "every fake error case emits at least one token first". Ca này đối chứng nhánh KHÔNG
/// một token nào từng tới trước khi provider trượt -- `Channel` phải RỖNG, không một khung nào
/// (khác ca `a_provider_error_after_partial_tokens_...` ngay trên, nơi MỘT token đã tới).
///
/// 🔵 THÊM 2026-09-22 (Story 4.10, Phase 3 — nửa MỚI của Task 9).
#[test]
fn a_provider_error_with_zero_tokens_received_leaves_the_channel_empty_and_propagates_the_error() {
    let provider = FakeProvider {
        tokens: vec![],
        finish: FakeFinish::Err(FakeProviderError("dropped-before-first-token".to_owned())),
    };
    let prepared = dummy_prepared();
    let (channel, received) = collecting_channel();
    let should_cancel = || false;

    let err =
        tauri::async_runtime::block_on(run_translate_call(&provider, &prepared, &channel, &should_cancel))
            .expect_err("mot loi provider TRUOC token dau tien van phai la Err");

    assert_eq!(err, FakeProviderError("dropped-before-first-token".to_owned()));
    let got = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    assert!(got.is_empty(), "khong token nao tung gui -- Channel phai RONG, khong mot khung nao");
}

// ═════════════════════════════════════════════════════════════════════════════════
// 3b. Story 4.11 -- `run_translate_call` carries `TranslateUsage` through unchanged, đúng ba
// hàng còn lại của I/O Matrix spec 4.11 canh được ở seam NÀY (usage đã tới, provider không
// gửi usage, huỷ trước khi usage tới).
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix "Usage arrives" -- provider (giả) trả về usage cùng lượt `Done`, và
/// `run_translate_call` phải mang NGUYÊN VẸN giá trị đó ra ngoài, không đánh rơi hay đúc lại.
#[test]
fn usage_reported_by_the_provider_is_carried_through_run_translate_call_unchanged() {
    let usage = TranslateUsage { prompt_tokens: 100, completion_tokens: 312, total_tokens: 412, cost_usd: Some(0.00412) };
    let provider = FakeProvider { tokens: vec!["Xin chào"], finish: FakeFinish::DoneWithUsage(usage) };
    let prepared = dummy_prepared();
    let (channel, _received) = collecting_channel();
    let should_cancel = || false;

    let outcome =
        tauri::async_runtime::block_on(run_translate_call(&provider, &prepared, &channel, &should_cancel))
            .expect("khong duoc loi");

    assert_eq!(outcome, TranslateOutcome::Done(Some(usage)), "usage phai di qua NGUYEN VEN, khong doi mot truong nao");
}

/// I/O Matrix "Provider sends no usage" -- stream kết thúc `Done` mà không một khung `usage`
/// nào từng tới (`FakeFinish::Done` không mang usage) -- outcome phải mang `None`, KHÔNG một
/// `TranslateUsage` giả với các trường bằng `0` (§Always spec 4.11: "never `0`, never an empty
/// currency").
#[test]
fn a_stream_that_ends_done_without_ever_seeing_a_usage_frame_carries_none() {
    let provider = FakeProvider { tokens: vec!["Xin ", "chào"], finish: FakeFinish::Done };
    let prepared = dummy_prepared();
    let (channel, _received) = collecting_channel();
    let should_cancel = || false;

    let outcome =
        tauri::async_runtime::block_on(run_translate_call(&provider, &prepared, &channel, &should_cancel))
            .expect("khong duoc loi");

    assert_eq!(outcome, TranslateOutcome::Done(None));
}

/// I/O Matrix "Cancelled mid-flight" -- huỷ TRƯỚC khi provider (giả) kịp trả usage (dù công
/// thức của nó CÓ mang usage nếu chạy hết) -- outcome phải là `Cancelled`, kiểu của biến thể
/// đó KHÔNG có chỗ để mang một usage nào (không cần assert thêm gì ngoài biến thể đúng: đây là
/// bảo đảm ở TẦNG KIỂU, không phải một giá trị có thể lỡ tay đúc sai).
#[test]
fn cancelling_before_the_usage_frame_arrives_reports_cancelled_with_no_usage_to_carry() {
    let usage = TranslateUsage { prompt_tokens: 9, completion_tokens: 9, total_tokens: 18, cost_usd: None };
    let provider = FakeProvider { tokens: vec!["partial"], finish: FakeFinish::DoneWithUsage(usage) };
    let prepared = dummy_prepared();
    let (channel, received) = collecting_channel();
    let calls = AtomicUsize::new(0);
    // Huy NGAY sau token dau tien -- provider (gia) khong bao gio toi duoc nhanh tra usage.
    let should_cancel = || calls.fetch_add(1, Ordering::SeqCst) >= 1;

    let outcome =
        tauri::async_runtime::block_on(run_translate_call(&provider, &prepared, &channel, &should_cancel))
            .expect("huy la Ok(Cancelled), khong phai mot Err");

    assert_eq!(outcome, TranslateOutcome::Cancelled);
    let got = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    assert_eq!(got, vec!["partial".to_owned()]);
}

/// `AiTranslateUsageWire::from(TranslateUsage)` -- chuyển đổi 1:1, không đánh rơi/đúc lại
/// trường nào (kể cả `cost_usd: None`, ca "usage đã tới nhưng mô hình không có giá").
#[test]
fn ai_translate_usage_wire_from_translate_usage_copies_every_field_verbatim() {
    let usage = TranslateUsage { prompt_tokens: 7, completion_tokens: 11, total_tokens: 18, cost_usd: None };
    let wire = auratranslate_lib::commands::aitranslate::AiTranslateUsageWire::from(usage);
    assert_eq!(wire.prompt_tokens, 7);
    assert_eq!(wire.completion_tokens, 11);
    assert_eq!(wire.total_tokens, 18);
    assert_eq!(wire.cost_usd, None);

    let priced = TranslateUsage { prompt_tokens: 100, completion_tokens: 312, total_tokens: 412, cost_usd: Some(0.00412) };
    let priced_wire = auratranslate_lib::commands::aitranslate::AiTranslateUsageWire::from(priced);
    assert_eq!(priced_wire.cost_usd, Some(0.00412));
}

/// 🔴 Rà soát coordinator — `single_run_outcome_wire` (`commands/aitranslate.rs`) là seam
/// THUẦN mà `wire::ai_translate_segment` (một `#[tauri::command]`, không gọi được từ đây) dùng
/// để đúc `AiTranslateOutcomeWire` -- trước ca này, KHÔNG một test nào canh rằng `usage` thật
/// sự đi ra dây cho lượt dịch MỘT segment: thay `usage.map(AiTranslateUsageWire::from)` bằng
/// `None` bên trong hàm đó (đúng hình dạng nhánh batch hợp lệ đứng cạnh -- một lỗi copy-paste
/// dễ mắc) vẫn qua sạch mọi gate/test khác. Ca này gọi THẲNG hàm ánh xạ, không đi vòng qua một
/// webview giả.
#[test]
fn single_run_outcome_wire_carries_the_real_usage_through_for_done_and_carries_nothing_for_cancelled() {
    use auratranslate_lib::commands::aitranslate::{AiTranslateOutcomeWire, single_run_outcome_wire};

    let usage = TranslateUsage { prompt_tokens: 100, completion_tokens: 312, total_tokens: 412, cost_usd: Some(0.00412) };
    match single_run_outcome_wire(TranslateOutcome::Done(Some(usage))) {
        AiTranslateOutcomeWire::Done { usage: Some(wire) } => {
            assert_eq!(wire.total_tokens, 412, "usage THAT phai di ra day, khong duoc thay bang None");
            assert_eq!(wire.cost_usd, Some(0.00412));
        }
        other => panic!("Done(Some(usage)) phai anh xa thanh Done{{ usage: Some(..) }}, nhan duoc {other:?}"),
    }

    match single_run_outcome_wire(TranslateOutcome::Done(None)) {
        AiTranslateOutcomeWire::Done { usage: None } => {}
        other => panic!("Done(None) (provider khong tra usage) phai anh xa thanh Done{{ usage: None }}, nhan duoc {other:?}"),
    }

    match single_run_outcome_wire(TranslateOutcome::Cancelled) {
        AiTranslateOutcomeWire::Cancelled => {}
        other => panic!("Cancelled phai anh xa thanh Cancelled, nhan duoc {other:?}"),
    }
}

/// AC5 — batch: usage rời rạc theo TỪNG câu qua `AiTranslateBatchEventWire::Done`, và một câu
/// KHÔNG báo usage (provider giả không mang usage cho câu đó) không kéo câu khác xuống theo —
/// mỗi khung chỉ mang sự thật CỦA CHÍNH NÓ (Rust không tự cộng dồn; cộng dồn + phân biệt
/// "toàn phần"/"một phần" là việc của `aiTranslateBatchState.ts`, canh ở
/// `tests/frontend/aiTranslateBatch.test.ts`).
#[test]
fn a_batch_carries_usage_per_sentence_and_a_sentence_with_no_usage_does_not_affect_its_neighbours() {
    let ids = [1_601_i64, 1_602, 1_603];
    let items: Vec<PreparedBatchItem> =
        ids.iter().map(|id| dummy_batch_item(*id, &format!("Prompt for {id}"))).collect();
    let usage_1 = TranslateUsage { prompt_tokens: 10, completion_tokens: 20, total_tokens: 30, cost_usd: Some(0.0002) };
    let usage_3 = TranslateUsage { prompt_tokens: 5, completion_tokens: 5, total_tokens: 10, cost_usd: None };
    let recipes = vec![
        MultiItemRecipe {
            tokens: vec!["one"],
            finish: FakeFinish::DoneWithUsage(usage_1),
            trigger_cancel_after_token: None,
        },
        MultiItemRecipe { tokens: vec!["two"], finish: FakeFinish::Done, trigger_cancel_after_token: None },
        MultiItemRecipe {
            tokens: vec!["three"],
            finish: FakeFinish::DoneWithUsage(usage_3),
            trigger_cancel_after_token: None,
        },
    ];
    let provider = MultiItemProvider {
        recipes,
        call_index: AtomicUsize::new(0),
        call_count: AtomicUsize::new(0),
        cancel_flag: Arc::new(AtomicBool::new(false)),
    };
    let (channel, received) = collecting_batch_channel();
    let should_cancel = || false;

    let outcome = tauri::async_runtime::block_on(run_batch_call(&provider, &items, &channel, &should_cancel))
        .expect("khong duoc loi");
    assert_eq!(outcome, AiTranslateBatchOutcome::Done);

    let got = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    let usage_shape = |u: TranslateUsage| UsageShape {
        prompt_tokens: u.prompt_tokens,
        completion_tokens: u.completion_tokens,
        total_tokens: u.total_tokens,
        cost_usd: u.cost_usd,
    };
    let expected = vec![
        BatchEventShape::Token { segment_id: ids[0], text: "one".to_owned() },
        BatchEventShape::Done { segment_id: ids[0], usage: Some(usage_shape(usage_1)) },
        BatchEventShape::Token { segment_id: ids[1], text: "two".to_owned() },
        BatchEventShape::Done { segment_id: ids[1], usage: None },
        BatchEventShape::Token { segment_id: ids[2], text: "three".to_owned() },
        BatchEventShape::Done { segment_id: ids[2], usage: Some(usage_shape(usage_3)) },
    ];
    assert_eq!(
        got, expected,
        "moi khung Done phai mang DUNG usage cua CHINH cau do -- cau khong bao usage (id[1]) \
         khong lam sai lech usage cua hai cau con lai"
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
///
/// 🔵 SỬA 2026-09-22 (Story 4.10, Phase 1) — trước bản sửa này ca này canh một NHÃN GỘP DUY
/// NHẤT (`AiTranslateProviderCallFailed`) cho cả bảy biến thể; Quyết định 2 spec 4.10 tách sáu
/// khoá họ nguyên nhân, và non-2xx là khoá `AiTranslateProviderRefused` (`["status"]` bắt buộc).
#[test]
fn provider_returns_a_non_2xx_maps_to_a_non_retryable_ipc_error_naming_the_status() {
    let err: IpcError = OpenAiClientError::NonSuccessStatus { status: 401 }.into();
    assert_eq!(err.code(), "ai_translate.provider_refused");
    assert_eq!(err.message_key(), MessageKey::AiTranslateProviderRefused);
    assert_eq!(err.params().get("status"), Some(&"401".to_owned()));
    assert!(!err.retryable(), "non-2xx khong duoc thu lai");
}

/// I/O Matrix "Stream ends without [DONE]" — `IpcError` retryable, KHÔNG tự động thử lại ở
/// bất kỳ tầng nào (đó là quyết định của chỗ gọi/người dùng bấm lại, không phải cơ chế tự
/// động — AD-22).
///
/// 🔵 SỬA 2026-09-22 (Story 4.10, Phase 1) — khoá đổi từ `AiTranslateProviderCallFailed` sang
/// `AiTranslateStreamEndedWithoutDone` (Quyết định 2 spec 4.10, cùng lý do trên).
#[test]
fn stream_ended_without_done_maps_to_a_retryable_ipc_error() {
    let err: IpcError = OpenAiClientError::StreamEndedWithoutDone.into();
    assert_eq!(err.code(), "ai_translate.stream_ended_without_done");
    assert_eq!(err.message_key(), MessageKey::AiTranslateStreamEndedWithoutDone);
    assert!(err.params().is_empty());
    assert!(err.retryable(), "rot ket noi GIUA CHUNG co the thu lai o luot bam lai");
}

/// Đối chứng ĐỦ TÁM biến thể (7 cũ + `ApiKeyHeaderInvalid` tách khỏi `RequestFailed` ở Task 1
/// spec 4.10) — khoá cả bảng `match` của `openai_client_error_family`
/// (`impl From<OpenAiClientError> for IpcError` gọi xuống nó) lại một lần, để một biến thể mới
/// thêm vào tương lai không lặng lẽ rơi vào nhánh mặc định sai.
///
/// 🔵 SỬA 2026-09-22 (Story 4.10, Phase 1) — tên hàm VÀ nội dung đổi từ "mọi biến thể ánh xạ
/// cùng MỘT khoá gộp" sang "mọi biến thể ánh xạ khoá HỌ của chính nó" (Quyết định 2). Mảng giữ
/// nguyên 7 phần tử cũ CỘNG một phần tử mới cho `ApiKeyHeaderInvalid` = 8.
#[test]
fn every_openai_client_error_variant_maps_to_its_own_family_key_with_the_documented_retryable_flag() {
    let cases: [(OpenAiClientError, &str, MessageKey, bool); 8] = [
        (
            OpenAiClientError::ClientBuildFailed { detail: "x".to_owned() },
            "ai_translate.client_build_failed",
            MessageKey::AiTranslateClientBuildFailed,
            false,
        ),
        (
            OpenAiClientError::RequestFailed { detail: "x".to_owned() },
            "ai_translate.provider_unreachable",
            MessageKey::AiTranslateProviderUnreachable,
            true,
        ),
        (
            OpenAiClientError::NonSuccessStatus { status: 500 },
            "ai_translate.provider_refused",
            MessageKey::AiTranslateProviderRefused,
            false,
        ),
        (
            OpenAiClientError::ReadFailed { detail: "x".to_owned() },
            "ai_translate.provider_unreachable",
            MessageKey::AiTranslateProviderUnreachable,
            true,
        ),
        (
            OpenAiClientError::StreamEndedWithoutDone,
            "ai_translate.stream_ended_without_done",
            MessageKey::AiTranslateStreamEndedWithoutDone,
            true,
        ),
        (
            OpenAiClientError::MalformedEvent { detail: "x".to_owned() },
            "ai_translate.reply_unreadable",
            MessageKey::AiTranslateReplyUnreadable,
            false,
        ),
        (
            OpenAiClientError::BufferOverflow { size: 2 * 1024 * 1024 },
            "ai_translate.reply_unreadable",
            MessageKey::AiTranslateReplyUnreadable,
            false,
        ),
        (
            OpenAiClientError::ApiKeyHeaderInvalid { detail: "x".to_owned() },
            "ai_translate.api_key_header_invalid",
            MessageKey::AiTranslateApiKeyHeaderInvalid,
            false,
        ),
    ];
    for (variant, expected_code, expected_key, expected_retryable) in cases {
        let label = format!("{variant:?}");
        let err: IpcError = variant.into();
        assert_eq!(err.code(), expected_code, "{label} phai mang code cua HO nguyen nhan");
        assert_eq!(err.message_key(), expected_key, "{label}");
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

// ═════════════════════════════════════════════════════════════════════════════════
// 5. Story 4.9, Phase 4a — I/O Matrix của batch (`prepare_batch_call` + `run_batch_call`)
// ═════════════════════════════════════════════════════════════════════════════════
//
// Chín hàng ma trận I/O spec 4.9. Sáu hàng canh được ở tầng Rust này (giống 4.8's file, hai
// hàng "Promote while generating"/"Caret moves during generation" là frontend-only): "Batch
// over a selection", "Cancel mid-batch", "Error mid-batch", "Omitted segment inside the
// selection", "AI not configured", "Selection of exactly one". Ba hàng còn lại là trạng thái
// **frontend-only**, không một trường Rust nào biết chúng: "Work or Chapter changes while an
// AI call runs" (module reset, `tests/frontend/aiTranslate*.test.ts`), "Selection changes
// while a batch runs" (đã đóng băng CẤU TRÚC ngay khi `prepare_batch_call` trả một
// `Vec<PreparedBatchItem>` sở hữu -- `run_batch_call` không giữ tham chiếu nào tới lựa chọn
// đang sống của webview để mà thấy nó đổi), và "Empty selection" (nút lệnh bị khoá ở tầng
// dispatch, không một lời gọi IPC nào được gửi).
//
// 🔵 SỬA 2026-09-22 (Story 4.10, Phase 1) — đoạn ⚠️ nguyên bản dưới đây (Phase 4a) đã SAI ngay
// từ Story 4.9 Phase 4b: `batch_stopped_error` được đổi thành `pub` ở chính phase đó (xem
// doc-comment của nó ở `commands/aitranslate.rs`, "`pub` (Story 4.9, Phase 4b)"), nên ca
// `batch_stopped_error_names_the_sentence_and_maps_the_documented_retryable_flag` (§4a trên) ĐÃ
// gọi thẳng được hàm thật từ lâu — không còn cần `AppHandle` thật/một lệnh Tauri thật nào. Tên
// hàm phân loại dùng chung cũng đổi: `openai_client_error_is_retryable` → `openai_client_error_
// family` (Quyết định 2 spec 4.10, sáu khoá họ thay vì một nhãn gộp), và nhãn `ai_translate.
// batch_stopped` không còn được dùng nữa — `batch_stopped_error` giờ trả về MỘT trong sáu khoá
// họ nguyên nhân, giống hệt lượt dịch MỘT segment, chỉ mang thêm `segment_id`.

/// Mirror THUẦN của `AiTranslateUsageWire` (`commands/aitranslate.rs`, Story 4.11) -- cùng
/// lý do [`BatchEventShape`] ngay dưới có mirror riêng của nó: kiểu sản phẩm chỉ derive
/// `Serialize`.
#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
struct UsageShape {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
    cost_usd: Option<f64>,
}

/// Mirror THUẦN của `AiTranslateBatchEventWire` chỉ để giải mã byte THẬT đã đi qua
/// `Channel::send` -- kiểu sản phẩm chỉ derive `Serialize` (một chiều gửi ra), không
/// `Deserialize`, nên một crate test khác không giải mã ngược được kiểu đó thẳng. Hình dạng
/// JSON (`tag = "kind"`, `rename_all = "snake_case"`) sao lại NGUYÊN VĂN từ
/// `commands/aitranslate.rs` -- một đổi hình dạng bên sản phẩm mà không sửa mirror này sẽ
/// làm MỌI ca dưới đây đỏ vì lỗi giải mã, không lặng lẽ trôi qua.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum BatchEventShape {
    Token { segment_id: i64, text: String },
    // Story 4.11 -- `usage` them vao `AiTranslateBatchEventWire::Done` (commands/aitranslate.rs);
    // mirror THUAN rieng (khong tai dung `AiTranslateUsageWire` san pham: kieu do chi derive
    // `Serialize`, khong `Deserialize`, dung khuon doc-comment tren -- mot crate test KHAC
    // khong giai ma nguoc duoc no thang).
    Done { segment_id: i64, usage: Option<UsageShape> },
    Skipped { segment_id: i64 },
}

/// Dựng một `Channel<AiTranslateBatchEventWire>` THẬT, thu lại các sự kiện đã gửi qua
/// `BatchEventShape` (cùng khuôn `collecting_channel` ở §3, mở rộng cho kiểu enum của batch),
/// cho một `hook` chạy TRÊN MỖI sự kiện ngay khi nó tới -- chỗ Test "huỷ đúng giữa hai câu"
/// dưới đây cắm cờ huỷ vào.
fn collecting_batch_channel_with_hook(
    hook: impl FnMut(&BatchEventShape) + Send + 'static,
) -> (tauri::ipc::Channel<AiTranslateBatchEventWire>, Arc<Mutex<Vec<BatchEventShape>>>) {
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_in = Arc::clone(&received);
    // `Channel::new` doi `Fn + Send + Sync` -- boc `hook` (chi `FnMut`) trong mot `Mutex` de
    // co `Sync` thay vi doi chu ky ham nay thanh `Fn` (moi ca goi hook deu can MUTATE mot cai
    // gi do, vd. bat mot co dung chung).
    let hook = Mutex::new(hook);
    let channel = tauri::ipc::Channel::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            let event: BatchEventShape = serde_json::from_str(&json)
                .unwrap_or_else(|e| panic!("giai ma AiTranslateBatchEventWire that bai: {e} -- json={json}"));
            (hook.lock().unwrap_or_else(std::sync::PoisonError::into_inner))(&event);
            received_in.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(event);
        }
        Ok(())
    });
    (channel, received)
}

fn collecting_batch_channel()
-> (tauri::ipc::Channel<AiTranslateBatchEventWire>, Arc<Mutex<Vec<BatchEventShape>>>) {
    collecting_batch_channel_with_hook(|_| {})
}

fn dummy_batch_item(segment_id: i64, prompt: &str) -> PreparedBatchItem {
    PreparedBatchItem::ToTranslate {
        segment_id,
        prepared: PreparedTranslateCall {
            endpoint: "https://api.example.invalid/v1/chat/completions".to_owned(),
            model: "gpt-test".to_owned(),
            temperature: None,
            max_tokens: None,
            api_key: "sk-fake-batch-provider-test".to_owned(),
            prompt: prompt.to_owned(),
        },
    }
}

/// Một chuỗi 5 câu tiếng Anh phân biệt được bằng số thứ tự -- đủ để cả `prepare_batch_call`
/// (đọc Chương thật, cần thứ tự tài liệu) lẫn phép đối chứng "sắp xếp lại theo tài liệu,
/// không theo thứ tự `segment_ids` gửi lên" dùng chung MỘT fixture.
const FIVE_SENTENCE_TEXT: &str = "Sentence one arrives. Sentence two arrives. \
Sentence three arrives. Sentence four arrives. Sentence five arrives.";

/// "Công thức" của MỘT lời gọi provider trong một lô nhiều câu -- chỉ số THỨ TỰ lời gọi
/// (không phải `segment_id`) chọn công thức nào chạy, đúng cách [`MultiItemProvider`] tiêu
/// thụ nó.
struct MultiItemRecipe {
    tokens: Vec<&'static str>,
    finish: FakeFinish,
    /// Sau khi gửi token Ở CHỈ SỐ này (0-based) của CHÍNH công thức này, bật cờ huỷ dùng
    /// chung -- mô phỏng "người dùng bấm huỷ đúng lúc câu này đang chảy".
    trigger_cancel_after_token: Option<usize>,
}

/// `TranslationProvider` GIẢ cho một LÔ nhiều câu -- mở rộng [`FakeProvider`] (§3, một câu)
/// sang một DANH SÁCH công thức tiêu thụ THEO THỨ TỰ lời gọi. `call_count` đếm MỌI lời gọi
/// `translate(...)`, kể cả một lời gọi lẽ ra không được phép xảy ra -- các ca "không câu nào
/// SAU được gọi" cấp đúng số công thức cho số câu ĐÁNG được gọi; một lời gọi THỪA sẽ panic ở
/// `expect` dưới đây (chỉ số vượt `recipes.len()`) thay vì lặng lẽ trôi qua.
struct MultiItemProvider {
    recipes: Vec<MultiItemRecipe>,
    call_index: AtomicUsize,
    call_count: AtomicUsize,
    cancel_flag: Arc<AtomicBool>,
}

impl TranslationProvider for MultiItemProvider {
    type Error = FakeProviderError;

    async fn translate(
        &self,
        _request: TranslateRequest<'_>,
        on_token: &mut dyn FnMut(&str),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<TranslateOutcome, Self::Error> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let idx = self.call_index.fetch_add(1, Ordering::SeqCst);
        let recipe = self.recipes.get(idx).unwrap_or_else(|| {
            panic!(
                "MultiItemProvider bi goi lan thu {} nhung chi cap {} cong thuc -- mot cau KHONG \
                 DUOC goi da bi goi",
                idx + 1,
                self.recipes.len()
            )
        });
        for (i, token) in recipe.tokens.iter().enumerate() {
            if should_cancel() {
                return Ok(TranslateOutcome::Cancelled);
            }
            on_token(token);
            if recipe.trigger_cancel_after_token == Some(i) {
                self.cancel_flag.store(true, Ordering::SeqCst);
            }
        }
        if should_cancel() {
            return Ok(TranslateOutcome::Cancelled);
        }
        match &recipe.finish {
            FakeFinish::Done => Ok(TranslateOutcome::Done(None)),
            FakeFinish::DoneWithUsage(u) => Ok(TranslateOutcome::Done(Some(*u))),
            FakeFinish::Err(e) => Err(e.clone()),
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────────────
// 5a. `prepare_batch_call` -- hàm THUẦN, đọc Chương thật
// ───────────────────────────────────────────────────────────────────────────────────

/// I/O Matrix "Batch over a selection" (nửa sắp xếp) -- `prepare_batch_call` trả các hàng
/// theo ĐÚNG thứ tự tài liệu, KHÔNG theo thứ tự `segment_ids` được gửi lên (§Always spec 4.9:
/// "the batch is exactly the user's selection, translated in document order").
#[test]
fn prepare_batch_call_returns_items_in_document_order_regardless_of_the_input_order_of_segment_ids()
 {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-batch-order-test-key");

    let global_dir = temp_dir("batch-order-global");
    let work_dir = temp_dir("batch-order-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "BatchOrder", "en", FIVE_SENTENCE_TEXT);
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "Plain", "{{source_segment}}")
        .expect("tao bo prompt");

    let doc_order: Vec<i64> =
        read_open_chapter_segments(Some(&open)).expect("nap chuong").segments.iter().map(|s| s.id).collect();
    assert_eq!(doc_order.len(), 5, "fixture 5 cau phai tach thanh 5 segment");

    // Gui LEN theo thu tu XAO TRON co y -- khac han thu tu tai lieu.
    let shuffled = [doc_order[2], doc_order[0], doc_order[4], doc_order[1], doc_order[3]];
    let record = fresh_record();
    let outcome = prepare_batch_call(Some(&global), Some(&open), &record, Some("Plain"), &shuffled)
        .expect("prepare batch khong duoc loi");
    let items = match outcome {
        PrepareBatchOutcome::Ready(items) => items,
        PrepareBatchOutcome::NotConfigured => panic!("phai san sang"),
    };

    let returned_order: Vec<i64> = items
        .iter()
        .map(|item| match item {
            PreparedBatchItem::ToTranslate { segment_id, .. } => *segment_id,
            PreparedBatchItem::Omitted { segment_id } => *segment_id,
        })
        .collect();
    assert_eq!(
        returned_order, doc_order,
        "prepare_batch_call phai sap lai THEO TAI LIEU, khong theo thu tu segment_ids gui len"
    );

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// I/O Matrix "Selection of exactly one" -- một lô của ĐÚNG một segment chạy qua path THUẦN
/// của batch (`Ready(vec![... 1 phần tử ...])`), không một nhánh rẽ lặng lẽ nào rơi về hình
/// dạng khác.
#[test]
fn a_selection_of_exactly_one_segment_runs_through_the_batch_prepare_path_as_a_batch_of_one() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-batch-of-one-test-key");

    let global_dir = temp_dir("batch-of-one-global");
    let work_dir = temp_dir("batch-of-one-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "BatchOfOne", "en", "A dragon roared.");
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "Plain", "{{source_segment}}")
        .expect("tao bo prompt");

    let segment_id = first_segment_id(&open);
    let record = fresh_record();
    let outcome = prepare_batch_call(Some(&global), Some(&open), &record, Some("Plain"), &[segment_id])
        .expect("prepare batch khong duoc loi");
    match outcome {
        PrepareBatchOutcome::Ready(items) => {
            assert_eq!(items.len(), 1, "mot lua chon dung MOT segment phai tra ve DUNG mot phan tu");
            match &items[0] {
                PreparedBatchItem::ToTranslate { segment_id: got, prepared } => {
                    assert_eq!(*got, segment_id);
                    assert!(!prepared.prompt.is_empty());
                }
                PreparedBatchItem::Omitted { .. } => panic!("segment nay khong bi cat, phai la ToTranslate"),
            }
        }
        PrepareBatchOutcome::NotConfigured => panic!("phai san sang"),
    }

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// I/O Matrix "Omitted segment inside the selection" (nửa PHÂN LOẠI) — segment `is_omitted`
/// phải được `prepare_batch_call` phân loại `PreparedBatchItem::Omitted`, không
/// `ToTranslate`. **Đây là ca dùng cho counter-check-bằng-gỡ của "the omitted-segment guard"**
/// (§Tasks spec 4.9's Phase 4 cuối cùng): gỡ khối `if row.is_omitted { ... continue; }` khỏi
/// `prepare_batch_call` làm hàng này rơi xuống nhánh `assemble_and_record_prompt` như mọi hàng
/// khác và trở thành `ToTranslate` -- ca này đỏ ngay tại đúng assert dưới, không phải một lỗi
/// biên dịch.
#[test]
fn an_omitted_segment_inside_the_selection_is_classified_as_omitted_not_to_translate() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-batch-omitted-test-key");

    let global_dir = temp_dir("batch-omitted-global");
    let work_dir = temp_dir("batch-omitted-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "BatchOmitted", "en", FIVE_SENTENCE_TEXT);
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "Plain", "{{source_segment}}")
        .expect("tao bo prompt");

    let chapter = read_open_chapter_segments(Some(&open)).expect("nap chuong");
    assert_eq!(chapter.segments.len(), 5, "fixture 5 cau phai tach thanh 5 segment");
    let all_ids: Vec<i64> = chapter.segments.iter().map(|s| s.id).collect();
    let omitted_id = all_ids[2];
    set_segment_omitted(Some(&open), omitted_id, true).expect("dat co cat bo that bai");

    let record = fresh_record();
    let outcome = prepare_batch_call(Some(&global), Some(&open), &record, Some("Plain"), &all_ids)
        .expect("prepare batch khong duoc loi");
    let items = match outcome {
        PrepareBatchOutcome::Ready(items) => items,
        PrepareBatchOutcome::NotConfigured => panic!("phai san sang -- cac hang khac can dich"),
    };

    assert_eq!(items.len(), 5);
    for (i, item) in items.iter().enumerate() {
        if i == 2 {
            assert!(
                matches!(item, PreparedBatchItem::Omitted { segment_id } if *segment_id == omitted_id),
                "hang thu 3 (segment_id={omitted_id}, is_omitted=true) phai la PreparedBatchItem::Omitted"
            );
        } else {
            assert!(
                matches!(item, PreparedBatchItem::ToTranslate { .. }),
                "hang thu {} khong bi cat, phai la ToTranslate",
                i + 1
            );
        }
    }

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// I/O Matrix "Omitted segment inside the selection" (nửa KHOÁ) — một lô mà MỌI segment đều
/// `is_omitted` không được chạm khoá keychain: đo bằng cách xoá khoá đi (không lưu gì) rồi
/// vẫn mong `Ready`, không `NotConfigured` -- nếu code đọc keychain cho lô này, nó sẽ thấy
/// "chưa có khoá" và rơi nhầm về `NotConfigured`.
#[test]
fn a_batch_of_only_omitted_segments_never_reads_the_keychain() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();

    let global_dir = temp_dir("batch-all-omitted-global");
    let work_dir = temp_dir("batch-all-omitted-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "BatchAllOmitted", "en", "Sentence one. Sentence two.");
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");

    let chapter = read_open_chapter_segments(Some(&open)).expect("nap chuong");
    assert_eq!(chapter.segments.len(), 2, "fixture 2 cau phai tach thanh 2 segment");
    let all_ids: Vec<i64> = chapter.segments.iter().map(|s| s.id).collect();
    for id in &all_ids {
        set_segment_omitted(Some(&open), *id, true).expect("dat co cat bo that bai");
    }

    let record = fresh_record();
    let outcome = prepare_batch_call(Some(&global), Some(&open), &record, Some("Plain"), &all_ids)
        .expect("prepare batch khong duoc loi");
    match outcome {
        PrepareBatchOutcome::Ready(items) => {
            assert_eq!(items.len(), 2);
            assert!(
                items.iter().all(|i| matches!(i, PreparedBatchItem::Omitted { .. })),
                "ca hai hang deu bi cat -- ca hai phai la Omitted"
            );
        }
        PrepareBatchOutcome::NotConfigured => panic!(
            "mot lo TOAN cau da cat khong duoc doc keychain -- khong khoa nao duoc luu nhung \
             ket qua khong duoc la NotConfigured vi KHONG lan nao thu doc khoa"
        ),
    }

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// I/O Matrix "AI not configured" (áp cho lô) -- `NotConfigured` cho CẢ lô, và lựa chọn không
/// bị chạm: không một bản ghi prompt nào được tạo cho bất kỳ segment nào trong lô.
#[test]
fn ai_not_configured_for_a_batch_reports_not_configured_before_any_provider_call_and_leaves_the_selection_untouched()
 {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();

    let global_dir = temp_dir("batch-not-configured-global");
    let work_dir = temp_dir("batch-not-configured-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "BatchNotConfigured", "en", "Sentence one. Sentence two.");
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    // KHONG luu khoa nao -- endpoint/model du nhung "key_configured == Some(false)".

    let chapter = read_open_chapter_segments(Some(&open)).expect("nap chuong");
    let all_ids: Vec<i64> = chapter.segments.iter().map(|s| s.id).collect();

    let record = fresh_record();
    let outcome = prepare_batch_call(Some(&global), Some(&open), &record, Some("Plain"), &all_ids)
        .expect("khong duoc la mot Err -- 'chua cau hinh' la mot TRANG THAI");
    assert!(matches!(outcome, PrepareBatchOutcome::NotConfigured));
    assert!(
        read_last_assembled_prompt(&record).is_none(),
        "lua chon phai VAN NGUYEN -- khong mot ban ghi prompt nao duoc tao khi chua cau hinh"
    );

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// I/O Matrix "No Work open" (áp cho LÔ) -- tái dùng ĐÚNG khoá `WorkNoneOpen`, cùng khoá mà
/// lượt dịch MỘT segment dùng (`no_work_open_reuses_the_existing_work_none_open_key`). Ca đó
/// chỉ canh nhánh ĐƠN -- `prepare_batch_call` từ chối TRƯỚC khi đụng tới `segment_ids` (dòng
/// `open.ok_or_else(crate::commands::chapter::no_work_open)?` chạy trước bất kỳ điều gì khác),
/// nên §Coverage spec 4.10 nêu tên đây là lỗ chưa canh: "`work.none_open` is untested on the
/// batch path".
///
/// 🔵 THÊM 2026-09-22 (Story 4.10, Phase 3 — nửa MỚI của Task 9).
#[test]
fn no_work_open_on_a_batch_reuses_the_existing_work_none_open_key() {
    let global_dir = temp_dir("batch-no-work-open-global");
    let global = open_global(&global_dir);
    let record = fresh_record();

    let err = expect_prepare_batch_err(
        prepare_batch_call(Some(&global), None, &record, Some("Plain"), &[1]),
        "khong Tac pham nao dang mo phai la mot Err -- ca cho MOT LO",
    );
    assert_eq!(err.code(), "work.none_open");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);
    assert!(!err.retryable());

    drop(global);
    cleanup(&global_dir);
}

/// I/O Matrix "segment not in chapter" (áp cho LÔ) -- tái dùng ĐÚNG khoá
/// `AiPromptSegmentNotInChapter`, cùng khoá mà lượt dịch MỘT segment dùng
/// (`segment_not_in_chapter_reuses_the_existing_ai_prompt_segment_not_in_chapter_key`). Ca đó
/// chỉ canh nhánh ĐƠN -- ca dưới đây là nhánh LÔ của `prepare_batch_call`, đúng dòng `if let
/// Some(&missing) = requested.difference(&found).next() { return Err(segment_not_in_chapter(...)) }`,
/// với một id bịa TRỘN giữa hai id thật (không phải id bịa duy nhất trong lô).
#[test]
fn segment_not_in_chapter_on_a_batch_names_the_one_bogus_id_among_real_ones() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-batch-not-in-chapter-test-key");

    let global_dir = temp_dir("batch-not-in-chapter-global");
    let work_dir = temp_dir("batch-not-in-chapter-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "BatchNotInChapter", "en", "Sentence one. Sentence two.");
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "Plain", "{{source_segment}}")
        .expect("tao bo prompt");

    let chapter = read_open_chapter_segments(Some(&open)).expect("nap chuong");
    assert_eq!(chapter.segments.len(), 2, "fixture 2 cau phai tach thanh 2 segment");
    let real_ids: Vec<i64> = chapter.segments.iter().map(|s| s.id).collect();
    let bogus_id = real_ids.iter().copied().max().unwrap_or(0) + 999_999;
    // Id bia NAM GIUA hai id that -- doi chung "phan loai tung id", khong chi "danh sach dau
    // hay cuoi bi tu choi".
    let mixed = [real_ids[0], bogus_id, real_ids[1]];

    let record = fresh_record();
    let err = expect_prepare_batch_err(
        prepare_batch_call(Some(&global), Some(&open), &record, Some("Plain"), &mixed),
        "mot id khong thuoc Chuong dang mo phai la mot Err -- ca cho MOT LO",
    );

    assert_eq!(err.code(), "ai_prompt.segment_not_in_chapter");
    assert_eq!(err.message_key(), MessageKey::AiPromptSegmentNotInChapter);
    assert_eq!(err.params().get("segment_id"), Some(&bogus_id.to_string()));
    assert!(!err.retryable());
    assert!(
        read_last_assembled_prompt(&record).is_none(),
        "tu choi TRUOC khi bat ky prompt nao trong lo duoc lap -- 0 luot ghi mot phan"
    );

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

/// I/O Matrix "Keychain refuses to answer" (áp cho LÔ) -- cùng khoá
/// `AiConfigKeychainUnavailable` mà lượt dịch MỘT segment dùng
/// (`keychain_refusing_to_answer_surfaces_the_existing_ai_config_keychain_unavailable_key`). Ca
/// đó chỉ canh nhánh ĐƠN -- ca dưới đây là nhánh LÔ, đúng dòng `Err(_unavailable) => return
/// Err(keychain_unavailable())` bên trong khối "đọc khoá MỘT LẦN cho cả lô, CHỈ KHI ít nhất
/// một hàng cần dịch". Một hàng bị cắt (`is_omitted`), một hàng còn lại CẦN dịch -- đúng điều
/// kiện `needs_translation` mà nhánh này canh, không phải "mọi hàng đều cần dịch".
#[test]
fn keychain_refusing_to_answer_on_a_batch_surfaces_the_existing_key_when_at_least_one_row_still_needs_translation()
 {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();

    let global_dir = temp_dir("batch-keychain-unavailable-global");
    let work_dir = temp_dir("batch-keychain-unavailable-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "BatchKeychainUnavailable", "en", "Sentence one. Sentence two.");
    write_field(&global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");

    let chapter = read_open_chapter_segments(Some(&open)).expect("nap chuong");
    assert_eq!(chapter.segments.len(), 2, "fixture 2 cau phai tach thanh 2 segment");
    let all_ids: Vec<i64> = chapter.segments.iter().map(|s| s.id).collect();
    set_segment_omitted(Some(&open), all_ids[0], true).expect("dat co cat bo that bai");

    inject_one_shot_keychain_error();

    let record = fresh_record();
    let err = expect_prepare_batch_err(
        prepare_batch_call(Some(&global), Some(&open), &record, Some("Plain"), &all_ids),
        "keychain tu choi tra loi phai la mot Err -- ca cho MOT LO",
    );

    assert_eq!(err.message_key(), MessageKey::AiConfigKeychainUnavailable);
    assert!(err.retryable(), "mot keychain bi khoa/tu choi quyen co the thanh cong o luot bam lai");
    assert!(
        err.params().values().all(|v| v != "sk-decoy" && !v.contains("Bearer")),
        "gia tri khoa KHONG BAO GIO duoc lot vao tham so loi"
    );
    assert!(
        read_last_assembled_prompt(&record).is_none(),
        "khoa doc TRUOC lop lap prompt dau tien -- 0 luot ghi mot phan khi keychain tu choi"
    );

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

// ───────────────────────────────────────────────────────────────────────────────────
// 5b. `run_batch_call` + `MultiItemProvider` -- seam ASYNC, `Channel` THẬT
// ───────────────────────────────────────────────────────────────────────────────────

/// I/O Matrix "Batch over a selection" (nửa CHẢY) -- 5 câu, mỗi sự kiện mang ĐÚNG
/// `segment_id` của câu nó thuộc về, tới THEO THỨ TỰ tài liệu, outcome `Done`.
#[test]
fn batch_over_a_selection_streams_each_sentence_in_document_order_and_finishes_done() {
    let ids = [601_i64, 602, 603, 604, 605];
    let items: Vec<PreparedBatchItem> =
        ids.iter().map(|id| dummy_batch_item(*id, &format!("Prompt for {id}"))).collect();
    let recipes = ids
        .iter()
        .map(|id| MultiItemRecipe {
            tokens: vec![Box::leak(format!("token-{id}").into_boxed_str())],
            finish: FakeFinish::Done,
            trigger_cancel_after_token: None,
        })
        .collect();
    let provider = MultiItemProvider {
        recipes,
        call_index: AtomicUsize::new(0),
        call_count: AtomicUsize::new(0),
        cancel_flag: Arc::new(AtomicBool::new(false)),
    };
    let (channel, received) = collecting_batch_channel();
    let should_cancel = || false;

    let outcome = tauri::async_runtime::block_on(run_batch_call(&provider, &items, &channel, &should_cancel))
        .expect("khong duoc loi");
    assert_eq!(outcome, AiTranslateBatchOutcome::Done);
    assert_eq!(provider.call_count.load(Ordering::SeqCst), 5);

    let got = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    let mut expected = Vec::new();
    for id in ids {
        expected.push(BatchEventShape::Token { segment_id: id, text: format!("token-{id}") });
        expected.push(BatchEventShape::Done { segment_id: id, usage: None });
    }
    assert_eq!(got, expected, "moi su kien phai mang DUNG segment_id va toi DUNG THU TU tai lieu");
}

/// I/O Matrix "Cancel mid-batch" — batch 12, huỷ giữa lúc câu 6 đang chảy: phần dở của câu 6
/// bị bỏ (không `Done` cho nó), câu 1-5 giữ kết quả, câu 7-12 KHÔNG BAO GIỜ được gọi (đo bằng
/// `call_count` VÀ bằng việc `MultiItemProvider` chỉ cấp đúng 6 công thức -- gọi lần thứ 7 sẽ
/// panic thay vì lặng lẽ trôi qua).
#[test]
fn cancel_while_sentence_six_of_twelve_streams_discards_its_partial_text_keeps_one_through_five_and_never_calls_seven_through_twelve()
 {
    let ids: Vec<i64> = (701..=712).collect();
    let items: Vec<PreparedBatchItem> =
        ids.iter().map(|id| dummy_batch_item(*id, &format!("Prompt for {id}"))).collect();

    let mut recipes: Vec<MultiItemRecipe> = (0..5)
        .map(|i| MultiItemRecipe {
            tokens: vec![Box::leak(format!("token-{}", ids[i]).into_boxed_str())],
            finish: FakeFinish::Done,
            trigger_cancel_after_token: None,
        })
        .collect();
    // Cau thu 6 (chi so 5): gui DUNG mot token ROI bat co huy -- mo phong "nguoi dung bam
    // huy dung luc cau nay dang chay".
    recipes.push(MultiItemRecipe {
        tokens: vec![Box::leak(format!("partial-{}", ids[5]).into_boxed_str())],
        finish: FakeFinish::Done,
        trigger_cancel_after_token: Some(0),
    });
    assert_eq!(recipes.len(), 6, "chi cap cong thuc cho 6 cau DAU -- cau 7-12 khong duoc goi");

    let provider = MultiItemProvider {
        recipes,
        call_index: AtomicUsize::new(0),
        call_count: AtomicUsize::new(0),
        cancel_flag: Arc::new(AtomicBool::new(false)),
    };
    let cancel_flag = Arc::clone(&provider.cancel_flag);
    let should_cancel = move || cancel_flag.load(Ordering::SeqCst);
    let (channel, received) = collecting_batch_channel();

    let outcome = tauri::async_runtime::block_on(run_batch_call(&provider, &items, &channel, &should_cancel))
        .expect("huy giua chung phai la Ok(Cancelled), khong phai mot Err");
    assert_eq!(outcome, AiTranslateBatchOutcome::Cancelled);
    assert_eq!(
        provider.call_count.load(Ordering::SeqCst),
        6,
        "provider chi duoc goi dung 6 lan -- cau 7-12 khong bao gio duoc goi"
    );

    let got = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    let mut expected = Vec::new();
    for i in 0..5 {
        expected.push(BatchEventShape::Token { segment_id: ids[i], text: format!("token-{}", ids[i]) });
        expected.push(BatchEventShape::Done { segment_id: ids[i], usage: None });
    }
    expected.push(BatchEventShape::Token { segment_id: ids[5], text: format!("partial-{}", ids[5]) });
    assert_eq!(
        got, expected,
        "cau 1-5 giu ket qua DA XONG (Token+Done); cau 6 chi co Token cua phan da nhan TRUOC \
         khi huy, KHONG mot Done nao cho no; khong su kien nao cho cau 7-12"
    );
}

/// I/O Matrix "Error mid-batch" — provider lỗi ở câu 6/12: batch DỪNG NGAY, đặt tên đúng câu
/// (`segment_id` trong `Err`), câu 1-5 giữ kết quả, câu 7-12 không bao giờ được gọi.
#[test]
fn provider_error_on_sentence_six_of_twelve_stops_the_batch_names_it_and_never_calls_seven_through_twelve()
 {
    let ids: Vec<i64> = (801..=812).collect();
    let items: Vec<PreparedBatchItem> =
        ids.iter().map(|id| dummy_batch_item(*id, &format!("Prompt for {id}"))).collect();

    let mut recipes: Vec<MultiItemRecipe> = (0..5)
        .map(|i| MultiItemRecipe {
            tokens: vec![Box::leak(format!("token-{}", ids[i]).into_boxed_str())],
            finish: FakeFinish::Done,
            trigger_cancel_after_token: None,
        })
        .collect();
    recipes.push(MultiItemRecipe {
        tokens: vec!["boom"],
        finish: FakeFinish::Err(FakeProviderError("dropped".to_owned())),
        trigger_cancel_after_token: None,
    });
    assert_eq!(recipes.len(), 6, "chi cap cong thuc cho 6 cau DAU -- cau 7-12 khong duoc goi");

    let provider = MultiItemProvider {
        recipes,
        call_index: AtomicUsize::new(0),
        call_count: AtomicUsize::new(0),
        cancel_flag: Arc::new(AtomicBool::new(false)),
    };
    let should_cancel = || false;
    let (channel, received) = collecting_batch_channel();

    let (failed_segment_id, err) =
        tauri::async_runtime::block_on(run_batch_call(&provider, &items, &channel, &should_cancel))
            .expect_err("mot loi provider phai truyen nguyen qua run_batch_call");

    assert_eq!(failed_segment_id, ids[5], "loi phai dat DUNG TEN cau ma provider tra loi that bai");
    assert_eq!(err, FakeProviderError("dropped".to_owned()));
    assert_eq!(provider.call_count.load(Ordering::SeqCst), 6);

    let got = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    let mut expected = Vec::new();
    for i in 0..5 {
        expected.push(BatchEventShape::Token { segment_id: ids[i], text: format!("token-{}", ids[i]) });
        expected.push(BatchEventShape::Done { segment_id: ids[i], usage: None });
    }
    expected.push(BatchEventShape::Token { segment_id: ids[5], text: "boom".to_owned() });
    assert_eq!(got, expected, "cau 1-5 giu ket qua, cau 6 chi co Token cua phan da nhan, khong Done");
}

/// I/O Matrix "Error with zero tokens received" (nhánh LÔ) — cùng lỗ §Coverage spec 4.10 nêu
/// tên cho nhánh ĐƠN (`a_provider_error_with_zero_tokens_received_...` ở §3), áp cho
/// `run_batch_call`: câu 6/12 trượt TRƯỚC khi gửi bất kỳ token nào -- không một `Token` nào cho
/// câu đó, chỉ dừng lô ngay (không `Done`). Câu 1-5 vẫn giữ kết quả, câu 7-12 chưa từng được
/// gọi -- cùng phép đo `call_count` các ca "Error mid-batch"/"Cancel mid-batch" ngay trên dùng.
///
/// 🔵 THÊM 2026-09-22 (Story 4.10, Phase 3 — nửa MỚI của Task 9).
#[test]
fn provider_error_with_zero_tokens_received_on_a_batch_sentence_leaves_no_event_for_that_sentence_and_stops_the_batch()
 {
    let ids: Vec<i64> = (901..=912).collect();
    let items: Vec<PreparedBatchItem> =
        ids.iter().map(|id| dummy_batch_item(*id, &format!("Prompt for {id}"))).collect();

    let mut recipes: Vec<MultiItemRecipe> = (0..5)
        .map(|i| MultiItemRecipe {
            tokens: vec![Box::leak(format!("token-{}", ids[i]).into_boxed_str())],
            finish: FakeFinish::Done,
            trigger_cancel_after_token: None,
        })
        .collect();
    // Cau thu 6: KHONG mot token nao truoc khi loi.
    recipes.push(MultiItemRecipe {
        tokens: vec![],
        finish: FakeFinish::Err(FakeProviderError("dropped-before-first-token".to_owned())),
        trigger_cancel_after_token: None,
    });
    assert_eq!(recipes.len(), 6, "chi cap cong thuc cho 6 cau DAU -- cau 7-12 khong duoc goi");

    let provider = MultiItemProvider {
        recipes,
        call_index: AtomicUsize::new(0),
        call_count: AtomicUsize::new(0),
        cancel_flag: Arc::new(AtomicBool::new(false)),
    };
    let should_cancel = || false;
    let (channel, received) = collecting_batch_channel();

    let (failed_segment_id, err) =
        tauri::async_runtime::block_on(run_batch_call(&provider, &items, &channel, &should_cancel))
            .expect_err("mot loi provider TRUOC token dau tien van phai la Err");

    assert_eq!(failed_segment_id, ids[5], "loi phai dat DUNG TEN cau ma provider tra loi that bai");
    assert_eq!(err, FakeProviderError("dropped-before-first-token".to_owned()));
    assert_eq!(provider.call_count.load(Ordering::SeqCst), 6);

    let got = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    let mut expected = Vec::new();
    for i in 0..5 {
        expected.push(BatchEventShape::Token { segment_id: ids[i], text: format!("token-{}", ids[i]) });
        expected.push(BatchEventShape::Done { segment_id: ids[i], usage: None });
    }
    // Cau 6: KHONG mot su kien nao -- khong Token (khong token nao tung gui), khong Done.
    assert_eq!(
        got, expected,
        "cau 1-5 giu ket qua DA XONG; cau 6 KHONG mot su kien nao (loi truoc token dau tien)"
    );
}

/// I/O Matrix "Error mid-batch" — cột Error-Handling, phần Phase 4a KHÔNG canh được
/// (`batch_stopped_error` từng `private`, xem doc-comment tại nguồn). Ca ngay TRÊN chỉ đo được
/// `run_batch_call` trả `Err((segment_id, P::Error))` với `P::Error` GIẢ (`FakeProviderError`)
/// -- không một ca nào gọi ĐÚNG hàm đúc `IpcError` thật (`OpenAiClientError` thật) cho tới bản
/// sửa này. Hai biến thể, đúng hai hàng đã có tên trong
/// `every_openai_client_error_variant_maps_to_its_own_family_key_...` ở trên (`NonSuccessStatus`
/// ⇒ không thử lại, `StreamEndedWithoutDone` ⇒ có thể thử lại) -- cùng phân loại VÀ cùng khoá họ
/// nguyên nhân mà lượt dịch MỘT segment dùng (Quyết định 2, spec 4.10: "No per-path duplicate of
/// the family"), chỉ mang thêm `segment_id` -- đúng câu batch dừng ở đó.
///
/// 🔵 SỬA 2026-09-22 (Story 4.10, Phase 1) — trước bản sửa này ca này canh một NHÃN GỘP RIÊNG
/// cho đường batch (`AiTranslateBatchStopped`, mang `segment_id` BẮT BUỘC); Quyết định 2 gỡ khoá
/// đó, batch dùng CHUNG sáu khoá họ với lượt dịch MỘT segment. `status` (ca non-2xx) giờ là tham
/// số bắt buộc của `AiTranslateProviderRefused` và PHẢI có mặt — đây cũng là phép sửa "cho
/// `batch_stopped_error` cái `{status}` nó đánh rơi hôm nay" mà Task 4 spec 4.10 đòi.
///
/// 🔵 SỬA 2026-09-22 (Story 4.10, Phase 3 — nửa MỚI của Task 9: "extend `:1733` to every
/// variant"). Phase 1 chỉ canh HAI trong tám biến thể (đúng hai hàng §Coverage spec 4.10 nêu tên
/// đã đo được lỗ: "`:1733` checks only two of the seven variants for batch"). Bản sửa này mở
/// rộng thành ĐỦ TÁM — cùng khuôn ĐÚNG mảng `every_openai_client_error_variant_maps_to_its_own_
/// family_key_with_the_documented_retryable_flag` ở §4a (nhánh ĐƠN), chỉ khác chỗ gọi
/// (`batch_stopped_error(segment_id, variant)` thay vì `variant.into()`) và hai khẳng định thêm
/// mọi biến thể đều PHẢI mang đúng `segment_id`, còn `status` chỉ có ở đúng một biến thể
/// (`NonSuccessStatus`) — hai mảng này PHẢI đứng cạnh nhau khi đọc, không phải hai bản chép độc
/// lập: một biến thể mới thêm vào `OpenAiClientError` mà chỉ một trong hai mảng cập nhật sẽ để
/// lộ lệch ở CHÍNH bản sửa thêm biến thể đó, không phải ở đây.
#[test]
fn batch_stopped_error_names_the_sentence_and_maps_the_documented_retryable_flag() {
    const SEGMENT_ID: i64 = 850;
    let cases: [(OpenAiClientError, &str, MessageKey, bool); 8] = [
        (
            OpenAiClientError::ClientBuildFailed { detail: "x".to_owned() },
            "ai_translate.client_build_failed",
            MessageKey::AiTranslateClientBuildFailed,
            false,
        ),
        (
            OpenAiClientError::RequestFailed { detail: "x".to_owned() },
            "ai_translate.provider_unreachable",
            MessageKey::AiTranslateProviderUnreachable,
            true,
        ),
        (
            OpenAiClientError::NonSuccessStatus { status: 500 },
            "ai_translate.provider_refused",
            MessageKey::AiTranslateProviderRefused,
            false,
        ),
        (
            OpenAiClientError::ReadFailed { detail: "x".to_owned() },
            "ai_translate.provider_unreachable",
            MessageKey::AiTranslateProviderUnreachable,
            true,
        ),
        (
            OpenAiClientError::StreamEndedWithoutDone,
            "ai_translate.stream_ended_without_done",
            MessageKey::AiTranslateStreamEndedWithoutDone,
            true,
        ),
        (
            OpenAiClientError::MalformedEvent { detail: "x".to_owned() },
            "ai_translate.reply_unreadable",
            MessageKey::AiTranslateReplyUnreadable,
            false,
        ),
        (
            OpenAiClientError::BufferOverflow { size: 2 * 1024 * 1024 },
            "ai_translate.reply_unreadable",
            MessageKey::AiTranslateReplyUnreadable,
            false,
        ),
        (
            OpenAiClientError::ApiKeyHeaderInvalid { detail: "x".to_owned() },
            "ai_translate.api_key_header_invalid",
            MessageKey::AiTranslateApiKeyHeaderInvalid,
            false,
        ),
    ];
    for (variant, expected_code, expected_key, expected_retryable) in cases {
        let label = format!("{variant:?}");
        let is_non_2xx = matches!(variant, OpenAiClientError::NonSuccessStatus { .. });
        let err = batch_stopped_error(SEGMENT_ID, variant);
        assert_eq!(err.code(), expected_code, "{label}");
        assert_eq!(err.message_key(), expected_key, "{label}");
        assert_eq!(
            err.params().get("segment_id"),
            Some(&SEGMENT_ID.to_string()),
            "{label}: segment_id phai co mat o MOI bien the, khong chi hai bien the cu"
        );
        if is_non_2xx {
            assert_eq!(err.params().get("status"), Some(&"500".to_owned()), "{label}: status khong con bi rot");
        } else {
            assert!(
                err.params().get("status").is_none(),
                "{label}: chi NonSuccessStatus moi mang status"
            );
        }
        assert_eq!(
            err.retryable(),
            expected_retryable,
            "{label} phai anh xa retryable={expected_retryable}, cung phan loai voi loi don"
        );
    }
}

/// I/O Matrix "Omitted segment inside the selection" (nửa CHẢY) — segment 3/5 mang
/// `Omitted`: một sự kiện `Skipped` gửi cho ĐÚNG segment đó, KHÔNG một lời gọi provider nào
/// (đo bằng `call_count` == 4, không phải 5), và lô tiếp tục sang câu 4.
#[test]
fn an_omitted_segment_sends_a_skipped_event_with_no_provider_call_and_the_batch_continues() {
    let ids = [901_i64, 902, 903, 904, 905];
    let items: Vec<PreparedBatchItem> = vec![
        dummy_batch_item(ids[0], "Prompt 1"),
        dummy_batch_item(ids[1], "Prompt 2"),
        PreparedBatchItem::Omitted { segment_id: ids[2] },
        dummy_batch_item(ids[3], "Prompt 4"),
        dummy_batch_item(ids[4], "Prompt 5"),
    ];
    let recipes: Vec<MultiItemRecipe> = [ids[0], ids[1], ids[3], ids[4]]
        .iter()
        .map(|id| MultiItemRecipe {
            tokens: vec![Box::leak(format!("token-{id}").into_boxed_str())],
            finish: FakeFinish::Done,
            trigger_cancel_after_token: None,
        })
        .collect();
    let provider = MultiItemProvider {
        recipes,
        call_index: AtomicUsize::new(0),
        call_count: AtomicUsize::new(0),
        cancel_flag: Arc::new(AtomicBool::new(false)),
    };
    let should_cancel = || false;
    let (channel, received) = collecting_batch_channel();

    let outcome = tauri::async_runtime::block_on(run_batch_call(&provider, &items, &channel, &should_cancel))
        .expect("khong duoc loi");
    assert_eq!(outcome, AiTranslateBatchOutcome::Done);
    assert_eq!(
        provider.call_count.load(Ordering::SeqCst),
        4,
        "segment bi cat KHONG duoc goi provider -- chi 4/5 cau that su goi provider"
    );

    let got = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    let expected = vec![
        BatchEventShape::Token { segment_id: ids[0], text: format!("token-{}", ids[0]) },
        BatchEventShape::Done { segment_id: ids[0], usage: None },
        BatchEventShape::Token { segment_id: ids[1], text: format!("token-{}", ids[1]) },
        BatchEventShape::Done { segment_id: ids[1], usage: None },
        BatchEventShape::Skipped { segment_id: ids[2] },
        BatchEventShape::Token { segment_id: ids[3], text: format!("token-{}", ids[3]) },
        BatchEventShape::Done { segment_id: ids[3], usage: None },
        BatchEventShape::Token { segment_id: ids[4], text: format!("token-{}", ids[4]) },
        BatchEventShape::Done { segment_id: ids[4], usage: None },
    ];
    assert_eq!(got, expected, "Skipped phai toi DUNG vi tri, lo phai tiep tuc sang cau ke tiep");
}

/// Counter-check bằng gỡ của "between-sentence `should_cancel()`" (`aitranslate.rs:548`,
/// §Tasks spec 4.9's Phase 4 cuối cùng) — huỷ xảy ra ĐÚNG lúc câu 1 vừa `Done` và câu 2 CHƯA
/// bắt đầu: không một khung SSE nào đang chảy lúc đó để mà cơ chế huỷ-giữa-khung (đã có từ
/// Story 4.8, không đổi ở đây) bắt được. Nếu vòng lặp không tự hỏi `should_cancel()` GIỮA HAI
/// CÂU, câu 3 sẽ vẫn bị gọi dù người dùng đã bấm huỷ -- đúng câu doc-comment `run_batch_call`
/// tự nêu. Cờ huỷ được bật NGAY TRONG hook của `Channel` thật, đúng lúc sự kiện `Done` của
/// câu 2 được gửi -- không một `should_cancel()` nào bên trong `provider.translate` của câu 2
/// còn dịp đọc thấy cờ TRƯỚC khi nó được bật (nó đã trả `Done` va roi khoi ham TRUOC khi hook
/// chay), nên chỉ cơ chế GIỮA HAI CÂU của `run_batch_call` mới bắt được lượt huỷ này.
#[test]
fn cancelling_exactly_between_two_sentences_never_calls_the_provider_for_the_next_sentence() {
    let ids = [1001_i64, 1002, 1003];
    let items: Vec<PreparedBatchItem> = ids.iter().map(|id| dummy_batch_item(*id, "Prompt")).collect();
    // CHI cap 2 cong thuc -- cau thu 3 KHONG DUOC goi; neu bi goi, `MultiItemProvider` panic
    // vi vuot chi so thay vi lang le sinh su kien.
    let recipes = vec![
        MultiItemRecipe {
            tokens: vec!["token-1"],
            finish: FakeFinish::Done,
            trigger_cancel_after_token: None,
        },
        MultiItemRecipe {
            tokens: vec!["token-2"],
            finish: FakeFinish::Done,
            trigger_cancel_after_token: None,
        },
    ];
    let provider = MultiItemProvider {
        recipes,
        call_index: AtomicUsize::new(0),
        call_count: AtomicUsize::new(0),
        cancel_flag: Arc::new(AtomicBool::new(false)),
    };
    let cancel_flag_for_should_cancel = Arc::clone(&provider.cancel_flag);
    let should_cancel = move || cancel_flag_for_should_cancel.load(Ordering::SeqCst);

    let cancel_flag_for_hook = Arc::clone(&provider.cancel_flag);
    let second_id = ids[1];
    let (channel, received) = collecting_batch_channel_with_hook(move |event| {
        if *event == (BatchEventShape::Done { segment_id: second_id, usage: None }) {
            cancel_flag_for_hook.store(true, Ordering::SeqCst);
        }
    });

    let outcome = tauri::async_runtime::block_on(run_batch_call(&provider, &items, &channel, &should_cancel))
        .expect("huy giua hai cau phai la Ok(Cancelled)");
    assert_eq!(
        outcome,
        AiTranslateBatchOutcome::Cancelled,
        "huy dung luc giua hai cau phai duoc doc lai boi VONG LAP, khong phai boi provider"
    );
    assert_eq!(
        provider.call_count.load(Ordering::SeqCst),
        2,
        "cau thu 3 KHONG DUOC goi provider -- huy da xay ra TRUOC no bat dau, giua cau 1 va cau 2"
    );

    let got = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    let expected = vec![
        BatchEventShape::Token { segment_id: ids[0], text: "token-1".to_owned() },
        BatchEventShape::Done { segment_id: ids[0], usage: None },
        BatchEventShape::Token { segment_id: ids[1], text: "token-2".to_owned() },
        BatchEventShape::Done { segment_id: ids[1], usage: None },
    ];
    assert_eq!(got, expected, "cau 1 va cau 2 hoan tat sach; khong su kien nao cho cau 3");
}
