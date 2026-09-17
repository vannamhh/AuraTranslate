//! Smart RAG Injector — Story 4.6, AD-14, FR70 (nửa Glossary). Hai lớp tách hẳn: một hàm TẠP
//! ([`gather_glossary_context`], đọc `Store`, chính xác MỘT lượt gọi Glossary) và một hàm
//! THUẦN ([`assemble_prompt`], không `Store`, không `ScopeResolver`, không khớp, không đồng
//! hồ, không I/O — nhận kết quả hàm tạp đã gom sẵn). Cùng nhau chúng khớp chữ ký AD-14
//! `(source sentence, scope, Glossary, TM) -> assembled prompt`; tách riêng, chỉ nửa THUẦN là
//! thứ cả ma trận I/O của spec 4.6 nghiệm thu — Epic 7 thêm tham số TM thật vào CẢ HAI hàm mà
//! không đổi hình dạng chữ ký.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 `ai/` GỌI ĐÚNG MỘT CỬA VÀO GLOSSARY — `confirmed_terms_for_injection`
//! ─────────────────────────────────────────────────────────────────────────────
//! `core::glossary::confirmed_terms_for_injection` là cửa DUY NHẤT (Decision 5, spec 4.6):
//! nó đã tra hai tầng, phân xử chồng nhau (Decision 4: một mục *chờ chốt* che một mục *đã
//! chốt* thì cả hai đều KHÔNG được chèn) và lọc `is_confirmed` — module này KHÔNG tự phân xử
//! gì, KHÔNG gọi bất kỳ hàm phơi-dữ-liệu-thô nào khác của Glossary (kể cả hàm dựng lưới của
//! Story 3.4). `tests/ai_boundary.rs` cưỡng chế đúng NĂM tên được phép: cửa này, kiểu nó trả
//! về (`GlossaryInjectionOutcome`), `GlossaryError`, `match_lang_for_source_lang`, và
//! `GlossaryTier` (rà soát 2026-09-18 — ledger phải mang lại `tier` của cửa để Story 4.7 và
//! AC "lưới/ledger cùng span" đọc được nó; xem `InjectedGlossaryTerm`/`SuppressedGlossaryTerm`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BA-TRẠNG-THÁI, MỌI NƠI MỘT KẾT QUẢ CÓ THỂ VẮNG MẶT (root `AGENTS.md`: rỗng im lặng là
//! lớp lỗi trung tâm)
//! ─────────────────────────────────────────────────────────────────────────────
//! Glossary: *chưa hỏi* ([`GlossaryInjectionStatus::NotAsked`] — thân không mang
//! `{{glossary_terms}}`, KHÔNG lượt gọi Glossary nào chạy) tách hẳn khỏi *đã hỏi, không thấy
//! gì* ([`GlossaryInjectionStatus::Asked`] với `injected` rỗng). TM: *chưa dựng*
//! ([`TmInjectionStatus::NotBuiltYet`], tham số là `None` — TM chưa tồn tại tới Epic 7) tách
//! hẳn khỏi *đã tra, không khớp gì* ([`TmInjectionStatus::Searched`] với lát cắt rỗng) — hai
//! trạng thái đó không bao giờ được viết cùng một cách.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! MỘT LƯỢT QUÉT TRÊN THÂN GỐC — nội dung tiêm KHÔNG BAO GIỜ bị quét lại
//! ─────────────────────────────────────────────────────────────────────────────
//! [`assemble_prompt`] duyệt `body` (thân prompt gốc) đúng MỘT LẦN, trái sang phải, và ghi
//! phần thay thế thẳng vào chuỗi kết quả. Một bản dịch đã chốt mang literal
//! `{{source_segment}}` vẫn đi vào prompt NGUYÊN VĂN — nó không bao giờ được coi là một
//! marker thứ hai, vì con trỏ quét đã đi QUA điểm chèn đó trước khi nội dung được đặt vào.

use crate::core::glossary::{
    GlossaryError, GlossaryInjectionOutcome, GlossaryTier, confirmed_terms_for_injection,
    match_lang_for_source_lang,
};
use crate::core::promptset::{PromptVariable, scan_markers};
use crate::core::scope::ScopeResolver;
use crate::core::store::Store;
use crate::core::tm::SimilarSegment;

// ═════════════════════════════════════════════════════════════════════════════════
// Ledger — những gì Story 4.7 (prompt inspector) đọc lại
// ═════════════════════════════════════════════════════════════════════════════════

/// Một cặp Glossary đã tiêm vào prompt.
///
/// 🔴 **Mang `start`/`end`/`tier` — rà soát 2026-09-18.** Bản đầu chỉ mang `source_term`/
/// `translation`, đánh rơi đúng ba trường `GlossaryInjectionTerm` (cửa Glossary) đã tính sẵn
/// — mà không có chúng, AC 2 của spec ("mỗi thuật ngữ ledger tiêm là một dấu ĐÃ CHỐT ở ĐÚNG
/// CÙNG SPAN trên lưới") không kiểm được TẠI ledger, chỉ kiểm được tại cửa. `start`/`end` là
/// ĐIỂM MÃ, cùng đơn vị `GlossaryMark` của lưới (Story 3.4) — so sánh trực tiếp, không quy
/// đổi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectedGlossaryTerm {
    /// Thuật ngữ nguồn.
    pub source_term: String,
    /// Bản dịch đã chốt được tiêm.
    pub translation: String,
    /// Vị trí ĐIỂM MÃ bắt đầu (bao gồm) trong câu nguồn — cùng đơn vị `GlossaryMark::start`.
    pub start: usize,
    /// Vị trí ĐIỂM MÃ kết thúc (không bao gồm).
    pub end: usize,
    /// Tầng thắng (AD-18).
    pub tier: GlossaryTier,
}

/// Một mục ĐÃ CHỐT bị một mục CHỜ CHỐT che khi hai bên chồng nhau (Decision 4, spec 4.6) —
/// không được tiêm, nhưng ledger vẫn gọi tên nó cho Story 4.7's *"considered but not
/// injected"*. Mang cùng ba trường `start`/`end`/`tier` với [`InjectedGlossaryTerm`], cùng lý
/// do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuppressedGlossaryTerm {
    /// Thuật ngữ nguồn của mục đã chốt bị che.
    pub source_term: String,
    /// Bản dịch đã chốt của mục bị che.
    pub translation: String,
    /// Vị trí ĐIỂM MÃ bắt đầu (bao gồm) của mục bị che.
    pub start: usize,
    /// Vị trí ĐIỂM MÃ kết thúc (không bao gồm).
    pub end: usize,
    /// Tầng của mục bị che.
    pub tier: GlossaryTier,
}

/// Trạng thái Glossary của MỘT lượt gọi [`assemble_prompt`] — hai giá trị, không collapse.
///
/// 🔴 `NotAsked` và `Asked { injected: vec![], .. }` là HAI biến thể `enum` khác nhau, không
/// một cờ `bool` cạnh một `Vec` rỗng — cùng kỷ luật mà `library_work.status IS NULL` giữ ở
/// tầng SQL (root `AGENTS.md`: "một giá trị có thể UNKNOWN nhận `Option`, không bao giờ một
/// `0`/mặc định lặng").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlossaryInjectionStatus {
    /// Thân không mang `{{glossary_terms}}` — [`gather_glossary_context`] không gọi Glossary,
    /// đúng I/O Matrix *"Marker absent … no Glossary query runs at all"*.
    NotAsked,
    /// Đã hỏi — `injected` có thể rỗng (*"asked and empty"*), và `suppressed_by_pending_overlap`
    /// gọi tên mọi mục đã chốt bị Decision 4 che.
    Asked {
        injected: Vec<InjectedGlossaryTerm>,
        suppressed_by_pending_overlap: Vec<SuppressedGlossaryTerm>,
    },
}

/// Trạng thái TM của MỘT lượt gọi [`assemble_prompt`] — ba giá trị đúng nghĩa đen của tên nó:
/// [`Self::NotBuiltYet`] khác [`Self::Searched`] dù `Searched` mang lát cắt RỖNG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TmInjectionStatus {
    /// TM chưa tồn tại (trước Epic 7) — tham số `tm` của [`assemble_prompt`] là `None`.
    NotBuiltYet,
    /// TM đã tra — `Vec` rỗng nghĩa là *"đã tra, không khớp gì"*, KHÔNG lẫn với
    /// [`Self::NotBuiltYet`].
    Searched(Vec<SimilarSegment>),
}

/// Kết quả đầy đủ của MỘT lượt [`assemble_prompt`] — thứ Story 4.7 đọc để vẽ prompt inspector
/// (FR71) và Story 4.8 đọc để quyết có gửi được hay không.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectionLedger {
    pub glossary: GlossaryInjectionStatus,
    pub tm: TmInjectionStatus,
    /// Toàn văn (`"{{foo}}"`) mỗi dấu ngoặc lạ trong thân — [`scan_markers`], không một bản
    /// quét thứ hai.
    pub unknown_markers: Vec<String>,
    /// `true` khi thân KHÔNG mang `{{source_segment}}` — Decision 6: assembling vẫn thành
    /// công (§Never của spec cấm mọi hành vi từ chối ở đây), nhưng một prompt không có câu để
    /// dịch không được rời khỏi hàm này mà tự mô tả là "ổn".
    pub source_segment_missing: bool,
}

// ═════════════════════════════════════════════════════════════════════════════════
// Lớp TẠP — MỘT lượt gọi Glossary, bỏ qua hoàn toàn khi thân không cần nó
// ═════════════════════════════════════════════════════════════════════════════════

/// Nửa KHÔNG THUẦN của AD-14 — đọc `Store` qua đúng MỘT cửa Glossary. Bỏ qua HOÀN TOÀN lượt
/// gọi đó khi `body` không mang `{{glossary_terms}}` (I/O Matrix *"Marker absent"*): không
/// query nào chạy, và [`GlossaryInjectionStatus::NotAsked`] tự nó nói vì sao.
///
/// `source_lang` đi qua [`match_lang_for_source_lang`] để suy `MatchLang` — chính hằng số
/// module này được phép gõ, cùng cách `commands::glossary` đã làm.
///
/// # Lỗi
/// [`GlossaryError`] truyền thẳng từ tầng gom dữ liệu (I/O Matrix *"Store unreadable ⇒ lỗi
/// truyền từ tầng gom, assembler không bao giờ thấy nó"*) — [`assemble_prompt`] không nhận
/// tham số này, nên nó không có cơ hội thấy lỗi.
pub fn gather_glossary_context(
    body: &str,
    resolver: &ScopeResolver,
    global: &Store,
    work: Option<&Store>,
    source_lang: &str,
    sentence: &str,
) -> Result<GlossaryInjectionStatus, GlossaryError> {
    if scan_markers(body).glossary_terms_missing {
        return Ok(GlossaryInjectionStatus::NotAsked);
    }

    let lang = match_lang_for_source_lang(source_lang);
    let GlossaryInjectionOutcome { injected, suppressed_by_pending_overlap } =
        confirmed_terms_for_injection(resolver, global, work, sentence, lang)?;

    Ok(GlossaryInjectionStatus::Asked {
        injected: injected
            .into_iter()
            .map(|t| InjectedGlossaryTerm {
                source_term: t.source_term,
                translation: t.translation,
                start: t.start,
                end: t.end,
                tier: t.tier,
            })
            .collect(),
        suppressed_by_pending_overlap: suppressed_by_pending_overlap
            .into_iter()
            .map(|t| SuppressedGlossaryTerm {
                source_term: t.source_term,
                translation: t.translation,
                start: t.start,
                end: t.end,
                tier: t.tier,
            })
            .collect(),
    })
}

// ═════════════════════════════════════════════════════════════════════════════════
// Lớp THUẦN — Story 4.6's assembler
// ═════════════════════════════════════════════════════════════════════════════════

/// Dựng khối văn bản mà `{{glossary_terms}}` mở rộng thành — Decision 2: bare
/// `"source term → confirmed translation"`, một cặp một dòng, theo đúng thứ tự ledger báo cáo
/// (thứ tự [`InjectedGlossaryTerm`] đã mang sẵn — không sắp lại).
///
/// 🔴 `.iter()` trên TOÀN BỘ `injected`, không `.take(1)` — Pass 1 (2026-09-17) đo được một
/// lượt `.take(1)` ở đúng vị trí này để lọt `ai_rag_contract` (19 passed, 0 failed) vì mọi ca
/// đọc pair qua `.contains` một cặp, không so KHỚP TOÀN VĂN với hai cặp trở lên. Ca đối chứng
/// mới (`ai_rag_contract.rs`) so `result.prompt` khớp CHÍNH XÁC với hai cặp.
///
/// 🔵 **THÊM 2026-09-18 (rà soát) — GOM theo `source_term`, chỉ ở ĐÂY.** Một thuật ngữ khớp
/// HAI lần trong cùng một câu (hai span khác nhau) cho HAI phần tử `InjectedGlossaryTerm`
/// khác nhau trong `injected` — đúng, ledger phải giữ nguyên (mỗi lần khớp là một dấu thật
/// trên lưới, AC 2 cần cả hai). Nhưng khối văn bản gửi cho nhà cung cấp không cần lặp lại
/// CÙNG một cặp `"X → Y"` hai lần — dòng thứ hai không mang thêm thông tin nào cho mô hình,
/// chỉ tốn token BYOK. Gom ở ĐÂY, không gom ở `injected` của ledger: hai mối quan tâm khác
/// nhau (Story 4.7 vẽ *"đã tiêm ở đâu"*, dùng TỪNG dấu; nhà cung cấp chỉ cần biết TỪ này dịch
/// là gì, một lần).
fn render_glossary_pairs(injected: &[InjectedGlossaryTerm]) -> String {
    let mut seen_source_terms: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    injected
        .iter()
        .filter(|t| seen_source_terms.insert(t.source_term.as_str()))
        .map(|t| format!("{} → {}", t.source_term, t.translation))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Định danh biến số + phần thay thế của nó tại MỘT lượt gặp — dùng nội bộ bởi
/// [`expand_prompt_body`].
fn replacement_for<'a>(var: PromptVariable, sentence: &'a str, glossary_block: &'a str) -> &'a str {
    match var {
        PromptVariable::GlossaryTerms => glossary_block,
        PromptVariable::SourceSegment => sentence,
        // 🔵 Story 4.6 chỉ XOÁ marker này (câu chuyện "TM searched" của I/O Matrix nói thẳng:
        // "the marker line is still removed this story") — tiêm nội dung TM thật là việc của
        // Epic 7, không đổi chữ ký `assemble_prompt` khi tới lượt.
        PromptVariable::TmSimilarSegments => "",
    }
}

/// `out` (đã ghi tới đúng vị trí NGAY SAU dấu `\n` kết thúc "dòng trước") kết ở một DÒNG
/// TRẮNG — tức dòng thật sự sẽ hiện ra ngay phía trên vị trí đang xét hôm nay là rỗng/chỉ
/// khoảng trắng.
///
/// 🔴 **Đọc từ những gì ĐÃ GHI (`out`), không đọc từ `body` gốc — rà soát 2026-09-18.** Bản
/// trước đọc "dòng trước" từ `body` GỐC, nên hai marker-chỉ-có-marker LIÊN TIẾP nhau (mỗi
/// marker rỗng, đứng một mình trên dòng của nó) không phối hợp được: lượt gỡ dòng THỨ NHẤT
/// không thấy "dòng sau" (đó là dòng marker THỨ HAI, chưa gỡ) là trắng, nên không co; lượt gỡ
/// dòng THỨ HAI đọc "dòng trước" từ `body` GỐC — vẫn là văn bản `{{...}}` của marker thứ NHẤT,
/// không phải dòng trắng thật đứng trước nó SAU khi marker thứ nhất đã biến mất — nên cũng
/// không co. Kết quả: hai dòng trắng hai bên bị BỎ SÓT, không co lại thành một. Đọc từ `out`
/// (thứ THỰC SỰ sẽ xuất ra) sửa đúng lỗi này: `out` sau lượt gỡ dòng thứ nhất **đã** kết ở một
/// dòng trắng (dòng trắng gốc phía trước marker thứ nhất), nên lượt gỡ dòng thứ hai đọc đúng.
fn out_ends_with_a_blank_line(out: &str) -> bool {
    let Some(without_trailing_newline) = out.strip_suffix('\n') else {
        return false; // "dong hien tai" chua ket thuc -- khong co dong TRUOC nao de xet
    };
    without_trailing_newline
        .rsplit('\n')
        .next()
        .unwrap_or("")
        .trim()
        .is_empty()
}

/// Mở rộng MỌI marker ratify trong `body` — **một lượt quét, trái sang phải, trên thân GỐC**
/// (Task của spec: nội dung tiêm không bao giờ bị quét lại — xem doc-comment đầu tệp). Trả
/// thêm `source_segment_seen`: `true` nếu ít nhất MỘT lần lượt quét này (không phải một phép
/// so chuỗi thứ hai) nhận diện đúng `{{source_segment}}` như một biến số ratify.
///
/// Ba luật, đúng Quyết định #2/#3 của spec 4.6:
/// 1. Marker lạ (`{{chapter_context}}`, …) — giữ NGUYÊN VĂN, không thay gì.
/// 2. Phần thay thế RỖNG (không có gì tiêm) **và** marker đứng MỘT MÌNH trên dòng của nó (chỉ
///    khoảng trắng ở cả hai phía) — GỠ CẢ DÒNG, kể cả dấu xuống dòng của nó; nếu việc gỡ đó
///    đưa hai dòng trắng liền kề nhau, một dòng bị bỏ (Decision 3, "LOCAL" — chỉ tính hai dòng
///    NGAY SÁT lượt gỡ này, không đụng một dòng trắng nào khác trong thân).
/// 3. Phần thay thế **nhiều dòng** (chứa `\n` — chỉ `{{glossary_terms}}` với 2+ cặp làm được
///    điều này ở story này) **và** marker CHIA sẻ dòng với văn bản khác — chèn thêm một dấu
///    xuống dòng trước khối (nếu có văn bản trước marker trên cùng dòng) và một dấu xuống
///    dòng sau khối (nếu có văn bản sau) — khối bắt đầu dòng riêng, văn bản còn lại xuống
///    dòng riêng, không bao giờ nối giữa dòng (matrix: *"Marker shares its line, two or more
///    pairs"*). Phần thay thế MỘT dòng (hoặc rỗng, không đứng một mình) không cần cách ly —
///    nó nối vào dòng như văn bản thường, đúng cách `{{source_segment}}` luôn hoạt động.
fn expand_prompt_body(body: &str, sentence: &str, glossary_block: &str) -> (String, bool) {
    let mut out = String::with_capacity(body.len() + glossary_block.len());
    let mut cursor = 0usize;
    let mut source_segment_seen = false;

    loop {
        let Some(open_rel) = body[cursor..].find("{{") else {
            out.push_str(&body[cursor..]);
            break;
        };
        let open_at = cursor + open_rel;
        let after_open = open_at + 2;

        let Some(close_rel) = body[after_open..].find("}}") else {
            out.push_str(&body[cursor..]);
            break;
        };
        let token_start = after_open;
        let token_end = after_open + close_rel;
        let token = &body[token_start..token_end];

        // "{{" mo coi dung TRUOC mot marker that -- cung guard `scan_markers` dung: ban than
        // no la van ban thuong, khong xu ly nhu mot marker; sao chep qua no roi quet lai TU
        // BEN TRONG token vua tim thay.
        if token.contains("{{") {
            out.push_str(&body[cursor..after_open]);
            cursor = after_open;
            continue;
        }

        let close_at = token_end + 2; // vi tri NGAY SAU "}}"

        let Some(var) = PromptVariable::from_token(token) else {
            // Token khong biet -- giu NGUYEN VAN toan bo marker (Quyet dinh #3, spec 4.4).
            out.push_str(&body[cursor..close_at]);
            cursor = close_at;
            continue;
        };
        if var == PromptVariable::SourceSegment {
            source_segment_seen = true;
        }

        let replacement = replacement_for(var, sentence, glossary_block);

        let line_start = body[..open_at].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = body[close_at..].find('\n').map(|i| close_at + i).unwrap_or(body.len());
        let before_on_line = &body[line_start..open_at];
        let after_on_line = &body[close_at..line_end];
        let alone_on_line = before_on_line.trim().is_empty() && after_on_line.trim().is_empty();

        if replacement.is_empty() && alone_on_line {
            // Quyet dinh 3 -- go CA DONG (ke ca dau xuong dong CUA CHINH no), khong chi go
            // van ban marker.
            out.push_str(&body[cursor..line_start]);

            let mut next_cursor = line_end;
            if next_cursor < body.len() {
                next_cursor += 1; // nuot dau `\n` ket dong nay
            }

            // Ve THU HAI cua Quyet dinh 3 -- neu luot go NAY dua hai dong trang lai gan
            // nhau, bo MOT. `prev_line_blank` doc tu `out` (xem doc-comment cua
            // `out_ends_with_a_blank_line`) de mot chuoi marker-roi-lien-tiep phoi hop dung;
            // `next_line_blank` doc tu `body` GOC la dung, vi dong SAU (neu no cung la mot
            // marker rong) se tu no gay ra mot luot go RIENG o vong lap ke tiep.
            let prev_line_blank = out_ends_with_a_blank_line(&out);
            let next_line_blank = body[next_cursor..]
                .split('\n')
                .next()
                .unwrap_or("")
                .trim()
                .is_empty();

            if prev_line_blank && next_line_blank && out.ends_with('\n') {
                out.pop();
            }

            cursor = next_cursor;
            continue;
        }

        // Truong hop chung: van ban truoc marker giu nguyen, roi chen phan thay the.
        out.push_str(&body[cursor..open_at]);

        let needs_isolation = replacement.contains('\n');
        if needs_isolation && !before_on_line.trim().is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(replacement);
        if needs_isolation && !after_on_line.trim().is_empty() {
            out.push('\n');
        }

        cursor = close_at;
    }

    (out, source_segment_seen)
}

/// **Assembler THUẦN của Smart RAG Injector** (AD-14) — nửa nghiệm thu bởi toàn bộ ma trận
/// I/O của spec 4.6. Không `Store`, không `ScopeResolver`, không khớp thuật ngữ, không đồng
/// hồ, không I/O: đầu vào giống hệt nhau cho ra `prompt` giống hệt nhau, TỪNG BYTE.
///
/// `glossary` đến từ [`gather_glossary_context`]; `tm` là `None` cho tới khi Epic 7 tồn tại
/// (`TmInjectionStatus::NotBuiltYet`) hoặc `Some(&[...])` một khi nó tra xong
/// (`TmInjectionStatus::Searched`, kể cả lát cắt rỗng).
pub fn assemble_prompt(
    body: &str,
    sentence: &str,
    glossary: GlossaryInjectionStatus,
    tm: Option<&[SimilarSegment]>,
) -> (String, InjectionLedger) {
    let glossary_block = match &glossary {
        GlossaryInjectionStatus::NotAsked => String::new(),
        GlossaryInjectionStatus::Asked { injected, .. } => render_glossary_pairs(injected),
    };
    let tm_status = match tm {
        None => TmInjectionStatus::NotBuiltYet,
        Some(segments) => TmInjectionStatus::Searched(segments.to_vec()),
    };

    let (prompt, source_segment_seen) = expand_prompt_body(body, sentence, &glossary_block);

    let unknown_markers = scan_markers(body).unknown_markers;
    // 🔴 **KHÔNG `body.contains(marker())` — rà soát 2026-09-18.** Một chuỗi con thô khớp cả
    // khi `{{source_segment}}` nằm LỌT bên trong một hình dạng lạ hơn, vd.
    // `"{{{source_segment}}"` (ba dấu `{` mở) — `.contains` tìm thấy `"{{source_segment}}"`
    // bắt đầu từ điểm mã thứ hai và báo "có mặt", trong khi CHÍNH lượt quét-và-thay ở trên
    // token hoá phần đó thành `"{source_segment"` (một token LẠ, không khớp biến số nào) và
    // để nguyên văn — không câu nào thật sự được tiêm vào prompt. Cờ này phải đến từ ĐÚNG
    // lượt quét đã tiêm (`source_segment_seen`), không phải một phép so chuỗi thứ hai có thể
    // trôi khỏi nó.
    let source_segment_missing = !source_segment_seen;

    let ledger = InjectionLedger {
        glossary,
        tm: tm_status,
        unknown_markers,
        source_segment_missing,
    };

    (prompt, ledger)
}
