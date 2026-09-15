# Bàn đo Story 6.18 — NFR3/NFR4/NFR5 trên thư viện 5.000 Chương thật

Chạy từ gốc kho:

```sh
_bmad-output/implementation-artifacts/6-18-ban-do/run.sh
```

Một lệnh: dựng thư viện 5.000 Chương THẬT (50 Tác phẩm × 100 Chương × 10 segment) qua
`confirm_bilingual_import` + `lifecycle::set_chapter_status` + `Indexer::rebuild` —
**không một fixture SQL thô nào** (khác Story 5.14) — đọc lại quần thể độc lập với builder,
chạy ba session NFR3 release đọc thư viện đó, dựng app Tauri release có probe production
**cộng bốn tệp `resources/dict/*.db` thật qua build-time `--config` merge**, rồi chạy mười
session app cho cold/warm startup và hai fixture memory (`full`/`frontier`). Mỗi pha bộ nhớ
lấy 10 mẫu idle của **PID app cộng mọi WebKit mới sinh**. HOME/DB/.app/dist/target chỉ sống
ở vị trí nháp hoặc đường build đã bị Git bỏ qua; `trap` giết app và xoá HOME nháp kể cả khi
lượt đo trượt.

Khác biệt với `5-14-ban-do` (Code Map spec 6.18, task 5):

- Thư viện là SẢN PHẨM THẬT (Epic 5 retro F5), không một `.atproj` tổng hợp bằng SQL;
- **10 session** NFR4/NFR5, không 3 (§Always spec 6.18: "≥10 sessions, cold and warm");
- App release build có **lớp từ điển thật** (`build.sh` từ chối build nếu
  `src-tauri/resources/dict/*.db` rỗng) — §Always: *"The run is invalid unless the probe
  reads a loaded-layer count > 0 before the first sample"*; `run.sh` đọc số lớp từ chính
  marker `usable` (được `lib.rs::nfr_bench` gắn thêm server-side) và từ chối trước mẫu đầu
  tiên nếu bằng 0;
- Chuyển trạng thái Chương giữa hai lượt đo full/frontier đi qua MỘT ca `#[ignore]` mới,
  `story_6_18_bench_transition.rs` — gọi `lifecycle::set_chapter_status` một Chương một lần
  rồi `Indexer::rebuild`, **không một câu `sqlite3 UPDATE chapter SET status=...`** như
  `5-14-ban-do/run.sh::set_fixture_status` cũ (§Always: "Chapter status only through
  `lifecycle::set_chapter_status`" áp dụng cho MỌI lượt đổi, không riêng lúc dựng);
- Command `nfr_bench` (lib.rs, feature `nfr-bench`) được **tổng quát hoá** ba biến môi
  trường thay vì hai literal cứng của Story 5.14: `AURA_NFR_BENCH_WORK_NAME` ("NFR Story
  6.18 Work 00" thay "5.14 Fixture"), `AURA_NFR_BENCH_WORKS` (50 thay 1),
  `AURA_NFR_BENCH_READING_FULL_SEGMENTS` (1.000 thay 50.000) — không sửa mã cho mỗi bàn đo
  mới, và `5-14-ban-do` chạy nguyên như cũ vì cả ba biến đều có mặc định khớp Story 5.14;
- Không có `reading-run-raw.tsv`/bench `read_reading_run` riêng: hình dạng Reading của 6.18
  (một Tác phẩm, ≤1.000 segment) không so sánh được với 50.000 segment của 5.14 — Design
  Notes spec 6.18 nói rõ số NFR5 pha Reading, không thêm một bench pure-function riêng.

Hàng rào trước khi chạy:

- chỉ cho phép các contract test/artefact/tracking của Story 5.14 VÀ 6.18, cùng đúng ba móc
  đo feature-gated: `nfr-bench` trong `src-tauri/Cargo.toml`, module `nfr_bench` +
  `[profile.bench-release]` trong `src-tauri/src/lib.rs`/`Cargo.toml`;
- `build.sh` từ chối build nếu `src-tauri/resources/dict/*.db` (git-ignored, tải qua GitHub
  Release cục bộ) rỗng — đo trên 0 lớp là ĐÚNG khuyết tật Story 6.18 phải sửa, không phải một
  hàng rào được nới; app release sau build được kiểm lại có ≥1 tệp `.db` dưới
  `Contents/Resources/dict/`;
- `tauri.conf.json` không bị sửa; `--config` chỉ merge thêm một khoá `bundle.resources` lúc
  build, không đụng CSP/`assetProtocol` (dict đọc bằng Rust filesystem, không qua asset
  protocol);
- fixture export chỉ nhận đích có marker `auratranslate-nfr-bench-` và thư mục rỗng; không
  `AURA_NFR_BENCH_RESUME` (cùng lý do provenance của 5-14-ban-do);
- trước BẤT KỲ lượt đo nào, quần thể được đọc lại ĐỘC LẬP với builder (mở thẳng 50
  `project.db`, không qua `Indexer`) và phải khớp đúng 50/5.000/50.000 cùng trạng thái
  `done` toàn bộ — lệch một chỗ nối ⇒ `die` trước khi build app;
- usable đòi grid có đủ 50 Tác phẩm thật, gồm đúng Tác phẩm đích Reading, VÀ ≥1 lớp từ điển
  đã nạp — fixture rỗng, gỡ probe usable, hoặc 0 lớp sẽ timeout/từ chối thành `unknown` và
  `run.sh` thoát đỏ với một thông điệp có tên;
- một mẫu memory thiếu WebKit mới sinh, PID chết, `phys_footprint` hoặc RSS sẽ được giữ
  thành hàng `error`; app PID đơn lẻ không bao giờ được nhận;
- trạng thái `done`/`not_started` CHỈ đổi qua `lifecycle::set_chapter_status` (task 5,
  `story_6_18_bench_transition.rs`), không một câu SQL ghi nào ở BẤT KỲ đường đo nào.

Không dùng bàn phím, chuột, `wdio`, debug build hay dữ liệu Library thật trong lượt đo. Trong
lúc chạy không mở thêm ứng dụng WebKit: tập WebKit của AuraTranslate được nhận bằng hiệu PID
trước/sau spawn, nên một WebKit ngoài phạm vi mới sinh giữa session sẽ làm bẩn tập đo.

Đầu ra giữ lại: `environment.txt`, `fixture.txt`, ba tệp `*-raw.tsv`
(`nfr3`/`startup`/`memory`, cộng `transition-raw.tsv` không gating nhưng giữ để tham khảo) và
`REPORT.md`. `summarize.mjs` đòi ĐỦ ma trận 10 session — một lượt chẩn đoán thu hẹp qua
`AURA_NFR_BENCH_SESSIONS` sẽ làm nó ném lỗi CHỦ Ý, không in một report thiếu như đã đủ.

Phán quyết ở đây **THAY THẾ** bản sơ bộ Story 5.14, không cộng thêm — nhưng bản thân việc ghi
verdict A6/A7/A8 và đóng Q4 vào `prd.md`/`SPEC.md`/`requirements.md` vẫn cần Ice ký trên đúng
con số đã in (spec 6.18 §Always), không phải việc của `run.sh`.
