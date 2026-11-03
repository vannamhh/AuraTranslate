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
    AiTranslateScope, PrepareOperationOutcome, PrepareOutcome, PreparedTranslateCall, SingleRun,
    SingleRunError, batch_panicked_error, batch_stopped_error, prepare_resend_call,
    prepare_translate_call, prepare_translate_operation, run_translate_call,
};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::promptset::prompt_set_create;
use auratranslate_lib::commands::segment::{
    TRANSLATION_ORIGIN_OTHER, confirm_segment, promote_ai_translation, read_open_chapter_segments,
    set_segment_omitted,
};
use auratranslate_lib::core::ai::batch::system_with_block_format;
use auratranslate_lib::core::ai::client::{
    ChunkUsage, OpenAiClientError, SseChunk, SseEventOutcome, build_request_body,
    enforce_sse_buffer_cap, interpret_sse_event, split_sse_frames,
};
use auratranslate_lib::core::aiconfig::{AiConfigField, AiConfigTier, write_field};
use auratranslate_lib::core::i18n::{IpcError, MessageKey};
use auratranslate_lib::core::promptset::PromptSetTier;
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};
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
    read_open_chapter_segments(Some(open))
        .expect("nap chuong")
        .segments[0]
        .id
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

/// Same reason as [`expect_prepare_err`]: `PrepareOperationOutcome::Ready` carries the key.
fn expect_prepare_operation_err(
    result: Result<PrepareOperationOutcome, IpcError>,
    msg: &str,
) -> IpcError {
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
            conn.query_row(
                "SELECT translation_origin FROM segment WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
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
    assert!(
        events1.is_empty(),
        "chua du mot su kien tron ven -- 0 su kien"
    );
    assert_eq!(
        remaining1, chunk1,
        "phan con lai phai la TOAN BO khoi dau, chua mat byte nao"
    );

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
    assert!(
        events.is_empty(),
        "chua co dau ngan cach dong trang -- 0 su kien TRON VEN"
    );
    assert_eq!(
        remaining, buf,
        "toan bo buffer o lai cho lan doc ke tiep -- khong byte nao mat"
    );
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
        assert_eq!(
            events,
            vec![String::new()],
            "mot data: rong van la MOT su kien, khong bi nuot"
        );
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
    assert!(
        events.is_empty(),
        "khung chi-chu-thich khong duoc tao mot su kien nao"
    );
    assert!(remaining.is_empty());
}

/// Đối chứng review — bộ đệm tích luỹ CHƯA-ĐỦ-một-sự-kiện bị từ chối một khi vượt
/// [`enforce_sse_buffer_cap`]'s trần, thay vì phình vô hạn khi một endpoint sai hình/ác ý không
/// bao giờ gửi dấu phân cách `\n\n`/`\r\n\r\n`.
#[test]
fn a_buffer_that_never_sees_a_boundary_is_rejected_once_it_crosses_the_cap() {
    let just_under = vec![b'x'; 1024 * 1024];
    assert!(
        enforce_sse_buffer_cap(&just_under).is_ok(),
        "dung tran -- chua duoc tu choi"
    );

    let just_over = vec![b'x'; 1024 * 1024 + 1];
    let err = enforce_sse_buffer_cap(&just_over).expect_err("vuot tran phai la mot Err");
    assert_eq!(
        err,
        OpenAiClientError::BufferOverflow {
            size: just_over.len()
        }
    );
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
        system: None,
    };
    let json = serde_json::to_string(&build_request_body(&request)).expect("serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse lai");
    assert_eq!(
        value["stream_options"]["include_usage"],
        serde_json::json!(true)
    );
}

/// 🔴 Counter-check trực tiếp cho khuyết tật nêu ở §Code Map spec 4.11 — một khung `usage`
/// mang `choices: []` (hình dạng THẬT của khung usage cuối của một provider tương thích
/// OpenAI, `stream_options.include_usage == true`) phải cho ra một `SseEventOutcome::Chunk` mang `usage`,
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
        SseEventOutcome::Chunk(SseChunk {
            text: None,
            usage: Some(ChunkUsage {
                prompt_tokens: Some(100),
                completion_tokens: Some(312),
                total_tokens: Some(412),
            }),
            finish_reason: None,
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
        SseEventOutcome::Chunk(SseChunk {
            text: None,
            usage: Some(ChunkUsage {
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: Some(412)
            }),
            finish_reason: None,
        }),
        "usage MOT PHAN van la Usage that -- total_tokens=412 co nghia, hai truong con lai None"
    );
}

/// 🔴 Rà soát coordinator — `ChunkUsage::into_translate_usage` không được đúc một chi phí BỊA
/// từ hai trường VẮNG MẶT. Cùng khung `{"total_tokens":412}` ở ca ngay trên: token count 412
/// phải lên màn hình (thật), nhưng `cost_usd` PHẢI là `None` (không phải một `Some(0.0)` giả —
/// công thức giá cần CẢ HAI `prompt_tokens`/`completion_tokens`, và ở đây cả hai đều không
/// biết, không phải bằng `0`).
#[test]
fn into_translate_usage_does_not_fabricate_a_cost_when_the_prompt_and_completion_split_is_unknown()
{
    let partial = ChunkUsage {
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: Some(412),
    };
    let usage = partial.into_translate_usage("claude-sonnet-5");
    assert_eq!(
        usage.total_tokens, 412,
        "so token THAT phai len man hinh du thieu chi tiet vao/ra"
    );
    assert_eq!(
        usage.cost_usd, None,
        "khong du CA HAI prompt_tokens/completion_tokens thi KHONG duoc tinh gia -- None, khong phai Some(0.0) gia"
    );
}

/// Ca ĐỐI CHỨNG DƯƠNG cạnh ca trên — khi CẢ HAI `prompt_tokens`/`completion_tokens` đều có
/// mặt (dù `total_tokens` vắng mặt, một hình dạng hợp lệ khác), giá VẪN được tính, và
/// `total_tokens` rơi về đúng tổng hai chiều đã biết.
#[test]
fn into_translate_usage_computes_a_cost_and_falls_back_to_the_sum_when_total_tokens_is_absent_but_both_parts_are_known()
 {
    let usage = ChunkUsage {
        prompt_tokens: Some(100),
        completion_tokens: Some(312),
        total_tokens: None,
    }
    .into_translate_usage("claude-sonnet-5");
    assert_eq!(
        usage.total_tokens, 412,
        "total_tokens vang mat -- roi ve dung tong hai chieu da biet"
    );
    assert!(
        usage.cost_usd.is_some(),
        "ca hai chieu deu biet -- gia PHAI duoc tinh"
    );
}

/// Hình Gemini: một khung mang CẢ `delta.content` LẪN `usage` phải giữ cả hai trong cùng một
/// `Chunk`, chữ không bị bỏ.
#[test]
fn a_frame_carrying_both_usage_and_delta_content_keeps_the_text_and_the_usage() {
    let both = r#"{"choices":[{"delta":{"content":"Xin"}}],"usage":{"total_tokens":412}}"#;
    let outcome = interpret_sse_event(both).expect("khong duoc la Err");
    assert_eq!(
        outcome,
        SseEventOutcome::Chunk(SseChunk {
            text: Some("Xin".to_owned()),
            usage: Some(ChunkUsage {
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: Some(412)
            }),
            finish_reason: None,
        }),
        "khung co ca chu lan usage phai giu CA HAI"
    );
}

/// Nhiều khung Gemini liên tiếp (mỗi khung chữ + usage) cho ra từng đoạn chữ đúng thứ tự, mỗi
/// khung kèm usage của nó.
#[test]
fn consecutive_gemini_frames_each_yield_their_own_text_in_order_with_usage() {
    let frames = [
        (
            r#"{"choices":[{"delta":{"content":"Xin "}}],"usage":{"total_tokens":10}}"#,
            "Xin ",
            10,
        ),
        (
            r#"{"choices":[{"delta":{"content":"chao"}}],"usage":{"total_tokens":11}}"#,
            "chao",
            11,
        ),
        (
            r#"{"choices":[{"delta":{"content":"!"}}],"usage":{"total_tokens":12}}"#,
            "!",
            12,
        ),
    ];
    let mut joined = String::new();
    let mut last_total = None;
    for (frame, text, total) in frames {
        match interpret_sse_event(frame).expect("khung hop le") {
            SseEventOutcome::Chunk(chunk) => {
                assert_eq!(chunk.text.as_deref(), Some(text));
                joined.push_str(&chunk.text.expect("text"));
                last_total = chunk.usage.and_then(|u| u.total_tokens);
                assert_eq!(last_total, Some(total));
            }
            other => panic!("mong Chunk, nhan {other:?}"),
        }
    }
    assert_eq!(joined, "Xin chao!");
    assert_eq!(last_total, Some(12));
}

/// `finish_reason` nằm ở khung KHÔNG có chữ (khung cuối) vẫn được đọc ra, cùng `text: None`.
#[test]
fn a_finish_reason_on_a_frame_without_text_is_read_with_no_text() {
    let frame = r#"{"choices":[{"delta":{},"finish_reason":"content_filter"}]}"#;
    let outcome = interpret_sse_event(frame).expect("khong duoc la Err");
    assert_eq!(
        outcome,
        SseEventOutcome::Chunk(SseChunk {
            text: None,
            usage: None,
            finish_reason: Some("content_filter".to_owned()),
        })
    );
}

/// Khung `role` thuần (không chữ, không usage, không lý do) vẫn là `Ignore`.
#[test]
fn a_role_only_frame_is_still_ignored() {
    let frame = r#"{"choices":[{"delta":{"role":"assistant"}}]}"#;
    assert_eq!(
        interpret_sse_event(frame).expect("khong duoc la Err"),
        SseEventOutcome::Ignore
    );
}

/// Một khung giữa dòng (không usage, `delta.content` có mặt) vẫn cho `Chunk` chỉ có chữ như
/// trước.
#[test]
fn a_token_chunk_without_usage_still_reads_as_text_only() {
    let token_chunk = r#"{"choices":[{"delta":{"content":"Xin"}}]}"#;
    let outcome = interpret_sse_event(token_chunk).expect("khung token hop le khong duoc la Err");
    assert_eq!(
        outcome,
        SseEventOutcome::Chunk(SseChunk {
            text: Some("Xin".to_owned()),
            usage: None,
            finish_reason: None,
        })
    );
}

/// `core::ai::pricing::estimate_cost_usd` — mô hình CÓ hàng trong bảng giá trả `Some`, tính
/// đúng công thức (giá/triệu token × số token / 1_000_000, cộng hai chiều vào/ra).
#[test]
fn estimate_cost_usd_computes_a_price_for_a_seeded_model_id() {
    let cost = auratranslate_lib::core::ai::pricing::estimate_cost_usd("claude-sonnet-5", 100, 312)
        .expect("claude-sonnet-5 phai co hang trong bang gia");
    let expected = (100.0 / 1_000_000.0 * 2.0) + (312.0 / 1_000_000.0 * 10.0);
    assert!(
        (cost - expected).abs() < 1e-12,
        "cong thuc gia phai dung: got={cost}, expected={expected}"
    );
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
    assert_eq!(
        auratranslate_lib::core::ai::pricing::estimate_cost_usd("llama3", 100, 312),
        None
    );
    assert_eq!(
        auratranslate_lib::core::ai::pricing::estimate_cost_usd("qwen2.5:14b", 100, 312),
        None
    );
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
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-ac1-test-key");

    let global_dir = temp_dir("ac1-global");
    let work_dir = temp_dir("ac1-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "AC1", "en", "A dragon roared.");

    write_field(
        &global,
        AiConfigField::Endpoint,
        "https://api.example.invalid/v1/chat/completions",
    )
    .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "Plain",
        "{{source_segment}}",
    )
    .expect("tao bo prompt");

    let segment_id = first_segment_id(&open);
    let record = fresh_record();

    let outcome = prepare_translate_call(
        Some(&global),
        Some(&open),
        &record,
        Some("Plain"),
        segment_id,
    )
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
    assert_eq!(
        prepared.endpoint,
        "https://api.example.invalid/v1/chat/completions"
    );
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
    let outcome = prepare_translate_call(
        Some(&global),
        Some(&open),
        &record,
        Some("Plain"),
        segment_id,
    )
    .expect("khong duoc la mot Err -- 'chua cau hinh' la mot TRANG THAI");
    assert!(matches!(outcome, PrepareOutcome::NotConfigured));
    assert!(
        read_last_assembled_prompt(&record).is_none(),
        "producer cua 4.7 KHONG duoc goi khi chua cau hinh -- khong ban ghi nao duoc tao"
    );

    // Nua ca thu hai -- endpoint co, model rong: cung mot trang thai, khong phai mot Err rieng.
    write_field(
        &global,
        AiConfigField::Endpoint,
        "https://api.example.invalid/v1/chat/completions",
    )
    .expect("ghi endpoint");
    let outcome2 = prepare_translate_call(
        Some(&global),
        Some(&open),
        &record,
        Some("Plain"),
        segment_id,
    )
    .expect("khong duoc la mot Err");
    assert!(
        matches!(outcome2, PrepareOutcome::NotConfigured),
        "model van rong -- van chua cau hinh"
    );

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
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();

    let global_dir = temp_dir("not-configured-no-key-global");
    let work_dir = temp_dir("not-configured-no-key-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "NotConfiguredNoKey", "en", "A dragon roared.");
    write_field(
        &global,
        AiConfigField::Endpoint,
        "https://api.example.invalid/v1/chat/completions",
    )
    .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");

    let segment_id = first_segment_id(&open);
    let record = fresh_record();
    let outcome = prepare_translate_call(
        Some(&global),
        Some(&open),
        &record,
        Some("Plain"),
        segment_id,
    )
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
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();

    let global_dir = temp_dir("keychain-unavailable-global");
    let work_dir = temp_dir("keychain-unavailable-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "KeychainUnavailable", "en", "A dragon roared.");
    write_field(
        &global,
        AiConfigField::Endpoint,
        "https://api.example.invalid/v1/chat/completions",
    )
    .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");

    inject_one_shot_keychain_error();

    let segment_id = first_segment_id(&open);
    let record = fresh_record();
    let err = expect_prepare_err(
        prepare_translate_call(
            Some(&global),
            Some(&open),
            &record,
            Some("Plain"),
            segment_id,
        ),
        "keychain tu choi tra loi phai la mot Err",
    );

    assert_eq!(err.message_key(), MessageKey::AiConfigKeychainUnavailable);
    assert!(
        err.retryable(),
        "mot keychain bi khoa/tu choi quyen co the thanh cong o luot bam lai"
    );
    assert!(
        err.params()
            .values()
            .all(|v| v != "sk-decoy" && !v.contains("Bearer")),
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
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-omitted-test-key");

    let global_dir = temp_dir("omitted-global");
    let work_dir = temp_dir("omitted-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Omitted", "en", "A dragon roared.");
    write_field(
        &global,
        AiConfigField::Endpoint,
        "https://api.example.invalid/v1/chat/completions",
    )
    .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "Plain",
        "{{source_segment}}",
    )
    .expect("tao bo prompt");

    let segment_id = first_segment_id(&open);
    set_segment_omitted(Some(&open), segment_id, true).expect("dat co cat bo that bai");

    let record = fresh_record();
    let err = expect_prepare_err(
        prepare_translate_call(
            Some(&global),
            Some(&open),
            &record,
            Some("Plain"),
            segment_id,
        ),
        "segment is_omitted phai bi tu choi",
    );

    assert_eq!(err.code(), "ai_translate.segment_omitted");
    assert_eq!(err.message_key(), MessageKey::AiTranslateSegmentOmitted);
    assert_eq!(
        err.params().get("segment_id"),
        Some(&segment_id.to_string())
    );
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
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-not-in-chapter-test-key");

    let global_dir = temp_dir("not-in-chapter-global");
    let work_dir = temp_dir("not-in-chapter-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "NotInChapter", "en", "A dragon roared.");
    write_field(
        &global,
        AiConfigField::Endpoint,
        "https://api.example.invalid/v1/chat/completions",
    )
    .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "Plain",
        "{{source_segment}}",
    )
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
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-decision6-test-key");

    let global_dir = temp_dir("decision6-global");
    let work_dir = temp_dir("decision6-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "Decision6", "en", "A dragon roared.");
    write_field(
        &global,
        AiConfigField::Endpoint,
        "https://api.example.invalid/v1/chat/completions",
    )
    .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    // Temperature/MaxTokens CÓ Ý không ghi gì -- đúng trạng thái "chưa đặt" thật, cùng
    // `aiConfigState.ts:69`'s giá trị khởi tạo rỗng.
    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "Plain",
        "{{source_segment}}",
    )
    .expect("tao bo prompt");

    let segment_id = first_segment_id(&open);
    let record = fresh_record();
    let outcome = prepare_translate_call(
        Some(&global),
        Some(&open),
        &record,
        Some("Plain"),
        segment_id,
    )
    .expect("prepare khong duoc loi");
    match outcome {
        PrepareOutcome::Ready(p) => {
            assert_eq!(
                p.temperature, None,
                "chua dat phai la None, khong mot so duc san thay ho"
            );
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

#[test]
fn an_explicit_default_at_either_tier_leaves_temperature_and_max_tokens_unsent() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-explicit-default-test-key");

    let global_dir = temp_dir("explicit-default-global");
    let work_dir = temp_dir("explicit-default-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "ExplicitDefault", "en", "A dragon roared.");
    write_field(
        &global,
        AiConfigField::Endpoint,
        "https://api.example.invalid/v1/chat/completions",
    )
    .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    write_field(&global, AiConfigField::Temperature, "0.3").expect("ghi temperature");
    write_field(&global, AiConfigField::MaxTokens, "4096").expect("ghi max_tokens");
    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "Plain",
        "{{source_segment}}",
    )
    .expect("tao bo prompt");

    let segment_id = first_segment_id(&open);
    let record = fresh_record();
    let prepared = |label: &str| match prepare_translate_call(
        Some(&global),
        Some(&open),
        &record,
        Some("Plain"),
        segment_id,
    )
    .expect("prepare khong duoc loi")
    {
        PrepareOutcome::Ready(p) => (p.temperature, p.max_tokens),
        PrepareOutcome::NotConfigured => panic!("{label}: khong duoc la chua cau hinh"),
    };

    assert_eq!(
        prepared("doi chung so"),
        (Some(0.3), Some(4096)),
        "doi chung: so cua Toan cuc phai den duoc lenh goi"
    );

    write_field(&open.store, AiConfigField::Temperature, "default").expect("ghi default Work");
    write_field(&open.store, AiConfigField::MaxTokens, "default").expect("ghi default Work");
    assert_eq!(
        prepared("default o tang Work"),
        (None, None),
        "default o tang Work phai de len so cua Toan cuc"
    );

    write_field(&open.store, AiConfigField::Temperature, "0.9").expect("ghi so Work");
    write_field(&open.store, AiConfigField::MaxTokens, "512").expect("ghi so Work");
    write_field(&global, AiConfigField::Temperature, "default").expect("ghi default Global");
    write_field(&global, AiConfigField::MaxTokens, "default").expect("ghi default Global");
    assert_eq!(
        prepared("so o tang Work tren default Toan cuc"),
        (Some(0.9), Some(512))
    );

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
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-two-producer-test-key");

    let global_dir = temp_dir("two-producer-global");
    let work_dir = temp_dir("two-producer-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "TwoProducer", "en", "A dragon roared.");
    write_field(
        &global,
        AiConfigField::Endpoint,
        "https://api.example.invalid/v1/chat/completions",
    )
    .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "Plain",
        "{{source_segment}}",
    )
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
        prepared.system.as_deref(),
        "gpt-test",
        "2026-09-21T00:00:00.000Z",
        None,
    );
    let wire_translate = read_last_assembled_prompt(&record_translate).expect("phai co ban ghi");

    // Giống hệt -- CHỈ một hàm ghi record (Quyết định 2).
    assert_eq!(
        prepared.prompt, wire_assemble_only.prompt,
        "ca hai duong phai lap DUNG mot prompt"
    );
    assert_eq!(wire_translate.prompt, wire_assemble_only.prompt);
    assert_eq!(wire_translate.segment_id, wire_assemble_only.segment_id);
    assert_eq!(wire_translate.chapter_id, wire_assemble_only.chapter_id);
    assert_eq!(
        wire_translate.prompt_set_name,
        wire_assemble_only.prompt_set_name
    );
    assert_eq!(
        serde_json::to_value(&wire_translate.ledger).expect("serialize ledger 1"),
        serde_json::to_value(&wire_assemble_only.ledger).expect("serialize ledger 2"),
        "ledger phai giong het nhau -- khong mot duong nao lap lai/thay bien"
    );

    // Khác biệt DUY NHẤT được phép: các sự thật ĐÃ GỬI.
    assert_eq!(
        wire_assemble_only.sent_at, None,
        "assemble-only KHONG BAO GIO la mot luot gui"
    );
    assert_eq!(wire_assemble_only.sent_model, None);
    assert!(
        wire_translate.sent_at.is_some(),
        "duong dich, sau khi gui thanh cong, phai co sent_at"
    );
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
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-mid-flight-test-key");

    let global_dir = temp_dir("mid-flight-global");
    let work_dir = temp_dir("mid-flight-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "MidFlight", "en", "A dragon roared.");
    write_field(
        &global,
        AiConfigField::Endpoint,
        "https://api.example.invalid/v1/chat/completions",
    )
    .expect("ghi endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "First",
        "{{source_segment}}",
    )
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
    let prepared = match prepare_translate_call(
        Some(&global),
        Some(&open),
        &record,
        Some("First"),
        segment_id,
    )
    .expect("prepare khong duoc loi")
    {
        PrepareOutcome::Ready(p) => p,
        PrepareOutcome::NotConfigured => panic!("phai san sang"),
    };
    let sent_prompt = prepared.prompt.clone();

    // Lượt 2 -- TRONG LÚC lượt dịch (lượt 1) còn treo, người dùng lắp lại prompt cho CÙNG
    // segment với bộ "Second" -- `assemble_and_record_prompt` GHI ĐÈ đúng bản ghi đó.
    let replaced = assemble_and_record_prompt(
        Some(&global),
        Some(&open),
        &record,
        Some("Second"),
        segment_id,
    )
    .expect("lap rap lai khong duoc loi");
    assert_ne!(
        replaced.prompt, sent_prompt,
        "fixture phai tao ra hai prompt KHAC nhau"
    );

    // Lượt 1 cuối cùng trả `Done` -- chỗ gọi thật (`wire::ai_translate_segment`) truyền đúng
    // chuỗi ĐÃ GỬI của lượt 1, không đọc lại `record` để lấy chuỗi "hiện có".
    mark_prompt_as_sent(
        &record,
        segment_id,
        &sent_prompt,
        prepared.system.as_deref(),
        "gpt-test",
        "2026-09-21T00:00:00.000Z",
        None,
    );

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
            FakeFinish::Done => Ok(TranslateOutcome::Done {
                usage: None,
                finish_reason: None,
            }),
            FakeFinish::DoneWithUsage(u) => Ok(TranslateOutcome::Done {
                usage: Some(*u),
                finish_reason: None,
            }),
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
        system: None,
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
            received_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(text);
        }
        Ok(())
    });
    (channel, received)
}

/// I/O Matrix "Translate a segment" (nửa "tokens arrive ... render as they land") + AC2 —
/// token tới ĐÚNG THỨ TỰ, mỗi token một lần gửi `Channel`, trạng thái kết thúc `Done`.
#[test]
fn tokens_arrive_on_the_channel_in_order_as_they_land_and_the_call_ends_done() {
    let provider = FakeProvider {
        tokens: vec!["<<<SEGMENT 1\n", "Xin ", "chào", "\nSEGMENT>>>\n"],
        finish: FakeFinish::Done,
    };
    let prepared = dummy_prepared();
    let (channel, received) = collecting_channel();
    let should_cancel = || false;

    let outcome = tauri::async_runtime::block_on(run_translate_call(
        &provider,
        &prepared,
        &channel,
        &should_cancel,
    ))
    .expect("khong duoc loi");

    assert_eq!(
        outcome,
        SingleRun::Done {
            usage: None,
            finish_reason: None,
            block_open: false
        }
    );
    let got = received
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(
        got,
        vec!["Xin ".to_owned(), "chào".to_owned()],
        "token phai toi DUNG THU TU"
    );
}

/// I/O Matrix "Cancel mid-stream" — Đối chứng bằng phép ĐO, không bằng cờ: sau khi huỷ, KHÔNG
/// khung nào gửi thêm (§Verification spec 4.8: "assert that no further Channel message
/// arrives, not merely that the state changed").
#[test]
fn cancel_mid_stream_sends_no_further_channel_message_and_reports_cancelled() {
    let provider = FakeProvider {
        tokens: vec!["<<<SEGMENT 1\n", "one", "two", "three"],
        finish: FakeFinish::Done,
    };
    let prepared = dummy_prepared();
    let (channel, received) = collecting_channel();
    let calls = AtomicUsize::new(0);
    // Huỷ đúng SAU khi đã nhận một token đầu tiên -- "two"/"three" không được phép tới.
    let should_cancel = || calls.fetch_add(1, Ordering::SeqCst) >= 2;

    let outcome = tauri::async_runtime::block_on(run_translate_call(
        &provider,
        &prepared,
        &channel,
        &should_cancel,
    ))
    .expect("huy la Ok(Cancelled), khong phai mot Err");

    assert_eq!(outcome, SingleRun::Cancelled);
    let got = received
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
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
        tokens: vec!["<<<SEGMENT 1\n", "partial "],
        finish: FakeFinish::Err(FakeProviderError("dropped".to_owned())),
    };
    let prepared = dummy_prepared();
    let (channel, received) = collecting_channel();
    let should_cancel = || false;

    let err = tauri::async_runtime::block_on(run_translate_call(
        &provider,
        &prepared,
        &channel,
        &should_cancel,
    ))
    .expect_err("mot loi provider phai truyen nguyen qua run_translate_call");

    assert_eq!(
        err,
        SingleRunError::Provider(FakeProviderError("dropped".to_owned()))
    );
    let got = received
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
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

    let err = tauri::async_runtime::block_on(run_translate_call(
        &provider,
        &prepared,
        &channel,
        &should_cancel,
    ))
    .expect_err("mot loi provider TRUOC token dau tien van phai la Err");

    assert_eq!(
        err,
        SingleRunError::Provider(FakeProviderError("dropped-before-first-token".to_owned()))
    );
    let got = received
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert!(
        got.is_empty(),
        "khong token nao tung gui -- Channel phai RONG, khong mot khung nao"
    );
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
    let usage = TranslateUsage {
        prompt_tokens: 100,
        completion_tokens: 312,
        total_tokens: 412,
        cost_usd: Some(0.00412),
    };
    let provider = FakeProvider {
        tokens: vec!["<<<SEGMENT 1\nXin chào\nSEGMENT>>>\n"],
        finish: FakeFinish::DoneWithUsage(usage),
    };
    let prepared = dummy_prepared();
    let (channel, _received) = collecting_channel();
    let should_cancel = || false;

    let outcome = tauri::async_runtime::block_on(run_translate_call(
        &provider,
        &prepared,
        &channel,
        &should_cancel,
    ))
    .expect("khong duoc loi");

    assert_eq!(
        outcome,
        SingleRun::Done {
            usage: Some(usage),
            finish_reason: None,
            block_open: false
        },
        "usage phai di qua NGUYEN VEN, khong doi mot truong nao"
    );
}

/// I/O Matrix "Provider sends no usage" -- stream kết thúc `Done` mà không một khung `usage`
/// nào từng tới (`FakeFinish::Done` không mang usage) -- outcome phải mang `None`, KHÔNG một
/// `TranslateUsage` giả với các trường bằng `0` (§Always spec 4.11: "never `0`, never an empty
/// currency").
#[test]
fn a_stream_that_ends_done_without_ever_seeing_a_usage_frame_carries_none() {
    let provider = FakeProvider {
        tokens: vec!["<<<SEGMENT 1\n", "Xin ", "chào", "\nSEGMENT>>>\n"],
        finish: FakeFinish::Done,
    };
    let prepared = dummy_prepared();
    let (channel, _received) = collecting_channel();
    let should_cancel = || false;

    let outcome = tauri::async_runtime::block_on(run_translate_call(
        &provider,
        &prepared,
        &channel,
        &should_cancel,
    ))
    .expect("khong duoc loi");

    assert_eq!(
        outcome,
        SingleRun::Done {
            usage: None,
            finish_reason: None,
            block_open: false
        }
    );
}

/// I/O Matrix "Cancelled mid-flight" -- huỷ TRƯỚC khi provider (giả) kịp trả usage (dù công
/// thức của nó CÓ mang usage nếu chạy hết) -- outcome phải là `Cancelled`, kiểu của biến thể
/// đó KHÔNG có chỗ để mang một usage nào (không cần assert thêm gì ngoài biến thể đúng: đây là
/// bảo đảm ở TẦNG KIỂU, không phải một giá trị có thể lỡ tay đúc sai).
#[test]
fn cancelling_before_the_usage_frame_arrives_reports_cancelled_with_no_usage_to_carry() {
    let usage = TranslateUsage {
        prompt_tokens: 9,
        completion_tokens: 9,
        total_tokens: 18,
        cost_usd: None,
    };
    let provider = FakeProvider {
        tokens: vec!["<<<SEGMENT 1\n", "partial", "more"],
        finish: FakeFinish::DoneWithUsage(usage),
    };
    let prepared = dummy_prepared();
    let (channel, received) = collecting_channel();
    let calls = AtomicUsize::new(0);
    // Huy NGAY sau token dau tien -- provider (gia) khong bao gio toi duoc nhanh tra usage.
    let should_cancel = || calls.fetch_add(1, Ordering::SeqCst) >= 2;

    let outcome = tauri::async_runtime::block_on(run_translate_call(
        &provider,
        &prepared,
        &channel,
        &should_cancel,
    ))
    .expect("huy la Ok(Cancelled), khong phai mot Err");

    assert_eq!(outcome, SingleRun::Cancelled);
    let got = received
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(got, vec!["partial".to_owned()]);
}

/// `AiTranslateUsageWire::from(TranslateUsage)` -- chuyển đổi 1:1, không đánh rơi/đúc lại
/// trường nào (kể cả `cost_usd: None`, ca "usage đã tới nhưng mô hình không có giá").
#[test]
fn ai_translate_usage_wire_from_translate_usage_copies_every_field_verbatim() {
    let usage = TranslateUsage {
        prompt_tokens: 7,
        completion_tokens: 11,
        total_tokens: 18,
        cost_usd: None,
    };
    let wire = auratranslate_lib::commands::aitranslate::AiTranslateUsageWire::from(usage);
    assert_eq!(wire.prompt_tokens, 7);
    assert_eq!(wire.completion_tokens, 11);
    assert_eq!(wire.total_tokens, 18);
    assert_eq!(wire.cost_usd, None);

    let priced = TranslateUsage {
        prompt_tokens: 100,
        completion_tokens: 312,
        total_tokens: 412,
        cost_usd: Some(0.00412),
    };
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
fn single_run_outcome_wire_carries_the_real_usage_through_for_done_and_carries_nothing_for_cancelled()
 {
    use auratranslate_lib::commands::aitranslate::{
        AiTranslateOutcomeWire, single_run_outcome_wire,
    };

    let usage = TranslateUsage {
        prompt_tokens: 100,
        completion_tokens: 312,
        total_tokens: 412,
        cost_usd: Some(0.00412),
    };
    match single_run_outcome_wire(SingleRun::Done {
        usage: Some(usage),
        finish_reason: None,
        block_open: false,
    }) {
        AiTranslateOutcomeWire::Done {
            usage: Some(wire),
            maybe_truncated: _,
        } => {
            assert_eq!(
                wire.total_tokens, 412,
                "usage THAT phai di ra day, khong duoc thay bang None"
            );
            assert_eq!(wire.cost_usd, Some(0.00412));
        }
        other => panic!(
            "Done(Some(usage)) phai anh xa thanh Done{{ usage: Some(..) }}, nhan duoc {other:?}"
        ),
    }

    match single_run_outcome_wire(SingleRun::Done {
        usage: None,
        finish_reason: None,
        block_open: false,
    }) {
        AiTranslateOutcomeWire::Done {
            usage: None,
            maybe_truncated: _,
        } => {}
        other => panic!(
            "Done(None) (provider khong tra usage) phai anh xa thanh Done{{ usage: None }}, nhan duoc {other:?}"
        ),
    }

    match single_run_outcome_wire(SingleRun::Cancelled) {
        AiTranslateOutcomeWire::Cancelled => {}
        other => panic!("Cancelled phai anh xa thanh Cancelled, nhan duoc {other:?}"),
    }
}

/// `maybe_truncated` bật khi khối 1 chưa đóng hoặc `finish_reason` khác `stop`, và chỉ tắt khi
/// khối đóng với `stop` (AD-65 mục 5).
#[test]
fn single_run_outcome_wire_sets_maybe_truncated_for_an_open_block_or_a_non_stop_finish_reason() {
    use auratranslate_lib::commands::aitranslate::{
        AiTranslateOutcomeWire, single_run_outcome_wire,
    };

    let truncated = |finish_reason: Option<&str>, block_open: bool| match single_run_outcome_wire(
        SingleRun::Done {
            usage: None,
            finish_reason: finish_reason.map(str::to_owned),
            block_open,
        },
    ) {
        AiTranslateOutcomeWire::Done {
            maybe_truncated, ..
        } => maybe_truncated,
        other => panic!("phai Done, nhan duoc {other:?}"),
    };
    assert!(!truncated(Some("stop"), false), "khoi dong + stop");
    assert!(truncated(Some("stop"), true), "khoi chua dong");
    assert!(truncated(None, false), "khong co finish_reason");
    assert!(truncated(Some("length"), false), "finish_reason khac stop");
}

/// AC2's nửa "grep tìm 0 `emit`/`listen`" — canh bằng chính văn bản nguồn của tầng lệnh, đúng
/// khuôn các gate quét-mã-nguồn khác của kho (`ai_boundary.rs`).
#[test]
fn the_command_file_carries_zero_emit_or_listen_calls_on_the_translate_path() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("commands")
        .join("aitranslate.rs");
    let src = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("khong doc duoc {}: {e}", path.display()));
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
fn provider_returns_a_non_2xx_maps_to_the_key_of_its_status_and_names_the_status() {
    let table: [(u16, &str, MessageKey, bool); 7] = [
        (
            401,
            "ai_translate.provider_key_rejected",
            MessageKey::AiTranslateProviderKeyRejected,
            false,
        ),
        (
            403,
            "ai_translate.provider_key_rejected",
            MessageKey::AiTranslateProviderKeyRejected,
            false,
        ),
        (
            402,
            "ai_translate.provider_payment_required",
            MessageKey::AiTranslateProviderPaymentRequired,
            false,
        ),
        (
            404,
            "ai_translate.provider_endpoint_not_found",
            MessageKey::AiTranslateProviderEndpointNotFound,
            false,
        ),
        (
            429,
            "ai_translate.provider_rate_limited",
            MessageKey::AiTranslateProviderRateLimited,
            false,
        ),
        (
            503,
            "ai_translate.provider_server_error",
            MessageKey::AiTranslateProviderServerError,
            true,
        ),
        (
            418,
            "ai_translate.provider_refused",
            MessageKey::AiTranslateProviderRefused,
            false,
        ),
    ];
    for (status, code, key, retryable) in table {
        let err: IpcError = OpenAiClientError::NonSuccessStatus {
            status,
            provider_message: None,
        }
        .into();
        assert_eq!(err.code(), code, "status {status}");
        assert_eq!(err.message_key(), key, "status {status}");
        assert_eq!(
            err.params().get("status"),
            Some(&status.to_string()),
            "status {status}"
        );
        assert!(
            err.params().get("provider_message").is_none(),
            "status {status}"
        );
        assert_eq!(err.retryable(), retryable, "status {status}");
    }
}

#[test]
fn provider_message_rides_in_params_only_when_present_on_both_translate_paths() {
    let with = || OpenAiClientError::NonSuccessStatus {
        status: 402,
        provider_message: Some("het tien".to_owned()),
    };
    let single: IpcError = with().into();
    let batch = batch_stopped_error(2, 7, 9, with());
    for err in [single, batch] {
        assert_eq!(
            err.params().get("provider_message"),
            Some(&"het tien".to_owned())
        );
        assert_eq!(err.params().get("status"), Some(&"402".to_owned()));
    }
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
    assert_eq!(
        err.message_key(),
        MessageKey::AiTranslateStreamEndedWithoutDone
    );
    assert!(err.params().is_empty());
    assert!(
        err.retryable(),
        "rot ket noi GIUA CHUNG co the thu lai o luot bam lai"
    );
}

/// `EmptyReply` đi qua họ lỗi riêng: không thử lại; `finish_reason` chỉ làm tham số và đổi khoá
/// khi có.
#[test]
fn empty_reply_maps_to_a_non_retryable_ipc_error_with_finish_reason_only_when_present() {
    let without: IpcError = OpenAiClientError::EmptyReply {
        finish_reason: None,
    }
    .into();
    assert_eq!(without.code(), "ai_translate.reply_empty");
    assert_eq!(without.message_key(), MessageKey::AiTranslateReplyEmpty);
    assert!(without.params().get("finish_reason").is_none());
    assert!(!without.retryable());

    let with: IpcError = OpenAiClientError::EmptyReply {
        finish_reason: Some("content_filter".to_owned()),
    }
    .into();
    assert_eq!(with.code(), "ai_translate.reply_empty");
    assert_eq!(
        with.message_key(),
        MessageKey::AiTranslateReplyEmptyWithReason
    );
    assert_eq!(
        with.params().get("finish_reason"),
        Some(&"content_filter".to_owned())
    );
    assert!(!with.retryable());
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
fn every_openai_client_error_variant_maps_to_its_own_family_key_with_the_documented_retryable_flag()
{
    let cases: [(OpenAiClientError, &str, MessageKey, bool); 8] = [
        (
            OpenAiClientError::ClientBuildFailed {
                detail: "x".to_owned(),
            },
            "ai_translate.client_build_failed",
            MessageKey::AiTranslateClientBuildFailed,
            false,
        ),
        (
            OpenAiClientError::RequestFailed {
                detail: "x".to_owned(),
            },
            "ai_translate.provider_unreachable",
            MessageKey::AiTranslateProviderUnreachable,
            true,
        ),
        (
            OpenAiClientError::NonSuccessStatus {
                status: 500,
                provider_message: None,
            },
            "ai_translate.provider_server_error",
            MessageKey::AiTranslateProviderServerError,
            true,
        ),
        (
            OpenAiClientError::ReadFailed {
                detail: "x".to_owned(),
            },
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
            OpenAiClientError::MalformedEvent {
                detail: "x".to_owned(),
            },
            "ai_translate.reply_unreadable",
            MessageKey::AiTranslateReplyUnreadable,
            false,
        ),
        (
            OpenAiClientError::BufferOverflow {
                size: 2 * 1024 * 1024,
            },
            "ai_translate.reply_unreadable",
            MessageKey::AiTranslateReplyUnreadable,
            false,
        ),
        (
            OpenAiClientError::ApiKeyHeaderInvalid {
                detail: "x".to_owned(),
            },
            "ai_translate.api_key_header_invalid",
            MessageKey::AiTranslateApiKeyHeaderInvalid,
            false,
        ),
    ];
    for (variant, expected_code, expected_key, expected_retryable) in cases {
        let label = format!("{variant:?}");
        let err: IpcError = variant.into();
        assert_eq!(
            err.code(),
            expected_code,
            "{label} phai mang code cua HO nguyen nhan"
        );
        assert_eq!(err.message_key(), expected_key, "{label}");
        assert_eq!(
            err.retryable(),
            expected_retryable,
            "{label} phai anh xa retryable={expected_retryable}"
        );
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
        system: None,
    };
    let json = serde_json::to_string(&build_request_body(&request)).expect("serialize");
    assert!(
        !json.contains("temperature"),
        "Quyet dinh 6: truong nay phai VANG MAT (khong ca 'null') khi None: {json}"
    );
    assert!(
        !json.contains("max_tokens"),
        "Quyet dinh 6: truong nay phai VANG MAT khi None: {json}"
    );
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
        system: None,
    };
    let json = serde_json::to_string(&build_request_body(&request)).expect("serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse lai");
    assert_eq!(value["temperature"], serde_json::json!(0.7));
    assert_eq!(value["max_tokens"], serde_json::json!(512));
}

/// AD-14 tại điểm ra dây cuối cùng — `content` mang ĐÚNG `request.prompt`, không lắp lại.
#[test]
fn a_request_with_a_system_prompt_sends_system_then_user_messages_verbatim() {
    let request = TranslateRequest {
        endpoint: "https://api.example.invalid/v1/chat/completions",
        model: "gpt-test",
        temperature: None,
        max_tokens: None,
        api_key: "sk-test",
        prompt: "Dịch câu này giúp tôi.",
        system: Some("  Dòng một\n\nDòng hai  "),
    };
    let json = serde_json::to_string(&build_request_body(&request)).expect("serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse lai");
    let messages = value["messages"].as_array().expect("messages la mang");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["role"], "system");
    assert_eq!(messages[0]["content"], "  Dòng một\n\nDòng hai  ");
    assert_eq!(messages[1]["role"], "user");
    assert_eq!(messages[1]["content"], "Dịch câu này giúp tôi.");
}

#[test]
fn a_request_without_a_system_prompt_or_with_an_empty_one_sends_only_the_user_message() {
    for system in [None, Some(""), Some("  \n ")] {
        let request = TranslateRequest {
            endpoint: "https://api.example.invalid/v1/chat/completions",
            model: "gpt-test",
            temperature: None,
            max_tokens: None,
            api_key: "sk-test",
            prompt: "Dịch câu này giúp tôi.",
            system,
        };
        let json = serde_json::to_string(&build_request_body(&request)).expect("serialize");
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse lai");
        let messages = value["messages"].as_array().expect("messages la mang");
        assert_eq!(messages.len(), 1, "system={system:?}");
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "Dịch câu này giúp tôi.");
    }
}

/// What the inspector shows is what the wire carries: the record's `system` and `prompt` flow
/// through `prepare_translate_call` and `run_translate_call` into the request body unchanged.
#[test]
fn a_translate_call_sends_exactly_the_system_and_prompt_the_inspector_record_holds() {
    let fx = resend_fixture("system-bytes", true);
    write_field(&fx.global, AiConfigField::Provider, "ollama").expect("ghi provider");
    let typed = " Dong 1\r\n\r\n  Dong 2\t ";
    write_field(&fx.global, AiConfigField::SystemPrompt, typed).expect("ghi system");

    let prepared = match prepare_translate_call(
        Some(&fx.global),
        Some(&fx.open),
        &fx.record,
        Some("Plain"),
        fx.segment_id,
    )
    .expect("khong loi")
    {
        PrepareOutcome::Ready(p) => p,
        PrepareOutcome::NotConfigured => panic!("phai san sang"),
    };
    let record = read_last_assembled_prompt(&fx.record).expect("phai co ban ghi");
    let expected_system = system_with_block_format(Some(typed));
    assert_eq!(record.system.as_deref(), Some(expected_system.as_str()));
    assert_eq!(prepared.system.as_deref(), Some(expected_system.as_str()));
    assert_eq!(prepared.prompt, record.prompt);

    let request = TranslateRequest {
        endpoint: &prepared.endpoint,
        model: &prepared.model,
        temperature: prepared.temperature,
        max_tokens: prepared.max_tokens,
        api_key: &prepared.api_key,
        prompt: &prepared.prompt,
        system: prepared.system.as_deref(),
    };
    let json = serde_json::to_string(&build_request_body(&request)).expect("serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse lai");
    assert_eq!(value["messages"][0]["role"], "system");
    assert_eq!(value["messages"][0]["content"], expected_system.as_str());
    assert_eq!(value["messages"][1]["role"], "user");
    assert_eq!(value["messages"][1]["content"], record.prompt.as_str());
    fx.teardown();
}

#[test]
fn a_resend_uses_the_system_prompt_recorded_with_the_first_send_not_the_current_config() {
    let fx = resend_fixture("system-resend", true);
    write_field(&fx.global, AiConfigField::Provider, "ollama").expect("ghi provider");
    write_field(&fx.global, AiConfigField::SystemPrompt, "Ban dau").expect("ghi system");
    assemble_and_record_prompt(
        Some(&fx.global),
        Some(&fx.open),
        &fx.record,
        Some("Plain"),
        fx.segment_id,
    )
    .expect("lap rap");

    write_field(&fx.global, AiConfigField::SystemPrompt, "Da sua sau do").expect("sua system");
    let (_, outcome) =
        prepare_resend_call(Some(&fx.global), Some(&fx.open), &fx.record).expect("khong loi");
    let prepared = match outcome {
        PrepareOutcome::Ready(p) => p,
        PrepareOutcome::NotConfigured => panic!("phai san sang"),
    };
    let expected_system = system_with_block_format(Some("Ban dau"));
    assert_eq!(prepared.system.as_deref(), Some(expected_system.as_str()));
    let recorded = read_last_assembled_prompt(&fx.record).expect("ban ghi");
    assert_eq!(recorded.system, prepared.system);
    fx.teardown();
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

    let outcome = promote_ai_translation(Some(&open), segment_id, "Con rồng gầm.", false)
        .expect("promote khong duoc loi");
    assert!(
        !outcome.needs_confirmation,
        "segment chua tung dich -- khong co gi de mat"
    );
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
    confirm_segment(Some(&open), segment_id).expect("confirm khong duoc loi");
    assert_eq!(
        read_origin(&open, segment_id),
        TRANSLATION_ORIGIN_OTHER,
        "AC4: xac nhan khong sua mot chu phai GIU NGUYEN 'other'"
    );

    drop(open);
    cleanup(&work_dir);
}

/// **AD-49 iii** — `promote_ai_translation` KHÔNG được ghi đè vô điều
/// kiện một bản nháp chưa từng ký. Cùng khuôn `restore_segment_version`
/// (`segment_contract.rs::restoring_over_an_unsigned_draft_holds_the_write_until_the_caller_confirms`):
/// khi văn bản hiện tại KHÔNG rỗng và KHÔNG có bản sao trong `segment_version`, lượt gọi
/// `force = false` giữ lại lượt ghi.
///
/// ⚠️ Lượt promote ĐẦU không tạo một hàng `segment_version` nào (đúng doc-comment của hàm —
/// chỉ `confirm_segment` làm điều đó), nên bản nháp nó để lại chính là ca "chưa từng ký"
/// cần cho ca này, không cần dựng fixture bằng SQL trực tiếp.
#[test]
fn promoting_over_an_unsigned_draft_holds_the_write_until_the_caller_confirms() {
    let work_dir = temp_dir("promote-l12359-holds");
    let open = open_work(&work_dir, "PromoteHolds", "en", "A dragon roared.");
    let segment_id = first_segment_id(&open);

    let first = promote_ai_translation(Some(&open), segment_id, "Ban nhap AI thu nhat.", false)
        .expect("promote thu nhat khong duoc loi");
    assert!(
        !first.needs_confirmation,
        "segment chua tung dich -- khong co gi de mat"
    );

    let held = promote_ai_translation(Some(&open), segment_id, "Ban nhap AI thu hai.", false)
        .expect("mot luot giu lai KHONG phai mot loi -- no la mot ket qua");
    assert!(
        held.needs_confirmation,
        "van ban hien tai chua tung duoc ky (khong co ban sao trong segment_version) ⇒ PHAI \
         hoi lai truoc khi ghi de -- AD-49 iii, cung khuon restore_segment_version"
    );
    assert_eq!(
        held.unsigned_draft.as_deref(),
        Some("Ban nhap AI thu nhat."),
        "ban nhap sap mat phai di ra ngoai de webview HIEN NO RA, khong chi noi \"co thu se mat\""
    );
    assert_eq!(
        held.target_text, "Ban nhap AI thu nhat.",
        "khong ghi mot byte nao ⇒ target_text tra ve phai la ban DANG CO, khong phai de xuat moi"
    );

    let on_disk = read_open_chapter_segments(Some(&open)).expect("nap lai segment that bai");
    let seg = on_disk
        .segments
        .iter()
        .find(|s| s.id == segment_id)
        .expect("khong thay segment");
    assert_eq!(
        seg.target_text, "Ban nhap AI thu nhat.",
        "mot luot GIU LAI khong duoc dung toi mot byte nao cua dia"
    );

    let forced = promote_ai_translation(Some(&open), segment_id, "Ban nhap AI thu hai.", true)
        .expect("promote voi force khong duoc loi");
    assert!(!forced.needs_confirmation, "voi `force` thi no phai ghi");
    assert_eq!(forced.target_text, "Ban nhap AI thu hai.");
    assert_eq!(read_origin(&open, segment_id), TRANSLATION_ORIGIN_OTHER);

    drop(open);
    cleanup(&work_dir);
}

#[test]
fn promoting_over_a_target_that_is_only_whitespace_writes_without_asking() {
    let work_dir = temp_dir("promote-blank");
    let open = open_work(&work_dir, "PromoteBlank", "en", "A dragon roared.");
    let segment_id = first_segment_id(&open);
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET target_text = ?1 WHERE id = ?2",
                ("\n \t", segment_id),
            )
        })
        .expect("xoa trang");

    let outcome = promote_ai_translation(Some(&open), segment_id, "Con rồng gầm.", false)
        .expect("promote khong duoc loi");

    assert!(
        !outcome.needs_confirmation,
        "o chi con khoang trang la chua dich -- khong co gi de mat"
    );
    assert_eq!(outcome.target_text, "Con rồng gầm.");
    drop(open);
    cleanup(&work_dir);
}

/// Một segment đã về hưu (tombstone) KHÔNG nhận bản dịch AI: trả `segment.retired` TRƯỚC mọi
/// ghi, hàng tombstone giữ nguyên văn bản lẫn xuất xứ. Counter-check: gỡ nhánh `retired` khỏi
/// `promote_ai_translation` ⇒ ca này đỏ vì `Ok` thay cho `Err(SegmentRetired)`.
#[test]
fn promoting_into_a_retired_segment_is_refused_and_leaves_the_tombstone_untouched() {
    let work_dir = temp_dir("promote-retired");
    let open = open_work(&work_dir, "PromoteRetired", "en", "A dragon roared.");
    let segment_id = first_segment_id(&open);

    open.store
        .write(move |tx| {
            tx.execute(
                "UPDATE segment SET retired_at = '2026-09-30T00:00:00.000Z' WHERE id = ?1",
                [segment_id],
            )
        })
        .expect("ve huu segment that bai");

    let (text_before, origin_before) = read_text_and_origin(&open, segment_id);

    for force in [false, true] {
        let err = promote_ai_translation(Some(&open), segment_id, "Con rồng gầm.", force)
            .expect_err("promote vao segment da ve huu phai la Err");
        assert_eq!(err.message_key(), MessageKey::SegmentRetired);
        assert_eq!(err.code(), "segment.retired");
        assert!(!err.retryable());
    }

    assert_eq!(
        read_text_and_origin(&open, segment_id),
        (text_before, origin_before),
        "mot luot promote bi tu choi khong duoc ghi mot byte nao vao hang tombstone"
    );

    drop(open);
    cleanup(&work_dir);
}

#[test]
fn promoting_into_a_segment_outside_the_open_chapter_is_refused_and_writes_nothing() {
    let work_dir = temp_dir("promote-other-chapter");
    let mut open = open_work(&work_dir, "PromoteOtherChapter", "en", "A dragon roared.");
    let segment_id = first_segment_id(&open);
    let first_chapter = open.chapter_id;

    let second_chapter = open
        .store
        .write(move |tx| {
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (2, NULL, 'Other.', 'not_started', strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [],
            )?;
            Ok(tx.last_insert_rowid())
        })
        .expect("chen Chuong thu hai that bai");
    open.chapter_id = second_chapter;

    let (text_before, origin_before) = read_text_and_origin(&open, segment_id);

    for force in [false, true] {
        let err = promote_ai_translation(Some(&open), segment_id, "Con rồng gầm.", force)
            .expect_err("promote vao segment ngoai Chuong dang mo phai la Err");
        assert_eq!(err.code(), "segment.not_in_open_chapter");
        assert_eq!(err.message_key(), MessageKey::SegmentNotInOpenChapter);
        assert_eq!(
            err.params().get("segment_id"),
            Some(&segment_id.to_string())
        );
        assert_eq!(
            err.params().get("chapter_id"),
            Some(&second_chapter.to_string())
        );
        assert!(!err.retryable());
    }

    assert_eq!(
        read_text_and_origin(&open, segment_id),
        (text_before, origin_before)
    );
    assert_ne!(first_chapter, second_chapter);

    drop(open);
    cleanup(&work_dir);
}

fn read_text_and_origin(open: &OpenWork, id: i64) -> (String, String) {
    open.store
        .read(move |conn| {
            conn.query_row(
                "SELECT target_text, translation_origin FROM segment WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
        })
        .expect("doc hang segment that bai")
}

/// Panic/bị huỷ của tác vụ blocking là một sự cố NỘI BỘ: khoá riêng, không retryable, khác họ
/// "provider không tới được". Ca này canh hàm dựng lỗi, không canh chỗ gọi trong `wire`.
#[test]
fn a_panicked_blocking_task_is_an_internal_failure_not_a_network_error() {
    let err = batch_panicked_error();
    assert_eq!(err.code(), "ai_translate.internal_failure");
    assert_eq!(err.message_key(), MessageKey::AiTranslateInternalFailure);
    assert_ne!(
        err.message_key(),
        MessageKey::AiTranslateProviderUnreachable
    );
    assert!(!err.retryable());
}

struct RecordingProvider {
    seen_prompts: Arc<Mutex<Vec<String>>>,
    usage: Option<TranslateUsage>,
}

impl TranslationProvider for RecordingProvider {
    type Error = FakeProviderError;

    async fn translate(
        &self,
        request: TranslateRequest<'_>,
        on_token: &mut dyn FnMut(&str),
        _should_cancel: &dyn Fn() -> bool,
    ) -> Result<TranslateOutcome, Self::Error> {
        self.seen_prompts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(request.prompt.to_owned());
        on_token("<<<SEGMENT 1\nXin chào\nSEGMENT>>>\n");
        Ok(TranslateOutcome::Done {
            usage: self.usage,
            finish_reason: None,
        })
    }
}

const RESEND_USAGE: TranslateUsage = TranslateUsage {
    prompt_tokens: 100,
    completion_tokens: 312,
    total_tokens: 412,
    cost_usd: Some(0.00412),
};

struct ResendFixture {
    global_dir: PathBuf,
    work_dir: PathBuf,
    global: Store,
    open: OpenWork,
    segment_id: i64,
    record: LastAssembledPromptState,
}

impl ResendFixture {
    fn teardown(self) {
        let Self {
            global_dir,
            work_dir,
            global,
            open,
            ..
        } = self;
        drop(open);
        drop(global);
        cleanup(&work_dir);
        cleanup(&global_dir);
    }
}

fn resend_fixture(tag: &str, configure: bool) -> ResendFixture {
    let global_dir = temp_dir(&format!("{tag}-global"));
    let work_dir = temp_dir(&format!("{tag}-work"));
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, tag, "en", "A dragon roared.");
    if configure {
        write_field(
            &global,
            AiConfigField::Endpoint,
            "https://api.example.invalid/v1/chat/completions",
        )
        .expect("ghi endpoint");
        write_field(&global, AiConfigField::Model, "gpt-test").expect("ghi model");
    }
    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Global,
        "Plain",
        "Dich: {{source_segment}}",
    )
    .expect("tao bo prompt");
    let segment_id = first_segment_id(&open);
    let record = fresh_record();
    assemble_and_record_prompt(
        Some(&global),
        Some(&open),
        &record,
        Some("Plain"),
        segment_id,
    )
    .expect("lap rap khong duoc loi");
    ResendFixture {
        global_dir,
        work_dir,
        global,
        open,
        segment_id,
        record,
    }
}

fn set_record_prompt(record: &LastAssembledPromptState, prompt: &str) {
    record
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_mut()
        .expect("phai co ban ghi")
        .prompt = prompt.to_owned();
}

#[test]
fn resend_sends_the_recorded_prompt_byte_for_byte_writes_no_segment_and_stamps_the_usage() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-resend-test-key");
    let fx = resend_fixture("resend-bytes", true);

    // Chuoi khong the tai lap boi mot lan lap rap lai: moi lan lap rap tu bo "Plain" cho
    // "Dich: A dragon roared.", nen chi doc tu ban ghi moi cho ra chuoi nay.
    let recorded = "  Chuoi la\tcua ban ghi \u{1F409}\n--\nA dragon roared.  ";
    set_record_prompt(&fx.record, recorded);
    let before = read_text_and_origin(&fx.open, fx.segment_id);
    let source_before = read_open_chapter_segments(Some(&fx.open))
        .expect("nap chuong")
        .segments[0]
        .source_text
        .clone();

    let (segment_id, outcome) =
        prepare_resend_call(Some(&fx.global), Some(&fx.open), &fx.record).expect("khong loi");
    assert_eq!(
        segment_id, fx.segment_id,
        "ket qua thuoc segment cua ban ghi"
    );
    let prepared = match outcome {
        PrepareOutcome::Ready(p) => p,
        PrepareOutcome::NotConfigured => panic!("phai san sang"),
    };
    assert_eq!(prepared.prompt, recorded);

    let seen = Arc::new(Mutex::new(Vec::new()));
    let provider = RecordingProvider {
        seen_prompts: Arc::clone(&seen),
        usage: Some(RESEND_USAGE),
    };
    let (channel, received) = collecting_channel();
    let done =
        tauri::async_runtime::block_on(run_translate_call(&provider, &prepared, &channel, &|| {
            false
        }))
        .expect("khong duoc loi");
    assert_eq!(
        done,
        SingleRun::Done {
            usage: Some(RESEND_USAGE),
            finish_reason: None,
            block_open: false
        }
    );
    assert_eq!(
        *seen.lock().expect("khoa"),
        vec![recorded.to_owned()],
        "provider phai nhan DUNG chuoi cua ban ghi, tung byte"
    );
    assert_eq!(*received.lock().expect("khoa"), vec!["Xin chào".to_owned()]);

    let SingleRun::Done { usage, .. } = done else {
        panic!("phai Done")
    };
    mark_prompt_as_sent(
        &fx.record,
        segment_id,
        &prepared.prompt,
        prepared.system.as_deref(),
        &prepared.model,
        "2026-10-15T00:00:00.000Z",
        usage.map(auratranslate_lib::commands::aitranslate::AiTranslateUsageWire::from),
    );
    let wire = read_last_assembled_prompt(&fx.record).expect("phai co ban ghi");
    assert_eq!(wire.prompt, recorded, "gui lai khong doi ban ghi");
    assert_eq!(wire.sent_at.as_deref(), Some("2026-10-15T00:00:00.000Z"));
    assert_eq!(wire.sent_model.as_deref(), Some("gpt-test"));
    let stamped = wire.sent_usage.expect("usage phai duoc dong dau");
    assert_eq!(stamped.total_tokens, 412);
    assert_eq!(stamped.prompt_tokens, 100);
    assert_eq!(stamped.completion_tokens, 312);
    assert_eq!(stamped.cost_usd, Some(0.00412));

    assert_eq!(
        read_text_and_origin(&fx.open, fx.segment_id),
        before,
        "gui lai khong duoc ghi target_text hay translation_origin"
    );
    assert_eq!(
        read_open_chapter_segments(Some(&fx.open))
            .expect("nap chuong")
            .segments[0]
            .source_text,
        source_before
    );
    fx.teardown();
}

#[test]
fn resend_with_nothing_assembled_is_an_error_not_a_silent_empty_send() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-resend-none-key");
    let global_dir = temp_dir("resend-none-global");
    let global = open_global(&global_dir);

    let err = expect_prepare_err(
        prepare_resend_call(Some(&global), None, &fresh_record()).map(|(_, o)| o),
        "khong co ban ghi",
    );
    assert_eq!(err.code(), "ai_translate.no_prompt_to_resend");
    assert!(!err.retryable());

    drop(global);
    cleanup(&global_dir);
}

#[test]
fn resend_reports_not_configured_without_a_key_or_endpoint_and_leaves_the_record_unstamped() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    let fx = resend_fixture("resend-notconf-key", true);
    let (segment_id, no_key) =
        prepare_resend_call(Some(&fx.global), Some(&fx.open), &fx.record).expect("khong loi");
    assert_eq!(segment_id, fx.segment_id);
    assert!(matches!(no_key, PrepareOutcome::NotConfigured));
    fx.teardown();

    save_key("sk-resend-notconf-key");
    let fx = resend_fixture("resend-notconf-endpoint", false);
    let (_, no_endpoint) =
        prepare_resend_call(Some(&fx.global), Some(&fx.open), &fx.record).expect("khong loi");
    assert!(matches!(no_endpoint, PrepareOutcome::NotConfigured));
    let wire = read_last_assembled_prompt(&fx.record).expect("phai co ban ghi");
    assert_eq!(wire.sent_at, None);
    assert!(wire.sent_usage.is_none());
    fx.teardown();
}

#[test]
fn a_send_whose_provider_returned_no_usage_stamps_sent_at_but_leaves_usage_none_not_zero() {
    let fx = resend_fixture("stamp-no-usage", true);
    let recorded = read_last_assembled_prompt(&fx.record).expect("phai co ban ghi");
    let prompt = recorded.prompt;
    mark_prompt_as_sent(
        &fx.record,
        fx.segment_id,
        &prompt,
        recorded.system.as_deref(),
        "gpt-test",
        "2026-10-15T00:00:00.000Z",
        None,
    );
    let wire = read_last_assembled_prompt(&fx.record).expect("phai co ban ghi");
    assert!(wire.sent_at.is_some());
    assert!(
        wire.sent_usage.is_none(),
        "provider khong tra usage la None, khong phai mot usage toan so 0"
    );
    let json = serde_json::to_value(&wire).expect("serialize");
    assert_eq!(json["sent_usage"], serde_json::json!(null));
    fx.teardown();
}

#[test]
fn stamped_usage_serializes_with_the_exact_field_names_the_webview_guard_reads() {
    let fx = resend_fixture("stamp-usage-json", true);
    let recorded = read_last_assembled_prompt(&fx.record).expect("phai co ban ghi");
    let prompt = recorded.prompt;
    mark_prompt_as_sent(
        &fx.record,
        fx.segment_id,
        &prompt,
        recorded.system.as_deref(),
        "gpt-test",
        "2026-10-15T00:00:00.000Z",
        Some(auratranslate_lib::commands::aitranslate::AiTranslateUsageWire::from(RESEND_USAGE)),
    );
    let json = serde_json::to_value(read_last_assembled_prompt(&fx.record).expect("ban ghi"))
        .expect("json");
    assert_eq!(
        json["sent_usage"],
        serde_json::json!({
            "prompt_tokens": 100,
            "completion_tokens": 312,
            "total_tokens": 412,
            "cost_usd": 0.00412
        })
    );
    fx.teardown();
}

#[test]
fn a_record_reassembled_after_a_send_carries_no_stale_sent_facts_and_a_late_stamp_is_ignored() {
    let fx = resend_fixture("stamp-reassembled", true);
    let recorded = read_last_assembled_prompt(&fx.record).expect("phai co ban ghi");
    let old_prompt = recorded.prompt;
    mark_prompt_as_sent(
        &fx.record,
        fx.segment_id,
        &old_prompt,
        recorded.system.as_deref(),
        "gpt-test",
        "2026-10-15T00:00:00.000Z",
        Some(auratranslate_lib::commands::aitranslate::AiTranslateUsageWire::from(RESEND_USAGE)),
    );

    prompt_set_create(
        Some(&fx.global),
        Some(&fx.open),
        PromptSetTier::Global,
        "Other",
        "Doi bo: {{source_segment}}",
    )
    .expect("tao bo prompt thu hai");
    let reassembled = assemble_and_record_prompt(
        Some(&fx.global),
        Some(&fx.open),
        &fx.record,
        Some("Other"),
        fx.segment_id,
    )
    .expect("lap lai");
    assert_ne!(reassembled.prompt, old_prompt);
    assert_eq!(reassembled.sent_at, None);
    assert_eq!(reassembled.sent_model, None);
    assert!(
        reassembled.sent_usage.is_none(),
        "lap lai phai xoa usage cua lot gui truoc"
    );

    mark_prompt_as_sent(
        &fx.record,
        fx.segment_id,
        &old_prompt,
        Some(auratranslate_lib::core::aiconfig::DEFAULT_SYSTEM_PROMPT),
        "gpt-test",
        "2026-10-15T00:01:00.000Z",
        Some(auratranslate_lib::commands::aitranslate::AiTranslateUsageWire::from(RESEND_USAGE)),
    );
    let wire = read_last_assembled_prompt(&fx.record).expect("phai co ban ghi");
    assert_eq!(wire.sent_at, None, "dau cham cua chuoi cu bi bo qua");
    assert!(wire.sent_usage.is_none());
    fx.teardown();
}

// ═════════════════════════════════════════════════════════════════════════════════
// Grouped operation: the sync prepare layer and the error mapping (AD-60)
// ═════════════════════════════════════════════════════════════════════════════════

fn configure_endpoint(global: &Store) {
    write_field(
        global,
        AiConfigField::Endpoint,
        "https://api.example.invalid/v1/chat/completions",
    )
    .expect("ghi endpoint");
    write_field(global, AiConfigField::Model, "gpt-test").expect("ghi model");
}

fn create_plain_set(global: &Store, open: &OpenWork) {
    prompt_set_create(
        Some(global),
        Some(open),
        PromptSetTier::Global,
        "Plain",
        "{{source_segment}}",
    )
    .expect("tao bo prompt");
}

fn chapter_scope() -> AiTranslateScope {
    AiTranslateScope::Chapter
}

#[test]
fn an_operation_reports_not_configured_without_a_key_or_with_an_empty_endpoint_or_model() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    let global_dir = temp_dir("op-not-configured-global");
    let work_dir = temp_dir("op-not-configured-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "OpNotConfigured", "en", "One. Two.");
    create_plain_set(&global, &open);

    let outcome =
        prepare_translate_operation(Some(&global), Some(&open), Some("Plain"), &chapter_scope())
            .unwrap_or_else(|e| panic!("chua cau hinh la mot trang thai: {e:?}"));
    assert!(matches!(outcome, PrepareOperationOutcome::NotConfigured));

    configure_endpoint(&global);
    let outcome =
        prepare_translate_operation(Some(&global), Some(&open), Some("Plain"), &chapter_scope())
            .unwrap_or_else(|e| panic!("chua co khoa la mot trang thai: {e:?}"));
    assert!(
        matches!(outcome, PrepareOperationOutcome::NotConfigured),
        "endpoint va model da co nhung chua luu khoa"
    );

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

#[test]
fn an_operation_without_an_open_work_reuses_the_work_none_open_key() {
    let global_dir = temp_dir("op-no-work-global");
    let global = open_global(&global_dir);
    let err = expect_prepare_operation_err(
        prepare_translate_operation(Some(&global), None, Some("Plain"), &chapter_scope()),
        "khong Tac pham nao dang mo",
    );
    assert_eq!(err.code(), "work.none_open");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);
    drop(global);
    cleanup(&global_dir);
}

#[test]
fn a_selected_id_outside_the_open_chapter_is_refused_naming_that_id_before_any_plan() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    save_key("sk-op-not-in-chapter");
    let global_dir = temp_dir("op-not-in-chapter-global");
    let work_dir = temp_dir("op-not-in-chapter-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "OpNotInChapter", "en", "One. Two.");
    configure_endpoint(&global);
    create_plain_set(&global, &open);

    let real: Vec<i64> = read_open_chapter_segments(Some(&open))
        .expect("nap chuong")
        .segments
        .iter()
        .map(|s| s.id)
        .collect();
    let bogus = real.iter().copied().max().unwrap_or(0) + 999_999;
    let err = expect_prepare_operation_err(
        prepare_translate_operation(
            Some(&global),
            Some(&open),
            Some("Plain"),
            &AiTranslateScope::Selection {
                segment_ids: vec![real[0], bogus, real[1]],
            },
        ),
        "id la giua hai id that",
    );
    assert_eq!(err.code(), "ai_prompt.segment_not_in_chapter");
    assert_eq!(err.message_key(), MessageKey::AiPromptSegmentNotInChapter);
    assert_eq!(err.params().get("segment_id"), Some(&bogus.to_string()));

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

#[test]
fn an_operation_with_nothing_to_translate_never_reads_the_keychain() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    let global_dir = temp_dir("op-all-omitted-global");
    let work_dir = temp_dir("op-all-omitted-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "OpAllOmitted", "en", "One. Two.");
    configure_endpoint(&global);
    let ids: Vec<i64> = read_open_chapter_segments(Some(&open))
        .expect("nap chuong")
        .segments
        .iter()
        .map(|s| s.id)
        .collect();
    for id in &ids {
        set_segment_omitted(Some(&open), *id, true).expect("dat co cat bo");
    }

    let outcome = prepare_translate_operation(
        Some(&global),
        Some(&open),
        Some("Plain"),
        &AiTranslateScope::Selection { segment_ids: ids },
    )
    .unwrap_or_else(|e| panic!("khong duoc loi: {e:?}"));
    match outcome {
        PrepareOperationOutcome::Ready(prepared) => assert!(prepared.plan.is_empty()),
        PrepareOperationOutcome::NotConfigured => {
            panic!("khong co cau nao can dich thi khong doc khoa, nen khong the la NotConfigured")
        }
    }

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

#[test]
fn a_keychain_that_refuses_to_answer_surfaces_the_existing_key_when_something_needs_translating() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();
    let global_dir = temp_dir("op-keychain-global");
    let work_dir = temp_dir("op-keychain-work");
    let global = open_global(&global_dir);
    let open = open_work(&work_dir, "OpKeychain", "en", "One. Two.");
    configure_endpoint(&global);
    create_plain_set(&global, &open);
    inject_one_shot_keychain_error();

    let err = expect_prepare_operation_err(
        prepare_translate_operation(Some(&global), Some(&open), Some("Plain"), &chapter_scope()),
        "keychain tu choi tra loi",
    );
    assert_eq!(err.message_key(), MessageKey::AiConfigKeychainUnavailable);
    assert!(err.retryable());

    drop(open);
    drop(global);
    cleanup(&work_dir);
    cleanup(&global_dir);
}

#[test]
fn a_stopped_operation_names_the_call_and_its_first_and_last_sentence_in_every_family() {
    let cases: [(OpenAiClientError, &str, MessageKey, bool); 8] = [
        (
            OpenAiClientError::ClientBuildFailed {
                detail: "x".to_owned(),
            },
            "ai_translate.client_build_failed",
            MessageKey::AiTranslateClientBuildFailed,
            false,
        ),
        (
            OpenAiClientError::RequestFailed {
                detail: "x".to_owned(),
            },
            "ai_translate.provider_unreachable",
            MessageKey::AiTranslateProviderUnreachable,
            true,
        ),
        (
            OpenAiClientError::NonSuccessStatus {
                status: 500,
                provider_message: None,
            },
            "ai_translate.provider_server_error",
            MessageKey::AiTranslateProviderServerError,
            true,
        ),
        (
            OpenAiClientError::ReadFailed {
                detail: "x".to_owned(),
            },
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
            OpenAiClientError::MalformedEvent {
                detail: "x".to_owned(),
            },
            "ai_translate.reply_unreadable",
            MessageKey::AiTranslateReplyUnreadable,
            false,
        ),
        (
            OpenAiClientError::BufferOverflow {
                size: 2 * 1024 * 1024,
            },
            "ai_translate.reply_unreadable",
            MessageKey::AiTranslateReplyUnreadable,
            false,
        ),
        (
            OpenAiClientError::ApiKeyHeaderInvalid {
                detail: "x".to_owned(),
            },
            "ai_translate.api_key_header_invalid",
            MessageKey::AiTranslateApiKeyHeaderInvalid,
            false,
        ),
    ];
    for (variant, code, key, retryable) in cases {
        let label = format!("{variant:?}");
        let is_non_2xx = matches!(variant, OpenAiClientError::NonSuccessStatus { .. });
        let err = batch_stopped_error(2, 850, 899, variant);
        assert_eq!(err.code(), code, "{label}");
        assert_eq!(err.message_key(), key, "{label}");
        assert_eq!(err.retryable(), retryable, "{label}");
        assert_eq!(
            err.params().get("call_index"),
            Some(&"2".to_owned()),
            "{label}"
        );
        assert_eq!(
            err.params().get("first_segment_id"),
            Some(&"850".to_owned()),
            "{label}"
        );
        assert_eq!(
            err.params().get("last_segment_id"),
            Some(&"899".to_owned()),
            "{label}"
        );
        assert!(err.params().get("segment_id").is_none(), "{label}");
        assert_eq!(
            err.params().get("status").is_some(),
            is_non_2xx,
            "{label}: only a non-2xx carries status"
        );
    }
}

#[test]
fn the_scope_argument_reads_the_two_shapes_the_webview_sends() {
    let selection: AiTranslateScope =
        serde_json::from_str(r#"{"kind":"selection","segment_ids":[3,5]}"#).expect("selection");
    assert!(matches!(
        selection,
        AiTranslateScope::Selection { ref segment_ids } if segment_ids == &[3, 5]
    ));
    let chapter: AiTranslateScope = serde_json::from_str(r#"{"kind":"chapter"}"#).expect("chapter");
    assert!(matches!(chapter, AiTranslateScope::Chapter));
}
