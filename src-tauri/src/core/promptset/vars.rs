//! Từ vựng biến số đóng của thân prompt — Quyết định #2 của spec 4.4, cộng phép quét dấu
//! ngoặc `{{...}}` trả về CẢ HAI điều kiện cảnh báo (Quyết định #3, I/O Matrix hai hàng
//! cuối).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 ĐÚNG BA TÊN, VÀ `{{chapter_context}}` KHÔNG CÓ MẶT — CÓ CHỦ Ý
//! ─────────────────────────────────────────────────────────────────────────────
//! `{{glossary_terms}}` · `{{source_segment}}` · `{{tm_similar_segments}}` — Quyết định #2:
//! *"Until now these names existed only as mockup content; this is where they become
//! code."* `{{chapter_context}}` mockup đánh dấu *chưa dùng* và bị loại có chủ ý; nó KHÔNG
//! được thêm vào [`PromptVariable::ALL`]. Story 4.6 (`RagInjector`) tiêu thụ đúng enum này
//! thay vì tự khai một danh sách thứ hai có thể trôi khỏi đây — cùng lý do `message_keys!`
//! tồn tại (`src-tauri/AGENTS.md:15`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHÔNG BAO GIỜ TỪ CHỐI MỘT THÂN PROMPT VÌ DẤU NGOẶC CỦA NÓ
//! ─────────────────────────────────────────────────────────────────────────────
//! Quyết định #3: một `{{foo}}` lạ được lưu NGUYÊN VĂN và cảnh báo, GỌI TÊN token đó — từ
//! chối sẽ cấm luôn việc gõ `{{` như văn bản thường. Thiếu hẳn `{{glossary_terms}}` cũng
//! không bị từ chối — chỉ cảnh báo rằng Glossary Enforcement đang tắt cho bộ này (I/O
//! Matrix). [`scan_markers`] vì thế không bao giờ trả `Err`; nó chỉ trả cảnh báo.

/// Ba biến số ratify — và đúng ba, không hơn.
///
/// [`Self::ALL`] là nguồn sự thật DUY NHẤT mà cả màn soạn thảo (danh sách chèn biến) lẫn
/// [`scan_markers`] đọc — không có bản chép tay thứ hai ở đâu khác trong repo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PromptVariable {
    /// Thuật ngữ Glossary đã tiêm — nửa Glossary của FR70 (AD-14). Đây là biến số DUY NHẤT
    /// mà sự VẮNG MẶT của nó tự nó đã là một cảnh báo riêng — xem [`MarkerWarnings`].
    GlossaryTerms,
    /// Câu nguồn đang dịch.
    SourceSegment,
    /// Các cặp TM tương tự — chữ ký của `RagInjector` (AD-14) chừa tham số này RỖNG cho
    /// tới Epic 7; biến số vẫn ratify ở đây vì mockup đã vẽ nó.
    TmSimilarSegments,
}

impl PromptVariable {
    /// Cả ba, và đúng ba — thứ tự này là thứ tự hiển thị của danh sách chèn biến ở màn
    /// soạn thảo.
    pub const ALL: &'static [PromptVariable] = &[
        PromptVariable::GlossaryTerms,
        PromptVariable::SourceSegment,
        PromptVariable::TmSimilarSegments,
    ];

    /// Tên bên trong dấu ngoặc — KHÔNG mang `{{`/`}}`. Định danh máy đọc (AD-21, NFR16).
    pub const fn as_str(self) -> &'static str {
        match self {
            PromptVariable::GlossaryTerms => "glossary_terms",
            PromptVariable::SourceSegment => "source_segment",
            PromptVariable::TmSimilarSegments => "tm_similar_segments",
        }
    }

    /// Toàn văn dấu ngoặc như nó xuất hiện trong một thân prompt — thứ màn soạn thảo chèn
    /// vào con trỏ và thứ `RagInjector` (Story 4.6) tìm-và-thay.
    pub const fn marker(self) -> &'static str {
        match self {
            PromptVariable::GlossaryTerms => "{{glossary_terms}}",
            PromptVariable::SourceSegment => "{{source_segment}}",
            PromptVariable::TmSimilarSegments => "{{tm_similar_segments}}",
        }
    }

    /// 🔵 **THÊM 2026-09-18 (Story 4.6, rà soát) — `pub(crate)`, không còn `fn` riêng của
    /// module.** `core::ai::rag::expand_prompt_body` cần đúng phép tra này (token → biến số
    /// đã ratify) để mở rộng marker; trước bản vá này nó tự chép lại thân hàm bằng
    /// `PromptVariable::ALL.iter().copied().find(...)` — một bản chép tay THỨ HAI, đúng thứ
    /// doc-comment của [`PromptVariable::ALL`] cam kết không tồn tại trong kho.
    pub(crate) fn from_token(token: &str) -> Option<Self> {
        PromptVariable::ALL.iter().copied().find(|v| v.as_str() == token)
    }
}

/// Kết quả của [`scan_markers`] — CẢ HAI điều kiện cảnh báo của Quyết định #3, không bao
/// giờ một `Err`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarkerWarnings {
    /// Toàn văn (`"{{foo}}"`) của mỗi dấu ngoặc KHÔNG khớp [`PromptVariable::ALL`], theo
    /// thứ tự gặp lần đầu trong thân — một token lạ lặp lại nhiều lần chỉ được liệt MỘT
    /// lần. `{{chapter_context}}` rơi vào đây, đúng như Quyết định #2 đòi.
    pub unknown_markers: Vec<String>,
    /// `true` khi thân KHÔNG chứa `{{glossary_terms}}` dù chỉ một lần — I/O Matrix: "Save a
    /// prompt with no `{{glossary_terms}}` ... a warning says Glossary Enforcement is off
    /// for this set".
    pub glossary_terms_missing: bool,
}

impl MarkerWarnings {
    /// Không cảnh báo nào — thân sạch, mang đúng `{{glossary_terms}}` và không token lạ.
    pub fn is_clean(&self) -> bool {
        self.unknown_markers.is_empty() && !self.glossary_terms_missing
    }
}

/// Quét thân prompt tìm `{{...}}`, phân loại theo [`PromptVariable::ALL`], KHÔNG BAO GIỜ
/// sửa/từ chối thân — chỉ trả cảnh báo (Quyết định #3).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 KHÔNG BẮT LỒNG NHAU, KHÔNG BẮT `{{` MỒ CÔI — VÀ ĐÓ LÀ CHỦ Ý
/// ─────────────────────────────────────────────────────────────────────────────
/// Quét tuyến tính trái sang phải: tìm `"{{"`, tìm `"}}"` GẦN NHẤT sau nó, phần ở giữa là
/// token. Một `"{{"` không có `"}}"` đi theo sau nó trong phần thân CÒN LẠI dừng lượt quét
/// tại đó — phần thân còn lại (mang `{{` mồ côi) không bị quét tiếp, và ĐÓ KHÔNG PHẢI một
/// lỗi: Quyết định #3 nói thẳng một thân được phép gõ `{{` như văn bản thường, nên một dấu
/// mở không khép không được phép làm cả phép quét sụp đổ hay ném lỗi.
pub fn scan_markers(body: &str) -> MarkerWarnings {
    let mut unknown_markers: Vec<String> = Vec::new();
    let mut has_glossary_terms = false;
    let mut rest = body;

    while let Some(open_at) = rest.find("{{") {
        let after_open = &rest[open_at + 2..];
        let Some(close_at) = after_open.find("}}") else {
            break;
        };
        let token = &after_open[..close_at];

        // 🔵 SUA (Phase 2, do bang do voi ma san pham that): mot "{{" mo coi DUNG TRUOC
        // mot marker that lam token GOM CA marker do -- "ghi chu {{ rieng: {{glossary_terms}}"
        // tra token " rieng: {{glossary_terms", nuot mat marker that. Khi token con mang
        // mot "{{" khac, "{{" hien tai chi la van ban thuong (Quyet dinh #3 cho phep) --
        // khong xu ly no nhu mot marker, chi doi lai vi tri quet toi DUNG "{{" ben trong ma
        // token vua tim thay, roi quet lai tu do.
        if token.contains("{{") {
            rest = after_open;
            continue;
        }

        match PromptVariable::from_token(token) {
            Some(PromptVariable::GlossaryTerms) => has_glossary_terms = true,
            Some(_) => {}
            None => {
                let marker_text = format!("{{{{{token}}}}}");
                if !unknown_markers.contains(&marker_text) {
                    unknown_markers.push(marker_text);
                }
            }
        }

        rest = &after_open[close_at + 2..];
    }

    MarkerWarnings {
        unknown_markers,
        glossary_terms_missing: !has_glossary_terms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_three_ratified_names_round_trip_through_as_str_and_marker() {
        for v in PromptVariable::ALL {
            assert_eq!(v.marker(), format!("{{{{{}}}}}", v.as_str()));
            assert_eq!(PromptVariable::from_token(v.as_str()), Some(*v));
        }
    }

    #[test]
    fn a_clean_body_carrying_all_three_has_no_warnings() {
        let body = "Dich: {{source_segment}}\nThuat ngu: {{glossary_terms}}\nTM: {{tm_similar_segments}}";
        let warnings = scan_markers(body);
        assert!(warnings.is_clean());
        assert!(warnings.unknown_markers.is_empty());
        assert!(!warnings.glossary_terms_missing);
    }

    #[test]
    fn an_unknown_marker_is_named_verbatim_and_not_rejected() {
        let warnings = scan_markers("Dung {{glosary_terms}} thay vi ten dung.");
        assert_eq!(warnings.unknown_markers, vec!["{{glosary_terms}}".to_owned()]);
        // Ten go sai khong duoc tinh la co mat -- glossary_terms van thieu.
        assert!(warnings.glossary_terms_missing);
    }

    #[test]
    fn chapter_context_is_excluded_from_the_ratified_vocabulary_and_reads_as_unknown() {
        let warnings = scan_markers("{{chapter_context}} chua dung.");
        assert_eq!(warnings.unknown_markers, vec!["{{chapter_context}}".to_owned()]);
    }

    #[test]
    fn missing_glossary_terms_warns_without_rejecting() {
        let warnings = scan_markers("{{source_segment}} khong co thuat ngu.");
        assert!(warnings.glossary_terms_missing);
        assert!(warnings.unknown_markers.is_empty());
        assert!(!warnings.is_clean());
    }

    #[test]
    fn a_repeated_unknown_token_is_reported_once() {
        let warnings = scan_markers("{{foo}} va lai {{foo}} lan nua.");
        assert_eq!(warnings.unknown_markers, vec!["{{foo}}".to_owned()]);
    }

    #[test]
    fn an_unclosed_double_brace_does_not_panic_or_reject() {
        let warnings = scan_markers("mo ngoac {{ khong bao gio khep.");
        assert!(warnings.unknown_markers.is_empty());
        assert!(warnings.glossary_terms_missing);
    }

    #[test]
    fn an_empty_body_is_clean_of_unknown_markers_but_missing_glossary_terms() {
        let warnings = scan_markers("");
        assert!(warnings.unknown_markers.is_empty());
        assert!(warnings.glossary_terms_missing);
    }

    /// 🔴 Doi chung do cua Phase 2 -- CONTROL line trong bao cao khiem khuyet, giu nguyen
    /// de mot lan sua sau nay khong the lam vo phep phat hien ma khong bi bat.
    #[test]
    fn the_measured_control_line_reports_glossary_terms_present_and_no_unknown_markers() {
        let warnings = scan_markers("{{glossary_terms}}");
        assert!(!warnings.glossary_terms_missing);
        assert!(warnings.unknown_markers.is_empty());
    }

    /// 🔴 Doi chung do cua Phase 2 -- ACTUAL line do duoc tren ma san pham that: mot "{{" mo
    /// coi TRUOC mot marker that khong duoc phep nuot marker do. Truoc ban va: tra
    /// `glossary_terms_missing = true` va `unknown_markers = ["{{ rieng: {{glossary_terms}}"]`
    /// -- CA HAI deu sai.
    #[test]
    fn a_stray_opening_brace_before_a_real_marker_does_not_swallow_it() {
        let warnings = scan_markers("ghi chu {{ rieng: {{glossary_terms}}");
        assert!(warnings.unknown_markers.is_empty());
        assert!(!warnings.glossary_terms_missing);
    }

    /// Cung mot dang mo coi, nhung marker theo sau la MOT token la -- token la van phai duoc
    /// goi ten dung, khong bi phan mo coi phia truoc lam sai lech vi tri.
    #[test]
    fn a_stray_opening_brace_before_an_unknown_marker_still_names_it_correctly() {
        let warnings = scan_markers("note {{ stray: {{foo}} more");
        assert_eq!(warnings.unknown_markers, vec!["{{foo}}".to_owned()]);
    }
}
