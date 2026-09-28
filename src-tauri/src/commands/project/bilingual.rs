//! Nhập tài liệu song ngữ hai cột (Story 6.16, FR115) — tách ra khỏi
//! `commands/project/mod.rs`, không đổi hành vi.
//!
//! `use super::*;` mang mọi kiểu/hàm dùng chung của `commands::project` vào đây.

use super::*;

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.16 — Nhập tài liệu song ngữ hai cột (FR115, AD-39 · AD-37/46 · AD-47 ③)
// ═════════════════════════════════════════════════════════════════════════════════
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 STATE TÁI DÙNG, KHÔNG MỘT HỘP THỨ BA
// ─────────────────────────────────────────────────────────────────────────────
// Vai cột (`source_column`/`target_column`) và cờ tiêu đề là tham số MỖI LƯỢT xem trước/xác
// nhận, cùng khuôn `chapter_pattern` — KHÔNG một `Mutex<...>` mới cạnh
// `Tier2BlockOverridesState`/`ChapterOriginOverridesState`. `PendingImportSourceState` (đã
// có, Story 6.3) giữ nguyên vai trò: `stash_pending_import_source`/`cancel_import_preview`
// dùng ĐƯỢC NGUYÊN cho `PipelineShape::Bilingual` — chỉ byte thô của tệp được cất, đổi vai
// cột/tiêu đề không đọc lại đĩa (§I/O Matrix "Swap columns"/"Header checkbox on": "counts
// rebuild"/"rebuilds the preview in memory").

/// Một hàng lệch cặp trên dây — Story 6.16, mở rộng Story 6.17 (FR116) với đủ dữ kiện để
/// webview dựng màn quy nhóm KHÔNG cần hỏi lại Rust: `source_sentences`/`target_line` cũng là
/// ẢNH CHỤP webview echo lại nguyên vẹn trong [`BilingualRegroupingWire`] (staleness check của
/// [`crate::core::segment::bilingual::resolve`]); `candidate_positions`/`initial_cuts`/
/// `proposed_cuts` xem doc-comment các hàm cùng tên ở `core::segment::bilingual`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BilingualMismatchWire {
    pub chapter_index: usize,
    pub row_number: usize,
    pub source_sentences: Vec<String>,
    pub target_line: String,
    pub target_sentence_count: usize,
    pub candidate_positions: Vec<usize>,
    pub initial_cuts: Vec<usize>,
    pub proposed_cuts: Vec<usize>,
}

impl From<&crate::core::segment::bilingual::BilingualMismatch> for BilingualMismatchWire {
    fn from(m: &crate::core::segment::bilingual::BilingualMismatch) -> Self {
        BilingualMismatchWire {
            chapter_index: m.chapter_index,
            row_number: m.row_number,
            source_sentences: m.source_sentences.clone(),
            target_line: m.target_line.clone(),
            target_sentence_count: m.target_sentence_count,
            candidate_positions: m.candidate_positions.clone(),
            initial_cuts: m.initial_cuts.clone(),
            proposed_cuts: m.proposed_cuts.clone(),
        }
    }
}

/// Một lượt quy nhóm trên dây — Story 6.17 (FR116). `kind`/`cuts` tách rời (không một enum
/// gắn thẻ) — cùng khuôn phẳng [`ChapterPatternWire`] đã theo cho `kind`. `cuts` bị bỏ qua khi
/// `kind == Skip` (webview gửi `[]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum BilingualRegroupingKindWire {
    #[serde(rename = "cuts")]
    Cuts,
    #[serde(rename = "skip")]
    Skip,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BilingualRegroupingWire {
    pub row_number: usize,
    pub source_sentences: Vec<String>,
    pub target_line: String,
    pub kind: BilingualRegroupingKindWire,
    pub cuts: Vec<usize>,
}

impl From<BilingualRegroupingWire> for crate::core::segment::bilingual::BilingualRegrouping {
    fn from(w: BilingualRegroupingWire) -> Self {
        use crate::core::segment::bilingual::{BilingualRegrouping, BilingualRegroupingAction};
        let action = match w.kind {
            BilingualRegroupingKindWire::Cuts => BilingualRegroupingAction::Cuts(w.cuts),
            BilingualRegroupingKindWire::Skip => BilingualRegroupingAction::Skip,
        };
        BilingualRegrouping {
            row_number: w.row_number,
            source_sentences: w.source_sentences,
            target_line: w.target_line,
            action,
        }
    }
}

/// Kết quả chạy TRỌN chuỗi bảy bước cho MỘT ứng viên bảng mã — Story 6.16. `chapter_count`/
/// `pair_count`/`mismatches` đều RỖNG/0 khi ứng viên này "không ra chữ" trong cửa sổ bằng
/// chứng (`preview == None`, cùng khuôn `EncodingCandidateWire`).
#[derive(Debug, Clone, serde::Serialize)]
pub struct BilingualEncodingCandidateWire {
    pub label: String,
    pub encoding: String,
    pub preview: Option<String>,
    pub row_count: usize,
    pub chapter_count: usize,
    pub pair_count: usize,
    /// **THÊM (vòng rà đối kháng, Story 6.17, FR116)** — tổng câu đích của mọi hàng đã giải
    /// quyết bằng Skip (nguồn rỗng) khi ứng viên này chạy TRỌN chuỗi — §I/O Matrix "Skip
    /// blank source": "count shown in preview". Cạnh `pair_count`, cùng lý do: cả hai đều là
    /// một tổng TÍNH LẠI mỗi lượt chạy, không suy từ `mismatches` (hàng Skip đã biến mất khỏi
    /// danh sách đó ngay khi được giải quyết — xem doc-comment
    /// [`crate::core::segment::pipeline::PipelineOutput::bilingual_skipped_target_sentence_count`]).
    pub skipped_target_sentence_count: usize,
    pub mismatches: Vec<BilingualMismatchWire>,
    /// **THÊM (Story 6.16b)** — khối tách Chương (tầng 4, Story 6.6/6.10), tính bằng CHÍNH
    /// [`build_chapter_split_preview_wire`] mà đường tệp/dán tay/URL đã dùng (§Approach spec
    /// 6.16b: "reuse, do not rebuild") — không một hàng rào Tukey thứ hai. `broken_item_count`
    /// LUÔN `0` (đường song ngữ không có khái niệm mục hỏng, §Always spec 6.16b). `origin_overrides`
    /// LUÔN rỗng (`&[]`) — màn xem trước song ngữ không có bề mặt sửa xuất xứ.
    ///
    /// 🔴 **`None` ⇔ `outcome` (chạy `run_pipeline` cho CHÍNH ứng viên này) là `None`/`Err` —
    /// KHÔNG chỉ khi `preview == None`.** Hai điều kiện KHÔNG đồng bộ: `preview` chỉ nói "bảng
    /// mã này ra chữ được trên cửa sổ bằng chứng" (`encoding::render_candidates`), còn
    /// `run_pipeline` chạy TRỌN bảy bước (kể cả table-parse) trên TOÀN văn bản — một ứng viên
    /// có thể ra chữ được (`preview: Some`) mà vẫn hỏng ở bước bảng (`TooFewColumns`,
    /// `UnterminatedQuotedField`) hoặc bất kỳ lỗi `run_pipeline` nào khác của CHÍNH ứng viên
    /// đó, cho `chapters: None` trong khi `preview` vẫn `Some` — xem
    /// `a_table_parse_failure_on_a_non_selected_candidate_leaves_its_chapters_none_others_unaffected`
    /// (`bilingual_import_contract.rs`).
    pub chapters: Option<ChapterSplitPreviewWire>,
}

/// Dải năm ứng viên trên dây — Story 6.16, cùng khuôn [`ImportEncodingPreview`].
#[derive(Debug, Clone, serde::Serialize)]
pub struct BilingualImportEncodingPreview {
    pub confidence: ConfidenceWire,
    pub selected_encoding: String,
    pub candidates: Vec<BilingualEncodingCandidateWire>,
    /// Tối đa [`crate::core::segment::pipeline::BILINGUAL_SAMPLE_ROW_CAP`] hàng đầu tiên
    /// của bảng mã ĐANG CHỌN — mọi cột, để webview dựng thẻ chọn cột nguồn/đích với dữ liệu
    /// THẬT thay vì tên cột suông. Rỗng khi tệp rỗng hoặc bảng mã đang chọn không ra chữ.
    pub sample_rows: Vec<Vec<String>>,
    /// Tổng số hàng của bảng mã ĐANG CHỌN — §I/O Matrix "Fewer than 2 columns" cũng lộ ra ở
    /// đây khi chuỗi từ chối: `0` cùng `candidates` rỗng thân trong (chuỗi trả `Err`, không
    /// một ứng viên nào chạy được tới cuối).
    pub row_count: usize,
    /// Số cột rộng nhất đếm được ở bảng mã ĐANG CHỌN — webview dùng để dựng danh sách lựa
    /// chọn cột nguồn/đích (0-based, `0..column_count`). `0` khi tệp rỗng.
    pub column_count: usize,
}

/// **Hàm thuần** — dò bảng mã VÀ chạy TRỌN chuỗi bảy bước cho MỖI ứng viên (AD-39: "table
/// parsing happens right after decode, inside the chain, and re-runs on every encoding
/// candidate"). Cùng khuôn [`preview_import_encoding`]: KHÔNG tự đọc gì, KHÔNG tự lưu
/// state — vỏ `mod wire` cấp `shape` (đọc tệp MỘT LẦN ở `preview_bilingual_import_from_file`,
/// hoặc clone từ ô đang chờ ở `rebuild_bilingual_import_preview`) rồi gọi hàm này.
///
/// `bilingual_source_column`/`bilingual_target_column`/`bilingual_has_header` là tham số MỖI
/// LƯỢT gọi (xem §Design Notes đầu mục) — đổi cột/tiêu đề chỉ đòi gọi lại hàm này với
/// CÙNG `shape` (byte thô không đổi), 0 lượt đọc đĩa thêm.
///
/// `cleanup_rules` — luật làm sạch đã phân giải hai tầng, cùng nguồn mà lượt xác nhận đọc lại
/// lúc xác nhận (§Always spec 6.16: "Cleanup and normalize run per cell, both columns").
///
/// # Lỗi
/// 🔵 **SỬA 2026-09-11 (Story 6.16, bước nghiệm thu)** — bản đầu nuốt MỌI `Err` của chuỗi
/// bằng `.ok()`: một tệp một cột, hay một ô mở ngoặc kép không đóng, vẫn hiện một màn xem
/// trước toàn số 0 với nút xác nhận BẬT, và lỗi chỉ lộ ra khi bấm xác nhận — trái hàng I/O
/// Matrix "Fewer than 2 columns: Refused before preview". Nay hai lỗi HÌNH DẠNG BẢNG của ứng
/// viên ĐANG CHỌN được trả lên (`import.bilingual_too_few_columns`,
/// `import.bilingual_unterminated_quoted_field`); lỗi của các ứng viên KHÁC vẫn chỉ làm ứng
/// viên đó rỗng, cùng khuôn dung thứ của [`preview_import_encoding`].
pub fn preview_bilingual_import(
    shape: &PipelineShape,
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    chapter_pattern: Option<&ChapterPattern>,
    bilingual_source_column: usize,
    bilingual_target_column: usize,
    bilingual_has_header: bool,
    // 🔴 **THÊM 2026-09-12 (Story 6.17, FR116)** — quy nhóm câu đích đang chờ, tham số MỖI
    // LƯỢT gọi cùng khuôn ba tham số vai cột/tiêu đề ngay trên. `preview_bilingual_import_from_file`
    // (lượt MỞ đầu tiên) luôn truyền `&[]`; `rebuild_bilingual_import_preview` truyền lại danh
    // sách hiện hành mỗi lượt người dùng sửa một chỗ cắt.
    regroupings: &[crate::core::segment::bilingual::BilingualRegrouping],
) -> Result<BilingualImportEncodingPreview, IpcError> {
    let PipelineShape::Bilingual { input, .. } = shape else {
        // Chỉ `mod wire` dựng `shape` cho hàm này, luôn từ `import_bilingual_file` — nhánh
        // này là phòng thủ kiểu (một lỗi lập trình, không một đường sản phẩm), không phải
        // một trạng thái người dùng gây ra được.
        return Ok(BilingualImportEncodingPreview {
            confidence: ConfidenceWire::SelfDeclared,
            selected_encoding: encoding_rs::UTF_8.name().to_owned(),
            candidates: Vec::new(),
            sample_rows: Vec::new(),
            row_count: 0,
            column_count: 0,
        });
    };
    let bytes: &[u8] = match input {
        ChapterInput::RawBytes { bytes, .. } => bytes,
        // `import_bilingual_file` luôn dựng `RawBytes` — nhánh này không nên chạm trên
        // đường sản phẩm, cùng lý lẽ nhánh `PipelineShape` không khớp ở trên.
        ChapterInput::AlreadyText(_) => &[],
    };
    let verdict = encoding::detect(bytes);

    let mut selected_sample_rows: Vec<Vec<String>> = Vec::new();
    let mut selected_row_count = 0usize;
    let mut selected_column_count = 0usize;
    let mut selected_seen = false;
    // Lỗi hình dạng bảng của ứng viên ĐANG CHỌN — xem mục "# Lỗi" của doc-comment.
    let mut selected_refusal: Option<ImportError> = None;

    let candidates: Vec<BilingualEncodingCandidateWire> = if bytes.is_empty() {
        Vec::new()
    } else {
        // Đường song ngữ không bao giờ bóc nội dung chính (không HTML, chỉ bảng CSV/TSV) —
        // `extract_main_content` luôn `false`, cùng lý do `Blob` ở `preview_import_encoding`.
        encoding::render_candidates(bytes, source_lang, false, "")
            .into_iter()
            .map(|c| {
                let is_selected = c.wire_id == verdict.encoding.name();
                let result = encoding::encoding_for_wire_id(c.wire_id).map(|enc| {
                    run_pipeline(
                        PipelineInput::with_encoding(shape.clone(), enc, source_lang)
                            .with_cleanup_rules(cleanup_rules.to_vec())
                            .with_chapter_pattern(chapter_pattern.cloned())
                            .with_bilingual_columns(
                                bilingual_source_column,
                                bilingual_target_column,
                                bilingual_has_header,
                            )
                            .with_bilingual_regroupings(regroupings.to_vec()),
                    )
                });
                let outcome = match result {
                    Some(Ok(o)) => Some(o),
                    Some(Err(err)) => {
                        if is_selected && selected_refusal.is_none() && is_bilingual_table_refusal(&err) {
                            selected_refusal = Some(err);
                        }
                        None
                    }
                    None => None,
                };

                let (chapter_count, pair_count, mismatches, row_count, column_count, skipped_target_sentence_count, chapters_wire) =
                    match &outcome {
                        Some(o) => {
                            let pair_count: usize = o
                                .chapters
                                .iter()
                                .filter_map(|c| c.bilingual_segments.as_ref())
                                .map(|s| s.len())
                                .sum();
                            let mismatches: Vec<BilingualMismatchWire> =
                                o.bilingual_mismatches.iter().map(BilingualMismatchWire::from).collect();
                            let column_count =
                                o.bilingual_sample_rows.iter().map(Vec::len).max().unwrap_or(0);
                            // **THÊM (Story 6.16b)** — tầng 4 (Story 6.10), TÍNH LẠI trên CHÍNH
                            // lượt chạy chuỗi thật vừa dựng `o.chapters` ở trên (không một lượt
                            // `run_pipeline` thứ hai). `broken_item_count = 0`, `origin_overrides
                            // = &[]` — xem doc-comment `BilingualEncodingCandidateWire::chapters`.
                            let chapters_wire = build_chapter_split_preview_wire(&o.chapters, 0, &[]);
                            (
                                o.chapters.len(),
                                pair_count,
                                mismatches,
                                o.bilingual_row_count,
                                column_count,
                                o.bilingual_skipped_target_sentence_count,
                                Some(chapters_wire),
                            )
                        }
                        None => (0, 0, Vec::new(), 0, 0, 0, None),
                    };

                if is_selected && !selected_seen {
                    selected_seen = true;
                    if let Some(o) = &outcome {
                        selected_sample_rows = o.bilingual_sample_rows.clone();
                        selected_row_count = row_count;
                        selected_column_count = column_count;
                    }
                }

                BilingualEncodingCandidateWire {
                    label: c.label.to_owned(),
                    encoding: c.wire_id.to_owned(),
                    preview: c.preview,
                    row_count,
                    chapter_count,
                    pair_count,
                    skipped_target_sentence_count,
                    mismatches,
                    chapters: chapters_wire,
                }
            })
            .collect()
    };

    if let Some(err) = selected_refusal {
        return Err(err.into());
    }

    Ok(BilingualImportEncodingPreview {
        confidence: verdict.confidence.into(),
        selected_encoding: verdict.encoding.name().to_owned(),
        candidates,
        sample_rows: selected_sample_rows,
        row_count: selected_row_count,
        column_count: selected_column_count,
    })
}

/// Hai lỗi HÌNH DẠNG BẢNG mà màn xem trước song ngữ phải TỪ CHỐI thay vì hiện một dải số 0 —
/// xem mục "# Lỗi" của [`preview_bilingual_import`].
fn is_bilingual_table_refusal(err: &ImportError) -> bool {
    matches!(
        err,
        ImportError::BilingualTooFewColumns { .. }
            | ImportError::BilingualUnterminatedQuotedField { .. }
            // 🔴 THÊM 2026-09-12 (Story 6.17, FR116) — một `Skip` gửi sai (cả hai phía đều có
            // câu) phải TỚI được webview như một lỗi typed, không bị nuốt cùng khuôn "ứng viên
            // này không ra chữ" — xem §I/O Matrix "Skip refused".
            | ImportError::BilingualSkipNotAllowed { .. }
    )
}

/// **Hàm thuần** — lõi lượt xác nhận song ngữ: CLONE nguồn đang chờ từ
/// [`PendingImportSourceState`] (tái dùng, xem §Design Notes đầu mục), gọi [`create_work`],
/// dọn ô đang chờ khi và chỉ khi THÀNH CÔNG — cùng khuôn
/// [`confirm_import_with_encoding`].
///
/// # Lỗi
/// - `state` rỗng ⇒ `import.no_pending_source`;
/// - `encoding_wire_id` không giải ngược được ⇒ `import.unrecognized_encoding`;
/// - còn hàng lệch cặp ⇒ `import.bilingual_mismatched_rows` — [`create_work`] tự kiểm lại
///   (Rust-side, §Boundaries), nên đây LUÔN đúng dù webview có quên gọi
///   [`preview_bilingual_import`] lại sau lượt sửa cuối hay không.
pub fn confirm_bilingual_import(
    documents_root: &Path,
    state: &PendingImportSourceState,
    name: &str,
    source_lang: &str,
    genre: &str,
    encoding_wire_id: &str,
    cleanup_rules: Vec<CleanupRule>,
    chapter_pattern: Option<ChapterPattern>,
    bilingual_source_column: usize,
    bilingual_target_column: usize,
    bilingual_has_header: bool,
    // 🔴 **THÊM 2026-09-12 (Story 6.17, FR116)** — quy nhóm câu đích, cùng khuôn ba tham số vai
    // cột/tiêu đề ngay trên (tham số MỖI LƯỢT, không state).
    regroupings: Vec<crate::core::segment::bilingual::BilingualRegrouping>,
) -> Result<OpenWork, IpcError> {
    let chosen = encoding::encoding_for_wire_id(encoding_wire_id).ok_or_else(|| {
        IpcError::from(ImportError::UnrecognizedEncoding { wire_id: encoding_wire_id.to_owned() })
    })?;

    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let shape = guard.as_ref().map(|p| p.shape.clone()).ok_or_else(no_pending_import_source)?;

    let opened = create_work(
        documents_root,
        name,
        source_lang,
        genre,
        shape,
        chosen,
        // 🔵 **SỬA 2026-09-11 (Story 6.16, bước nghiệm thu)** — bản đầu truyền `Vec::new()`
        // ở đây VÀ ở màn xem trước, nên nhánh làm sạch theo ô của `Step::CleanByRules` chạy
        // trên 0 luật: luật người dùng đã bật không bao giờ tới đường song ngữ (§Always spec
        // 6.16: "Cleanup and normalize run per cell, both columns"). Vỏ `wire` phân giải hai
        // tầng LÚC XÁC NHẬN, cùng kỷ luật `confirm_import_with_encoding`. Override khối và
        // override xuất xứ vẫn rỗng: hai bề mặt đó không có trên màn xem trước song ngữ.
        cleanup_rules,
        chapter_pattern,
        Vec::new(),
        bilingual_source_column,
        bilingual_target_column,
        bilingual_has_header,
        &[],
        &std::sync::Mutex::new(Vec::new()),
        None,
        &regroupings,
    )?;

    *guard = None;
    Ok(opened)
}
