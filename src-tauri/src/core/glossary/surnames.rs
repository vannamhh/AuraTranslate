//! Bảng họ phổ biến tiếng Trung (*Bách gia tính* — "Trăm họ") — Story 3.5, dùng để NỚI
//! ngưỡng quét cho một hình dạng chuỗi cụ thể (xem [`super::scan`]).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO KHÔNG ĐI QUA `tools/dict-build` — 0 CỬA NFR15
//! ─────────────────────────────────────────────────────────────────────────────
//! *Bách gia tính* là một văn bản đời Bắc Tống (thế kỷ 11) — nó không mang bản quyền để
//! rà (NFR15 nói về giấy phép PHẦN MỀM đưa vào cây phụ thuộc, không áp cho một danh sách
//! họ đã là dữ kiện công cộng gần một nghìn năm). Đưa nó qua `tools/dict-build` sẽ bắt
//! dựng lại **cả bốn** tệp `.db` + bốn SHA-256 + một bản phát hành GitHub Release
//! (`dict-manifest.toml`, AD-25) — một chi phí vận hành lớn cho đúng một mảng ~100 chuỗi
//! không đổi. Mảng hằng ngay trong mã Rust là hình dạng RẺ và ĐÚNG cho dữ liệu này.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! CHỈ NỚI NGƯỠNG, KHÔNG PHẢI MỘT NGUỒN TỪ ĐIỂN THỨ HAI
//! ─────────────────────────────────────────────────────────────────────────────
//! Module này KHÔNG tự nhận diện "đây là tên người" — nó chỉ trả lời "ký tự này có nằm
//! trong bảng họ phổ biến không", và [`super::scan`] dùng câu trả lời đó để hạ ngưỡng đúng
//! MỘT bậc cho một hình dạng chuỗi hẹp (2–3 ký tự, ký tự đầu là họ). Không sinh cột mới,
//! không sinh `candidate_origin` mới — xem §Design Notes của story.
//!
//! ⚠️ Mọi chuỗi trong `src-tauri/src/**` viết KHÔNG DẤU; doc-comment có dấu là hợp lệ.

/// ~100 họ phổ biến nhất theo *Bách gia tính* (giản thể). Ký tự đơn — bảng không mang họ
/// ghép (`欧阳`, `司马`, …): I/O Matrix của story chỉ đòi ký tự ĐẦU của một chuỗi 2–3 ký tự
/// nằm trong bảng, và một họ ghép hai ký tự vẫn khớp qua ký tự đầu của nó (`欧` có trong
/// bảng ⇒ `欧阳` vẫn được nới, dù bảng không liệt `欧阳` nguyên khối).
///
/// Thứ tự: nguyên văn *Bách gia tính*, không sắp lại — một danh sách chép từ một nguồn có
/// thứ tự riêng thì giữ thứ tự đó là bằng chứng nó THẬT được chép, không bị gõ tay rồi xáo.
pub const COMMON_SURNAMES: &[char] = &[
    '赵', '钱', '孙', '李', '周', '吴', '郑', '王', '冯', '陈', '褚', '卫', '蒋', '沈', '韩', '杨',
    '朱', '秦', '尤', '许', '何', '吕', '施', '张', '孔', '曹', '严', '华', '金', '魏', '陶', '姜',
    '戚', '谢', '邹', '喻', '柏', '水', '窦', '章', '云', '苏', '潘', '葛', '奚', '范', '彭', '郎',
    '鲁', '韦', '昌', '马', '苗', '凤', '花', '方', '俞', '任', '袁', '柳', '酆', '鲍', '史', '唐',
    '费', '廉', '岑', '薛', '雷', '贺', '倪', '汤', '滕', '殷', '罗', '毕', '郝', '邬', '安', '常',
    '乐', '于', '时', '傅', '皮', '卞', '齐', '康', '伍', '余', '元', '卜', '顾', '孟', '平', '黄',
    '和', '穆', '萧', '尹', '姚', '邵', '湛', '汪', '祁', '毛', '禹', '狄', '米', '贝', '明', '臧',
    '计', '伏', '成', '戴', '谈', '宋', '茅', '庞', '熊', '纪', '舒', '屈', '项', '祝', '董', '梁',
    '杜', '阮', '蓝', '闵', '席', '季', '麻', '强', '贾', '路', '娄', '危', '江', '童', '颜', '郭',
    '梅', '盛', '林', '刁', '钟', '徐', '邱', '骆', '高', '夏', '蔡', '田', '樊', '胡', '凌', '霍',
    '虞', '万', '支', '柯', '昝', '管', '卢', '莫', '经', '房', '裘', '缪', '干', '解', '应', '宗',
    '丁', '宣', '贲', '邓', '郁', '单', '杭', '洪', '包', '诸', '左', '石', '崔', '吉', '钮', '龚',
    '程', '嵇', '邢', '滑', '裴', '陆', '荣', '翁', '荀', '羊', '於', '惠', '甄', '曲', '家', '封',
    '芮', '羿', '储', '靳', '汲', '邴', '糜', '松', '井', '段', '富', '巫', '乌', '焦', '巴', '弓',
    '牧', '隗', '山', '谷', '车', '侯', '宓', '蓬', '全', '郗', '班', '仰', '秋', '仲', '伊', '宫',
    '宁', '仇', '栾', '暴', '甘', '钭', '厉', '戎', '祖', '武', '符', '刘', '景', '詹', '束', '龙',
    '叶', '幸', '司', '韶', '郜', '黎', '蓟', '薄', '印', '宿', '白', '怀', '蒲', '邰', '从', '鄂',
];

/// Alias phồn thể → giản thể cần cho phép nới ngưỡng họ.
///
/// 🔴 Không trộn alias vào [`COMMON_SURNAMES`]: tham số `surnames` của thuật toán quét là
/// một điểm tiêm có chủ cho test, và bảng trên cố ý giữ nguyên văn bản giản thể. Alias là
/// một bước chuẩn hoá hình dạng chữ trước phép tra bảng, không phải một họ mới. Story 3.5
/// review đo đúng ca `蕭炎` ở `threshold - 1`; thiếu cặp này làm cùng một họ đi hai ngưỡng
/// chỉ vì cách mã hoá chữ.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔵 **THÊM 2026-08-26 (cụm F ⑤) — năm cặp `陳/陈 張/张 劉/刘 楊/杨 黃/黄`, Ice chốt.**
/// ─────────────────────────────────────────────────────────────────────────────
/// **Phép đo đã chạy** (2026-08-26, `src-tauri/resources/dict/dict-core.db`, `dict_entry`
/// DISTINCT `headword`/`headword_simp`, cả hai dài đúng 1 ký tự, khác nhau): **7.362** cặp
/// phồn→giản một ký tự; **134** cặp có vế giản NẰM trong [`COMMON_SURNAMES`], phủ
/// **110/272** họ; **0** cặp mơ hồ (một phồn thể ứng với nhiều giản thể). Năm cặp trên là
/// NĂM trong 134 cặp đó — mỗi vế giản (`陈 张 刘 杨 黄`) đều có sẵn trong
/// [`COMMON_SURNAMES`], và không vế phồn nào (`陳 張 劉 楊 黃`) tự nó là một họ KHÁC trong
/// bảng, nên cả năm qua được cả hai mệnh đề mà ca quần thể ④c canh.
///
/// 🔴 **KHÔNG nhập trọn 134 cặp đo được — chính phép đo đó bác nó, bằng HAI lý do khác
/// nhau, không phải một.**
///
/// **Lớp 1 — alias sai NỚI NGƯỠNG cho chuỗi không mang hình dạng tên người.** Đúng cho
/// `鬍→胡` (râu), `週→周` (tuần), `鬱→郁` (u uất), `餘→余`, `衚→胡` (ngõ hẹp) — đo lại 2026-08-26:
/// cả NĂM vế phồn này đều KHÔNG có trong [`COMMON_SURNAMES`], nên `effective_threshold` sẽ
/// đi qua nhánh alias và hạ ngưỡng cho một chữ không phải họ. Đây là lớp lỗi mà tham số
/// `surnames` cố ý tách ra để KHÔNG trộn (xem đoạn trên).
///
/// **Lớp 2 — `於→于` KHÔNG thuộc lớp 1.** 🔵 **SỬA 2026-08-26 (vòng rà 1) — bản đầu của
/// đoạn này viết SAI cả VỊ TRÍ lẫn CƠ CHẾ.** Đo lại: `於` nằm ở **hàng 13, cột 11** của
/// [`COMMON_SURNAMES`] (không phải "hàng 6, cột 1" — hàng 6 cột 2 mới là `于`, vế GIẢN của
/// chính cặp này, một chữ khác hẳn). Và mệnh đề *"alias `於→于` sai sẽ nới ngưỡng"* cũng SAI
/// về cơ chế: `effective_threshold` tra `surnames.contains(&first) ||
/// TRADITIONAL_SURNAME_ALIASES.iter().find_map(...)…` — `於` ĐÃ có sẵn trong
/// `COMMON_SURNAMES`, nên vế ĐẦU của phép `||` đã đoản mạch đúng TRƯỚC khi bảng alias được
/// tra tới; thêm cặp `('於','于')` **không đổi một chút hành vi thời gian chạy nào** (ca
/// quần thể ④c xác nhận điều này bằng cách khác: nó đỏ không phải vì ngưỡng đổi, mà vì bản
/// thân mệnh đề "vế phồn không được tự nó là một họ khác" bị vi phạm). Lý do THẬT để loại
/// `於→于` là **MÔ HÌNH, không phải NGƯỠNG**: alias đó khai `於` và `于` là CÙNG một họ (một
/// bước chuẩn hoá hình dạng chữ, đúng vai của bảng), trong khi [`COMMON_SURNAMES`] coi
/// chúng là HAI họ RIÊNG (hai mục khác nhau trong *Bách gia tính*) — trộn hai họ khác nhau
/// thành một là sai về NGỮ NGHĨA của bảng, không phải một lỗ hổng ngưỡng.
///
/// Phần còn lại của bảng 134 cặp là một mục nợ **có chủ**, kèm nguyên bảng đo — xem
/// `deferred-work.md` §Cụm F — không phải một lượt gõ tay ở đây.
///
/// 🔵 **MỞ PHẠM VI 2026-08-26 (cụm F) — `pub`, từ `pub(super)`.** Ca quần thể ④c
/// (`glossary_scan_contract.rs`) đọc THẲNG mảng này để khẳng định cả hai mệnh đề mà một
/// cặp mơ hồ (`於→于`) sẽ vi phạm — cùng khuôn [`COMMON_SURNAMES`] đã `pub` sẵn cho đúng lý
/// do đó. Bản thân bảng vẫn KHÔNG được trộn vào `COMMON_SURNAMES` (xem đoạn đầu doc-comment
/// này) — mở phạm vi chỉ đổi ai ĐỌC được nó, không đổi việc nó có được TRA hay không.
pub const TRADITIONAL_SURNAME_ALIASES: &[(char, char)] =
    &[('蕭', '萧'), ('陳', '陈'), ('張', '张'), ('劉', '刘'), ('楊', '杨'), ('黃', '黄')];
