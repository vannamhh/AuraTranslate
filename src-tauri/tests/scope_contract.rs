//! Hành vi của phân giải hai tầng — Story 1.8, AC1 tới AC5.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `store_contract.rs` (*hành vi*) / `store_boundary.rs`
//! (*ranh giới cây nguồn*). Vế *"cưỡng chế bằng test"* của AC1 nằm ở `scope_boundary.rs`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY — thừa kế nguyên vẹn từ `store_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng** (pid + `AtomicU64`). `cargo test` chạy song song
//!    trong cùng một tiến trình; hai ca dùng chung một `.db` sẽ đỏ ngẫu nhiên và bị đọc
//!    thành flaky. ⛔ Không thêm `tempfile`.
//! 2. **Drop `Store` TRƯỚC khi xoá thư mục.** Windows từ chối xoá tệp đang mở — một
//!    `remove_dir_all` sớm cho ra một test đỏ **chỉ trên nhánh Windows**, đúng lớp lỗi
//!    NFR14 mà CI hai nền tảng của Story 1.3 tồn tại để bắt.
//! 3. ⛔ **Không `sleep` dài.** Phần lớn ca ở đây là hàm thuần và không chạm đĩa gì cả.
//! 4. **Không ca nào treo khi nó trượt.**
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ TỆP NÀY ⛔ KHÔNG `use rusqlite` — và đó là một mệnh đề, không phải may mắn
//! ─────────────────────────────────────────────────────────────────────────────
//! `deferred-work.md:179` ghi rằng `tests/**` được **miễn trừ** khỏi phép quét ranh giới,
//! và mở lại mục đó nếu test mới chạm `rusqlite` trực tiếp. Không cần: `Store::write` nhận
//! một closure lấy `&Transaction` — kiểu **tái xuất** từ `core::store` — nên ca ghi thẳng
//! một hàng vào `global.db` viết được mà không gõ tên crate. Miễn trừ ở lại nguyên trạng.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::Duration;

use auratranslate_lib::commands::config::{bootstrap_config, put_config};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::scope::kinds::{ScopeKind, Semantics};
use auratranslate_lib::core::scope::{
    DEFAULT_MODE, DEFAULT_THEME, ScopeError, ScopeResolver, Tier, load_global_config, save_value,
};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction, Tuning};

// ═════════════════════════════════════════════════════════════════════════════════
// Hạ tầng dùng chung
// ═════════════════════════════════════════════════════════════════════════════════

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

/// Một thư mục tạm **của riêng ca này**. Xem luật 1.
fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, AtomicOrdering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-scope-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));
    dir
}

/// ⚠️ Gọi **sau** khi `Store` đã drop. Xem luật 2.
fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

/// `Tuning` cho các ca không quan tâm tới checkpoint — nhịp chậm, ngưỡng vô cực.
fn quiet_tuning() -> Tuning {
    Tuning {
        checkpoint_tick: Duration::from_millis(50),
        idle_before_passive: Duration::from_secs(3600),
        wal_threshold_bytes: u64::MAX,
        close_truncate_budget: Duration::from_secs(5),
        ..Tuning::default()
    }
}

fn open_store(dir: &Path) -> Store {
    Store::open(StoreSpec {
        tuning: quiet_tuning(),
        ..StoreSpec::global(dir.join("global.db"))
    })
    .expect("mở kho global")
}

fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC4 — bảng ngữ nghĩa, khai tường minh, không mặc định ngầm
// ═════════════════════════════════════════════════════════════════════════════════

/// **Số biến thể viết TAY**, và đó là cả điểm của nó.
///
/// `ALL` sinh từ macro nên nó không bao giờ lệch khỏi `enum` — tức nó **không** bắt được
/// việc thêm một loại. Con số dưới đây là chỗ một con người phải ký: thêm hàng thứ mười
/// vào `scope_kinds!` làm ca này đỏ, và người sửa buộc phải mở AD-18 ra thay vì gõ tiếp.
const DECLARED_KIND_COUNT: usize = 9;

/// `ALL` phủ đúng mọi biến thể, và mỗi biến thể có khoá dây riêng.
#[test]
fn the_kind_table_has_every_variant() {
    assert_eq!(
        ScopeKind::ALL.len(),
        DECLARED_KIND_COUNT,
        "bảng AD-18 mở rộng có {DECLARED_KIND_COUNT} hàng. Lệch nghĩa là một loại vừa được \
         thêm (hoặc gỡ) mà không ai đối chiếu với AD-18 — mở \
         `ARCHITECTURE-SPINE.md#AD-18` rồi cập nhật cả hai chỗ."
    );

    let mut wires: Vec<&str> = ScopeKind::ALL.iter().map(|k| k.as_str()).collect();
    let before = wires.len();
    wires.sort_unstable();
    wires.dedup();
    assert_eq!(
        wires.len(),
        before,
        "hai biến thể trỏ về cùng một khoá dây — một lỗi gõ phím trong `scope_kinds!`. \
         Hậu quả: một loại im lặng đọc/ghi vào hàng của loại kia trong `config_value`. \
         Danh mục sau khi khử trùng: {wires:?}"
    );

    for kind in ScopeKind::ALL {
        assert_eq!(
            ScopeKind::from_wire(kind.as_str()),
            Some(*kind),
            "`from_wire` phải là nghịch đảo của `as_str` cho `{}` — nếu không, một giá trị \
             ghi được xuống đĩa mà không đọc lại được",
            kind.as_str()
        );

        // Khoá dây đi vào cột `kind` của `config_value` và lên dây IPC: `snake_case` ASCII,
        // ⛔ không hoa, ⛔ không dấu, ⛔ không gạch nối.
        assert!(
            !kind.as_str().is_empty()
                && kind
                    .as_str()
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_'),
            "khoá dây `{}` sai hình dạng — phải là `snake_case` ASCII",
            kind.as_str()
        );
    }

    assert_eq!(
        ScopeKind::from_wire("khong_ton_tai"),
        None,
        "một khoá lạ phải trả `None`, ⛔ không đoán về một biến thể nào"
    );
}

/// 🔴 Bảng AD-18, **đối chiếu từng hàng một**.
///
/// Không phải một phép kiểm hình thức: sáu hàng đầu là AD-18 nguyên văn, và ba hàng cuối
/// là phần **mở rộng** mà Ice ký 2026-08-04. Một `Override` lỡ tay đặt lên `Shortcut` mở
/// một tầng Tác phẩm mà UX đã cấm (`settings.html:246`), và không gì khác trong repo đỏ.
#[test]
fn the_semantics_table_matches_ad_18_row_by_row() {
    let expected: [(ScopeKind, Semantics); DECLARED_KIND_COUNT] = [
        (ScopeKind::Glossary, Semantics::Override),
        (ScopeKind::Prompt, Semantics::Override),
        (ScopeKind::AiConfig, Semantics::Override),
        (ScopeKind::TranslatorName, Semantics::Override),
        (ScopeKind::TranslationMemory, Semantics::Merge),
        (ScopeKind::ImportCleanupRule, Semantics::Merge),
        (ScopeKind::Shortcut, Semantics::GlobalOnly),
        (ScopeKind::LayoutPreset, Semantics::GlobalOnly),
        (ScopeKind::AppConfig, Semantics::GlobalOnly),
    ];

    for (kind, semantics) in expected {
        assert_eq!(
            kind.semantics(),
            semantics,
            "`{}` phải mang ngữ nghĩa {semantics:?} theo AD-18. Đọc `ARCHITECTURE-SPINE.md#AD-18` \
             trước khi đổi dòng này — bảng đó là nguồn, mã là bản sao có kiểu.",
            kind.as_str()
        );
    }

    // Mọi biến thể phải có mặt trong bảng đối chiếu, không chỉ đủ SỐ LƯỢNG: một hàng khai
    // trùng và một hàng thiếu cho ra cùng độ dài.
    for kind in ScopeKind::ALL {
        assert!(
            expected.iter().any(|(k, _)| k == kind),
            "`{}` không có trong bảng đối chiếu của test — thêm nó cùng lúc với hàng AD-18",
            kind.as_str()
        );
    }
}

/// Đúng **ba** loại `GlobalOnly`, và `config_value` phục vụ đúng ba loại đó.
///
/// Ca này canh mệnh đề mà `core::scope::store::load_global_config` dựa vào: nó phân giải
/// ba loại bằng tên. Một loại `GlobalOnly` thứ tư thêm vào mà quên nạp sẽ đọc thành *"cấu
/// hình rỗng"*, không thành lỗi.
#[test]
fn the_three_global_only_kinds_are_exactly_three() {
    let global_only: Vec<&str> = ScopeKind::ALL
        .iter()
        .filter(|k| matches!(k.semantics(), Semantics::GlobalOnly))
        .map(|k| k.as_str())
        .collect();

    assert_eq!(
        global_only,
        vec!["shortcut", "layout_preset", "app_config"],
        "`load_global_config` nạp ba loại này bằng tên. Thêm một loại `GlobalOnly` thứ tư \
         mà quên nạp nó nghĩa là nó đọc ra RỖNG thay vì ra lỗi — cập nhật cả hai chỗ."
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC2 — ghi đè theo TỪNG KHOÁ
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **Ca quan trọng nhất của cả story.**
///
/// Cài đặt sai *(work không rỗng thì trả work)* qua được mọi test viết cẩu thả và làm 411
/// mục Glossary toàn cục biến mất. Mệnh đề bắt được nó, và chỉ mệnh đề này: **một khoá chỉ
/// có ở Global phải còn trong kết quả, mang `tier: Global`.**
#[test]
fn an_override_keeps_keys_that_only_exist_in_the_global_tier() {
    let resolver = ScopeResolver::global_only();
    let global = map(&[("a", "ga"), ("b", "gb"), ("c", "gc")]);
    let work = map(&[("b", "wb")]);

    let out = resolver
        .apply_override(ScopeKind::Glossary, &global, Some(&work))
        .expect("`Glossary` khai `Override`");

    assert_eq!(
        out.len(),
        3,
        "kết quả phải là HỢP của khoá hai tầng (3 khoá), nhận {}. Một kết quả 1 khoá là \
         đúng chữ ký cài đặt *ghi đè theo cả tập*: người dùng thêm MỘT mục riêng cho Tác \
         phẩm và mất hết phần còn lại của Glossary toàn cục. AD-18: *tầng Tác phẩm thắng \
         theo từng thuật ngữ*.",
        out.len()
    );

    let a = out.get("a").expect("khoá `a` chỉ có ở Global phải còn trong kết quả");
    assert_eq!(a.value(), "ga");
    assert_eq!(a.tier(), Tier::Global);
    assert_eq!(a.shadowed(), None, "Global là tầng dưới cùng — không có gì dưới nó để che");

    let c = out.get("c").expect("khoá `c` chỉ có ở Global phải còn trong kết quả");
    assert_eq!(c.value(), "gc");
    assert_eq!(c.tier(), Tier::Global);

    let b = out.get("b").expect("khoá `b` có ở cả hai tầng");
    assert_eq!(b.value(), "wb", "trên khoá TRÙNG, tầng Tác phẩm thắng");
    assert_eq!(b.tier(), Tier::Work);
}

/// `shadowed` mang giá trị **bị che**, và nó là dữ liệu ba màn hình đã vẽ sẵn phụ thuộc.
///
/// `mockups/settings.html:172` vẽ *"Ghi đè Toàn cục — ở tầng Toàn cục đang là Anthropic"*
/// cạnh giá trị đang thắng; `mockups/glossary-manage.html:169` vẽ mục toàn cục *"đang bị
/// che"*. Không mang nó từ hôm nay thì hai màn hình đó phải tự truy vấn lại tầng Global —
/// đúng cái *"một truy vấn riêng"* mà Story 3.1 cấm.
#[test]
fn an_override_carries_the_shadowed_value() {
    let resolver = ScopeResolver::global_only();
    let global = map(&[("provider", "anthropic")]);
    let work = map(&[("provider", "openai"), ("temperature", "0.2")]);

    let out = resolver
        .apply_override(ScopeKind::AiConfig, &global, Some(&work))
        .expect("`AiConfig` khai `Override`");

    let provider = out.get("provider").expect("khoá `provider`");
    assert_eq!(provider.value(), "openai");
    assert_eq!(provider.tier(), Tier::Work);
    assert_eq!(
        provider.shadowed().map(String::as_str),
        Some("anthropic"),
        "giá trị bị che phải đi kèm kết quả — nếu không, màn Cài đặt phải truy vấn lại tầng \
         Global bằng một đường riêng (Story 3.1 cấm)"
    );

    let temperature = out.get("temperature").expect("khoá chỉ có ở tầng Work");
    assert_eq!(temperature.tier(), Tier::Work);
    assert_eq!(
        temperature.shadowed(),
        None,
        "một khoá KHÔNG tồn tại ở tầng Global thì không che gì cả — `shadowed` phải là \
         `None`, ⛔ không phải chuỗi rỗng"
    );
}

/// Tầng Work vắng mặt ⇒ kết quả là nguyên tầng Global. Trạng thái **duy nhất** hôm nay.
#[test]
fn an_override_without_a_work_tier_is_the_whole_global_tier() {
    let resolver = ScopeResolver::global_only();
    let global = map(&[("a", "ga"), ("b", "gb")]);

    let out = resolver
        .apply_override(ScopeKind::Glossary, &global, None)
        .expect("`Glossary` khai `Override`");

    assert_eq!(out.len(), 2);
    assert!(
        out.values().all(|r| r.tier() == Tier::Global && r.shadowed().is_none()),
        "chưa mở Tác phẩm nào ⇒ mọi mục mang `tier: Global` và không che gì"
    );
    assert!(
        !resolver.has_work_tier(),
        "`ScopeResolver::global_only()` ⛔ không được khai một tầng Tác phẩm"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC3 — hợp nhất, và TẦNG LÀ KHOÁ PHỤ
// ═════════════════════════════════════════════════════════════════════════════════

/// Cả hai tầng cùng áp, ⛔ **không khử trùng lặp** — AD-19 cùng triết lý: giữ nguyên bất đồng.
#[test]
fn a_merge_keeps_both_tiers_without_deduplicating() {
    let resolver = ScopeResolver::global_only();
    let global: Vec<String> = ["cat", "dog"].iter().map(|s| (*s).to_owned()).collect();
    let work: Vec<String> = ["cat", "bird"].iter().map(|s| (*s).to_owned()).collect();

    let out = resolver
        .apply_merge(ScopeKind::TranslationMemory, &global, Some(&work), None)
        .expect("`TranslationMemory` khai `Merge`");

    assert_eq!(
        out.len(),
        4,
        "hai tầng có 2 + 2 mục ⇒ kết quả 4 mục. Nhận {}. Một kết quả 3 mục nghĩa là `cat` \
         đã bị khử trùng lặp — AD-19: hai nguồn nói khác nhau là THÔNG TIN, không phải nhiễu.",
        out.len()
    );

    let cats: Vec<Tier> = out
        .iter()
        .filter(|t| t.value() == "cat")
        .map(|t| t.tier())
        .collect();
    assert_eq!(
        cats,
        vec![Tier::Work, Tier::Global],
        "`cat` phải xuất hiện HAI lần, mỗi lần mang nhãn tầng của CHÍNH NÓ — nhãn nằm trên \
         từng mục, ⛔ không phải cả tập mang một nhãn (Story 6.5 · UX-DR40)"
    );
}

/// 🔴 **Tầng là khoá PHỤ.** Bẫy 2, và AD-18 nói trước hậu quả của việc đảo nó.
///
/// AD-18: khoá chính là **xuất xứ** (FR118), khoá phụ là **tầng** — *"một cặp TM toàn cục
/// do chính người dùng dịch vẫn giống văn phong của họ hơn một cặp Tác phẩm do người khác
/// dịch."* Đảo hai khoá cho ra một danh sách **trông có thứ tự** và hỏng đúng mục đích của
/// FR70; AD-18 còn nói trước: *"Không khai thứ tự này thì Giai đoạn 4 và Giai đoạn 6 sẽ
/// cài lệch nhau."*
#[test]
fn the_tier_is_always_the_secondary_sort_key() {
    let resolver = ScopeResolver::global_only();

    // `.0` đóng vai XUẤT XỨ (khoá chính); `.1` chỉ để đọc kết quả. `core::scope` không
    // biết xuất xứ là gì — nó nhận bộ so sánh từ chỗ gọi (§Quyết định #4).
    let global = vec![(1u8, "g-tu-nguoi-dung".to_owned()), (2u8, "g-may-dich".to_owned())];
    let work = vec![(1u8, "w-tu-nguoi-dung".to_owned()), (2u8, "w-may-dich".to_owned())];
    let by_provenance = |a: &(u8, String), b: &(u8, String)| a.0.cmp(&b.0);

    let out = resolver
        .apply_merge(
            ScopeKind::TranslationMemory,
            &global,
            Some(&work),
            Some(&by_provenance as &dyn Fn(&(u8, String), &(u8, String)) -> Ordering),
        )
        .expect("`TranslationMemory` khai `Merge`");

    let order: Vec<(u8, Tier)> = out.iter().map(|t| (t.value().0, t.tier())).collect();
    assert_eq!(
        order,
        vec![
            (1, Tier::Work),
            (1, Tier::Global),
            (2, Tier::Work),
            (2, Tier::Global),
        ],
        "xuất xứ phân nhóm TRƯỚC, tầng chỉ phân xử BÊN TRONG một nhóm. Nhận {order:?}.\n\
         `[(1,Work),(2,Work),(1,Global),(2,Global)]` nghĩa là tầng đã thành khoá CHÍNH — \
         đúng Bẫy 2, và nó cho ra một danh sách trông có thứ tự trong khi FR70 đã hỏng."
    );
}

/// `primary = None` ⇒ tầng là khoá **duy nhất**, Work trước Global, thứ tự trong tầng ổn định.
#[test]
fn a_merge_without_a_primary_key_puts_work_before_global_and_stays_stable() {
    let resolver = ScopeResolver::global_only();
    let global: Vec<String> = ["g1", "g2", "g3"].iter().map(|s| (*s).to_owned()).collect();
    let work: Vec<String> = ["w1", "w2"].iter().map(|s| (*s).to_owned()).collect();

    let out = resolver
        .apply_merge(ScopeKind::ImportCleanupRule, &global, Some(&work), None)
        .expect("`ImportCleanupRule` khai `Merge`");

    let seen: Vec<&str> = out.iter().map(|t| t.value().as_str()).collect();
    assert_eq!(
        seen,
        vec!["w1", "w2", "g1", "g2", "g3"],
        "Work trước Global, và thứ tự BÊN TRONG mỗi tầng giữ nguyên thứ tự nguồn.\n\
         Thứ tự trong tầng đến từ tính ỔN ĐỊNH của phép sắp xếp — `sort_unstable_by` cho \
         ra hai kết quả khác nhau trên cùng dữ liệu, và luật làm sạch chạy theo thứ tự."
    );
}

/// Tầng Work vắng mặt ⇒ hợp nhất là nguyên tầng Global, vẫn mang nhãn tầng.
#[test]
fn a_merge_without_a_work_tier_is_the_whole_global_tier() {
    let resolver = ScopeResolver::global_only();
    let global: Vec<String> = ["a", "b"].iter().map(|s| (*s).to_owned()).collect();

    let out = resolver
        .apply_merge(ScopeKind::TranslationMemory, &global, None, None)
        .expect("`TranslationMemory` khai `Merge`");

    assert_eq!(out.len(), 2);
    assert!(out.iter().all(|t| t.tier() == Tier::Global));
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC1 — gọi sai ngữ nghĩa là một lỗi, ⛔ không phải một gợi ý
// ═════════════════════════════════════════════════════════════════════════════════

/// Gọi sai hàm cho `kind` ⇒ `Err`, ⛔ **không im lặng làm theo ý người gọi**.
///
/// 🔴 `Err` ở **cả debug lẫn release**, ⛔ không `panic!`: `Cargo.toml` ghim
/// `panic = "abort"` ở `[profile.release]`, nên một panic ở đây giết cả tiến trình và cuốn
/// theo writer nối tiếp của AD-11/AD-12.
#[test]
fn calling_the_wrong_resolver_for_a_kind_is_refused() {
    let resolver = ScopeResolver::global_only();
    let empty_map: BTreeMap<String, String> = BTreeMap::new();
    let empty_vec: Vec<String> = Vec::new();

    // `TranslationMemory` khai `Merge` — hỏi nó bằng `Override` là hỏi sai câu.
    let err = resolver
        .apply_override(ScopeKind::TranslationMemory, &empty_map, None)
        .expect_err("`Merge` ⛔ không được phân giải như `Override`");
    assert_eq!(
        err,
        ScopeError::WrongSemantics {
            kind: ScopeKind::TranslationMemory,
            declared: Semantics::Merge,
            called: Semantics::Override,
        },
        "lỗi phải nêu đích danh cả ba: loại nào, bảng khai gì, ai vừa hỏi gì"
    );

    // `Glossary` khai `Override` — hỏi nó bằng `Merge`.
    let err = resolver
        .apply_merge(ScopeKind::Glossary, &empty_vec, None, None)
        .expect_err("`Override` ⛔ không được phân giải như `Merge`");
    assert!(matches!(
        err,
        ScopeError::WrongSemantics {
            kind: ScopeKind::Glossary,
            declared: Semantics::Override,
            called: Semantics::Merge,
        }
    ));

    // `Shortcut` khai `GlobalOnly` — hỏi nó bằng `Override` mở đúng tầng mà UX đã cấm.
    let err = resolver
        .apply_override(ScopeKind::Shortcut, &empty_map, None)
        .expect_err("`GlobalOnly` ⛔ không được phân giải như `Override`");
    assert!(matches!(
        err,
        ScopeError::WrongSemantics {
            kind: ScopeKind::Shortcut,
            declared: Semantics::GlobalOnly,
            called: Semantics::Override,
        },
    ));

    // Và chiều ngược lại: một loại hai tầng ⛔ không được phân giải như chỉ-Global.
    let err = resolver
        .resolve_global_only(ScopeKind::Glossary, &empty_map, None)
        .expect_err("`Override` ⛔ không được phân giải như `GlobalOnly`");
    assert!(matches!(
        err,
        ScopeError::WrongSemantics {
            kind: ScopeKind::Glossary,
            declared: Semantics::Override,
            called: Semantics::GlobalOnly,
        },
    ));
}

/// AC5 — một loại `GlobalOnly` **từ chối** dữ liệu tầng Work.
///
/// ⛔ Không bỏ qua im lặng: bỏ qua im lặng là cách một tầng bị cấm vẫn được ghi xuống đĩa
/// rồi không bao giờ có tác dụng — hỏng đúng kiểu *"trông như đang chạy"*.
#[test]
fn a_global_only_kind_refuses_a_work_tier() {
    let resolver = ScopeResolver::global_only();
    let global = map(&[("mode.library", "Mod+1")]);
    let work = map(&[("mode.library", "Mod+9")]);

    let err = resolver
        .resolve_global_only(ScopeKind::Shortcut, &global, Some(&work))
        .expect_err("phím tắt chỉ tồn tại ở tầng Toàn cục — `settings.html:246`");

    assert_eq!(
        err,
        ScopeError::WorkTierForbidden {
            kind: ScopeKind::Shortcut
        }
    );
}

/// `Some(<rỗng>)` là hợp lệ và tương đương `None` — một map rỗng không khai tầng nào cả.
#[test]
fn a_global_only_kind_accepts_an_empty_work_tier() {
    let resolver = ScopeResolver::global_only();
    let global = map(&[("mode.library", "Mod+1")]);
    let empty: BTreeMap<String, String> = BTreeMap::new();

    let out = resolver
        .resolve_global_only(ScopeKind::Shortcut, &global, Some(&empty))
        .expect("một tầng Work RỖNG không khai một tầng nào — nó không phải một vi phạm");

    assert_eq!(out.len(), 1);
    assert_eq!(out["mode.library"].tier(), Tier::Global);
    assert_eq!(out["mode.library"].shadowed(), None);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC5 — vòng chạy END-TO-END THẬT trên `global.db`
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 AC5 nguyên văn, nghiệm thu bằng một hàng **ghi thẳng vào `global.db`**.
///
/// Đường đi đầy đủ: bảng `config_value` *(bước di trú 2)* → `Store::read` →
/// `ScopeResolver::resolve_global_only` → `bootstrap_config` → giá trị trên dây.
///
/// ⚠️ Ghi bằng SQL thô qua `Store::write` chứ không qua `save_value`, có chủ ý: nếu cả ghi
/// lẫn đọc đều đi qua mã của story này thì một quy ước tên cột sai vẫn khớp với chính nó.
/// Ở đây hàng được đặt xuống bằng tay theo đúng lược đồ đã khai, và đường đọc phải tìm ra.
#[test]
fn a_row_written_straight_into_global_db_resolves_back_through_the_scope_path() {
    let dir = temp_dir("e2e-read");
    let store = open_store(&dir);

    assert_eq!(
        store.schema_version(),
        2,
        "bước di trú 2 phải đã chạy — không có `config_value` thì AC5 không có gì để đọc"
    );

    store
        .write(|tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO config_value (kind, key, value, updated_at)
                 VALUES ('shortcut', 'mode.library', 'Mod+1',
                         strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
                (),
            )?;
            tx.execute(
                "INSERT INTO config_value (kind, key, value, updated_at)
                 VALUES ('app_config', 'theme', 'dark',
                         strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
                (),
            )?;
            tx.execute(
                "INSERT INTO config_value (kind, key, value, updated_at)
                 VALUES ('layout_preset', 'doc-dai', 'two-column',
                         strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
                (),
            )?;
            Ok(())
        })
        .expect("ghi ba hàng cấu hình");

    let config = load_global_config(&store).expect("phân giải tầng Global");
    assert_eq!(config.theme(), "dark", "theme phải đến TỪ ĐĨA, không từ mặc định");
    assert_eq!(
        config.mode(),
        DEFAULT_MODE,
        "chưa ai ghi chế độ ⇒ rơi về mặc định, ⛔ không phải chuỗi rỗng"
    );
    assert_eq!(
        config.shortcuts().get("mode.library").map(String::as_str),
        Some("Mod+1")
    );
    assert_eq!(
        config.layout_presets().get("doc-dai").map(String::as_str),
        Some("two-column")
    );
    assert!(
        config.tiers().iter().all(|t| *t == Tier::Global),
        "mọi mục phải mang `tier: Global` — đây là bằng chứng đường đọc đi QUA \
         `ScopeResolver` chứ không phải một truy vấn tắt chạy song song với nó (AC1)"
    );

    // Và vòng chạy đầy đủ tới hình dạng trên dây.
    let wire = bootstrap_config(Some(&store)).expect("bootstrap phải đọc được");
    assert_eq!(wire.theme, "dark");
    assert_eq!(wire.mode, DEFAULT_MODE);
    assert_eq!(wire.shortcuts.len(), 1);
    assert_eq!(wire.layout_presets.len(), 1);

    drop(store);
    cleanup(&dir);
}

/// Vòng ghi → đọc qua **đường sản phẩm**: `put_config` rồi `bootstrap_config`.
///
/// Đóng `deferred-work.md:140` — *"chế độ mặc định lúc khởi động là `library` và không phép
/// kiểm nào canh"*. Nay có một.
#[test]
fn the_last_mode_survives_a_write_and_a_reopen() {
    let dir = temp_dir("mode-roundtrip");

    {
        let store = open_store(&dir);
        assert_eq!(
            bootstrap_config(Some(&store)).expect("bootstrap").mode,
            DEFAULT_MODE,
            "kho rỗng ⇒ chế độ mặc định là `library` (PRD §5.2: Library là điểm vào ứng dụng)"
        );

        put_config(Some(&store), "app_config", "mode", "reading").expect("ghi chế độ");
        drop(store);
    }

    // Mở LẠI kho — giá trị phải nằm trên đĩa, không trong bộ nhớ của lượt trước.
    let store = open_store(&dir);
    let config = bootstrap_config(Some(&store)).expect("bootstrap sau khi mở lại");
    assert_eq!(
        config.mode, "reading",
        "chế độ cuối phải sống qua một lượt đóng/mở — nếu không thì `watch(currentMode)` \
         phía frontend đang ghi vào hư không"
    );

    // Ghi đè lên chính khoá đó ⇒ `ON CONFLICT` cập nhật, ⛔ không dựng hàng thứ hai.
    save_value(&store, "app_config", "mode", "workspace").expect("ghi lại chế độ");
    let rows: i64 = store
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM config_value WHERE kind = 'app_config' AND key = 'mode'",
                [],
                |r| r.get(0),
            )
        })
        .expect("đếm hàng");
    assert_eq!(
        rows, 1,
        "`PRIMARY KEY (kind, key)` + `ON CONFLICT DO UPDATE` ⇒ đúng MỘT hàng cho mỗi khoá. \
         Hai hàng nghĩa là lần đọc sau phụ thuộc vào thứ tự SQLite trả về."
    );
    assert_eq!(
        bootstrap_config(Some(&store)).expect("bootstrap").mode,
        "workspace"
    );

    drop(store);
    cleanup(&dir);
}

/// Kho rỗng ⇒ mặc định đầy đủ, ⛔ không chuỗi rỗng và ⛔ không lỗi.
///
/// ⚠️ Mặc định phải đến **từ Rust**: `cfg?.theme ?? 'light'` phía frontend chỉ bắt
/// `null`/`undefined`, còn `''` là một giá trị và nó đi thẳng vào `applyTheme('')`.
#[test]
fn an_empty_store_bootstraps_to_complete_defaults() {
    let dir = temp_dir("empty-bootstrap");
    let store = open_store(&dir);

    let config = bootstrap_config(Some(&store)).expect("kho rỗng ⛔ không phải một lỗi");
    assert_eq!(config.theme, DEFAULT_THEME);
    assert_eq!(config.mode, DEFAULT_MODE);
    assert!(
        config.shortcuts.is_empty() && config.layout_presets.is_empty(),
        "chưa ai đặt phím tắt hay preset nào ⇒ map rỗng, và `installCommands` dùng hợp âm \
         mặc định của nó"
    );

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §Quyết định #7 — từ vựng lỗi qua IPC đúng bằng từ vựng của `StoreError`
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 Mọi nhánh lỗi của hai hàm command chỉ sinh ra `IpcError` **dẫn xuất từ `StoreError`**.
///
/// `ScopeError` là **lỗi lập trình**: nó ⛔ không `impl From<..> for IpcError` và ⛔ không
/// bao giờ vượt ranh giới IPC. Story 1.7 §Completion Notes #3: *"⛔ Không khoá nào cho
/// tính năng chưa tồn tại"* — và story này ⛔ không thêm khoá `MessageKey` nào.
#[test]
fn every_command_error_comes_from_the_store_vocabulary() {
    // Ba nhánh lỗi tồn tại hôm nay, và cả ba phải nói bằng từ vựng của kho.
    let no_store = bootstrap_config(None).expect_err("kho vắng mặt là một lỗi");
    let no_store_write = put_config(None, "app_config", "mode", "reading")
        .expect_err("kho vắng mặt là một lỗi cả khi ghi");

    let dir = temp_dir("error-vocabulary");
    let store = open_store(&dir);
    let unknown_kind = put_config(Some(&store), "khong_ton_tai", "k", "v")
        .expect_err("một `kind` lạ ⛔ không được ghi im lặng");
    let wrong_semantics = put_config(Some(&store), "glossary", "k", "v").expect_err(
        "`config_value` phục vụ riêng ba loại `GlobalOnly` — một hàng `glossary` ở đó là \
         mầm của đúng lược đồ EAV mà §Quyết định #1 loại bỏ",
    );

    for (label, err) in [
        ("bootstrap_config(None)", &no_store),
        ("put_config(None, ..)", &no_store_write),
        ("put_config(.., \"khong_ton_tai\", ..)", &unknown_kind),
        ("put_config(.., \"glossary\", ..)", &wrong_semantics),
    ] {
        assert!(
            err.code().starts_with("store."),
            "{label} trả `code = {:?}`. Mọi lỗi qua ranh giới này phải thuộc từ vựng KHO — \
             một `code` mới nghĩa là một khoá `MessageKey` mới đã lẻn vào, hoặc `ScopeError` \
             đã vượt ranh giới IPC (§Quyết định #7).",
            err.code()
        );
        assert!(
            matches!(
                err.message_key(),
                MessageKey::StoreOpenFailed
                    | MessageKey::StoreReadFailed
                    | MessageKey::StoreWriteFailed
                    | MessageKey::StoreWalUnavailable
                    | MessageKey::StoreSchemaTooNew
            ),
            "{label} mang `message_key = {:?}` — ngoài năm khoá kho của Story 1.7",
            err.message_key()
        );
        assert_eq!(
            err.params().get("store").map(String::as_str),
            Some("global"),
            "{label}: `params` phải mang tên kho, và chỉ DỮ LIỆU — ⛔ không mang câu"
        );
    }

    // Không byte nào được ghi ở hai nhánh bị từ chối.
    let rows: i64 = store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM config_value", [], |r| r.get(0)))
        .expect("đếm hàng");
    assert_eq!(
        rows, 0,
        "một lượt ghi bị từ chối ⛔ không được để lại gì — `store.write_failed` nghĩa là \
         *thay đổi vừa rồi chưa được lưu*, và nó phải đúng theo nghĩa đen"
    );

    drop(store);
    cleanup(&dir);
}
