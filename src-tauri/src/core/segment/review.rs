//! Phán quyết *cần xem* / *sạch* cho từng Chương — hàng rào Tukey trên các số tóm tắt đã có
//! (Story 6.10, FR132). Module THUẦN: không I/O, không kho, không đồng hồ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 MỘT CƠ CHẾ CHO BA TÍN HIỆU SO-TƯƠNG-ĐỐI — Ice chốt 2026-09-08
//! ─────────────────────────────────────────────────────────────────────────────
//! Ba con số tóm tắt đã có/sắp có trên `ChapterSplitPreviewEntryWire` — `length`,
//! `cleanup_match_count`, `joined_line_count` — đều đủ để chỉ đích danh một Chương đáng nghi,
//! nếu so nó với các Chương KHÁC trong CÙNG lượt nhập, không so với một hằng số tuyệt đối chưa
//! ai đo (§Never spec 6.6: "không ngưỡng, không cờ"). Hàng rào Tukey (Tukey, 1977) là quy ước
//! thống kê CÓ TÊN — `1,5 × IQR` — không phải một hằng số tự đúc, nên nó đi qua lệnh cấm hằng
//! phù thuỷ 2026-09-05 đúng vì lý do lệnh cấm ấy tồn tại: nó không đo hình dạng của một lượt
//! nhập, nó đo hình dạng của MỌI phân phối một chiều.
//!
//! `length` chỉ xét hàng rào DƯỚI (*ngắn bất thường* = dưới `Q1 − 1,5×IQR`) — một Chương DÀI
//! bất thường không phải một dấu hiệu lỗi nhập. `cleanup_match_count`/`joined_line_count` chỉ
//! xét hàng rào TRÊN (*xoá quá nhiều*/*nối dòng cao* = trên `Q3 + 1,5×IQR`) — một Chương với
//! rất ÍT chỗ khớp luật làm sạch, hay rất ÍT dòng bị nối, không phải một dấu hiệu lỗi nhập.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 HAI TẦNG "KHÔNG ĐO ĐƯỢC" — đừng trộn (§Always spec 6.10)
//! ─────────────────────────────────────────────────────────────────────────────
//! ① Một tín hiệu KHÔNG ĐỦ DỮ LIỆU cho CẢ lượt nhập (dưới bốn giá trị đo được, hoặc `IQR = 0`
//! sau khi tính) — tín hiệu đó KHÔNG THAM GIA: không Chương nào bị nó phán, dù `needs_review`
//! hay `clean`. [`ReviewOutcome::participation`] nói ra tín hiệu nào đã tham gia.
//! ② Một tín hiệu CÓ hàng rào (đủ dữ liệu, `IQR > 0`) nhưng giá trị của CHÍNH Chương này là
//! `None` ⇒ Chương đó là *cần xem* với nguyên nhân [`ReviewCause::NotMeasured`] — KHÔNG BAO GIỜ
//! là *sạch*. Mặc định hoá `None` thành `0` ở đây sẽ cho Chương đó một giá trị nằm gọn trong
//! hàng rào, khai nó "sạch" trong khi CHƯA AI ĐO nó — đúng lớp lỗi rỗng-im-lặng mà
//! `AGENTS.md` §Known pitfalls gọi tên là trung tâm của dự án.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 "BỐN GIÁ TRỊ ĐO ĐƯỢC TỐI THIỂU" LÀ MỘT HỆ QUẢ SỐ HỌC, KHÔNG PHẢI MỘT NGƯỠNG (§Ask First)
//! ─────────────────────────────────────────────────────────────────────────────
//! Tứ phân vị ở đây dùng quy ước "bản lề Tukey" (Tukey hinges, phép trung vị LOẠI TRỪ): sắp
//! N giá trị tăng dần, loại phần tử giữa khi N lẻ, rồi `Q1`/`Q3` là TRUNG VỊ của nửa dưới/nửa
//! trên — mỗi nửa có đúng `floor(N/2)` phần tử. Để `Q1`/`Q3` là trung vị THẬT của một nửa (một
//! phép trung bình của ÍT NHẤT hai giá trị, không phải MỘT giá trị thô đứng thay), mỗi nửa cần
//! `floor(N/2) ≥ 2` — điều kiện đó đúng khi và chỉ khi `N ≥ 4`, với MỌI N (chẵn lẫn lẻ, vì
//! `floor(N/2)` đơn điệu không giảm và đạt 2 lần đầu tại N = 4). Đây LÀ điều kiện số học đã
//! nêu ở §Ask First — không phải một lựa chọn có thể chỉnh.
//!
//! ⚠️ **Khai tường minh so với `src/panels/lookupTiming.ts:90`** (bản `percentile()` DUY NHẤT
//! khác trong kho, đo 2026-09-08: `grep -ri "median|quantile|quartile|percentile|iqr|tukey"`
//! trên `src-tauri/src/**` cho 0 kết quả trước tệp này). Hai bản CỐ Ý LỆCH quy ước: bản kia
//! tính percentile-hạng-gần-nhất (nearest-rank, `ceil`) cho một bảng ĐỘ TRỄ (p50/p95/p99) —
//! đúng công cụ cho một chỉ số phân vị ĐƠN, không phải cho hàng rào ngoại lệ theo cặp Q1/Q3.
//! Bản lề Tukey là quy ước ĐÚNG cho hàng rào 1,5×IQR (đây là định nghĩa chuẩn của chính hàng
//! rào đó, Tukey 1977) — hai bản không dùng chung được vì phục vụ hai câu hỏi khác nhau, và
//! trộn chúng (dùng nearest-rank cho hàng rào) sẽ cho `Q1`/`Q3` không khớp định nghĩa mà hằng
//! số `1,5` được hiệu chỉnh cho.

/// Nguyên nhân *cần xem* của MỘT Chương — bốn khoá literal ĐÓNG (§Always spec 6.10: "bốn nhãn
/// nguyên nhân là bốn khoá literal riêng qua một `switch` cạn"). [`ReviewCause::NotMeasured`]
/// có thể đến từ BẤT KỲ tín hiệu nào trong ba tín hiệu trên — nó không phân biệt tín hiệu nào,
/// vì AC chỉ đòi phân biệt "không đo được" với ba dạng "đo được nhưng bất thường".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewCause {
    /// `length` dưới `Q1 − 1,5×IQR`.
    ShortLength,
    /// `cleanup_match_count` trên `Q3 + 1,5×IQR`.
    HighCleanupMatches,
    /// `joined_line_count` trên `Q3 + 1,5×IQR`.
    HighJoinedLines,
    /// Hàng rào của MỘT tín hiệu tồn tại (đủ dữ liệu cho cả lượt nhập), nhưng giá trị của
    /// CHÍNH Chương này là `None` — tầng ② của doc-comment đầu tệp. KHÔNG BAO GIỜ rơi vào
    /// nhánh sạch.
    NotMeasured,
}

/// Phán quyết của MỘT Chương — `needs_review == causes.is_empty() == false` là một BẤT BIẾN
/// giữ bởi [`classify`], không phải hai trường độc lập có thể lệch nhau.
///
/// 🔴 **BẤT BIẾN THỨ HAI (vòng rà đối kháng bước 4, 2026-09-08) — mỗi biến thể [`ReviewCause`]
/// xuất hiện TỐI ĐA MỘT LẦN trong `causes`.** [`ReviewCause::NotMeasured`] đặc biệt dễ vi phạm
/// điều này: nó có thể đến từ CẢ `cleanup_match_count` LẪN `joined_line_count` (hai tín hiệu
/// riêng, cùng nguyên nhân) khi một Chương thiếu CẢ HAI dưới CẢ HAI hàng rào cùng lúc — `classify`
/// gộp hai điều kiện đó thành ĐÚNG một `push`, không hai (xem thân hàm). Vi phạm bất biến này
/// làm `v-for :key="cause"` phía `ImportPreviewOverlay.vue` trùng khoá và nhãn "Không đo được"
/// hiện lặp trên cùng một hàng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewVerdict {
    pub needs_review: bool,
    pub causes: Vec<ReviewCause>,
}

/// Tín hiệu nào đã THAM GIA lượt phân loại này — `false` ⇒ tín hiệu đó không phán bất kỳ
/// Chương nào (dưới bốn giá trị đo được, hoặc `IQR = 0` — tầng ① của doc-comment đầu tệp).
/// `any()` — `true` khi ÍT NHẤT một trong ba `true` — là điều kiện tầng hiển thị dùng để phân
/// biệt *"0 Chương cần xem vì đã đo và sạch"* với *"0 Chương cần xem vì chưa đủ Chương để so"*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SignalParticipation {
    pub length: bool,
    pub cleanup_match_count: bool,
    pub joined_line_count: bool,
}

impl SignalParticipation {
    #[must_use]
    pub fn any(self) -> bool {
        self.length || self.cleanup_match_count || self.joined_line_count
    }
}

/// Kết quả một lượt [`classify`] — phán quyết cho TỪNG Chương, SONG SONG theo INDEX với đầu
/// vào, cộng cờ tham gia của cả ba tín hiệu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewOutcome {
    pub verdicts: Vec<ReviewVerdict>,
    pub participation: SignalParticipation,
}

/// Ba số tóm tắt của MỘT Chương — đầu vào của [`classify`]. `length` LUÔN đo được (mọi Chương
/// đều có `source_text`, kể cả rỗng ⇒ `0`) nên không phải `Option`; hai trường còn lại có thể
/// `None` (xem doc-comment `ImportedChapter::cleanup_report`/`::joined_line_count`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChapterMetrics {
    pub length: usize,
    pub cleanup_match_count: Option<usize>,
    pub joined_line_count: Option<usize>,
}

/// Hằng số DUY NHẤT được phép ngoài chính `1,5` — số bốn tối thiểu, xem doc-comment đầu tệp
/// cho lý do đây là một HỆ QUẢ số học, không phải một ngưỡng chỉnh được.
const MIN_MEASURED_VALUES: usize = 4;

/// `1,5` — hằng Tukey (1977), tên riêng, DUY NHẤT được phép (§Always spec 6.10).
const TUKEY_MULTIPLIER: f64 = 1.5;

/// Hàng rào Tukey của MỘT tín hiệu — `Q1`, `Q3`, `IQR` tính bằng bản lề Tukey (xem doc-comment
/// đầu tệp). `None` khi tín hiệu KHÔNG THAM GIA (dưới bốn giá trị đo được, hoặc `IQR = 0`).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Fence {
    q1: f64,
    q3: f64,
    iqr: f64,
}

impl Fence {
    fn lower(self) -> f64 {
        self.q1 - TUKEY_MULTIPLIER * self.iqr
    }

    fn upper(self) -> f64 {
        self.q3 + TUKEY_MULTIPLIER * self.iqr
    }
}

/// Trung vị của một lát ĐÃ SẮP TĂNG DẦN, không rỗng.
fn median_of_sorted(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

/// Hàng rào Tukey trên MỘT tập giá trị ĐÃ ĐO ĐƯỢC (bỏ qua `None` TRƯỚC khi gọi hàm này — xem
/// [`classify`]). `None` ⇒ tín hiệu không tham gia — xem doc-comment đầu tệp, tầng ①.
fn tukey_fence(measured: &[f64]) -> Option<Fence> {
    let mut sorted: Vec<f64> = measured.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("gia tri do duoc khong bao gio la NaN"));
    let n = sorted.len();
    let half = n / 2; // = floor(N/2), loại phần tử giữa khi N lẻ — bản lề Tukey.
    // `MIN_MEASURED_VALUES / 2` KHÔNG phải một hằng số thứ hai tự đúc — nó LÀ điều kiện
    // "N ≥ MIN_MEASURED_VALUES" viết lại theo `half` (xem doc-comment đầu tệp, mục "BỐN GIÁ
    // TRỊ ĐO ĐƯỢC TỐI THIỂU LÀ MỘT HỆ QUẢ SỐ HỌC"): mỗi nửa cần ÍT NHẤT hai phần tử để
    // `median_of_sorted` của nó là trung vị THẬT, và `half < MIN_MEASURED_VALUES / 2` là phép
    // so tương đương với `n < MIN_MEASURED_VALUES` khi `MIN_MEASURED_VALUES` chẵn (đúng ở
    // đây, `= 4`) — viết qua `half` để phép so đứng NGAY CẠNH đại lượng nó so, không đứng
    // cách xa ở một hằng số khác tên.
    if half < MIN_MEASURED_VALUES / 2 {
        return None; // dưới bốn giá trị đo được — xem doc-comment đầu tệp.
    }
    let lower_half = &sorted[..half];
    let upper_half = &sorted[n - half..];
    let q1 = median_of_sorted(lower_half);
    let q3 = median_of_sorted(upper_half);
    let iqr = q3 - q1;
    if iqr <= 0.0 {
        return None; // hàng rào suy biến — xem doc-comment đầu tệp, tầng ①.
    }
    Some(Fence { q1, q3, iqr })
}

/// Phân loại N Chương thành *cần xem*/*sạch* — hàm THUẦN, đúng khuôn module này.
///
/// `chapters` KHÔNG được chứa mục hỏng (§Never spec 6.10: "không dựng một Chương RỖNG giữ chỗ
/// cho link hỏng, và không cho mục hỏng vào phép tính trung vị") — chỗ gọi
/// (`commands::project::build_chapter_split_preview_wire`) chỉ nạp Chương THẬT vào đây.
#[must_use]
pub fn classify(chapters: &[ChapterMetrics]) -> ReviewOutcome {
    let length_fence = tukey_fence(&chapters.iter().map(|c| c.length as f64).collect::<Vec<_>>());
    let cleanup_measured: Vec<f64> =
        chapters.iter().filter_map(|c| c.cleanup_match_count).map(|v| v as f64).collect();
    let cleanup_fence = tukey_fence(&cleanup_measured);
    let joined_measured: Vec<f64> =
        chapters.iter().filter_map(|c| c.joined_line_count).map(|v| v as f64).collect();
    let joined_fence = tukey_fence(&joined_measured);

    let participation = SignalParticipation {
        length: length_fence.is_some(),
        cleanup_match_count: cleanup_fence.is_some(),
        joined_line_count: joined_fence.is_some(),
    };

    let verdicts = chapters
        .iter()
        .map(|chapter| {
            let mut causes = Vec::new();
            // 🔴 SỬA (vòng rà đối kháng bước 4, 2026-09-08) — `NotMeasured` gom vào MỘT cờ
            // riêng thay vì `push` trực tiếp ở cả hai nhánh `cleanup_fence`/`joined_fence`: một
            // Chương có thể thiếu CẢ HAI tín hiệu dưới CẢ HAI hàng rào cùng lúc, và bản trước
            // push hai lần cho đúng một nguyên nhân — vỡ bất biến "mỗi nguyên nhân tối đa một
            // lần" ở doc-comment [`ReviewVerdict`].
            let mut not_measured = false;

            if let Some(fence) = length_fence {
                // `length` LUÔN đo được (không `Option`) — chỉ nhánh so sánh, không nhánh
                // `NotMeasured` cho tín hiệu này.
                if (chapter.length as f64) < fence.lower() {
                    causes.push(ReviewCause::ShortLength);
                }
            }

            if let Some(fence) = cleanup_fence {
                match chapter.cleanup_match_count {
                    None => not_measured = true,
                    Some(v) if (v as f64) > fence.upper() => causes.push(ReviewCause::HighCleanupMatches),
                    Some(_) => {}
                }
            }

            if let Some(fence) = joined_fence {
                match chapter.joined_line_count {
                    None => not_measured = true,
                    Some(v) if (v as f64) > fence.upper() => causes.push(ReviewCause::HighJoinedLines),
                    Some(_) => {}
                }
            }

            if not_measured {
                causes.push(ReviewCause::NotMeasured);
            }

            ReviewVerdict { needs_review: !causes.is_empty(), causes }
        })
        .collect();

    ReviewOutcome { verdicts, participation }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(length: usize, cleanup: Option<usize>, joined: Option<usize>) -> ChapterMetrics {
        ChapterMetrics { length, cleanup_match_count: cleanup, joined_line_count: joined }
    }

    /// Dưới bốn giá trị đo được — KHÔNG tín hiệu nào tham gia, KHÔNG Chương nào bị phán oan.
    #[test]
    fn fewer_than_four_measured_values_makes_no_signal_participate() {
        let chapters = vec![metrics(10, Some(1), Some(1)), metrics(500, Some(1), Some(1)), metrics(12, Some(1), Some(1))];
        let outcome = classify(&chapters);
        assert!(!outcome.participation.any(), "N = 3 phai cho CA BA tin hieu khong tham gia");
        for verdict in &outcome.verdicts {
            assert!(!verdict.needs_review, "khong tin hieu nao tham gia thi khong Chuong nao bi phan");
            assert!(verdict.causes.is_empty());
        }
    }

    /// `IQR = 0` (mọi giá trị bằng nhau) — hàng rào suy biến, tín hiệu đó không tham gia,
    /// NHƯNG tín hiệu khác vẫn xét (đo trên tập dữ liệu riêng của nó).
    #[test]
    fn a_degenerate_iqr_excludes_only_that_signal() {
        // length bang nhau tuyet doi (IQR = 0) tren ca 20 Chuong; cleanup_match_count tan
        // rong THAT SU (0..18 cong mot gia tri 200 vuot han hang rao tinh tren nua tren) --
        // mot gia tri thap doc lap giua mot dam dong bang nhau KHONG lam Q3 doi (bi "chim"
        // trong trung vi nua tren), nen phai tan rong that su moi kiem duoc IQR > 0.
        let counts: Vec<usize> = (0..19).collect(); // 0..=18, 19 gia tri
        let chapters: Vec<ChapterMetrics> = counts
            .iter()
            .map(|&c| metrics(1000, Some(c), None))
            .chain(std::iter::once(metrics(1000, Some(200), None)))
            .collect();
        let outcome = classify(&chapters);
        assert!(!outcome.participation.length, "length bang nhau tuyet doi phai suy bien");
        assert!(outcome.participation.cleanup_match_count, "cleanup_match_count phai tham gia");
        assert!(!outcome.participation.joined_line_count, "joined_line_count toan None, khong tham gia");
        let last = outcome.verdicts.last().expect("con Chuong cuoi");
        assert!(last.needs_review, "Chuong voi 200 cho khop phai bi gan co xoa qua nhieu");
        assert_eq!(last.causes, vec![ReviewCause::HighCleanupMatches]);
    }

    /// Hàng rào TỒN TẠI (đủ dữ liệu, IQR > 0) nhưng giá trị của Chương k là `None` ⇒ Chương k
    /// là CẦN XEM với nguyên nhân `NotMeasured` — KHÔNG BAO GIỜ sạch. Đây là AC 2026-09-08.
    #[test]
    fn a_null_value_under_an_existing_fence_is_always_needs_review_never_clean() {
        // length VA joined_line_count BANG NHAU tuyet doi tren ca 10 Chuong (IQR = 0, khong
        // tham gia) -- chi cleanup_match_count tan rong that su (1..=10, tru mot gia tri
        // KHONG do duoc o Chuong thu 5, chi so 4) de day la tin hieu DUY NHAT co the phan.
        let mut chapters: Vec<ChapterMetrics> = (0..10).map(|i| metrics(100, Some(i + 1), Some(1))).collect();
        chapters[4].cleanup_match_count = None;
        let outcome = classify(&chapters);
        assert!(!outcome.participation.length);
        assert!(!outcome.participation.joined_line_count);
        assert!(outcome.participation.cleanup_match_count, "du du lieu tan rong de hang rao ton tai");
        let verdict = &outcome.verdicts[4];
        assert!(verdict.needs_review, "gia tri null duoi mot hang rao TON TAI phai la CAN XEM");
        assert_eq!(verdict.causes, vec![ReviewCause::NotMeasured]);
        // Doi chung: DUNG `usize` voi `unwrap_or(0)` se cho `0`, nam GON trong hang rao (moi
        // Chuong khac deu tu 1 den 10, `0` khong vuot hang rao TREN nao ca), va se KHONG bi
        // gan co gi ca -- day chinh la ca do AC 2026-09-08 sinh ra de chan;
        // tests/segment_contract.rs::* doi chung o tang day.
        for (i, v) in outcome.verdicts.iter().enumerate() {
            if i != 4 {
                assert!(!v.needs_review, "chi Chuong khong do duoc (chi so 4) moi can xem o day");
            }
        }
    }

    /// Một Chương không đo được KHÔNG được đếm vào nhóm sạch — đối chứng riêng cho đúng câu
    /// đó (§Task list spec 6.10: "kèm ca khẳng định một Chương không đo được KHÔNG được đếm
    /// vào M sạch").
    #[test]
    fn a_chapter_that_is_not_measured_is_never_counted_among_the_clean_ones() {
        // length tan rong nhung khong Chuong nao vuot hang rao DUOI (xac minh o duoi); chi
        // Chuong 0 KHONG do duoc joined_line_count, nam gia tri con lai (1..=5) tan rong that
        // su de hang rao TON TAI.
        let mut chapters: Vec<ChapterMetrics> =
            (0..6).map(|i| metrics(100 + i * 5, Some(2), Some(i))).collect();
        chapters[0].joined_line_count = None;
        let outcome = classify(&chapters);
        assert!(outcome.participation.joined_line_count, "du du lieu tan rong de hang rao ton tai");
        let clean_count = outcome.verdicts.iter().filter(|v| !v.needs_review).count();
        let needs_review_count = outcome.verdicts.len() - clean_count;
        assert!(outcome.verdicts[0].needs_review);
        assert_eq!(needs_review_count, 1, "dung MOT Chuong can xem -- Chuong khong do duoc");
        assert_eq!(clean_count, 5);
    }

    /// Hàng rào chỉ xét MỘT chiều theo đúng tín hiệu — `length` dưới thì mới bị gắn cờ (một
    /// Chương DÀI bất thường không phải lỗi), `cleanup_match_count`/`joined_line_count` trên
    /// thì mới bị gắn cờ (rất ÍT chỗ khớp/rất ÍT dòng nối không phải lỗi).
    #[test]
    fn each_signal_only_checks_the_fence_side_that_makes_sense_for_it() {
        // Nam gia tri length tan rong, MOT gia tri THAT DAI (khong duoc gan co); nam gia tri
        // cleanup_match_count tan rong, MOT gia tri THAT NHO -- 0 -- (khong duoc gan co, vi
        // cleanup chi xet hang rao TREN).
        let chapters = vec![
            metrics(100, Some(50), None),
            metrics(110, Some(52), None),
            metrics(120, Some(48), None),
            metrics(115, Some(51), None),
            metrics(5000, Some(0), None), // length rat DAI, cleanup rat NHO
        ];
        let outcome = classify(&chapters);
        let last = &outcome.verdicts[4];
        assert!(!last.causes.contains(&ReviewCause::ShortLength), "dai bat thuong khong phai loi");
        assert!(
            !last.causes.contains(&ReviewCause::HighCleanupMatches),
            "khop RAT IT khong phai loi -- cleanup chi xet hang rao TREN"
        );
    }

    /// GỠ điều kiện `IQR > 0` (đối chứng đỏ ② của §Verification spec 6.10, mô phỏng NGAY TRONG
    /// test bằng một bản dựng hàng rào KHÔNG kiểm `iqr <= 0.0`) — expected ĐỎ trên chính cổng
    /// [`a_degenerate_iqr_excludes_only_that_signal`] ở trên (mọi Chương length bằng nhau sẽ bị
    /// gắn cờ SAI nếu ai gỡ điều kiện đó khỏi [`tukey_fence`]). Ca này không lặp lại phép gỡ
    /// (không biên dịch một bản Rust thứ hai) — nó khẳng định TRỰC TIẾP rằng hàng rào suy biến
    /// (`iqr == 0`) không gắn cờ Chương nào, đúng NGAY tại hàm sản phẩm.
    #[test]
    fn a_degenerate_fence_never_flags_any_chapter_even_at_the_boundary_value() {
        let chapters: Vec<ChapterMetrics> = (0..8).map(|_| metrics(777, None, None)).collect();
        let outcome = classify(&chapters);
        assert!(!outcome.participation.length);
        assert!(outcome.verdicts.iter().all(|v| !v.needs_review));
    }

    /// **THÊM (vòng rà đối kháng bước 4, 2026-09-08)** — một Chương thiếu CẢ HAI tín hiệu
    /// (`cleanup_match_count`/`joined_line_count`) dưới CẢ HAI hàng rào TỒN TẠI cùng lúc phải
    /// cho `causes == [NotMeasured]` — ĐÚNG MỘT LẦN, không hai. Bản trước `push` một lần cho
    /// MỖI tín hiệu thiếu, nên ca này đỏ trước bản sửa: `causes` mang `[NotMeasured, NotMeasured]`.
    #[test]
    fn a_chapter_missing_both_signals_under_both_existing_fences_gets_not_measured_exactly_once() {
        // length hang nhau tuyet doi tren ca 10 Chuong (IQR = 0, khong tham gia) -- loai
        // ShortLength khoi phep so. cleanup_match_count VA joined_line_count moi thu tan rong
        // THAT SU rieng, du du lieu de hang rao TON TAI cho CA HAI; Chuong chi so 3 thieu CA
        // HAI gia tri.
        let mut chapters: Vec<ChapterMetrics> =
            (0..10).map(|i| metrics(100, Some(i + 1), Some((i + 1) * 2))).collect();
        chapters[3].cleanup_match_count = None;
        chapters[3].joined_line_count = None;

        let outcome = classify(&chapters);
        assert!(outcome.participation.cleanup_match_count, "du du lieu tan rong de hang rao ton tai");
        assert!(outcome.participation.joined_line_count, "du du lieu tan rong de hang rao ton tai");

        let verdict = &outcome.verdicts[3];
        assert!(verdict.needs_review);
        assert_eq!(
            verdict.causes,
            vec![ReviewCause::NotMeasured],
            "NotMeasured phai xuat hien DUNG MOT LAN du hai tin hieu cung thieu, khong hai lan \
             lam trung khoa o tang hien thi (`v-for :key=\"cause\"`)"
        );
    }
}
