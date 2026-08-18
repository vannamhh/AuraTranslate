//! Mối ghép giữa **nhịp flush của Editor** (TypeScript) và **cửa rảnh của luồng
//! checkpoint** (Rust) — Story 2.4 · Task 5 · §Điều kiện khởi hành mục 4, ràng buộc ①.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 TỆP NÀY ÔM ĐÚNG MỘT MỆNH ĐỀ, VÀ ĐÓ LÀ MỆNH ĐỀ DUY NHẤT CHƯA CÓ CHỦ
//! ─────────────────────────────────────────────────────────────────────────────
//! `Tuning::idle_before_passive` được đặt *"cố ý dài hơn nhịp flush 2 s của AD-35"*
//! (`core/store/mod.rs:207-208`). Đó là một ràng buộc **giữa hai workspace tách rời**, và
//! trước tệp này nó chỉ sống bằng **một câu chữ trong doc-comment**.
//!
//! ⚠️ Hai NỬA đơn-ngôn-ngữ đã có chủ rồi — đừng chép chúng vào đây, một mệnh đề hai chủ
//! là hai nguồn sự thật:
//!
//! | Mệnh đề | Chủ đang sống |
//! | --- | --- |
//! | `EDITOR_IDLE_MS == 2000` · `EDITOR_HARD_CAP_MS == 5000` | `tests/frontend/editorFlush.test.ts:56-57` (vitest) |
//! | hành vi của `createWriteSchedule` (idle + trần không reset) | `scripts/check-layout.mjs` Kiểm B |
//! | `Tuning::default()` dựng đúng sáu số | `store_contract.rs` |
//! | **quan hệ `idle_before_passive` ⟷ `EDITOR_IDLE_MS`** | 🔵 **tệp này, từ 2026-08-18** |
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO NÓ PHẢI LÀ MỘT PHÉP KIỂM, KHÔNG PHẢI MỘT DÒNG NHẮC
//! ─────────────────────────────────────────────────────────────────────────────
//! Lớp lỗi là **hai lượt sửa riêng biệt**: một lượt hạ `idle_before_passive`, một lượt
//! khác — có thể cách nhau nhiều tuần và do người khác làm — nâng `EDITOR_IDLE_MS`. Mỗi
//! lượt **một mình nó đều hợp lệ**, đi qua sạch `cargo test`, cả chín cổng đọc-tệp, vitest
//! và `npm run build`. Chỉ **tổ hợp** của hai lượt mới hỏng, và hỏng của nó là luồng
//! checkpoint chạy ĐÈ lên đường gõ — biểu hiện thành *"gõ bị khựng"*, đúng lớp triệu chứng
//! mà `writeSchedule.ts` đã phải trả giá một lần để tìm ra.
//!
//! ⚠️ Bản Task 5 cũ chỉ có **một dòng nhắc** cho ràng buộc này, và story tự ghi ra chỗ
//! yếu: *"hai lượt sửa riêng biệt (hạ cái này, nâng cái kia) lọt hết mọi cổng cho tới khi
//! Task 11 chạy xong"*. Task 11 là **cuối** story — tức khoảng hở dài bằng cả story.
//!
//! 🔵 **Dựng TRƯỚC lượt hiệu chỉnh, có chủ ý.** Task 5 viết *"sau khi cả hai số đã hiệu
//! chỉnh"*, nhưng thứ tệp này giữ là **quan hệ**, không phải **giá trị** — nó đúng cả
//! trước lẫn sau lượt hiệu chỉnh, và dựng trước nghĩa là chính lượt hiệu chỉnh cũng nằm
//! trong tầm canh của nó. Nếu số đo sau này đòi **phá** bất biến, phép kiểm này đỏ và
//! chuyện đó nổi lên thành một quyết định của Ice — đúng cái nó tồn tại để làm.
//!
//! Số hôm nay 2026-08-18, đo từ nguồn: `idle_before_passive = 5 s` · `EDITOR_IDLE_MS =
//! 2000 ms` ⇒ dư **3 000 ms**.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use auratranslate_lib::core::store::Tuning;

/// Tệp khai ba hằng nhịp flush của Editor, tính từ `CARGO_MANIFEST_DIR` (`src-tauri/`).
const EDITOR_FLUSH_TS: &str = "../src/panels/editorFlush.ts";

fn read_editor_flush() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(EDITOR_FLUSH_TS);
    // 🔴 Không đọc được là **lỗi hạ tầng**, không phải một phép kiểm đạt. Cùng luật với
    // `abort()` của các cổng `scripts/check-*.mjs`: đừng bao giờ báo một kết quả không có
    // thật. Tệp bị đổi chỗ ⇒ ca này chết ồn ào, và đó là hành vi đúng.
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("khong doc duoc {}: {e}", path.display()))
}

/// Bóc giá trị của `export const <name> = <số>` từ mã nguồn TypeScript.
///
/// 🔴 **Chỉ nhận ở vị trí KHAI BÁO.** `editorFlush.ts` nhắc cả ba tên trong doc-comment
/// nhiều lần (`[`EDITOR_IDLE_MS`]`, …); một phép so chuỗi con sẽ đếm cả những chỗ đó rồi
/// hoặc bắt được số sai, hoặc báo "khai trùng" oan. Phép kiểm chạy **theo dòng** và đòi
/// dòng **bắt đầu bằng** `export const <name>`, nên mọi lần nhắc trong chú thích — vốn
/// luôn mở đầu bằng ` * ` hay `//` — rơi ra ngoài.
///
/// ⚠️ Trả `Err` chứ không phải một giá trị mặc định ở **mọi** ca không chắc chắn. Một hằng
/// bị đổi tên mà phép kiểm lặng lẽ đi tiếp là một phép kiểm xanh giả — đúng thứ mà bất
/// biến này tồn tại để chống.
fn declared_ms(source: &str, name: &str) -> Result<u64, String> {
    let needle = format!("export const {name}");
    let mut hits: Vec<u64> = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim_start();
        let Some(after) = trimmed.strip_prefix(needle.as_str()) else {
            continue;
        };
        // Chặn khớp TIỀN TỐ: `EDITOR_IDLE_MS` không được nuốt `EDITOR_IDLE_MS_V2`.
        if after.starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '$') {
            continue;
        }
        let Some(eq) = after.find('=') else {
            return Err(format!(
                "dong khai `{name}` khong co dau `=`: {trimmed:?}"
            ));
        };
        let digits: String = after[eq + 1..]
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '_')
            .filter(|c| *c != '_')
            .collect();
        if digits.is_empty() {
            return Err(format!(
                "`export const {name}` co mat nhung ve phai khong phai mot so nguyen: {trimmed:?}"
            ));
        }
        let value: u64 = digits
            .parse()
            .map_err(|e| format!("`{name}` = {digits:?} khong doc ra so: {e}"))?;
        hits.push(value);
    }

    match hits.len() {
        0 => Err(format!(
            "khong thay `export const {name}` — hang da bi doi ten hoac doi cho. \
             Day la loi HA TANG cua phep kiem, khong phai mot ket qua dat."
        )),
        1 => Ok(hits[0]),
        n => Err(format!(
            "`export const {name}` khai {n} lan — khong biet tin cai nao"
        )),
    }
}

/// Bất biến ① của §mục 4: cửa rảnh của checkpoint phải **dài hơn HẲN** nhịp flush.
///
/// Vì sao **hẳn** chứ không phải `>=`: bằng nhau nghĩa là hai mốc rơi vào cùng một thời
/// điểm, tức đúng ca *"checkpoint đánh nhau với đường gõ"* mà `mod.rs:207-208` viết ra để
/// tránh. Một `>=` cho ca xấu nhất đi qua.
fn invariant_holds(idle_before_passive: Duration, editor_idle_ms: u64) -> bool {
    idle_before_passive > Duration::from_millis(editor_idle_ms)
}

#[test]
fn the_checkpoint_idle_gate_stays_strictly_longer_than_the_editor_flush_cadence() {
    let source = read_editor_flush();
    let editor_idle_ms =
        declared_ms(&source, "EDITOR_IDLE_MS").unwrap_or_else(|e| panic!("{e}"));
    let idle_before_passive = Tuning::default().idle_before_passive;

    assert!(
        invariant_holds(idle_before_passive, editor_idle_ms),
        "Bất biến ① của Story 2.4 §mục 4 bị phá: `Tuning::idle_before_passive` = {:?} \
         KHÔNG dài hơn `EDITOR_IDLE_MS` = {} ms.\n\
         \n\
         Hai số này sống ở hai workspace tách rời:\n\
         · Rust — `src-tauri/src/core/store/mod.rs`, `Tuning::default()`\n\
         · TypeScript — `src/panels/editorFlush.ts`\n\
         \n\
         Đổi một trong hai mà quên số kia làm luồng checkpoint chạy ĐÈ lên đường gõ. \
         Nếu đây là một lượt hiệu chỉnh CÓ CHỦ Ý của Story 2.4 thì nó là một quyết định \
         của Ice, không phải một lượt sửa hằng — xem AC5.",
        idle_before_passive,
        editor_idle_ms
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// TỰ KIỂM — chứng minh phép kiểm trên ĐỎ ĐƯỢC, và không đỏ oan
// ─────────────────────────────────────────────────────────────────────────────
// Luật của kho: *"một cổng chưa bao giờ đỏ là một cổng chưa ai biết nó có chạy không"*.
// Bốn ca dưới lái thẳng hai hàm thuần bằng chuỗi dựng tay, nên chúng không cần tệp thật
// và không chậm.

#[test]
fn the_parser_finds_exactly_one_declaration_in_the_real_file_despite_many_comment_mentions() {
    let source = read_editor_flush();

    // 🔴 Ca này KHÔNG ghim giá trị. `EDITOR_IDLE_MS == 2000` đã có chủ ở
    // `tests/frontend/editorFlush.test.ts:56-57`; ghim lại ở đây là dựng nguồn sự thật thứ
    // hai, và nó sẽ bắt lượt hiệu chỉnh của Story 2.4 phải sửa HAI chỗ cho một con số.
    // Thứ ca này nghiệm thu là **bộ bóc chạy đúng trên tệp thật**: `Ok` chỉ trả về khi tìm
    // thấy ĐÚNG MỘT dòng khai báo.
    for name in ["EDITOR_IDLE_MS", "EDITOR_HARD_CAP_MS", "EDITOR_RETRY_FLOOR_MS"] {
        assert!(
            declared_ms(&source, name).is_ok(),
            "khong boc duoc `{name}` tu tep that: {:?}",
            declared_ms(&source, name)
        );
    }

    // Và ca trên chỉ có nghĩa nếu tệp thật ĐÚNG LÀ có nhiều lần nhắc trong chú thích —
    // nếu không, "bỏ qua chú thích" là một mệnh đề chưa được thử.
    let mentions = source.matches("EDITOR_IDLE_MS").count();
    assert!(
        mentions > 1,
        "tep that chi nhac `EDITOR_IDLE_MS` {mentions} lan — ca nay khong con chung minh \
         duoc rang chu thich bi bo qua; kiem lai truoc khi tin no"
    );
}

#[test]
fn the_parser_refuses_a_source_where_the_constant_only_lives_in_a_comment() {
    let fake = " * Nhac toi [`EDITOR_IDLE_MS`] = 9999 trong mot chu thich\n\
                 // export const EDITOR_IDLE_MS = 9999\n";
    assert!(declared_ms(fake, "EDITOR_IDLE_MS").is_err());
}

#[test]
fn the_parser_refuses_a_renamed_or_duplicated_declaration() {
    // Đổi tên ⇒ Err, KHÔNG phải một giá trị mặc định lặng lẽ.
    let renamed = "export const EDITOR_IDLE_MILLIS = 2000\n";
    assert!(declared_ms(renamed, "EDITOR_IDLE_MS").is_err());

    // Khớp tiền tố KHÔNG được tính là khai báo.
    let prefix = "export const EDITOR_IDLE_MS_V2 = 2000\n";
    assert!(declared_ms(prefix, "EDITOR_IDLE_MS").is_err());

    // Khai hai lần ⇒ Err, không tự chọn một cái.
    let twice = "export const EDITOR_IDLE_MS = 2000\nexport const EDITOR_IDLE_MS = 7000\n";
    assert!(declared_ms(twice, "EDITOR_IDLE_MS").is_err());
}

#[test]
fn the_invariant_goes_red_on_an_equal_pair_and_on_an_inverted_pair() {
    // 🟢 hình dạng hôm nay
    assert!(invariant_holds(Duration::from_secs(5), 2_000));
    // 🔴 bằng nhau — hai mốc rơi cùng thời điểm
    assert!(!invariant_holds(Duration::from_secs(5), 5_000));
    // 🔴 đảo ngược — nhịp flush dài hơn cửa rảnh
    assert!(!invariant_holds(Duration::from_secs(2), 5_000));
}
