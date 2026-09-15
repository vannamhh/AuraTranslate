//! Story 6.18, task 5 — chuyển trạng thái Chương của MỘT Tác phẩm đã export
//! (`story_6_18_library.rs`) qua lại giữa "full" (`done`) và "frontier" (`not_started`) tại
//! THỜI ĐIỂM ĐO, không phải lúc dựng thư viện.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! Vì sao tệp này tồn tại tách riêng khỏi `story_6_18_library.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! §Always spec 6.18: *"Chapter status only through `lifecycle::set_chapter_status`"* — áp
//! dụng cho MỌI lượt đổi trạng thái, không riêng lượt dựng. Bàn đo Story 5.14 cũ
//! (`5-14-ban-do/run.sh::set_fixture_status`) đổi trạng thái bằng một câu `UPDATE` SQL thô
//! viết thẳng vào `project.db` giữa hai fixture NFR4/NFR5 — hợp lệ với fixture 5.14 (không
//! do §Always spec 6.18 chi phối) nhưng VI PHẠM đúng câu trên nếu bê nguyên sang thư viện
//! 6.18. `run.sh` của Story 6.18 (task 5) gọi ĐÚNG một ca `#[ignore]` ở đây giữa các khối đo
//! full/frontier, thay cho `sqlite3 UPDATE chapter SET status=...` trực tiếp.
//!
//! Chạy tay, SAU khi `story_6_18_library.rs` đã export:
//!
//!     AURA_6_18_LIBRARY_ROOT=<documents_root> AURA_6_18_BENCH_TARGET_STATUS=not_started \
//!       cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
//!         --test story_6_18_bench_transition -- --ignored --nocapture
//!
//! Tác phẩm bị chuyển LUÔN là Tác phẩm có TÊN nhỏ nhất theo thứ tự từ điển trong chỉ mục —
//! cùng phép chọn `min_by(|a, b| a.name.cmp(&b.name))` mà `story_6_18_library.rs` đã dùng cho
//! hình dạng frontier của chính nó ("NFR Story 6.18 Work 00"), để `run.sh` không phải biết
//! trước `work_id` sinh ra lúc dựng.

use std::path::PathBuf;

use auratranslate_lib::commands::lifecycle::set_chapter_status;
use auratranslate_lib::commands::project::open_work;
use auratranslate_lib::core::library::indexer::{Indexer, WorkQuery};
use auratranslate_lib::core::store::{SqlResult, Store, StoreSpec};

const SCRATCH_MARKER: &str = "auratranslate-nfr-bench-";

#[test]
#[ignore = "ban do chay tay Story 6.18: doi trang thai mot Tac pham that qua lifecycle::set_chapter_status, khong phai mot cong"]
fn toggles_the_frontier_work_between_full_and_frontier_through_product_lifecycle_code() {
    let raw_root = std::env::var("AURA_6_18_LIBRARY_ROOT").unwrap_or_else(|_| {
        panic!(
            "thiếu AURA_6_18_LIBRARY_ROOT -- chạy `story_6_18_library.rs` trước và trỏ biến \
             này vào `documents_root` nó in ra (dòng `STORY_6_18_POPULATION\\tstate=final_export`)"
        )
    });
    assert!(
        raw_root.contains(SCRATCH_MARKER),
        "AURA_6_18_LIBRARY_ROOT phải trỏ vào một HOME nháp mang marker {SCRATCH_MARKER}, nhận {raw_root}"
    );
    let documents_root = PathBuf::from(&raw_root);

    let target_status = std::env::var("AURA_6_18_BENCH_TARGET_STATUS").unwrap_or_else(|_| {
        panic!("thiếu AURA_6_18_BENCH_TARGET_STATUS (done|not_started)")
    });
    assert!(
        target_status == "done" || target_status == "not_started",
        "AURA_6_18_BENCH_TARGET_STATUS ngoài danh mục hai giá trị (done|not_started): {target_status}"
    );

    let global_store = Store::open(StoreSpec::global(documents_root.join("..").join("global.db")))
        .unwrap_or_else(|e| panic!("mở global.db: {e}"));
    let index_path = documents_root.join("..").join("library-index.db");
    assert!(
        index_path.exists(),
        "không thấy library-index.db cạnh {} -- chạy `story_6_18_library.rs` trước",
        documents_root.display()
    );
    let indexer = Indexer::open(index_path).expect("mở chỉ mục đã export");

    let report = indexer.list_works(WorkQuery::default()).expect("liệt kê Tác phẩm đã lập chỉ mục");
    let target = report
        .works
        .iter()
        .min_by(|a, b| a.name.cmp(&b.name))
        .unwrap_or_else(|| panic!("thư viện rỗng: 0 Tác phẩm trong chỉ mục {}", documents_root.display()))
        .clone();

    let mut opened = open_work(&target.work_id, Some(&target))
        .unwrap_or_else(|e| panic!("open_work({}) thất bại: {e:?}", target.work_id));

    let chapter_ids: Vec<i64> = opened
        .store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM chapter ORDER BY ord")?;
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            rows.collect::<SqlResult<Vec<_>>>()
        })
        .expect("đọc danh sách Chương của Tác phẩm đích");
    assert!(!chapter_ids.is_empty(), "Tác phẩm {} không có Chương nào để chuyển trạng thái", target.name);

    // Một lời gọi `set_chapter_status` MỘT Chương — không một câu `UPDATE` gộp viết tay
    // (§Always spec 6.18, cùng kỷ luật `story_6_18_library.rs::set_all_chapters`).
    for chapter_id in &chapter_ids {
        set_chapter_status(Some(&mut opened), *chapter_id, &target_status).unwrap_or_else(|e| {
            panic!("set_chapter_status({chapter_id}, {target_status:?}) thất bại: {e:?}")
        });
    }
    drop(opened);

    indexer
        .rebuild(&documents_root, Some(&global_store))
        .expect("rebuild sau lượt chuyển trạng thái đo");

    // Đọc lại ĐỘC LẬP với `Indexer`, cùng lý do `story_6_18_library.rs::read_back_population`:
    // một lỗi trong `rebuild` không được phép tự che một lượt chuyển trạng thái sai bằng
    // chính con số nó tính ra.
    let db_path = target.atproj_path.join("project.db");
    let store = Store::open(StoreSpec::project(db_path))
        .unwrap_or_else(|e| panic!("mở lại project.db của {}: {e}", target.name));
    let matched: i64 = store
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM chapter WHERE status = ?1",
                [target_status.as_str()],
                |row| row.get(0),
            )
        })
        .expect("đếm lại Chương sau lượt chuyển trạng thái");

    let verdict = if matched as usize == chapter_ids.len() { "matches_target" } else { "mismatch" };
    println!(
        "STORY_6_18_TRANSITION\twork_id={}\twork_name={}\ttarget_status={}\tchapters={}\tchanged={}\tverdict={}",
        target.work_id,
        target.name,
        target_status,
        chapter_ids.len(),
        matched,
        verdict
    );
    assert_eq!(
        matched as usize,
        chapter_ids.len(),
        "không đổi đủ {} Chương của {} sang {target_status}",
        chapter_ids.len(),
        target.name
    );

    drop(indexer);
    drop(global_store);
}
