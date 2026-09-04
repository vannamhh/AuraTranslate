//! Phát hiện bảng mã — FR126, Story 6.3, AD-39 bước 1.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BA TRẠNG THÁI TIN CẬY LÀ LUẬT CỦA TA — `chardetng` KHÔNG CẤP MỘT CON SỐ NÀO
//! ─────────────────────────────────────────────────────────────────────────────
//! Toàn bộ API công khai của `chardetng::EncodingDetector` là `new` · `feed` · `guess`
//! (`chardetng-1.0.0/src/lib.rs:2938,3002,3204`) — **không `guess_assess`, không điểm tin
//! cậy**. `feed` trả một `bool` nghĩa hẹp *"đã thấy ít nhất một byte không phải ASCII"*
//! (`:2929-2931`). [`Confidence`] là một phán quyết TA tự dựng, tất định, đo được — không
//! phải một con số thư viện cấp. Xem §Design Notes của spec 6.3 cho phép đo đầy đủ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 PHẠM VI CỦA PHÉP SO "CÙNG MỘT CHUỖI" LÀ BẮT BUỘC (vòng rà đối kháng 2026-09-04)
//! ─────────────────────────────────────────────────────────────────────────────
//! Phép so chạy trên [`EVIDENCE_WINDOW_BYTES`] byte ĐẦU — CÙNG cửa sổ mà `chardetng` nhìn
//! (`decode_evidence_window`/[`detect`] dùng đúng một biến `window`). Một phép so trên bản
//! ĐÃ CẮT NGẮN để hiển thị (6-8 ký tự) từng làm một tệp GBK mở đầu bằng `"Chapter 01\r\n"`
//! (12 ký tự ASCII) kết luận sai "tin cậy cao" — bốn ứng viên trùng nhau đúng trong 12 ký
//! tự đầu, dải không mở, trên chính loại tệp FR126 tồn tại để cứu. Cắt ngắn CHỈ xảy ra ở
//! [`render_candidates`], sau khi phán quyết đã chốt trên cửa sổ đầy đủ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 UTF-16 BỊ LOẠI KHỎI PHÉP BIỂU QUYẾT "CÙNG MỘT CHUỖI"
//! ─────────────────────────────────────────────────────────────────────────────
//! `b"abcd"` giải mã UTF-16LE ra `扡摣` — hợp lệ, không lỗi, khác hẳn bốn ứng viên byte
//! kia. Cho nó dự vote thì MỌI tệp ASCII thuần rơi xuống "tin cậy thấp" (Big5/GBK/GB18030
//! cũng khớp ASCII, nhưng UTF-16 luôn khác) — trái hàng ma trận đã đóng băng *"Tệp thuần
//! ASCII ⇒ tự đoán, tin cậy cao"*. `chardetng` cũng không bao giờ trả UTF-16
//! (`chardetng-1.0.0/src/lib.rs`, `grep "UTF-16"` cho 0 dòng) — đường DUY NHẤT UTF-16 vào
//! được là BOM ([`sniff_bom`]).

use encoding_rs::{Encoding, GB18030, GBK, UTF_8, UTF_16BE, UTF_16LE, BIG5};

/// Năm nhãn FR126 khai thành **DỮ LIỆU**, đúng thứ tự PRD `prd.md:355` — AC1 spec 6.3: cổng
/// đỏ khi một nhãn bị đổi CHỖ, không chỉ khi một nhãn biến mất.
pub const FR126_LABELS: [&str; 5] = ["UTF-8", "GB18030", "GBK", "Big5", "UTF-16"];

/// Bảng mã ĐẠI DIỆN cho từng nhãn ở CÙNG chỉ số với [`FR126_LABELS`] — dữ liệu song song,
/// một bảng, một chỗ (§Always spec 6.3).
///
/// ⚠️ Ô `UTF-16` dùng `UTF_16LE` khi KHÔNG có BOM (lựa chọn tuỳ ý, tài liệu hoá ở đây vì
/// không có căn cứ nào khác để chọn LE thay vì BE khi thiếu BOM) — ô này gần như luôn hiện
/// "không ra chữ" hoặc chữ vô nghĩa cho một nguồn KHÔNG PHẢI UTF-16 thật, và đó là hành vi
/// ĐÚNG: FR126 liệt đủ năm nhãn trong dải, kể cả khi một nhãn hiếm khi là lựa chọn đúng
/// khi thiếu BOM. Khi CÓ BOM, [`detect`] trả sớm ở nhánh "nguồn tự khai" và dải không bao
/// giờ mở — ô này không bao giờ được người dùng thấy trong ca đó.
const FR126_CANDIDATE_ENCODINGS: [&Encoding; 5] = [UTF_8, GB18030, GBK, BIG5, UTF_16LE];

/// Bốn ứng viên "byte-đơn-vị" tham gia phép biểu quyết "cùng một chuỗi" — LOẠI `UTF-16`
/// (xem doc-comment đầu tệp). Chỉ số vào [`FR126_CANDIDATE_ENCODINGS`]/[`FR126_LABELS`].
const VOTING_CANDIDATE_INDICES: [usize; 4] = [0, 1, 2, 3];

/// Cỡ cửa sổ bằng chứng — số byte ĐẦU dùng cho CẢ HAI nửa của phán quyết (`chardetng` đoán
/// VÀ phép so "cùng một chuỗi" của [`detect`]). 4 KiB đủ vượt qua một tiêu đề ASCII ngắn
/// (12 ký tự trong ca hồi quy vòng rà đối kháng 2026-09-04) trong khi vẫn giới hạn chi phí
/// cho một tệp ở trần [`super::import::MAX_IMPORT_BYTES`] (100 MB) — decode một cửa sổ nhỏ,
/// không toàn bộ tệp.
const EVIDENCE_WINDOW_BYTES: usize = 4096;

/// Số ký tự hiển thị của mỗi ô trong dải năm ứng viên — UX pattern epic 6: *"kèm bản dựng
/// thật 6-8 ký tự đầu Chương"*. Chọn cận trên (8) — nhiều chữ hơn cho mắt phân xử tốt hơn.
const PREVIEW_CHARS: usize = 8;

fn evidence_window(bytes: &[u8]) -> &[u8] {
    &bytes[..bytes.len().min(EVIDENCE_WINDOW_BYTES)]
}

/// Cắt `s` còn tối đa `n` KÝ TỰ (không byte) — an toàn cho UTF-8 đa byte.
fn truncate_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Giải mã `window` bằng `encoding`, DÒNG CHẢY (`last: false`) — một ký tự nhiều byte bị
/// cắt đúng ở biên cửa sổ KHÔNG được tính là "không ra chữ" (GIỮ, §Spec Change Log vòng rà
/// 1: `encoding_rs` không coi một chuỗi byte đang dang dở ở cuối input `last=false` là
/// `Malformed` — nó đợi thêm byte). Chỉ `DecoderResult::Malformed` (byte THẬT SỰ không hợp
/// lệ với bảng mã này) trả `None`.
///
/// 🔴 SỬA (vòng rà đối kháng 2, mục 12) — một chuỗi giải mã THÀNH CÔNG nhưng CHỈ TOÀN
/// KHOẢNG TRẮNG (byte non-whitespace window rơi hết vào vùng bị `window` cắt, hoặc nguồn
/// thật sự chỉ có khoảng trắng ở cửa sổ bằng chứng) vẫn trả `None` — với mắt người xem ô
/// xem trước, một chuỗi trắng và "không ra chữ" là CÙNG MỘT tín hiệu (ô trống), và để nó
/// đếm là "giải mã được" trong phép biểu quyết "cùng một chuỗi" của [`detect`] cho một tín
/// hiệu tin cậy giả (hai bảng mã cùng map một dải byte về toàn khoảng trắng không chứng
/// minh được gì về CHỮ THẬT). `out` RỖNG (window rỗng) không rơi vào nhánh này — xem
/// `render_candidates`/`detect` cho lý do window không bao giờ rỗng ở chỗ gọi thật.
fn decode_prefix_streaming(encoding: &'static Encoding, window: &[u8]) -> Option<String> {
    let mut decoder = encoding.new_decoder_without_bom_handling();
    let needed = decoder
        .max_utf8_buffer_length_without_replacement(window.len())
        .unwrap_or(window.len() * 4 + 4);
    let mut out = String::with_capacity(needed);
    let (result, _read) = decoder.decode_to_string_without_replacement(window, &mut out, false);
    let decoded = match result {
        encoding_rs::DecoderResult::InputEmpty => Some(out),
        // Không nên xảy ra khi `needed` được tính đúng — giữ phần đã giải mã thay vì
        // nuốt mất, đúng hơn là coi cả ô "không ra chữ" vì một lỗi TÍNH SỨC CHỨA.
        encoding_rs::DecoderResult::OutputFull => Some(out),
        encoding_rs::DecoderResult::Malformed { .. } => None,
    };
    decoded.filter(|s| s.is_empty() || !s.trim().is_empty())
}

/// Giải mã CẢ NĂM ứng viên trên CÙNG một `window` — nguồn DUY NHẤT mà [`detect`] (biểu
/// quyết trên bốn ứng viên đầu) và [`render_candidates`] (hiển thị cả năm) cùng đọc, đúng
/// bất biến "hai nửa phán quyết nhìn cùng một cửa sổ bằng chứng".
fn decode_all_five(window: &[u8]) -> [Option<String>; 5] {
    std::array::from_fn(|i| decode_prefix_streaming(FR126_CANDIDATE_ENCODINGS[i], window))
}

/// Ba trạng thái tin cậy — luật CỦA TA (xem doc-comment đầu tệp), không phải một điểm số
/// thư viện cấp. Quyết định "dải có MỞ hay không" sống ở TẦNG HIỂN THỊ
/// (`src/importPreviewState.ts`, so `confidence === 'low'`), không phải ở kiểu này — xem
/// doc-comment `commands::project::ImportEncodingPreview::candidates` cho lý do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// Nguồn tự khai bảng mã — có BOM, đã là văn bản (`ChapterInput::AlreadyText`), hoặc
    /// đầu vào rỗng. Mặc định dải KHÔNG mở.
    SelfDeclared,
    /// Tự đoán, tin cậy cao — đoán rơi trong FR126, ≥2 ứng viên giải mã được, mọi ứng viên
    /// giải mã được cho cùng một chuỗi. Mặc định dải KHÔNG mở.
    HighGuess,
    /// Tự đoán, tin cậy thấp — mọi ca còn lại. Mặc định dải MỞ.
    LowGuess,
}

/// Phán quyết bảng mã cho MỘT nguồn byte thô.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodingVerdict {
    /// Bảng mã đang CHỌN — mặc định của ô "đang chọn" trong dải, hoặc bảng mã DUY NHẤT
    /// khi tin cậy cao/nguồn tự khai.
    pub encoding: &'static Encoding,
    pub confidence: Confidence,
}

/// Ngửi BOM ở ĐẦU `bytes` — `EF BB BF` (UTF-8) · `FF FE` (UTF-16LE) · `FE FF` (UTF-16BE).
/// Đây là nguồn DUY NHẤT trả về UTF-16 với đúng thứ tự byte — không có BOM thì không có
/// cách nào phân biệt LE/BE mà không đoán mò (xem doc-comment [`FR126_CANDIDATE_ENCODINGS`]).
pub fn sniff_bom(bytes: &[u8]) -> Option<&'static Encoding> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Some(UTF_8);
    }
    // UTF-32LE's BOM (FF FE 00 00) bắt đầu bằng đúng hai byte của BOM UTF-16LE (FF FE) --
    // phải loại UTF-32 TRƯỚC, không thì 4 byte đầu của một tệp UTF-32LE bị đọc nhầm thành
    // UTF-16LE (rồi giải mã ra rác). UTF-32 không có trong FR126 nên khi gặp BOM của nó, ta
    // không tự nhận là "đã khai báo" -- rơi xuống `None` để `detect()` đoán bằng chardetng
    // (chardetng sẽ không đoán ra UTF-32, nhưng ít nhất không giả vờ chắc chắn sai).
    if bytes.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) || bytes.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
        return None;
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return Some(UTF_16LE);
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return Some(UTF_16BE);
    }
    None
}

/// Phán quyết bảng mã cho `bytes` — BOM trước ([`sniff_bom`]), rồi `chardetng` đoán, rồi
/// ánh xạ vào ba trạng thái [`Confidence`] (luật CỦA TA). Thứ tự BOM-trước-rồi-đoán là BẮT
/// BUỘC: `chardetng` không bao giờ trả UTF-16 (xem doc-comment đầu tệp), nên bỏ qua BOM là
/// đóng cửa DUY NHẤT mà UTF-16 vào được.
///
/// ⚠️ `bytes` RỖNG ⇒ [`Confidence::SelfDeclared`] (không byte nào để mắt phân xử, khuôn
/// UTF-8 — I/O Matrix hàng "Văn bản dán tay" áp dụng nghĩa tương tự cho `AlreadyText`,
/// nhưng nhánh đó không gọi hàm này — xem `commands::project::preview_import_encoding`).
pub fn detect(bytes: &[u8]) -> EncodingVerdict {
    if let Some(encoding) = sniff_bom(bytes) {
        return EncodingVerdict { encoding, confidence: Confidence::SelfDeclared };
    }
    if bytes.is_empty() {
        return EncodingVerdict { encoding: UTF_8, confidence: Confidence::SelfDeclared };
    }

    let window = evidence_window(bytes);
    let five = decode_all_five(window);

    let mut detector = chardetng::EncodingDetector::new(chardetng::Iso2022JpDetection::Deny);
    // "If you want to perform detection on just the prefix of a longer stream, do not
    // pass last=true after the prefix" (doc-comment `EncodingDetector::feed`) — `window`
    // có thể là một PHẦN của `bytes` lớn hơn, nên `last: false`.
    detector.feed(window, false);
    let guess = detector.guess(None, chardetng::Utf8Detection::Allow);

    let guess_index = FR126_CANDIDATE_ENCODINGS[..4].iter().position(|&e| e == guess);

    let decodable: Vec<usize> =
        VOTING_CANDIDATE_INDICES.iter().copied().filter(|&i| five[i].is_some()).collect();
    // 🔴 SỬA (vòng rà đối kháng 2026-09-04, defect #3) — `windows(2).all()` trên 0 hoặc 1
    // phần tử trả `true` một cách RỖNG. Gác `decodable.len() >= 2` TRƯỚC khi gọi `windows`
    // — một tệp mà KHÔNG bảng nào giải mã được (0 ứng viên) không còn rơi vào "cùng một
    // chuỗi" == true chỉ vì tập rỗng thoả mãn `all()` một cách vô nghĩa.
    let all_same = decodable.len() >= 2
        && decodable.windows(2).all(|w| five[w[0]] == five[w[1]]);

    // `all_same` đã tự gác `decodable.len() >= 2` bên trong nó (xem chú thích ngay trên) --
    // lặp lại điều kiện đó ở đây là một khoá THỪA, không đổi hành vi (vòng rà đối kháng 2,
    // mục 19).
    let confidence =
        if guess_index.is_some() && all_same { Confidence::HighGuess } else { Confidence::LowGuess };

    let encoding = match guess_index {
        // Đoán rơi trong FR126 — chọn nó, DÙ tin cậy cao hay thấp (ca "GBK/GB18030 hiện
        // chữ y hệt": vẫn chọn GBK, dải vẫn mở nếu Big5 khác chuỗi).
        Some(i) => FR126_CANDIDATE_ENCODINGS[i],
        // Đoán ngoài năm bảng (Shift_JIS/EUC-KR/windows-1252/…) ⇒ rơi về ứng viên GIẢI MÃ
        // ĐƯỢC đầu tiên theo thứ tự FR126 (hàng ma trận I/O Matrix "chardetng đoán ngoài
        // năm bảng"). Không ứng viên nào giải mã được ⇒ UTF-8 (mọi bảng đều thất bại,
        // không có gì hợp lý hơn để chọn mặc định).
        None => decodable.first().map(|&i| FR126_CANDIDATE_ENCODINGS[i]).unwrap_or(UTF_8),
    };

    EncodingVerdict { encoding, confidence }
}

/// Một ô trong dải năm ứng viên.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodingCandidate {
    /// Nhãn FR126 cho MẮT NGƯỜI — một trong [`FR126_LABELS`].
    pub label: &'static str,
    /// Định danh KHÔNG MẤT MÁT cho lượt xác nhận — `Encoding::name()` (tên WHATWG chuẩn,
    /// ví dụ `"UTF-16LE"` chứ không phải nhãn hiển thị gộp `"UTF-16"`). Đi ngược lại qua
    /// [`encoding_for_wire_id`]. Xem §Design Notes spec 6.3 "Nhãn đi qua dây phải KHÔNG
    /// MẤT MÁT" — lượt quay vòng `bảng mã → nhãn hiển thị → bảng mã` làm mất thứ tự byte
    /// của UTF-16; `wire_id` không đi qua nhãn hiển thị nên không có gì để mất.
    pub wire_id: &'static str,
    /// Bản dựng thật, tối đa [`PREVIEW_CHARS`] ký tự — `None` = "không ra chữ" với bảng mã
    /// này (byte không hợp lệ trong cửa sổ bằng chứng).
    pub preview: Option<String>,
}

/// Dựng dải NĂM Ô — một bản dựng thật cho mỗi nhãn FR126, theo ĐÚNG thứ tự
/// [`FR126_LABELS`]. Cắt ngắn hiển thị (6-8 ký tự) là việc CỦA HÀM NÀY và chỉ của nó — phán
/// quyết tin cậy ([`detect`]) không bao giờ so trên bản đã cắt (xem doc-comment đầu tệp).
pub fn render_candidates(bytes: &[u8]) -> Vec<EncodingCandidate> {
    let window = evidence_window(bytes);
    let five = decode_all_five(window);

    FR126_LABELS
        .iter()
        .zip(FR126_CANDIDATE_ENCODINGS.iter())
        .zip(five.iter())
        .map(|((&label, &encoding), decoded)| EncodingCandidate {
            label,
            wire_id: encoding.name(),
            preview: decoded.as_ref().map(|s| truncate_chars(s, PREVIEW_CHARS)),
        })
        .collect()
}

/// Đi NGƯỢC từ [`EncodingCandidate::wire_id`] (hoặc bất kỳ tên WHATWG hợp lệ nào) về
/// `&'static Encoding` — dùng ở lượt xác nhận, KHÔNG suy từ [`FR126_LABELS`] (mất thông tin
/// thứ tự byte của UTF-16, xem doc-comment [`EncodingCandidate::wire_id`]).
///
/// `None` ⇒ một nhãn KHÔNG NHẬN RA — chỗ gọi phải trả `IpcError` tường minh, KHÔNG âm thầm
/// rơi về UTF-8 (§Design Notes spec 6.3).
/// Danh sách CHO PHÉP — [`FR126_CANDIDATE_ENCODINGS`] cộng `UTF_16BE` (bảng mã DUY NHẤT
/// [`sniff_bom`] có thể trả về mà không có mặt trong bảng ứng viên mặc định LE).
///
/// 🔴 SỬA (vòng rà đối kháng 2, mục 6) — bản trước gọi thẳng `Encoding::for_label`, một hàm
/// nhận MỌI nhãn WHATWG hợp lệ (`Shift_JIS`, `EUC-KR`, `windows-1252`, `x-user-defined`,
/// `replacement`, …) — RỘNG HƠN hẳn hợp đồng mà chính doc-comment của
/// [`encoding_for_wire_id`] tự khai ("nhãn KHÔNG NHẬN RA ⇒ IpcError tường minh"). Danh sách
/// này là điều kiện để lời khai đó ĐÚNG: một nhãn hợp lệ về mặt WHATWG nhưng NGOÀI FR126 vẫn
/// phải bị từ chối.
const RECOGNIZED_ENCODINGS: [&Encoding; 6] = [UTF_8, GB18030, GBK, BIG5, UTF_16LE, UTF_16BE];

/// Đi NGƯỢC từ [`EncodingCandidate::wire_id`] (hoặc [`sniff_bom`]) về `&'static Encoding` —
/// dùng ở lượt xác nhận. CHỈ nhận `wire_id` mà chính module này từng CẤP (qua
/// [`RECOGNIZED_ENCODINGS`]) — không phải bất kỳ nhãn WHATWG hợp lệ nào.
pub fn encoding_for_wire_id(id: &str) -> Option<&'static Encoding> {
    let candidate = Encoding::for_label(id.as_bytes())?;
    RECOGNIZED_ENCODINGS.iter().copied().find(|&e| e == candidate)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═════════════════════════════════════════════════════════════════════════════════
    // sniff_bom
    // ═════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn sniff_bom_recognizes_all_three_boms_and_nothing_else() {
        assert_eq!(sniff_bom(&[0xEF, 0xBB, 0xBF, b'a']), Some(UTF_8));
        assert_eq!(sniff_bom(&[0xFF, 0xFE, b'a', 0]), Some(UTF_16LE));
        assert_eq!(sniff_bom(&[0xFE, 0xFF, 0, b'a']), Some(UTF_16BE));
        assert_eq!(sniff_bom(b"plain ascii"), None);
        assert_eq!(sniff_bom(&[]), None);
    }

    /// Vòng rà đối kháng 2, mục 10: BOM UTF-32LE (FF FE 00 00) bắt đầu bằng đúng hai byte
    /// của BOM UTF-16LE (FF FE) -- nếu không loại UTF-32 trước, 4 byte đầu của một tệp
    /// UTF-32LE thật bị nhận nhầm thành "đã khai báo UTF-16LE" rồi giải mã ra rác im lặng.
    #[test]
    fn sniff_bom_does_not_misread_a_utf32_bom_as_utf16() {
        // UTF-32LE: 'A' = 0x41 0x00 0x00 0x00 -- BOM rồi một ký tự.
        assert_eq!(sniff_bom(&[0xFF, 0xFE, 0x00, 0x00, 0x41, 0x00, 0x00, 0x00]), None);
        // UTF-32BE: BOM rồi 'A'.
        assert_eq!(sniff_bom(&[0x00, 0x00, 0xFE, 0xFF, 0x00, 0x00, 0x00, 0x41]), None);
    }

    // ═════════════════════════════════════════════════════════════════════════════════
    // detect — I/O Matrix
    // ═════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn empty_bytes_are_self_declared_utf8() {
        let v = detect(&[]);
        assert_eq!(v.confidence, Confidence::SelfDeclared);
        assert_eq!(v.encoding, UTF_8);
    }

    #[test]
    fn bom_short_circuits_to_self_declared_before_any_guessing() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        // Khong dau -- check:i18n Kiem A quet ca #[cfg(test)] trong src-tauri/src/**.
        bytes.extend_from_slice("mot doan van ban".as_bytes());
        let v = detect(&bytes);
        assert_eq!(v.confidence, Confidence::SelfDeclared);
        assert_eq!(v.encoding, UTF_8);
    }

    #[test]
    fn pure_ascii_is_high_confidence_because_every_candidate_agrees() {
        let bytes = b"Chapter One: a beast called Fire Dragon appeared at the gate.";
        let v = detect(bytes);
        assert_eq!(v.confidence, Confidence::HighGuess, "ASCII la tap con cua ca 4 bang");
    }

    #[test]
    fn a_gbk_file_with_a_short_ascii_header_still_reads_low_confidence() {
        // Ca hồi quy vòng rà đối kháng 2026-09-04, defect #2: "Chapter 01\r\n" la 12 ky tu
        // ASCII, roi mot doan chu Han ma GBK/Big5 giai ra HAI chuoi khac nhau.
        let (gbk_bytes, _, _) = GBK.encode("Chapter 01\r\n萧炎在东临村口的一处石壁上练习着最基础的吐纳法门。");
        let v = detect(&gbk_bytes);
        assert_eq!(
            v.confidence,
            Confidence::LowGuess,
            "tieu de ASCII ngan khong duoc che khuat sai lech o phan con lai cua cua so"
        );
    }

    #[test]
    fn no_decodable_candidate_is_low_confidence_not_a_vacuous_high() {
        // Defect #3: `windows(2).all()` tren 0/1 phan tu tra `true` mot cach RONG. Dung
        // byte KHONG hop le voi CA BON bang bang cach cho mot chuoi UTF-8 hop le nhung
        // KHONG hop le trong GBK/GB18030/Big5 (lead byte GBK hop le ma khong co trail byte
        // theo sau la mot cach chac; o day dung mot chuoi rieng cho tung bang qua feed
        // truc tiep -- muc tieu la 0 ung vien giai ma duoc trong ca bon).
        let bytes: &[u8] = &[0x81, 0x00, 0xFF, 0x80]; // khong hop le voi UTF-8/GBK/GB18030/Big5
        let v = detect(bytes);
        assert_eq!(
            v.confidence,
            Confidence::LowGuess,
            "0 ung vien giai ma duoc phai la tin cay THAP, khong phai 'cung mot chuoi' rong"
        );
    }

    #[test]
    fn a_guess_outside_fr126_falls_back_to_the_first_decodable_candidate_in_fr126_order() {
        // 🔴 SỬA (2026-09-04, phản biện Ice — bản trước dùng b"hello" KHÔNG ĐO ĐƯỢC nhánh
        // `None`). `b"hello"` là ASCII thuần: `chardetng` đoán UTF-8 trên nó (đo bằng
        // `EncodingDetector::guess` trực tiếp, `cargo test -- --nocapture`), nên `guess_index`
        // là `Some(0)` và test đi qua nhánh `Some`, KHÔNG BAO GIỜ chạm nhánh `None` mà tên nó
        // tuyên bố đang canh. Assert cũ (`FR126_CANDIDATE_ENCODINGS[..4].contains(...)`) đúng
        // ở CẢ HAI nhánh nên nó xanh vô điều kiện — đối chứng: thay `None => decodable.first()…`
        // trong `detect()` bằng `None => BIG5` (giá trị SAI trắng trợn) rồi `cargo test`: **0
        // ca đỏ**. Đây CHÍNH LÀ hàng ma trận I/O "chardetng đoán ngoài năm bảng" không có cổng
        // nào canh — spec 6.3 §I/O Matrix.
        //
        // 🔴 FIXTURE TẤT ĐỊNH — đo trước, không đoán (đo 2026-09-04, `probe_guess_outside_fr126`
        // tạm thời, đã gỡ sau khi xác nhận). Văn bản tiếng Nhật mã hoá Shift_JIS:
        // `EncodingDetector::guess` trả `Shift_JIS` — encoding này KHÔNG có mặt trong
        // `FR126_CANDIDATE_ENCODINGS` (chỉ UTF-8/GB18030/GBK/Big5/UTF-16LE), nên `guess_index`
        // TẤT ĐỊNH là `None`. Trong bốn ứng viên byte-đơn-vị, byte Shift_JIS này KHÔNG hợp lệ
        // dưới UTF-8 (không phải chuỗi UTF-8), nhưng ĐỌC ĐƯỢC dưới GB18030 và GBK (đo cùng lượt:
        // `decodable_in_fr126=["gb18030", "GBK"]`) — GB18030 đứng TRƯỚC GBK trong
        // `FR126_CANDIDATE_ENCODINGS`, nên ứng viên GIẢI MÃ ĐƯỢC ĐẦU TIÊN theo thứ tự đó là
        // GB18030.
        let (bytes, _, had_errors) = encoding_rs::SHIFT_JIS.encode(
            "これは日本語のテキストです。文字化けを避けるために十分な長さが必要です。",
        );
        assert!(!had_errors, "fixture phai ma hoa Shift_JIS sach");

        let v = detect(&bytes);

        assert_eq!(
            v.confidence,
            Confidence::LowGuess,
            "doan ngoai FR126 luon la tin cay THAP (guess_index=None loai truc tiep nhanh HighGuess)"
        );
        assert_eq!(
            v.encoding, GB18030,
            "nhanh None phai chon ung vien GIAI MA DUOC DAU TIEN theo thu tu FR126_CANDIDATE_ENCODINGS \
             -- GB18030 (chi so 1), khong phai UTF-8 (khong giai ma duoc byte Shift_JIS nay) va \
             khong phai GBK (chi so 2, dung THU HAI)"
        );
    }

    #[test]
    fn gbk_and_gb18030_candidates_render_the_identical_string() {
        let (gbk_bytes, _, _) = GBK.encode("萧炎登场");
        let candidates = render_candidates(&gbk_bytes);
        assert_eq!(candidates.len(), 5);
        assert_eq!(candidates[1].label, "GB18030");
        assert_eq!(candidates[2].label, "GBK");
        assert_eq!(
            candidates[1].preview, candidates[2].preview,
            "GBK va GB18030 dung chung mot decoder (encoding_rs-0.8.35/src/lib.rs:946)"
        );
    }

    #[test]
    fn render_candidates_is_always_five_cells_in_fr126_order() {
        let candidates = render_candidates(b"plain ascii text");
        assert_eq!(
            candidates.iter().map(|c| c.label).collect::<Vec<_>>(),
            FR126_LABELS.to_vec()
        );
    }

    #[test]
    fn a_streaming_decode_truncated_mid_multibyte_character_is_not_treated_as_undecodable() {
        // GIU (Spec Change Log vong ra 1): mot chuoi GBK hai byte bi CAT GIUA o dung bien
        // cua so bang chung khong duoc tinh la "khong ra chu".
        let (full, _, _) = GBK.encode("萧");
        // Chi lay byte DAU của ky tu hai byte -- mot chuoi dang do.
        let truncated = &full[..1];
        assert_eq!(
            decode_prefix_streaming(GBK, truncated),
            Some(String::new()),
            "byte dau dang cho -- InputEmpty, KHONG Malformed"
        );
    }

    #[test]
    fn a_truly_malformed_byte_sequence_is_undecodable() {
        // 0xFF khong bao gio la mot GBK lead byte hop le.
        assert_eq!(decode_prefix_streaming(GBK, &[0xFF, 0xFF]), None);
    }

    /// Vòng rà đối kháng 2, mục 12: một chuỗi giải mã THÀNH CÔNG nhưng chỉ toàn khoảng
    /// trắng phải bị coi như "không ra chữ" (`None`) -- cả cho ô xem trước LẪN cho phép
    /// biểu quyết "cùng một chuỗi" của `detect`. Trước lượt vá này, `decode_prefix_streaming`
    /// trả `Some("   \n\t  ")` cho cả bốn ứng viên byte-đơn-vị (khoảng trắng ASCII giống hệt
    /// nhau ở mọi bảng mã) -- `detect` khi đó vẫn kết luận đúng LowGuess CHỈ VÌ không có
    /// `guess_index` (chardetng không đoán ra một encoding cụ thể cho input toàn khoảng
    /// trắng); phép biểu quyết "cùng một chuỗi" tự nó (`all_same`) đã lặng lẽ là `true` một
    /// cách vô nghĩa (rơi vào chuỗi trắng không chứng minh gì) mà không lộ ra ngoài chỉ vì
    /// `guess_index` chặn trước đó -- một fixture khác (chardetng đoán trúng FR126 trên
    /// cùng input) có thể lộ HighGuess giả. Test này khoá ở tầng gốc: preview không còn là
    /// một chuỗi trắng.
    #[test]
    fn a_whitespace_only_decode_is_treated_as_no_text_not_a_vacuous_match() {
        assert_eq!(decode_prefix_streaming(UTF_8, b"   \n\t  "), None);
        assert_eq!(decode_prefix_streaming(GB18030, b"   \n\t  "), None);
        assert_eq!(decode_prefix_streaming(GBK, b"   \n\t  "), None);
        assert_eq!(decode_prefix_streaming(BIG5, b"   \n\t  "), None);

        let candidates = render_candidates(b"   \n\t  ");
        for label in ["UTF-8", "GB18030", "GBK", "Big5"] {
            let cell = candidates.iter().find(|c| c.label == label).unwrap();
            assert_eq!(cell.preview, None, "o {label} phai la None cho input toan khoang trang");
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════════
    // wire_id round-trip — "nhãn đi qua dây phải KHÔNG MẤT MÁT"
    // ═════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn wire_id_round_trips_losslessly_for_every_fr126_candidate_including_utf16_byte_order() {
        for &encoding in &FR126_CANDIDATE_ENCODINGS {
            let id = encoding.name();
            assert_eq!(
                encoding_for_wire_id(id),
                Some(encoding),
                "wire_id {id:?} phai giai nguoc dung ve chinh encoding da sinh ra no"
            );
        }
        // UTF-16BE khong nam trong FR126_CANDIDATE_ENCODINGS (mac dinh la LE khi khong
        // BOM) nhung van phai round-trip dung khi no la encoding DA CHON qua BOM.
        assert_eq!(encoding_for_wire_id(UTF_16BE.name()), Some(UTF_16BE));
    }

    #[test]
    fn an_unrecognized_wire_id_returns_none_not_a_silent_utf8_fallback() {
        assert_eq!(encoding_for_wire_id("not-a-real-encoding"), None);
    }

    /// 🔴 Vòng rà đối kháng 2, mục 6 — `Encoding::for_label` trần nhận MỌI nhãn WHATWG hợp
    /// lệ, không riêng FR126. `Shift_JIS`/`EUC-KR`/`windows-1252` là ba nhãn HỢP LỆ về mặt
    /// WHATWG (khác hẳn ca "not-a-real-encoding" ngay trên — chuỗi đó sai cú pháp từ đầu,
    /// không đo được liệu allowlist có thật sự lọc hay không) nhưng NGOÀI FR126 — đây mới là
    /// đối chứng cho đúng mệnh đề doc-comment tự khai ("nhãn KHÔNG NHẬN RA ⇒ IpcError").
    #[test]
    fn a_whatwg_valid_label_outside_fr126_is_rejected_not_silently_accepted() {
        for label in ["Shift_JIS", "EUC-KR", "windows-1252", "x-user-defined", "replacement"] {
            assert!(
                Encoding::for_label(label.as_bytes()).is_some(),
                "tien de: {label:?} phai la mot nhan WHATWG HOP LE (khac 'not-a-real-encoding')"
            );
            assert_eq!(
                encoding_for_wire_id(label),
                None,
                "{label:?} hop le ve WHATWG nhung NGOAI FR126 -- phai bi TU CHOI"
            );
        }
    }
}
