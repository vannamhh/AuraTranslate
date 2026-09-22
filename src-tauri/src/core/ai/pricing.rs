//! Bảng giá BUNDLED, keyed theo model id — Story 4.11 (Quyết định Ice, 2026-09-22).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHÔNG BAO GIỜ FETCH, KHÔNG BAO GIỜ ĐOÁN — bảng CÓ NGÀY, để một hàng cũ LỘ RA
//! ─────────────────────────────────────────────────────────────────────────────
//! `AD-15`/`SPEC.md:69` đóng cứng đúng BA điểm ra mạng app-wide; bảng giá không phải điểm
//! thứ tư — nó là một `const` biên dịch cùng nhị phân, không một request nào rời máy để đọc
//! nó. Mỗi hàng mang [`PRICES_AS_OF`] — không một cơ chế "tự cập nhật" nào ở đây; một giá
//! niêm yết đổi thì hàng này lặng lẽ SAI cho tới khi ai đó sửa lại bằng tay, và đó là lý do
//! ngày phải đứng CẠNH số, không phải trong log commit.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ MỘT ID TRÙNG TÊN KHÔNG CÓ NGHĨA LÀ CÙNG MỘT MÔ HÌNH
//! ─────────────────────────────────────────────────────────────────────────────
//! Bảng khoá THEO TÊN, không theo bất kỳ dấu hiệu cục bộ/đám mây nào (`provider` là chuỗi tự
//! do, FR66 cố ý cho cục bộ VÀ đám mây cùng một đường cấu hình) — một mô hình cục bộ hay một
//! proxy trả lời đúng một id đã niêm yết SẼ bị tính tiền như thể nó là bản đám mây thật. Đây
//! là giới hạn đã CHẤP NHẬN (I/O Matrix spec 4.11 "Model id collides with a table row"), không
//! phải một lỗi cần vá bằng cách dò `localhost` — dò như vậy sai cho một mô hình tự host trên
//! LAN và mở một bộ phân biệt FR66 cố ý không có.
//!
//! Không tokenizer, không đếm cục bộ ở đây (§Never spec 4.11) — module này chỉ NHÂN hai số đã
//! có sẵn (token đếm được từ provider) với một hằng số đã niêm yết, không tự suy ra token nào.
//!
//! Nguồn giá: `https://claude.com/pricing` (Anthropic), đọc trực tiếp lúc dựng story này.

/// Ngày mọi giá dưới đây được đọc từ trang niêm yết của nhà cung cấp — xem doc-comment đầu
/// tệp. Một test đọc hằng số này (`ai_translate_contract.rs`) để một hàng giá không âm thầm
/// trở thành cũ mà không ai thấy.
pub const PRICES_AS_OF: &str = "2026-09-22";

/// Một hàng của bảng giá — giá niêm yết trên MỘT TRIỆU token, tách input/output vì hai chiều
/// hầu như luôn khác giá nhau ở mọi nhà cung cấp tương thích OpenAI.
struct PriceRow {
    model_id: &'static str,
    input_usd_per_million_tokens: f64,
    output_usd_per_million_tokens: f64,
}

/// Bảng giá BUNDLED — chỉ những id kho này đã tự đặt tên hôm nay (`claude-sonnet-5`, mô hình
/// đứng sau CHÍNH agent đã viết story này). Một id cục bộ (`qwen2.5:14b`, `llama3` —
/// `aiconfig_contract.rs`) CỐ Ý vắng mặt: đó là quy tắc "mô hình cục bộ" (xem doc-comment
/// [`estimate_cost_usd`]), không một khoảng trống quên điền.
///
/// Giá đọc từ `https://claude.com/pricing`, ngày [`PRICES_AS_OF`]: Sonnet 5 — $2/triệu token
/// đầu vào, $10/triệu token đầu ra.
const PRICE_TABLE: &[PriceRow] = &[PriceRow {
    model_id: "claude-sonnet-5",
    input_usd_per_million_tokens: 2.0,
    output_usd_per_million_tokens: 10.0,
}];

/// `(model_id, số token vào, số token ra) -> Option<chi phí USD>` — HÀM THUẦN, không I/O nào.
/// Nhận hai số token TRẦN (không nhận `TranslateUsage` trọn vẹn — kiểu đó, khai ở
/// `ports::translation_provider`, chính là NƠI `cost_usd` do hàm này tính ra sẽ được đặt vào;
/// nhận nó làm đầu vào sẽ vòng tròn ngược). `None` khi `model_id` không có hàng trong
/// [`PRICE_TABLE`] — đây LÀ quy tắc "mô hình cục bộ" của Quyết định Ice 2026-09-22 (spec 4.11
/// §Always): absence from the table is the only signal, không có bộ dò `localhost` nào ở đây
/// hay bất kỳ đâu khác trong module này.
///
/// Không làm tròn ở đây — số `f64` trần đi ra, làm tròn/định dạng (dấu phẩy thập phân) là
/// việc của webview (spec 4.11 §Code Map: "The decimal comma is produced as a param, by a
/// pure function" ở phía `src/`, không ở đây).
pub fn estimate_cost_usd(model_id: &str, prompt_tokens: u32, completion_tokens: u32) -> Option<f64> {
    let row = PRICE_TABLE.iter().find(|r| r.model_id == model_id)?;
    let input_cost = f64::from(prompt_tokens) / 1_000_000.0 * row.input_usd_per_million_tokens;
    let output_cost = f64::from(completion_tokens) / 1_000_000.0 * row.output_usd_per_million_tokens;
    Some(input_cost + output_cost)
}
