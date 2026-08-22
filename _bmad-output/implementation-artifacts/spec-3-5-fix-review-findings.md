---
title: 'Story 3.5 — đóng 19 finding của vòng review'
type: 'bugfix'
created: '2026-08-22'
status: 'done'
baseline_commit: '6a517d11824886a001e480fb9fd9262833eafa02'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/implementation-artifacts/3-5-quet-ung-vien-khi-nhap-tai-lieu.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Vòng review Story 3.5 có 19 mục, còn 17 finding riêng sau khi gộp hai cặp trùng. Chúng có thể sinh ứng viên giả, để worker cũ tốn CPU, giữ mutex qua giao dịch, báo lưu giả và làm modal mất độc quyền; năm hợp đồng chưa có test chạy qua đường sản phẩm.

**Approach:** Cho lượt quét outcome phân biệt được, lọc Glossary hai tầng trước khi ghi, huỷ worker cũ và nhả state trước khi chờ writer; sửa frontend rồi canh từng regression ở Rust, vitest hoặc e2e đúng vai.

## Boundaries & Constraints

**Always:** Tần suất/dedup chạy trước tra từ điển. Layer lỗi không được hiểu là “không có”. Term đã có sau phân giải hai tầng bị loại; `WHERE NOT EXISTS` tầng Work vẫn giữ để chặn race. Import đã commit vẫn thành công nếu spawn lỗi. Cancellation chạy trong pha đếm và trước lookup/ghi. Modal đang lưu không đóng; modal mở chặn hợp âm toàn cục.

**Ask First:** Dừng nếu cần dependency/feature, AD, giao dịch chéo database, hoặc API `Store` rộng hơn một write-ticket package-private.

**Never:** Không ghi `glossary_entry`, không `ATTACH` global.db, không đổi ngưỡng mặc định, không đặt nghiệp vụ ở TypeScript, không dùng `Arc<Store>` để worker ghi kho cũ, không dùng sleep/timing làm bằng chứng concurrency, không sửa nợ review đã có chủ.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Dict không kết luận | Layer lỗi mở/lookup | 0 ghi; outcome `dictionary_inconclusive` | Chẩn đoán, tiến trình sống |
| Import liên tiếp | B thay A khi A quét | A dừng trong count/lookup, không ghi hoàn tất; B chạy | Cancellation không giả completed |
| Writer chậm | Batch đã enqueue | State nhả trước khi chờ | Lỗi là giá trị |
| Term có ở Global | Project mới | 0 hàng chờ, skipped tăng | Load/scope lỗi dừng lượt |
| Cụm hoa biến thể | Một/hai space, dấu phẩy | Một key `Fire Dragon`, count cộng dồn | Term không mang dấu câu |
| Họ phồn thể | `蕭炎` lặp `threshold − 1` | Nới như `萧炎` | Dưới nữa bị loại |
| IPC lỗi lạ | Có Tauri, put reject `Error` | Overlay ở lại, hiện `err.unknown` | Ngoài Tauri mới trả null |
| Save/modal | Save pending hoặc modal mở | Close/global shortcut bị chặn | Save xong mới tự đóng |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/project.rs:327-479` -- worker, guard, lookup, write, event; thêm cancellation/outcome, `thread::Builder`, lọc hai tầng và enqueue ngắn.
- `src-tauri/src/core/store/{mod.rs,writer.rs}` -- tách enqueue/reply hiện có thành write-ticket package-private; không mở writer mới.
- `src-tauri/src/core/glossary/{candidate_store.rs,store.rs}` -- batch chỉ thấy Work; reuse `load_tier` + `ScopeResolver::apply_override` một lần cho cả batch.
- `src-tauri/src/core/glossary/{scan.rs,surnames.rs}` -- chuẩn hoá token, cache context, alias phồn thể và cancellation; module vẫn thuần.
- `src/config/bootstrap.ts:249-271` -- chép `hasIpcBridge`/`UNKNOWN_IPC_ERROR` từ adapter project.
- `src/glossarySettingsState.ts:91-132` · `src/main.ts:430-463` · `src/GlossarySettingsOverlay.vue:15-105` -- guard close, keymap gate, selection surface.
- `src-tauri/tests/{glossary_scan_contract.rs,glossary_contract.rs,glossary_commands_contract.rs}` · `tests/frontend/**` · `e2e/**` -- chủ của test thuần, component và webview thật.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands/project.rs` + `src-tauri/src/core/store/{mod.rs,writer.rs}` -- cancellation generation, outcome, spawn lỗi thành giá trị và ticket nhả state trước wait; sửa comment hết đúng.
- [x] `src-tauri/src/core/glossary/{candidate_store.rs,store.rs,scan.rs,surnames.rs}` -- lọc batch hai tầng, chuẩn hoá cụm hoa, alias phồn thể, cache context và hook huỷ.
- [x] `src/config/bootstrap.ts` · `src/glossarySettingsState.ts` · `src/main.ts` · `src/GlossarySettingsOverlay.vue` · `scripts/check-commands.mjs` -- vá bốn lỗi frontend và nâng sàn selection surface theo số thật.
- [x] `src-tauri/tests/**` · `tests/frontend/**` · `e2e/**` -- ca đỏ→xanh cho Matrix và năm verification gap; e2e dùng `realClick`, đăng ký event trước import, Work mới cho mỗi ngưỡng.
- [x] `_bmad-output/implementation-artifacts/3-5-quet-ung-vien-khi-nhap-tai-lieu.md` -- nối append-only kết quả review, finding trùng/phóng đại và bằng chứng; không chạm khối frozen.

**Acceptance Criteria:**
- Given corpus tần suất 5 và config persisted 6 rồi 5, when import hai Work qua IPC, then command trả trước event và chỉ Work ngưỡng 5 có term.
- Given predicate đếm call, when term dưới/đủ ngưỡng, then chỉ term đủ ngưỡng bị lookup đúng một lần.
- Given ScanCandidate count/context khác default, when đọc qua command, then hai trường round-trip nguyên vẹn.
- Given App/overlay với registry thật, when mở, nhập sai, save trượt/thành công và đóng, then DOM/lỗi/focus đúng; webview xác nhận global shortcut bị chặn.
- Given pre-push và targeted e2e, when chạy, then mọi cổng xanh và số spec chạm bề mặt mới được ghi.

## Spec Change Log

## Design Notes

`Arc<Store>` bị loại vì worker có thể ghi kho cũ. Ticket enqueue dưới guard rồi nhả guard trước `wait`; generation chặn CPU và enqueue cũ. Finding “writer đang bận” bị thu hẹp: topology hiện tại chưa có job project khác xếp trước cùng mutex; lỗi thật là mutex bao batch đã đo 19 ms/969 hàng.

Lọc Global không gọi resolve từng term: batch nạp mỗi tier một lần, phân giải một lần, cộng term bị loại vào skipped rồi ghi Work bằng SQL hiện có. Snapshot hai database không atomic; không dựng giao dịch chéo.

## Verification

**Commands:**
- `npm run test` -- ✅ 30 tệp, 357 ca xanh.
- `npm run build` -- ✅ kiểm kiểu + Vite xanh.
- `cargo test --locked --manifest-path src-tauri/Cargo.toml` -- ✅ contract/concurrency xanh;
  các cụm trực tiếp: unit 23/23, scan 25/25, Glossary 63/63, command 6/6, boundary 11/11.
- `.githooks/pre-push` -- ✅ mười một cổng + Vitest + build + cargo test xanh trong 105 giây.
- `npm run test:e2e -- --spec e2e/specs/story-3-5-review.e2e.mjs` -- ✅ 2/2 ca event/threshold/modal xanh trên WKWebView thật.

## Suggested Review Order

**Worker, outcome và cancellation**

- Điểm vào chính: phân loại lookup, outcome và payload không kết luận.
  [`project.rs:322`](../../src-tauri/src/commands/project.rs#L322)

- Scope filter xong mới kiểm generation, enqueue dưới guard ngắn.
  [`project.rs:435`](../../src-tauri/src/commands/project.rs#L435)

- Quét ba trạng thái, huỷ được và chỉ dựng context cho `Missing`.
  [`scan.rs:152`](../../src-tauri/src/core/glossary/scan.rs#L152)

**Phân giải hai tầng và frontend**

- Query chỉ lấy key nhưng vẫn qua `ScopeResolver::apply_override`.
  [`store.rs:275`](../../src-tauri/src/core/glossary/store.rs#L275)

- Lỗi IPC lạ phân biệt Tauri thật với trình duyệt thường.
  [`bootstrap.ts:224`](../../src/config/bootstrap.ts#L224)

- Modal tự khai vai `display`, không thành nguồn Auto-Lookup.
  [`GlossarySettingsOverlay.vue:34`](../../src/GlossarySettingsOverlay.vue#L34)

- Global keymap bị chặn trong toàn vòng đời modal.
  [`main.ts:463`](../../src/main.ts#L463)

**Bằng chứng hồi quy**

- Unit khóa precedence, payload và cancellation muộn không write-ticket.
  [`project.rs:896`](../../src-tauri/src/commands/project.rs#L896)

- Contract khóa nhiều term cùng segment giữ nguyên context và output.
  [`glossary_scan_contract.rs:495`](../../src-tauri/tests/glossary_scan_contract.rs#L495)

- Vitest mount modal thật và kiểm selection contract lúc chạy.
  [`glossarySettings.test.ts:246`](../../tests/frontend/glossarySettings.test.ts#L246)

- WKWebView khóa một event, mode precondition và fallback save.
  [`story-3-5-review.e2e.mjs:82`](../../e2e/specs/story-3-5-review.e2e.mjs#L82)
