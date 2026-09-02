---
title: 'Story 5.14: Đo NFR3, NFR4, NFR5 và ghi lại trạng thái ba ngưỡng tạm'
type: 'chore'
created: '2026-09-01'
status: 'in-progress'
baseline_commit: 'af470305777b566fd0b8151a742798ac4851ffc8'
review_loop_iteration: 1
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** A6–A8 chưa có cùng một phép đo truy nguyên được ở quy mô giả định 5.000 Chương;
`read_reading_run` còn hai chi phí quy mô chưa đo. Fixture rỗng, PID Rust đơn lẻ hoặc số debug đều có
thể tạo một phán quyết xanh giả.

**Approach:** Dùng một fixture tổng hợp cho benchmark NFR3 release và bàn đo app release cô lập HOME
cho NFR4/NFR5; lưu mẫu thô, môi trường và phán quyết sơ bộ. A6–A8/Q4 vẫn mở tới Story 6.18, sau khi
FR14 tạo được 5.000 Chương qua sản phẩm.

## Boundaries & Constraints

**Always:** Fixture đúng 5.000 Chương, công bố số Work/segment/byte và kiểm quần thể trước khi bấm giờ.
Đo release không `wdio`/debug. NFR3 tách loại truy vấn; NFR4 đo tới grid có dữ liệu; NFR5 cộng app với
WebKit mới sinh. Mọi số mang commit, ngày, máy/OS, toolchain, profile, tải máy, cỡ mẫu; memory báo byte,
MB và MiB; kết luận đều ghi “sơ bộ”.

**Ask First:** Cần tối ưu mã sản phẩm, đổi ngưỡng/đơn vị hoặc thêm dependency. Nếu số nằm giữa
300.000.000 và 314.572.800 byte, dừng để Ice phân xử đơn vị. 🔵 **Ice đã cho phép 2026-09-02**
một command đo feature-gated trong release benchmark: chỉ `story-5-14-bench` được bật nó, feature
mặc định phải vắng command, không dependency/mạng/CSP/ATS mới, và command không được trở thành
đường sản phẩm hay ghi/đọc tệp ngoài HOME nháp của phép đo.

**Never:** Không commit `.db`, HOME nháp, `.app`, `dist/`, `target/`; không chạm dữ liệu thật; không dùng
`.works-block` làm usable, PID Rust làm toàn bộ memory, số debug làm phán quyết hay trộn nhiều đường vào
một p95. Không sửa ngưỡng để đổi màu và không đóng A6–A8/Q4.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| NFR3 | 50.000 segment; target/source, auto-widen, lenient | 10 warmup + ≥200 mẫu/ca; p50/p95/p99 và p95 xấu nhất | Sai count/mode/hit/truncated ⇒ vô hiệu |
| NFR4 | Launch lạnh và ấm của app release | Kết thúc khi grid có đúng fixture; báo từng mẫu, median/max | Probe không usable hoặc fixture rỗng ⇒ `unknown` |
| NFR5 frontier | 5.000 Chương, Chương đầu chưa `done` | Đo quét toàn bảng Chương mà không nạp dãy segment | Thiếu pha idle/tập PID ⇒ `unknown` |
| NFR5 full | 5.000 Chương `done`, rồi về Library | Đo Reading, KeepAlive và app + WebKit ở cả hai pha | PID/byte thiếu ⇒ giữ mẫu lỗi |

</frozen-after-approval>

## Code Map

- `src-tauri/tests/library_index_contract.rs:2062-2129,3061-3155` — fixture `.atproj` thật và bench
  NFR3 `#[ignore]`; nâng thành từng ca release và nguồn fixture tạm.
- `src-tauri/src/core/library/indexer.rs:744-889,1115-1247` — ba đường FTS và kết quả mode.
- `src-tauri/src/lib.rs:475-495,743-790` — startup đồng bộ `Indexer::rebuild`.
- `src/modes/LibraryMode.vue:125-138,745-797` · `src/modes/libraryWorks.ts:134-180` — mốc usable thật.
- `src-tauri/src/commands/segment.rs:1202-1299` — quét mọi Chương và clone dãy segment `done`.
- `src/modes/readingState.ts:43,113-119` · `src/modes/ReadingMode.vue:463-540` · `src/App.vue:291-306`
  — memory ReadingRun/DOM/KeepAlive.
- `_bmad-output/implementation-artifacts/2-4-ban-do/` — khuôn release/probe/HOME nháp/dấu sống.
- `_bmad-output/specs/spec-AuraTranslate/{SPEC.md,requirements.md}` · `deferred-work.md` — nơi ghi
  A6–A8/Q4 sơ bộ và món nợ `read_reading_run`.
- `5-14-ban-do/run.sh:23-405` · `probe.js` · `summarize.mjs` — review 2026-09-02 buộc hàng rào
  staged+untracked, manifest provenance, phase-file atomic, tập WebKit không đổi, ma trận raw đủ,
  và marker chứng minh DOM `full=50.000` / `frontier=0` trước khi lấy bộ nhớ.

## Tasks & Acceptance

**Execution:**

- [ ] `src-tauri/tests/{library_index_contract,segment_contract}.rs` — fixture dùng chung và bench
  `#[ignore]` có assertions, phân vị tách ca, elapsed frontier/full-run.
- [ ] `_bmad-output/implementation-artifacts/5-14-ban-do/` — README, probe/build/run với cây sạch,
  HOME nháp, dấu sống, hàng rào dữ liệu thật và cleanup có trap.
- [ ] Chạy ≥3 phiên; NFR4 lạnh/ấm, NFR5 10 mẫu idle/pha cho app + WebKit; lưu TSV, env và báo cáo.
- [ ] `_bmad-output/specs/spec-AuraTranslate/{SPEC.md,requirements.md}` — nối trạng thái sơ bộ vào
  A6–A8/Q4/NFR3–5, giữ nguyên ngưỡng/Q4.
- [ ] `deferred-work.md` · `sprint-status.yaml` — cập nhật đúng mức bằng chứng; Story 6.18 vẫn backlog.
- [ ] Lượt review 2026-09-02 — harness fail-closed với ma trận raw, provenance và DOM/PID; release
  command chỉ tồn tại trong `story-5-14-bench` theo quyền Ice vừa cấp.

**Acceptance Criteria:**

- Given fixture 5.000 Chương, when đo release, then mỗi NFR có mẫu thô, điều kiện và phán quyết sơ bộ
  `dưới ngưỡng`/`vượt ngưỡng`/`chưa phân xử`; không số nào được nâng thành đạt.
- Given usable bị gỡ hoặc fixture rỗng, when tự kiểm, then harness đỏ/unknown; số chỉ PID app bị từ chối.
- Given hai hình dạng `read_reading_run`, when đo, then quét 5.000 Chương và tải 50.000 segment có số
  riêng; chỉ đóng nợ phép đo khi đủ cả hai.

## Spec Change Log

- 2026-09-02 — Ice cho phép command đo feature-gated trong release benchmark sau loop review 1;
  tránh trạng thái cũ dùng hook không có quyền. Giữ nguyên giới hạn: không có command trong build
  mặc định, không dependency/mạng/CSP/ATS, và A6–A8/Q4 không được đóng.

## Verification

**Commands:**

- `cargo test --locked --manifest-path src-tauri/Cargo.toml` — contract xanh; bench vẫn `#[ignore]`.
- `cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml --test library_index_contract -- --ignored --nocapture` — đủ quần thể/phân vị NFR3.
  🔵 **Không `--release`** — dưới `release`, `crate-type` có `staticlib`/`cdylib` gặp `panic = "abort"` làm
  `auratranslate_lib` bị dựng hai bản ghi đè nhau, nên lượt biên dịch đỏ hay xanh theo thứ tự cache chứ không
  theo mã (đo 2026-09-02: lỗi đổi phía giữa hai test target trên cùng một cây). `[profile.bench-release]` kế
  thừa `release` và chỉ đổi `panic`; test target vốn luôn `unwind` nên số đo không đổi. `[profile.release]`
  không bị chạm và app NFR4/NFR5 vẫn dựng bằng `--release`.
- `_bmad-output/implementation-artifacts/5-14-ban-do/run.sh` — đo/cleanup xanh, sinh hồ sơ truy nguyên.
- `npm run build && npm test -- --run` — bundle và frontend không hồi quy.
