---
baseline_commit: 85f45299b80582464091ff8bf82160e6101a4476
---

# Story 1.7: Tầng ghi dữ liệu — một writer nối tiếp và lược đồ có phiên bản

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

**Covers:** AD-11 · AD-12 · AD-30 · NFR10 *(nửa cơ chế — nửa "xoá chỉ mục rồi dựng lại" nghiệm thu ở Epic 5)*
**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì

> 🔴 **Đây là story Rust đầu tiên có mã sản phẩm thật.** Story 1.2 dựng khung module rỗng, 1.5 dựng danh mục `MessageKey`, 1.6 giao 0 dòng Rust. Từ hôm nay `src-tauri/src/core/store/` có nội dung, và **mọi story ghi dữ liệu trong chín epic còn lại đi qua nó**. Một quyết định sai ở đây không lộ ra ở story sau — nó lộ ra ở Giai đoạn 2 dưới dạng "gõ bị khựng" hoặc ở tay người dùng dưới dạng dữ liệu hỏng.

---

## Story

As a người dịch,
I want ứng dụng không bao giờ khựng lại vì đang ghi dữ liệu và không bao giờ làm hỏng dữ liệu cũ khi tôi nâng cấp,
So that tôi tin được vào một công cụ mình dùng hàng năm.

---

## Acceptance Criteria

### AC1 — Đúng một kết nối ghi sau hàng đợi nối tiếp; đọc dùng pool song song trên WAL

**Given** mỗi kho ghi được
**When** mở
**Then** có **đúng một** kết nối ghi đặt sau một hàng đợi nối tiếp
**And** đọc dùng pool nhiều kết nối song song trên WAL

*Đạt nghĩa là:* `Store::open()` sinh ra **một** `Connection` ghi sống trong **một** luồng writer, nhận việc qua `std::sync::mpsc`; và **một** pool ≥ 2 kết nối đọc dùng song song được. Nghiệm thu bằng test: N luồng gọi `write()` đồng thời ⇒ các thao tác **không lồng nhau** (chứng minh bằng một bảng đếm ghi vào chính database, không bằng suy luận); M luồng gọi `read()` đồng thời ⇒ chạy chồng nhau thật.

### AC2 — Mọi ghi đi qua `store::Writer`; không module nào tự mở được kết nối ghi

**Given** bất kỳ module nào cần ghi
**When** thực hiện
**Then** đi qua `store::Writer` của kho tương ứng
**And** không module nào tự mở được kết nối ghi — cưỡng chế bằng test hoặc bằng khả năng hiển thị của kiểu

*Đạt nghĩa là* **cả hai** cơ chế, không phải một:
1. **Kiểu** — `rusqlite::Connection` không bao giờ thoát ra khỏi `core::store`. Đường đọc trả về kết nối đã đặt `PRAGMA query_only = 1`, nên **SQLite từ chối lệnh ghi**, không phải người viết tự nhớ.
2. **Test** — `tests/store_boundary.rs` quét cây nguồn: `Connection::open*` và `rusqlite::Connection` chỉ được xuất hiện dưới `src/core/store/**`. Có **sàn số tệp** để cây rỗng không đọc thành sạch.

### AC3 — `global.db` khởi tạo với ba PRAGMA đặt tường minh

**Given** `global.db`
**When** khởi tạo
**Then** `PRAGMA journal_mode = WAL`, `wal_autocheckpoint = 0`, `busy_timeout` đặt tường minh

*Đạt nghĩa là* ba PRAGMA được **đặt rồi ĐỌC LẠI để xác nhận**, và giá trị đọc về sai thì `open()` **trả lỗi**, không đi tiếp. Xem §Bẫy 1 — đặt mà không đọc lại là hỏng im lặng.

### AC4 — Luồng nền trên kết nối riêng: PASSIVE khi rảnh, TRUNCATE khi thoát

**Given** một luồng nền trên **kết nối riêng**
**When** người dùng ngừng thao tác một khoảng
**Then** gọi `wal_checkpoint(PASSIVE)`
**And** khi thoát ứng dụng gọi `wal_checkpoint(TRUNCATE)`

*Đạt nghĩa là:* luồng checkpoint có `Connection` **của riêng nó** (không mượn của writer, không mượn của pool); kết quả `(busy, log, checkpointed)` được **đọc và xét**, không vứt đi; và `RunEvent::Exit` của Tauri chạy TRUNCATE với **thời gian chờ có trần**.

### AC5 — `.db-wal` không phình vô hạn; có ngưỡng kích thước buộc checkpoint

**Given** thao tác liên tục hàng giờ
**When** theo dõi `.db-wal`
**Then** kích thước không phình vô hạn — có ngưỡng kích thước buộc checkpoint

*Đạt nghĩa là* **cơ chế** có thật và test chứng minh nó kích hoạt: ghi liên tục cho `.db-wal` vượt ngưỡng ⇒ checkpoint chạy **dù chưa tới lúc rảnh** ⇒ kích thước `.db-wal` **chững lại** thay vì tiếp tục lên.
⚠️ **PASSIVE không làm tệp nhỏ đi** — nó chép frame về database rồi cho SQLite **dùng lại** chỗ đó, nên tệp giữ nguyên cỡ và ngừng lớn. Đó chính là *"không phình vô hạn"*; ⛔ đừng viết test đòi nó co lại (§Bẫy 8). **Con số ngưỡng là tạm** — xem §Quyết định #8.

### AC6 — Lược đồ có phiên bản; di trú chỉ tiến, trong một giao dịch, sau khi sao lưu

**Given** `global.db` mang số phiên bản lược đồ
**When** mở bằng một bản ứng dụng mới hơn
**Then** chạy các bước di trú **chỉ tiến** trong một giao dịch, sau khi đã sao lưu

*Đạt nghĩa là:* phiên bản nằm ở `PRAGMA user_version`; các bước di trú chạy theo thứ tự tăng dần, **mỗi bước trong một giao dịch**, và **bản sao lưu được tạo trước bước đầu tiên** — bằng `wal_checkpoint(TRUNCATE)` rồi copy tệp, ⛔ không phải copy tệp trần (§Bẫy 5).

### AC7 — Phiên bản mới hơn ứng dụng ⇒ từ chối mở, không ghi một byte

**Given** một database mang phiên bản lược đồ **mới hơn** ứng dụng
**When** mở
**Then** ứng dụng **từ chối mở** và báo rõ
**And** không ghi vào nó một byte nào

*Đạt nghĩa là* test so **byte-for-byte** tệp `.db` trước và sau lần mở bị từ chối, **cộng** khẳng định `.db-wal` và `.db-shm` không được tạo ra. ⚠️ Đây là AC dễ trượt nhất của cả story — xem §Bẫy 4, thứ tự các bước trong `open()` quyết định nó đạt hay không.

---

## Tasks / Subtasks

- [x] **Task 1 — Đường cơ sở: chạy sáu lệnh trên cây sạch, ghi số vào §Debug Log References** (không AC)
  - [x] `npm run build` *(bắt buộc trước `cargo test` — `generate_context!` nhúng `dist/` lúc biên dịch)*
  - [x] `cargo test --manifest-path src-tauri/Cargo.toml` · `npm run check:deps` · `check:tokens` · `check:i18n` · `check:commands`
  - [x] Ghi lại: số tệp `.rs` dưới `src-tauri/**`, số test Rust đang có, `RS_FLOOR` hiện tại của `check-i18n.mjs`
  - [x] ⛔ Không sửa gì ở task này. Sáu lệnh phải exit 0 **trước** khi gõ dòng đầu tiên; một cái đỏ sẵn thì dừng và báo.

- [x] **Task 2 — `core::store` khung kiểu và lỗi** (AC1, AC2, AC7)
  - [x] `src-tauri/src/core/store/mod.rs` — `pub struct Store`, `pub struct StoreSpec`, `pub enum StoreKind`, `pub struct Tuning`, `pub enum StoreError`. Giữ nguyên doc-comment đang có, viết tiếp bên dưới.
  - [x] `StoreError` mang `MessageKey` + `params` **ngay từ hôm nay** (§Quyết định #6), và có `impl From<StoreError> for IpcError`.
  - [x] ⛔ `core::store` **không `use tauri::…`** — xem §Quyết định #1. Đường lấy `$APPDATA` nằm ở `lib.rs`.
  - [x] ⛔ `rusqlite::Connection` không xuất hiện trong bất kỳ chữ ký `pub` nào thoát khỏi module.

- [x] **Task 3 — Writer nối tiếp** (AC1, AC2)
  - [x] `store/writer.rs` — một luồng sở hữu `Connection` ghi; `mpsc::Receiver<Job>`; `Writer::write(closure) -> Result<T, StoreError>` chặn và trả kết quả qua kênh phản hồi riêng của từng lời gọi.
  - [x] Mỗi job chạy trong **một giao dịch**; job trả `Err` ⇒ rollback; job trả `Ok` ⇒ commit.
  - [x] 🔴 Luồng writer **không được panic**: lỗi là **giá trị** đi ngược qua kênh. `catch_unwind` **vô dụng** ở đây (§Bẫy 6).
  - [x] Writer chết hoặc kênh đứt ⇒ `StoreError::WriterGone`, ⛔ **không treo**. Test cho ca này.

- [x] **Task 4 — Pool đọc, cưỡng chế chỉ-đọc bằng SQLite** (AC1, AC2)
  - [x] `store/reader.rs` — `Mutex<Vec<Connection>>` + `Condvar`, kích thước từ `Tuning`. ⛔ Không thêm `r2d2`/`deadpool`/`parking_lot` (§Ranh giới phạm vi).
  - [x] Mỗi kết nối pool đặt `busy_timeout`, `wal_autocheckpoint = 0` **và `query_only = 1`**, cả ba **đọc lại để xác nhận**.
  - [x] `Store::read(|conn| …)` mượn một kết nối, trả lại khi xong **kể cả khi closure trả `Err`**.
  - [x] Test: một `INSERT` qua đường đọc phải **thất bại** với lỗi của SQLite — đó là bằng chứng của AC2 vế "khả năng hiển thị của kiểu".

- [x] **Task 5 — PRAGMA khởi tạo, đặt rồi ĐỌC LẠI** (AC3)
  - [x] `store/pragmas.rs` — hàm áp `journal_mode = WAL`, `wal_autocheckpoint = 0`, `busy_timeout = <Tuning>`, rồi đọc lại cả ba.
  - [x] `journal_mode` đọc về khác `"wal"` (không phân biệt hoa thường) ⇒ `StoreError::WalUnavailable`, ⛔ không đi tiếp.
  - [x] Test đối chứng âm: mở một database `:memory:` hoặc một cây thư mục dựng sẵn ở chế độ `delete` để chứng minh nhánh lỗi **thật sự chạy**, không chỉ tồn tại.

- [x] **Task 6 — Phiên bản lược đồ, từ chối mở lùi, di trú chỉ tiến** (AC6, AC7)
  - [x] `store/schema.rs` — `struct Migration { to_version: u32, sql: &'static str }`, `const GLOBAL_MIGRATIONS`, `GLOBAL_TARGET_VERSION`.
  - [x] 🔴 **Thứ tự trong `open()` là hợp đồng, không phải sở thích** (§Bẫy 4): mở kết nối → đọc `user_version` → **nếu > target thì đóng và trả lỗi NGAY** → mới đặt PRAGMA → mới di trú.
  - [x] Di trú: với mỗi bước, một giao dịch, chạy `sql`, `PRAGMA user_version = to_version`, commit. ⛔ Không có bước lùi, không có bước "sửa cho vừa".
  - [x] Sao lưu **trước bước đầu tiên** và chỉ khi `user_version >= 1`: `wal_checkpoint(TRUNCATE)` → xác nhận `busy == 0` → `fs::copy` sang `<tên>.db.bak-v<n>` cạnh tệp gốc.
  - [x] Di trú `global.db` bước 1: bảng `schema_migration_log` (§Quyết định #7).

- [x] **Task 7 — Luồng checkpoint** (AC4, AC5)
  - [x] `store/checkpoint.rs` — luồng nền, `Connection` **riêng**, đặt cùng bộ PRAGMA như pool *(trừ `query_only` — checkpoint cần ghi)*.
  - [x] Nhịp: thức dậy theo `tick`; chạy PASSIVE khi *(a)* đã qua `idle` kể từ job ghi cuối, **hoặc** *(b)* `.db-wal` vượt `wal_threshold_bytes` — vế (b) chạy **kể cả khi chưa rảnh** (AC5).
  - [x] Đọc `(busy, log, checkpointed)` của mỗi lượt; `busy != 0` ⇒ ghi chẩn đoán, ⛔ không coi là đã xong.
  - [x] `Store::close()` chạy TRUNCATE với **trần thời gian**, rồi dừng luồng. Trần hết mà chưa xong ⇒ ghi chẩn đoán rồi thoát, ⛔ không treo tiến trình (§Bẫy 7).

- [x] **Task 8 — Nối vào vòng đời ứng dụng** (AC3, AC4)
  - [x] `src-tauri/src/lib.rs`: đổi `builder.run(ctx)` thành `builder.build(ctx)?.run(|handle, event| …)`. *(Kiểm chứng 2026-08-04: `Builder::run` trong `tauri-2.11.5/src/app.rs:2449-2452` **chính là** `self.build(context)?.run(|_, _| {})` — phép đổi này không thay đổi hành vi nào khác.)*
  - [x] Trong `setup()`: `app.path().app_data_dir()` → `fs::create_dir_all` → `Store::open(global_spec)` → `app.manage(…)`.
  - [x] Trong callback: `RunEvent::Exit` ⇒ `Store::close()`.
  - [x] 🔴 **Kiểm chứng lại hai cổng của Story 1.2/1.3** sau khi sửa `lib.rs`: `npm run check:scope` và `npm run check:scope:bundled` phải vẫn exit 0. Móc self-check gọi `app.exit(code)`, và `AppHandle::exit` đi **qua vòng lặp sự kiện** (`tauri-2.11.5/src/app.rs:574-580`) nên callback mới **có chạy** trên đường đó — một `close()` treo ở đây làm hai cổng đỏ vì lý do không liên quan tới chúng.
  - [x] ⛔ Không thêm một `#[tauri::command]` nào (§Ranh giới phạm vi).

- [x] **Task 9 — Từ vựng lỗi** (AC3, AC6, AC7)
  - [x] Thêm khoá vào `message_keys!` trong `src-tauri/src/core/i18n/mod.rs` **kèm bảng tham số bắt buộc**, và chuỗi tương ứng vào `src/i18n/vi.json`.
  - [x] Bộ đề xuất: `err.store.schema_too_new` `["store","found","supported"]` · `err.store.open_failed` `["store"]` · `err.store.wal_unavailable` `["store","mode"]` · `err.store.write_failed` `["store"]`.
  - [x] ⚠️ Cả hai chiều bị khoá: `tests/ipc_contract.rs` đòi mọi khoá có trong `vi.json`, và `check:i18n` đối chiếu placeholder theo **cả hai chiều**. Thêm khoá mà quên chuỗi ⇒ đỏ; thêm chuỗi có `{x}` mà quên khai `params` ⇒ đỏ.
  - [x] ⚠️ Giọng văn theo UX-DR47 *(tiền lệ: Story 1.5 §AC5)*. ⛔ `params` mang **dữ liệu**, không mang câu.

- [x] **Task 10 — Test hành vi** (AC1–AC7)
  - [x] `src-tauri/tests/store_contract.rs`, khai phạm vi ở dòng 1 theo khuôn `config_invariants.rs` và `ipc_contract.rs`.
  - [x] Bảng ca tối thiểu — mỗi AC ít nhất một ca **dương** và một ca **âm**:

    | # | Ca | AC | Kỳ vọng |
    |---|---|---|---|
    | 1 | 8 luồng × 50 job ghi đồng thời | AC1 | Không job nào lồng nhau; tổng số bản ghi đúng |
    | 2 | 4 luồng đọc đồng thời khi writer đang chạy | AC1 | Đọc không bị chặn; kết quả nhất quán |
    | 3 | `INSERT` qua đường `read()` | AC2 | **Err** của SQLite |
    | 4 | Mở mới ⇒ đọc lại ba PRAGMA | AC3 | `wal` · `0` · đúng số `Tuning` |
    | 5 | `journal_mode` không đặt được | AC3 | `StoreError::WalUnavailable`, `open()` **trả Err** |
    | 6 | Ghi rồi để rảnh quá `idle` | AC4 | Một lượt PASSIVE chạy với `busy == 0` và `checkpointed > 0`. ⛔ **Không** assert tệp nhỏ đi |
    | 7 | Ghi liên tục cho `.db-wal` vượt ngưỡng, rồi ghi tiếp cùng lượng nữa | AC5 | Checkpoint chạy **trước** khi tới lúc rảnh; cỡ `.db-wal` sau đợt hai **không lớn hơn đáng kể** đợt một *(chững lại, không phình)* |
    | 8 | `close()` | AC4 | `.db-wal` về 0 byte hoặc biến mất — **chỉ TRUNCATE** làm được điều này |
    | 9 | DB `user_version = 0` (mới tinh) | AC6 | Di trú lên target; `schema_migration_log` có bản ghi |
    | 10 | DB `user_version = target - 1` | AC6 | Đúng **một** bước chạy; tệp `.bak-v…` tồn tại |
    | 11 | Một bước di trú ném lỗi giữa chừng | AC6 | Giao dịch rollback; `user_version` **không đổi** |
    | 12 | DB `user_version = target + 1` | AC7 | `Err`; **băm tệp `.db` không đổi**; `.db-wal`/`.db-shm` **không được tạo** |
    | 13 | Writer bị dừng rồi gọi `write()` | AC1 | `StoreError::WriterGone` trong thời gian hữu hạn, ⛔ không treo |

  - [x] `src-tauri/tests/store_boundary.rs` — quét cây nguồn cho AC2 vế test, có sàn số tệp.
  - [x] ⚠️ Mỗi test dùng thư mục tạm **riêng** (`std::env::temp_dir()` + pid + bộ đếm nguyên tử). ⛔ Không thêm `tempfile`.
  - [x] ⚠️ Dọn dẹp: **drop `Store` trước** khi xoá thư mục — Windows từ chối xoá tệp đang mở (NFR14).

- [x] **Task 11 — Nghiệm thu đỏ-rồi-xanh, có bảng** (AC1–AC7)
  - [x] Với mỗi ca **âm** ở Task 10: chứng minh test **đỏ** khi cơ chế bị gỡ, **xanh** khi có. Ghi lệnh + mã thoát vào §Debug Log References.
  - [x] Ít nhất **hai** ca đối chứng âm cho `store_boundary.rs`: thêm tạm một `Connection::open` ngoài `core::store` ⇒ đỏ; gỡ đi ⇒ xanh.
  - [x] Tiền lệ bắt buộc theo: Story 1.3 §Task 11 · 1.4 §Task 3 (28 ca) · 1.5 §Task 7 (21 ca) · 1.6 §Task 10 (28 ca).

- [x] **Task 12 — Sàn cổng và tài liệu module** (không AC)
  - [x] Nâng `RS_FLOOR` trong `scripts/check-i18n.mjs:223` cho khớp cây mới. ⚠️ Đọc `:190` trước — bộ đếm quét `src-tauri/**`, gồm cả `tests/` *(đang được miễn trừ khỏi Kiểm A ở `:128`, nhưng miễn trừ ≠ không đếm)*. Đặt sàn **dưới** số thật, đúng khuôn các sàn khác.
  - [x] Chứng minh sàn có tác dụng: đặt sàn cao hơn số thật ⇒ cổng đỏ; đặt lại ⇒ xanh.
  - [x] Cập nhật doc-comment đầu `core/store/mod.rs`: mô tả hình dạng đã dựng, ba `Tuning` tạm và **chủ sở hữu con số** *(Story 2.4)*.
  - [x] ⛔ **Không thêm bước CI mới** — xem §Quyết định #9.

- [x] **Task 13 — Đóng sổ nợ** (không AC)
  - [x] `deferred-work.md:22` *(`panic = "abort"` giết đường checkpoint của AD-12)* — mục này **ghi đích danh Story 1.7**. ⛔ **Không sửa `[profile.release]`** (§Câu hỏi cho Ice #1). Cập nhật mục đó: ghi thứ story này **đã** làm được *(writer không panic; TRUNCATE lúc thoát)* và thứ **vẫn còn hở** *(thoát cứng thì không có lần flush cuối)*, rồi giao lại đúng chủ.
  - [x] Mở mục mới trong `deferred-work.md` cho ba con số tạm của `Tuning`, giao **Story 2.4**.
  - [x] ⛔ Không sửa `_bmad-output/planning-artifacts/**` — tiền lệ quyết định #3 của Ice ở Story 1.3.

### Review Findings

*(Code review 2026-08-04 — Blind Hunter, Edge Case Hunter, Acceptance Auditor chạy song song trên diff so với baseline `85f4529`. Đã đọc nguồn thật trước khi chấm mức độ, không chỉ theo hunk diff. AC1–AC7 không có vi phạm nào được xác nhận.)*

**Decision needed:** *(cả hai đã được Ice chốt 2026-08-04 — xem mục Deferred bên dưới)*

**Patch:** *(cả 8 đã áp dụng 2026-08-04 — build sạch, clippy sạch, `check:i18n` xanh, 20/20 test store_boundary + store_contract xanh)*

- [x] [Review][Patch] Gọi lồng `Store::write()` từ trong một write job tự deadlock luồng writer [src-tauri/src/core/store/writer.rs:104] — chặn bằng cờ thread-local `ON_WRITER_THREAD`, trả `StoreError::WriteFailed` ngay thay vì enqueue+chặn.
- [x] [Review][Patch] Vòng lặp nền checkpoint có thể bỏ lỡ tín hiệu dừng (lost-wakeup), ăn hết một `checkpoint_tick` trong ngân sách đóng [src-tauri/src/core/store/checkpoint.rs:271] — đổi `stop_cv.wait_timeout(...)` thành `wait_timeout_while(..., |stop| !*stop)`.
- [x] [Review][Patch] `wal_len()` nuốt mọi lỗi `fs::metadata` thành 0 byte, không chỉ `NotFound` [src-tauri/src/core/store/mod.rs:649] — đổi chữ ký thành `io::Result<u64>`; chỉ `NotFound` = 0, lỗi khác trả ra và được ghi chẩn đoán ở chỗ gọi (`checkpoint.rs`).
- [x] [Review][Patch] `Shared::touch_write()` chạy vô điều kiện kể cả khi job ghi thất bại/rollback [src-tauri/src/core/store/writer.rs:81] — `Task` giờ trả `bool` (đã commit hay chưa); vòng lặp writer chỉ `touch_write()` khi `true`.
- [x] [Review][Patch] Dòng chẩn đoán bị lặp tiền tố "store[…]" hai lần [src-tauri/src/core/store/checkpoint.rs:345] — bỏ tiền tố thừa, dùng thẳng `Display` của `StoreError`.
- [x] [Review][Patch] `target_version()` được tính từ danh sách di trú CHƯA kiểm tra thứ tự tăng dần [src-tauri/src/core/store/mod.rs:493] — tách `schema::validate_strictly_increasing()`, gọi ở `Store::open()` bước 2 trước khi tin `target`, và `migrate()` gọi lại (idempotent, giá rẻ).
- [x] [Review][Patch] Doc-comment của `Store::close()` nói quá cơ chế "chờ reader rời đi" [src-tauri/src/core/store/mod.rs:579] — chỉnh lại: việc chờ diễn ra trong `wal_checkpoint(TRUNCATE)` (SQLite busy-wait), không phải ở `readers.close()`.
- [x] [Review][Patch] Doc-comment của `Store::read()` nói quá đảm bảo panic-safety dưới `panic = "abort"` của bản release [src-tauri/src/core/store/mod.rs:564] — chỉnh lại: chỉ đúng khi có unwind (dev/test), không đúng ở release.

**Deferred:**

- [x] [Review][Defer] Lỗi checkpoint/backup lúc chạy đều gắn nhãn `StoreError::OpenFailed` [src-tauri/src/core/store/pragmas.rs:249] — deferred, không chặn story này; xem lại khi có story nối chẩn đoán checkpoint lên UI.
- [x] [Review][Defer] Sao lưu bằng `fs::copy` không nguyên tử, không xác minh sau khi chép [src-tauri/src/core/store/schema.rs:137] — deferred, `open()` vẫn dừng đúng khi copy thất bại nên dữ liệu sống không gặp rủi ro; đáng làm cứng hơn nhưng không chặn story này.
- [x] [Review][Defer] `GLOBAL_TARGET_VERSION` nêu ở Task 6 chưa từng được tạo — thay bằng hàm `target_version()` nội bộ crate [src-tauri/src/core/store/schema.rs:82] — deferred, không ảnh hưởng AC nào; ghi lại cho minh bạch.
- [x] [Review][Defer] `Checkpointer::shutdown()` có thể để luồng nền treo lửng sau khi `close()` đã trả về [src-tauri/src/core/store/checkpoint.rs:228] — deferred, đánh đổi có chủ ý và đã ghi rõ; vô hại hôm nay vì chỗ gọi duy nhất là `RunEvent::Exit` ngay trước khi thoát tiến trình; chỉ thành rủi ro thật nếu một story sau thêm luồng khởi động lại kho mà không thoát tiến trình.
- [x] [Review][Defer] `Writer::shutdown()` không có trần thời gian cho `handle.join()`, dựa trên giả định "job không chặn/không gọi ra ngoài" chưa được cưỡng chế [src-tauri/src/core/store/writer.rs:159] — deferred (Ice chốt 2026-08-04). **Lý do:** giữ kỷ luật "không bỏ dở một giao dịch đang commit để tiết kiệm mili-giây trên đường thoát" cho v1; giám sát bằng review thủ công mỗi khi một story mới ghi qua tầng này thay vì cưỡng chế bằng cơ chế.
- [x] [Review][Defer] `ReaderPool::acquire()` chờ `Condvar` không trần, không đối xứng với bảo đảm hữu hạn (`WriterGone`) của đường ghi [src-tauri/src/core/store/reader.rs:107] — deferred (Ice chốt 2026-08-04). **Lý do:** đường đọc không có tác dụng phụ chờ đợi bên ngoài giống job ghi; rủi ro rò rỉ `Lease` thấp hơn rủi ro một job ghi bị chặn, chấp nhận cho v1.

---

## Dev Notes

### Ranh giới phạm vi — đọc trước khi gõ dòng đầu tiên

| Story này **có** làm | Story này **KHÔNG** làm |
|---|---|
| `src-tauri/src/core/store/**` — writer · reader · pragmas · schema · checkpoint | Bất kỳ bảng nghiệp vụ nào *(config, glossary, tm, segment…)* — story sở hữu năng lực tự thêm |
| Một thực thể `Store` cho **`global.db`** | `project.db` — **Story 1.15** · `library-index.db` — **Epic 5** |
| Cơ chế di trú **tổng quát** + bước 1 của `global.db` | Bất kỳ bước di trú nào cho lược đồ chưa tồn tại |
| Nối vào `setup()` và `RunEvent::Exit` của `lib.rs` | Một `#[tauri::command]` nào — xem mục ngay dưới |
| Khoá `MessageKey` cho **đúng** lỗi story này ném | Từ vựng lỗi cho tính năng chưa tồn tại |
| Ba con số `Tuning` **tạm**, khai là tạm | Hiệu chỉnh chúng bằng phép đo — **Story 2.4** |
| Test hành vi + test ranh giới nguồn | Một bước CI mới — §Quyết định #9 |
| `ScopeResolver`? **Không** — AD-18, **Story 1.8** | `meta.json` và số phiên bản của nó — **Story 1.15** (AD-33) |

⛔ **Không đụng tới:** `src-tauri/tauri.conf.json` · `src-tauri/capabilities/**` · `Cargo.toml` *(gồm cả `[profile.release]` — §Câu hỏi #1)* · `package.json` · `.github/workflows/ci.yml` · `src/selftest/**` · `src/tokens/**` · `src/commands/**` · `_bmad-output/planning-artifacts/**`.

⛔ **Không thêm một phụ thuộc nào.** Không `r2d2` / `r2d2_sqlite` / `deadpool` / `bb8`, không `parking_lot`, không `crossbeam`, không `tokio`, không `tempfile`, không `chrono` / `time`, ⛔ **không bật thêm feature nào của `rusqlite`** *(feature `backup` và `hooks` đều đang tắt — xem §Thông tin kỹ thuật)*. NFR15 đòi **mở tệp giấy phép trong nguồn đã tải mà đọc** rồi mới vào bảng Stack; đó là quyết định của Ice, không phải hệ quả phụ của story này. `check-deps.mjs` sẽ đỏ.

⛔ **Không dựng `tauri-plugin-sql`.** Bảng Stack loại nó đích danh với lý do là chính AD-11.

---

### 🔴 Vì sao story này vẫn KHÔNG có `#[tauri::command]`

`deferred-work.md:38-40` ghi một món nợ: *"`ipc_error_wire_shape` là một mệnh đề vòng… nhận lại ở **Story 1.8**, hoặc 1.9/1.11 nếu đường tra cứu chạm Rust trước"*. Story 1.7 **không** nhận món đó, và lý do phải nói ra để lượt sau không phải đoán:

Bảy AC của story này nói về **tầng ghi dữ liệu**, không về một bề mặt IPC. Không AC nào cần frontend gọi được cái gì. Một command dựng ra chỉ để "chứng minh cho thật" là **mã sản phẩm không ai gọi** — đúng ba lý do Story 1.5 và 1.6 đã từ chối, còn nguyên giá trị: nó tốn một lượt biên dịch profile `dev` riêng *(macOS hệ số ×10)*, nó cần webview để chạy, và vòng chạy thật đến **miễn phí** ở Story 1.8 khi `ScopeResolver` có nhu cầu đọc/ghi qua ranh giới.

Cái story này **có** đóng góp cho món nợ đó: `StoreError` mang sẵn `MessageKey` và `From<StoreError> for IpcError`, nên Story 1.8 chỉ phải **nối dây**, không phải phát minh một từ vựng lỗi thứ hai ở chỗ gọi.

---

### Trạng thái repo hiện tại — số, không phải mô tả

Đọc lúc dựng story, `HEAD = 85f4529`:

| | |
|---|---|
| `src-tauri/src/core/store/mod.rs` | **chỉ doc-comment** — 10 dòng, giao việc cho story này bằng chữ *"Story 1.7 sở hữu nội dung"* |
| Số tệp `.rs` dưới `src-tauri/**` | **19** *(17 dưới `src/`, 2 dưới `tests/`)* — `RS_FLOOR` của `check-i18n.mjs:223` đang là **14** |
| Test Rust đang có | `tests/config_invariants.rs` *(Story 1.2/1.3)* · `tests/ipc_contract.rs` *(Story 1.5)* |
| `src/i18n/vi.json` | **11 khoá** |
| `MessageKey` | **2 biến thể** — `Unknown` `[]`, `IoReadFailed` `["path"]` |
| Cổng đã có | `check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `check:scope` · `check:scope:bundled` |
| Bước CI trong job `check` | `check:deps` → `check:tokens` → `check:i18n` → `check:commands` → `npm run build` → `cargo test` → build/đo → `check:scope:bundled` → `check:scope` |
| `lib.rs` | `builder.run(generate_context!())` — **chưa có callback `RunEvent`**; có móc self-check `#[cfg(debug_assertions)]` gọi `app.exit(code)` |
| `ports/mod.rs` | **rỗng** — `ProjectStore` là cổng của **Story 1.15**, ⛔ không khai trait ở story này |
| Node máy Ice / CI | **v22.22.2** / `node-version: '22'` |
| Rust | `edition = "2024"`, `rust-version = "1.85"`, toolchain CI `1.97.1` |

**Sáu lệnh kế thừa** *(chép đúng, đừng phát minh lại)*:

```bash
npm run check:deps                                 # 13 phép kiểm — cây phụ thuộc
npm run check:tokens                               # 7 phép kiểm — token
npm run check:i18n                                 # 5 phép kiểm — chuỗi giao diện + hình dạng lỗi
npm run check:commands                             # 5 phép kiểm — CommandRegistry
npm run build                                      # vue-tsc ×2 + vite build
cargo test --manifest-path src-tauri/Cargo.toml    # CẦN `dist/` tồn tại
```

⚠️ `cargo test` **cần `dist/` tồn tại** — `generate_context!` nhúng `frontendDist: "../dist"` **lúc biên dịch**. Thiếu thì gãy ở khâu biên dịch, không ở một assert. Trên máy dev bẫy này vô hình vì `dist/` còn lại từ lượt trước.

---

### Tám cái bẫy — sáu trong tám cho ra một lượt CI XANH với dữ liệu sai

Tất cả đã kiểm chứng bằng cách **đọc nguồn `rusqlite-0.40.1` đã tải về máy**, không bằng trí nhớ. Đường dẫn ghi kèm để lượt sau kiểm lại được.

**1. 🔴 `PRAGMA journal_mode = WAL` đặt được mà KHÔNG bao giờ báo là nó trượt.**
`Connection::pragma_update` gọi `execute_batch` *(`src/pragma.rs:227-248`)*, và `execute_batch` trong 0.40.1 **cố ý nuốt** hàng trả về của PRAGMA: `src/lib.rs:555-560` viết `if !stmt.stmt.is_null() && stmt.step()? { if false { return Err(Error::ExecuteReturnedResults); } }` — nhánh `if false` là một no-op có chủ ý của thượng nguồn. `journal_mode` trả về **chế độ mới** dưới dạng một hàng, và hàng đó bị vứt. Nghĩa là trên một thư mục mà WAL không dùng được, `pragma_update` trả `Ok(())`, database ở lại chế độ `delete`, **mọi bảo đảm của NFR2 và NFR18 biến mất**, và không lỗi nào được ném.
→ **Đặt xong phải `query_row("PRAGMA journal_mode")` đọc lại và so.** Áp cho cả ba PRAGMA của AC3, không riêng cái này.

**2. 🔴 `PRAGMA wal_checkpoint(PASSIVE)` trả về ba cột, và cả hai cách gọi "tự nhiên" đều sai.**
Lệnh trả một hàng `(busy, log, checkpointed)`.
- `conn.execute("PRAGMA wal_checkpoint(PASSIVE)", [])` ⇒ **`Error::ExecuteReturnedResults`** *(`src/statement.rs:682`)* — sai ồn ào, dễ phát hiện.
- `conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE)")` ⇒ **`Ok(())`** và ba cột bị vứt — sai im lặng. Một lượt PASSIVE bị một reader chặn trả `busy = 1` và **không chép được frame nào**; đường mã đọc nó thành "đã checkpoint xong", `.db-wal` cứ phình, và ngưỡng của AC5 không bao giờ có cơ hội đúng.
→ **Dùng `query_row` và xét cả ba số.** `busy != 0` là một trạng thái phải ghi lại, không phải một thành công.

**3. ⚠️ `wal_autocheckpoint` và `busy_timeout` là trạng thái CỦA TỪNG KẾT NỐI, không phải của database.**
Đặt trên kết nối writer rồi tưởng cả kho đã yên là sai hình dạng. Pool đọc và luồng checkpoint mỗi cái là một `Connection` riêng và mỗi cái phải tự đặt. Với `busy_timeout`, quên trên pool nghĩa là reader nhận `SQLITE_BUSY` **ngay lập tức** trong lúc TRUNCATE đang chạy — biểu hiện thành "thỉnh thoảng tra cứu lỗi", không tái lập được.

**4. 🔴 `journal_mode = WAL` GHI VÀO database — nên AC7 trượt nếu đặt PRAGMA trước khi đọc phiên bản.**
AC7 nói nguyên văn *"không ghi vào nó một byte nào"*. Thứ tự tự nhiên nhất để viết — mở, đặt PRAGMA cho xong, rồi mới xét lược đồ — **vi phạm AC7**: chuyển một database từ `delete` sang `wal` viết lại header của tệp. Test so băm tệp sẽ bắt được, nhưng chỉ khi test được viết đúng thứ tự đó.
→ **Hợp đồng thứ tự trong `open()`:** mở → `PRAGMA user_version` *(chỉ đọc)* → **so với target và trả lỗi ngay nếu lớn hơn** → đặt PRAGMA → sao lưu → di trú.
→ ⚠️ Ca biên: `PRAGMA user_version` mặc định là **0**, nên "database mới tinh" và "database ở phiên bản 0" **không phân biệt được**. Quy ước phải khai tường minh: **0 = chưa có lược đồ**, bước di trú đầu tiên đánh số **1**.

**5. 🔴 Sao lưu bằng `fs::copy` tệp `.db` trần là một bản sao KHÔNG ĐẦY ĐỦ khi WAL đang bật.**
Dữ liệu đã commit nhưng chưa checkpoint sống trong `.db-wal`, không trong `.db`. Copy mình tệp `.db` cho ra một bản sao **thiếu đúng những thay đổi gần nhất** — và bản sao đó trông hoàn toàn hợp lệ. Đây là bản sao lưu mà AC6 dựa vào để cho phép di trú, nên nó hỏng ở đúng chỗ đắt nhất.
→ **`wal_checkpoint(TRUNCATE)` → xác nhận `busy == 0` → rồi mới `fs::copy`.**
→ ⛔ Feature `backup` của `rusqlite` **đang tắt** *(chỉ `bundled` được bật)*, nên `Connection::backup` **không tồn tại**; bật nó là thêm bề mặt API mới vào một crate đã ghim, không nằm trong phạm vi story này.

**6. 🔴 `panic = "abort"` làm `catch_unwind` vô dụng, và giết đường checkpoint.**
`Cargo.toml` `[profile.release]` đặt `panic = "abort"` *(cố ý đóng băng để giữ số đo NFR6 của Story 1.1 so sánh được)*. Hệ quả: một `panic!` trong luồng writer **chấm dứt tiến trình ngay** — không unwind, không `Drop`, không cơ hội flush WAL; và trên Windows release `windows_subsystem = "windows"` khiến nó cũng không in ra đâu. `deferred-work.md:22` đã ghi mục này và **giao đích danh Story 1.7**.
→ **Luồng writer không được panic. Lỗi là giá trị.** Mọi `unwrap()` / `expect()` trong `core::store` là một lỗi thiết kế, không phải một lối tắt.
→ ⛔ **Đừng "giải quyết" bằng cách sửa `[profile.release]`** — xem §Câu hỏi cho Ice #1.

**7. ⚠️ `close()` treo làm hai cổng của Story 1.2/1.3 đỏ vì lý do không liên quan.**
`wal_checkpoint(TRUNCATE)` **chờ mọi reader rời đi**. Một kết nối pool đang giữ một giao dịch đọc dài làm TRUNCATE chờ tới hết `busy_timeout`. Trên đường thoát, khoản chờ đó cộng vào thời gian đóng cửa sổ. Mà `scripts/check-scope.mjs` và `check-scope-bundled.mjs` chạy nhị phân **với timeout cứng** và đọc dòng `VERDICT:` — một `close()` chậm biến thành *"self-check chưa chạy tới nơi"* và exit 1, tức hai cổng đỏ vì tầng ghi dữ liệu, không vì phạm vi mà chúng canh.
→ **Trần thời gian ở `close()`.** Hết trần thì ghi chẩn đoán rồi thoát.

**8. 🔴 PASSIVE KHÔNG làm `.db-wal` nhỏ đi — và "sửa" điều đó bằng TRUNCATE là phá chính AD-12.**
Một lượt checkpoint chép frame từ WAL về database rồi **quay đầu đọc/ghi WAL về đầu tệp để dùng lại**. Tệp `.db-wal` **giữ nguyên cỡ**; nó chỉ ngừng lớn. Đó **chính là** thứ AC5 đòi — *"không phình vô hạn"*, không phải *"co lại"*. Chỉ `TRUNCATE` mới cắt tệp về 0.

Đường hỏng cụ thể, và nó rất dễ đi vào: dev viết test case 6 assert `.db-wal` nhỏ đi → đỏ → kết luận "PASSIVE không chạy" → đổi luồng nền sang TRUNCATE cho xanh. Lúc đó test xanh, AC5 trông như đạt, và **AD-12 bị vi phạm ở đúng chỗ nó tồn tại để bảo vệ**: TRUNCATE **chờ mọi reader rời đi**, nên nó là lượt checkpoint duy nhất có thể chặn — đặt nó vào đường chạy nền là dựng lại đúng cái gai trễ mà `wal_autocheckpoint = 0` vừa gỡ ra, và NFR2 mất hiệu lực. Không test nào đỏ, không lỗi nào được ném.
→ **PASSIVE ở đường nền; TRUNCATE chỉ ở `close()` và ngay trước khi sao lưu để di trú.** Bằng chứng của một lượt PASSIVE là `checkpointed > 0` với `busy == 0`, ⛔ không phải cỡ tệp.
→ ⚠️ Kéo theo: WAL chỉ được dùng lại khi lượt checkpoint chép **hết**. Một reader giữ ảnh chụp cũ làm `log > checkpointed`, và tệp **vẫn lớn tiếp** — đó là lý do §Quyết định #8 để `pool_size` nhỏ.

---

### Quyết định thiết kế — đã chốt, không phải lựa chọn của dev

#### #1 — `core::store` KHÔNG phụ thuộc `tauri`

`Store::open` nhận một `StoreSpec` mang `PathBuf`. Đường lấy `$APPDATA` — `app.path().app_data_dir()` — sống ở `lib.rs`, không ở trong module.

Ba lý do, cả ba đo được:
- **Test chạy được.** `tests/store_contract.rs` dựng `Store` trên thư mục tạm, không cần webview, không cần `AppHandle`, không cần một lượt biên dịch profile `dev` riêng. Đây là toàn bộ khác biệt giữa "13 ca chạy trong `cargo test`" và "một bảng nghiệm thu tay" như Story 1.6 phải chấp nhận.
- **AD-11 nói về kho dữ liệu, không về framework.** `project.db` của Story 1.15 nằm trong một `.atproj` do người dùng chọn — không phải `$APPDATA`. Một module biết `$APPDATA` sẽ phải học lại ở story đó.
- Doc-comment đang có của `store/mod.rs` đã ghi luật: *"Đường dẫn `$APPDATA` LUÔN lấy qua `app.path().app_data_dir()` — không viết cứng"*. Luật đó áp cho **chỗ gọi**; module thì nhận đường dẫn đã phân giải.

#### #2 — Pool đọc cưỡng chế bằng `query_only`, không bằng `SQLITE_OPEN_READ_ONLY`

Cả hai đều là cưỡng chế của SQLite chứ không phải kỷ luật, nhưng `READ_ONLY` mang một ràng buộc phụ trên database WAL: kết nối chỉ-đọc cần tệp `-shm` đã tồn tại và cần quyền phù hợp trên thư mục, nên nó gãy ở những ca biên mà `query_only` không có. `PRAGMA query_only = 1` là trạng thái kết nối, đọc lại xác nhận được, và SQLite trả lỗi cho mọi lệnh ghi.

⚠️ Đặt xong **phải đọc lại** — cùng lý do Bẫy 1.

#### #3 — Hàng đợi là `std::sync::mpsc`, không phải một crate mới

`Sender<T>` là `Send` **và `Sync`** kể từ Rust 1.72; toolchain CI là 1.97.1 và `rust-version` khai 1.85, nên `Store` để được trong `app.manage(…)` mà không cần bọc `Mutex`. ⛔ Không kéo `crossbeam` về cho một hàng đợi FIFO mà `std` đã có.

Hình dạng *(hình dạng, không phải bản chép — dev viết bản cuối)*:

```rust
pub fn write<T, F>(&self, job: F) -> Result<T, StoreError>
where
    F: FnOnce(&rusqlite::Transaction<'_>) -> rusqlite::Result<T> + Send + 'static,
    T: Send + 'static,
{
    let (reply_tx, reply_rx) = std::sync::mpsc::channel();
    self.jobs
        .send(Box::new(move |conn: &mut rusqlite::Connection| {
            let outcome = (|| {
                let tx = conn.transaction()?;
                let value = job(&tx)?;
                tx.commit()?;
                Ok(value)
            })();
            let _ = reply_tx.send(outcome); // chỗ gọi bỏ đi thì im lặng, KHÔNG panic
        }))
        .map_err(|_| StoreError::writer_gone())?;
    reply_rx.recv().map_err(|_| StoreError::writer_gone())?
}
```

⚠️ `let _ = reply_tx.send(...)` là cố ý: nếu chỗ gọi đã bỏ đi thì `send` trả `Err`, và một `unwrap()` ở đó giết luồng writer *(Bẫy 6)*.

#### #4 — `Connection` là `Send` nhưng KHÔNG `Sync`; đó là hàng rào, không phải phiền nhiễu

`rusqlite-0.40.1/src/lib.rs:364` chỉ có `unsafe impl Send for Connection {}` — **không** có `Sync`. `OpenFlags::default()` gồm `SQLITE_OPEN_NO_MUTEX` *(`src/lib.rs:1256-1266`)*, tức chế độ multi-thread: một kết nối không được dùng đồng thời từ hai luồng. Trình biên dịch cưỡng chế điều đó thay ta. ⛔ Đừng tìm cách lách bằng `Arc<Connection>` hay `unsafe impl`.

⚠️ `OpenFlags::default()` cũng gồm `SQLITE_OPEN_URI`. Mở **bằng cờ tường minh**, đừng dựa vào mặc định — đường dẫn thư mục người dùng chứa `?` sẽ bị đọc thành URI query.

#### #5 — Một `Store` cho mỗi kho, cùng một kiểu; hôm nay dựng đúng một thực thể

`StoreKind::{ Global, Project, LibraryIndex }` được khai **hết** vì AD-7 đã cố định năm loại kho và ranh giới sở hữu; nhưng chỉ `Global` được **dựng** hôm nay. ⛔ Không viết mã khởi tạo cho `project.db` *(Story 1.15)* hay `library-index.db` *(Epic 5)* — đó là mã không ai gọi, và AD-8 còn nói `library-index.db` **không di trú** *(xoá và dựng lại)*, tức nó cần một nhánh khác mà story đó phải tự quyết.

#### #6 — `StoreError` mang `MessageKey` từ hôm nay

Không phải để hiển thị *(chưa có gì hiển thị nó)*, mà để **Story 1.8 không phải phát minh một từ vựng lỗi thứ hai** ở chỗ gọi. `IpcError::new` là chỗ duy nhất `message_key` gặp `params` *(doc-comment của nó nói rõ vì sao)*, nên `From<StoreError> for IpcError` phải đi qua đó, ⛔ không dựng struct literal.

⚠️ `check-i18n.mjs` Kiểm A quét `src-tauri/**/*.rs` tìm **ký tự có dấu tiếng Việt ở vị trí mã**. `src-tauri/tests/**` được miễn trừ *(`:128`)* nhưng `src/core/store/**` thì **không**. Mọi chuỗi chẩn đoán trong module — kể cả trong `debug_assert!` — viết **không dấu**. Tiền lệ: `core/i18n/mod.rs` đã bị cổng bắt đúng ca này trong lượt review 2026-08-04 và phải viết lại thông báo không dấu.

#### #7 — Bước di trú 1 của `global.db` là `schema_migration_log`

Không phải một bảng nghiệp vụ — `global.db` chưa có nghiệp vụ nào *(config là Story 1.8, phím tắt là 1.21)*. Nhưng AC6 nói *"chạy các bước di trú chỉ tiến trong một giao dịch, sau khi đã sao lưu"*, và **không có bước nào thì AC6 không có gì để nghiệm thu trên đường sản phẩm** — chỉ nghiệm thu được bằng một bộ di trú giả trong test, tức lại là một mệnh đề vòng như `deferred-work.md:38` đã bắt ở Story 1.5.

```sql
CREATE TABLE schema_migration_log (
  version     INTEGER PRIMARY KEY,
  applied_at  TEXT NOT NULL,   -- ISO-8601 UTC
  app_version TEXT NOT NULL
);
```

- `applied_at` lấy bằng `strftime('%Y-%m-%dT%H:%M:%fZ','now')` **của chính SQLite** — ISO-8601 UTC theo Consistency Conventions, và ⛔ không phải thêm `chrono`/`time` cho một dòng.
- `app_version` lấy từ `env!("CARGO_PKG_VERSION")`.
- Bản ghi được chèn **trong cùng giao dịch** với bước di trú sinh ra nó. Ghi ngoài giao dịch là mở đúng ca "sổ nói đã chạy mà lược đồ thì chưa".

#### #8 — Ba con số `Tuning` là TẠM, và story này không được giả vờ chúng đã đo

`ARCHITECTURE-SPINE.md#Deferred` và `epics.md:454` đều xếp *"Ngưỡng kích thước WAL buộc checkpoint (AD-12) + nhịp flush cụ thể (AD-35)"* vào **Giai đoạn 2**, đo trên **Editor thật**, vì hai thứ đó **đánh đổi lẫn nhau**: phải đạt NFR18 *(mất ≤ 5 s)* mà không phạm NFR2 *(không frame nào vượt 50 ms)*. Không có Editor thì không có phép đo, và một con số đẹp không có phép đo đằng sau là thứ ba story nữa sẽ tin là đã hiệu chỉnh.

Mặc định của story này, khai là **tạm** ngay trong `Tuning` và trong doc-comment:

| Tham số | Giá trị tạm | Vì sao con số này |
|---|---|---|
| `pool_size` | **4** | Đủ để AC1 quan sát được đọc chồng nhau; nhỏ để TRUNCATE không phải chờ nhiều reader |
| `busy_timeout` | **5 000 ms** | Dài hơn hẳn một lượt checkpoint bình thường, ngắn hơn ngưỡng người dùng cho là treo |
| `checkpoint_tick` | **1 s** | Độ phân giải của cả hai điều kiện kích hoạt |
| `idle_before_passive` | **5 s** | ⚠️ **Cố ý dài hơn** nhịp flush 2 s của AD-35, để checkpoint không đánh nhau với đường gõ |
| `wal_threshold_bytes` | **4 MiB** | Bằng đúng ngưỡng autocheckpoint mặc định của SQLite *(1000 trang × 4096 B)* — ta tắt cái đó ở AC3 nên lấy lại đúng số nó bỏ lại, tức không đổi hành vi theo một hướng chưa ai đo |
| `close_truncate_budget` | **2 s** | Trần của Bẫy 7 |

⛔ Đừng chôn chúng thành số trần rải rác. Một `struct Tuning` với `Default`, và §Task 13 mở mục bàn giao cho **Story 2.4**.

#### #9 — Không thêm bước CI mới

Bốn cổng `.mjs` hiện có là cổng **frontend** — chúng tồn tại vì dự án không có bộ chạy test JS. Rust **đã có** `cargo test`, và `cargo test` **đã nằm trong** job `check` trên cả hai nền tảng. Mọi phép cưỡng chế của story này *(gồm cả phép quét cây nguồn cho AC2)* là test Rust, nên chúng vào CI **miễn phí**.

Điều này khớp với hai thứ đang có: khối *"CHỖ MÓC CHO EPIC SAU"* của `ci.yml` chỉ chờ **ba luật đã biết** *(1.4 ✅, 4.1, Epic 6)*; và §Ngân sách CI đang mở một mục lo về thời gian chạy trên Windows *(`deferred-work.md`, `timeout-minutes: 60`)*. Thêm một bước không cần thiết là đi ngược cả hai.

⛔ **Nhưng vẫn phải chạy `npm run check:scope` và `check:scope:bundled` bằng tay** sau khi sửa `lib.rs` — §Task 8.

---

### Testing standards

- **Test Rust đặt ở `src-tauri/tests/`** *(integration, `use auratranslate_lib::…`)*. Khuôn: khai **phạm vi ở dòng 1**, một tệp một mối quan tâm — tiền lệ `config_invariants.rs` *(bất biến cấu hình)* và `ipc_contract.rs` *(hợp đồng dây)*. Story này thêm **hai**: `store_contract.rs` *(hành vi)* và `store_boundary.rs` *(ranh giới cây nguồn)*.
- **Nghiệm thu đỏ-rồi-xanh là bắt buộc**, kèm ít nhất hai ca **đối chứng âm** mỗi cơ chế. Tiền lệ: 1.3 §Task 11 · 1.4 §Task 3 (28 ca) · 1.5 §Task 7 (21 ca) · 1.6 §Task 10 (28 ca).
- **`cargo test` chạy các test song song trong cùng một tiến trình.** Mỗi ca phải có thư mục tạm **riêng**; hai ca dùng chung một đường dẫn `.db` sẽ đỏ ngẫu nhiên và bị đọc thành flaky.
- **Dọn dẹp phải drop `Store` trước.** Windows từ chối xoá tệp đang mở — một `remove_dir_all` sớm cho ra một test đỏ **chỉ trên nhánh Windows** của ma trận, đúng lớp lỗi NFR14 mà Story 1.3 dựng CI để bắt.
- **Không đo thời gian bằng `sleep` dài.** Test của AC4/AC5 lái cơ chế bằng `Tuning` thu nhỏ *(tick và idle tính bằng chục mili-giây)*, không bằng cách chờ 5 giây thật — nhân với hai nền tảng thì đó là phút CI.
- **Lệnh chạy trước khi báo xong:** `npm run build` · `cargo test --manifest-path src-tauri/Cargo.toml` · `check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `check:scope` · `check:scope:bundled`. **Cả tám phải exit 0.**

---

### Bàn giao từ các story trước — thứ ảnh hưởng trực tiếp tới story này

**Từ Story 1.2 (scaffold):**
- Bảng Stack được cài **trọn** ở commit đầu, ghim bằng `=`: `rusqlite = "=0.40.1"` *(feature `bundled`)* và `libsqlite3-sys = "=0.38.1"` đã có mặt và **chưa có một dòng mã nào gọi tới**. Story này là chỗ tiêu thụ đầu tiên. ⛔ Không đổi số, ⛔ không thêm feature.
- `lib.rs` mang móc self-check `#[cfg(debug_assertions)]` mà **hai cổng đọc kết quả** — ⛔ sửa `lib.rs` mà chạm khối đó là làm mù `check:scope` và `check:scope:bundled`.
- `capabilities/main.json` giữ **ba** quyền tối thiểu, và `tests/config_invariants.rs` khoá đúng ba chuỗi đó. Story này ⛔ không cần thêm quyền nào: `app.path()` phía Rust không đi qua ACL.
- `assetProtocol.scope` ⛔ **không bao giờ chứa `$APPDATA`** — test `asset_protocol_scope_never_contains_appdata` cấm, và `global.db` sống ở đó. Webview không được thấy tệp này.

**Từ Story 1.5 (i18n · hình dạng lỗi):**
- `MessageKey` khai qua `macro_rules! message_keys!` — **một khai báo sinh ba thứ** *(`enum` · `ALL` · `as_str`)* **cộng** bảng `required_params()`. Thêm khoá là thêm **một dòng** trong macro; ⛔ đừng viết tay `ALL`.
- `IpcError` có **trường riêng tư**, dựng **chỉ** qua `IpcError::new` — đó là chỗ duy nhất `message_key` gặp `params`. ⛔ `#[serde(rename_all = "camelCase")]` là cấm.
- `IpcError::new` khi thiếu tham số: `debug_assert!` ở debug, rơi về `MessageKey::Unknown` ở release — ⛔ **không panic**, vì `panic = "abort"` trong đường báo lỗi giết luôn tiến trình *"và cuốn theo cả `core::store`"* — doc-comment đó viết sẵn cho story này.

**Từ Story 1.3 (CI):** job `check` là **workflow duy nhất**; `cargo test` chạy trên **cả hai** nền tảng sau `npm run build`. Nhánh Windows đang là nhánh chậm và `timeout-minutes: 60` là một phỏng đoán chưa đo — đó là lý do §Testing standards cấm `sleep` dài.

**Từ Story 1.6 (CommandRegistry):** không giao dây nào cho story này. Ghi ra để khỏi phải đi tìm: `deferred-work.md:131` nói **Story 1.8** *(không phải 1.7)* là chỗ nạp chế độ mặc định từ đĩa.

---

### Thông tin kỹ thuật — kiểm chứng 2026-08-04 bằng cách ĐỌC NGUỒN ĐÃ TẢI

Cùng phương pháp mà NFR15 đòi cho giấy phép: mở tệp trong `~/.cargo/registry/src/…` mà đọc, không tin trí nhớ và không tin tài liệu trên web.

| Sự thật | Đọc ở đâu | Vì sao story này cần |
|---|---|---|
| SQLite **3.53.2** *(bundled)* | `libsqlite3-sys-0.38.1/sqlite3/sqlite3.h:149` | Vượt xa sàn kiến trúc *(FTS5 `trigram` ≥ 3.34, `remove_diacritics 0` ≥ 3.27)*. `wal_checkpoint(TRUNCATE)` có từ 3.8.8 — an toàn |
| `execute_batch` **nuốt** hàng của PRAGMA | `rusqlite-0.40.1/src/lib.rs:555-560` | **Bẫy 1** |
| `Statement::execute` **từ chối** câu trả hàng | `rusqlite-0.40.1/src/statement.rs:682` | **Bẫy 2** |
| feature `hooks` **TẮT** ⇒ `Wal::checkpoint_v2` và `CheckpointMode` **không tồn tại** | `Cargo.toml:114` *(`hooks = []`, không nằm trong `default` hay `bundled`)*; `src/lib.rs:134-135` `#[cfg(feature = "hooks")]` | Checkpoint **phải** đi qua SQL, không có API Rust |
| feature `backup` **TẮT** ⇒ `Connection::backup` không tồn tại | `Cargo.toml:75` | **Bẫy 5** — sao lưu bằng `fs::copy` sau TRUNCATE |
| `default = ["cache", "ffi-sqlite-wasm-rs"]`; `bundled` kéo thêm `modern_sqlite` | `Cargo.toml:81-84, 106-109` | Danh sách feature thật đang bật, để không ai đoán |
| `Connection: Send`, **không `Sync`** | `rusqlite-0.40.1/src/lib.rs:364` | **Quyết định #4** |
| `OpenFlags::default()` = `READ_WRITE \| CREATE \| NO_MUTEX \| URI` | `rusqlite-0.40.1/src/lib.rs:1256-1266` | **Quyết định #4** — mở bằng cờ tường minh |
| `pragma_update(schema, name, value)` — `schema` là `Option<&str>`, truyền `None` | `rusqlite-0.40.1/src/pragma.rs:227` | Chữ ký đúng cho 0.40.1 |
| `pragma_update_and_check` tồn tại cho PRAGMA trả giá trị | `rusqlite-0.40.1/src/pragma.rs:249` | Đường thay thế hợp lệ cho Bẫy 1 |
| `Builder::run(ctx)` **chính là** `build(ctx)?.run(\|_, _\| {})` | `tauri-2.11.5/src/app.rs:2449-2452` | Task 8 — phép đổi không có tác dụng phụ |
| `RunEvent::{ Exit, ExitRequested, Ready }` tồn tại | `tauri-2.11.5/src/app.rs:220-250` | Task 8 — TRUNCATE ở `Exit` |
| `AppHandle::exit` đi **qua** vòng lặp sự kiện | `tauri-2.11.5/src/app.rs:574-580` | **Bẫy 7** — móc self-check cũng chạy callback mới |
| `std::sync::mpsc::Sender<T>: Sync` từ Rust **1.72** | Ghi chú phát hành Rust; toolchain CI **1.97.1**, `rust-version = 1.85` | **Quyết định #3** — `Store` vào `app.manage` không cần `Mutex` |

⚠️ **`rusqlite` 0.40.1 khai `edition = "2021"`** trong khi dự án là `edition = "2024"` — hợp lệ và không cần làm gì, ghi ra để không ai tưởng có gì phải sửa.

---

### Project Structure Notes

Cây mới, khớp `ARCHITECTURE-SPINE.md#Cây nguồn` *(dòng `store/ # Writer nối tiếp + Reader pool + checkpoint (AD-11, AD-12)`)*:

```text
src-tauri/src/core/store/
  mod.rs          # Store · StoreSpec · StoreKind · Tuning · StoreError · open()
  writer.rs       # luồng writer + hàng đợi nối tiếp        (AD-11)
  reader.rs       # pool + query_only                       (AD-11)
  pragmas.rs      # đặt RỒI ĐỌC LẠI ba PRAGMA của AC3       (AD-12)
  checkpoint.rs   # luồng nền · PASSIVE · TRUNCATE · ngưỡng (AD-12)
  schema.rs       # user_version · từ chối mở lùi · di trú  (AD-30)
src-tauri/tests/
  store_contract.rs   # 13 ca hành vi
  store_boundary.rs   # AC2 vế "cưỡng chế bằng test"
```

Hình dạng nhiều tệp trong một thư mục module là khuôn mà chính spine đã dùng cho `webimport/` *(`fetcher.rs` · `extractor.rs`)*. Rust `snake_case` theo Consistency Conventions.

**Không có chỗ lệch nào so với cây nguồn đã khai.** ⛔ Không thêm thư mục thứ mười ba vào `core/`; ⛔ không khai trait nào trong `ports/` *(`ProjectStore` là Story 1.15)*.

---

### References

- `_bmad-output/planning-artifacts/epics.md:1254-1296` — bảy AC nguyên văn của Story 1.7
- `epics.md:1298-1330` *(Story 1.8)* · `:1580-1625` *(Story 1.15)* — hai ranh giới liền kề
- `epics.md:410-416` — bốn dòng bất biến dữ liệu của Epic 1; `:454` — hàng Deferred giao ngưỡng WAL cho **Giai đoạn 2**
- `epics.md:326` NFR2 · `:344` NFR10 · `:358` NFR14 · `:368` NFR18 · `:282-294` FR96–FR102
- `ARCHITECTURE-SPINE.md#AD-11` — một writer duy nhất, hàng đợi nối tiếp, pool đọc
- `ARCHITECTURE-SPINE.md#AD-12` — ba PRAGMA, luồng nền trên kết nối riêng, ngưỡng WAL *(kèm ghi chú: WAL2 **không tồn tại** như tính năng đã phát hành)*
- `ARCHITECTURE-SPINE.md#AD-30` — lược đồ có phiên bản, mở tiến không mở lùi, `library-index.db` không di trú
- `ARCHITECTURE-SPINE.md#AD-7` — năm loại kho và quyền lúc chạy · `#AD-8` — chỉ mục là dẫn xuất · `#AD-9` — `.atproj` là thư mục
- `ARCHITECTURE-SPINE.md#AD-21` — hình dạng lỗi bốn trường · `#AD-35` — hợp đồng flush *(nhịp 2 s / trần 5 s; một flush chỉ xong **sau khi đã ghi vào WAL**)*
- `ARCHITECTURE-SPINE.md#Consistency Conventions` — *"Mọi ghi qua `store::Writer` của kho tương ứng"* · *"Lưu ISO-8601 UTC trong database"*
- `ARCHITECTURE-SPINE.md#Stack` — `rusqlite` 0.40.1 · `libsqlite3-sys` 0.38.1 · sàn SQLite của kiến trúc; §*"Không dùng, đã loại có lý do"* — `tauri-plugin-sql` bị loại **vì AD-11**
- `ARCHITECTURE-SPINE.md#Deferred` — hàng *"Ngưỡng kích thước WAL buộc checkpoint (AD-12) + nhịp flush cụ thể (AD-35)"*
- `src-tauri/src/core/store/mod.rs` — doc-comment giao việc, giữ nguyên và viết tiếp
- `src-tauri/src/core/i18n/mod.rs` — `message_keys!` · `IpcError::new` *(đọc doc-comment trước khi thêm khoá)*
- `src-tauri/tests/config_invariants.rs` · `ipc_contract.rs` — khuôn tệp test và cách khai phạm vi
- `deferred-work.md:22` — `panic = "abort"` giết đường checkpoint, **giao đích danh Story 1.7**
- `deferred-work.md:38-40` — món nợ `ipc_error_wire_shape`, giao **Story 1.8** *(không phải story này)*
- `.github/workflows/ci.yml` — job `check`, thứ tự bước, khối *"CHỖ MÓC CHO EPIC SAU"*
- `mockups/data-integrity.html` — bề mặt người dùng của FR96–FR102 *(tham chiếu ngữ cảnh; story này không dựng giao diện)*

---

### Câu hỏi cho Ice — đã có mặc định, không chặn

1. **`[profile.release]` `panic = "abort"` — giữ hay đổi?**
   `deferred-work.md:22` giao mục này cho Story 1.7 + lượt đo lại NFR6. Nhưng cùng tệp đó *(mục [D4])* ghi Ice đã chốt **không đụng `Cargo.toml`**, và cảnh báo rằng sửa profile làm **số `.dmg`/`.msi` khác đi**, nên nếu làm thì phải làm **trước** khi chốt baseline NFR6 — tức thuộc **Story 1.9 / 10.9**, không phải hôm nay.
   → **Mặc định: giữ nguyên.** Story này đóng phần đóng được bằng thiết kế *(writer không panic; TRUNCATE lúc thoát)* và ghi thẳng phần còn hở *(thoát cứng không có lần flush cuối)* vào `deferred-work.md`. ⛔ Không sửa `Cargo.toml`.

2. **`schema_migration_log` — có phải là bảng đúng cho bước di trú 1 không?**
   Xem §Quyết định #7. Đường thay thế duy nhất là để `global.db` ở phiên bản 0 và **không có bước di trú nào**, khi đó AC6 chỉ nghiệm thu được bằng một bộ di trú giả trong test — đúng hình dạng "mệnh đề vòng" mà lượt review Story 1.5 đã bắt.
   → **Mặc định: dựng bảng**, một cột `version` · `applied_at` · `app_version`, ghi trong cùng giao dịch với bước sinh ra nó.

3. **Sáu con số `Tuning` — chấp nhận mặc định tạm không?**
   Xem §Quyết định #8. Không con số nào đo được hôm nay vì chưa có Editor.
   → **Mặc định: dùng bảng ở #8**, khai là tạm trong mã, mở mục bàn giao **Story 2.4**. ⛔ Không viết trong Completion Notes rằng chúng đã hiệu chỉnh.

4. **`Store` mở lúc `setup()` hay lúc dùng lần đầu?**
   AC3 nói *"`global.db` **khi khởi tạo**"*, và mở sớm làm một `$APPDATA` không ghi được lộ ra ngay lúc khởi động thay vì lúc người dùng đang gõ.
   → **Mặc định: mở trong `setup()`.** ⚠️ Hệ quả phải nghiệm thu: một lỗi mở kho **không được** làm ứng dụng không khởi động nổi mà **không nói gì** — ghi chẩn đoán rõ, và kiểm chứng rằng `check:scope` / `check:scope:bundled` vẫn xanh.

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, dev-story workflow) — 2026-08-04.

### Debug Log References

#### Đường cơ sở (Task 1) — cây sạch tại `HEAD = 85f4529`, trước dòng mã đầu tiên

| Lệnh | Exit | Ghi chú |
|---|---|---|
| `npm run build` | 0 | chạy TRƯỚC `cargo test` — `generate_context!` nhúng `dist/` lúc biên dịch |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 0 | 20 test *(15 `config_invariants` + 5 `ipc_contract`)* |
| `npm run check:deps` | 0 | |
| `npm run check:tokens` | 0 | |
| `npm run check:i18n` | 0 | 18 tệp `.rs` sau miễn trừ *(20 qua `walk`, 2 miễn trừ)* |
| `npm run check:commands` | 0 | |

Số đo lúc bắt đầu: **20** tệp `.rs` dưới `src-tauri/**` *(17 `src/` + 2 `tests/` + `build.rs`)* · **20** test Rust · `RS_FLOOR = 14` ở `scripts/check-i18n.mjs:223`.

⚠️ §Trạng thái repo của story ghi **19** tệp `.rs`; số thật là **20** — bảng đó không tính `src-tauri/build.rs`, mà `walk()` của `check-i18n.mjs` thì có tính *(nó quét cả `src-tauri/`, không riêng `src-tauri/src/`)*. Sàn áp lên quần thể của cổng, nên số của cổng mới là số phải dùng.

#### Nghiệm thu ĐỎ-rồi-XANH (Task 11) — 17 ca, mỗi ca gỡ đúng một cơ chế rồi trả lại

Phương pháp: vá một chỗ → chạy đúng ca liên quan → ghi mã thoát → **khôi phục tệp từ bản sao** → chạy lại toàn bộ để xác nhận xanh. Bảng dưới đây là mã thoát của lượt **đỏ**; lượt **xanh** là trạng thái repo hiện tại *(40 test, exit 0)*.

| # | Cơ chế bị gỡ | Ca phải đỏ | Exit |
|---|---|---|---|
| 1 | Thêm `rusqlite::Connection::open` vào `core/scope/mod.rs` | `only_core_store_may_name_rusqlite` | 101 |
| 2 | Thêm `use rusqlite::{Connection as …}` ngoài store *(dạng ngoặc nhọn)* | `only_core_store_may_name_rusqlite` | 101 |
| 3 | `STORE_DIR` gõ sai ⇒ miễn trừ không khớp tệp nào | `only_core_store_may_name_rusqlite` | 101 |
| 4 | `RS_FLOOR` của test ranh giới đặt trên số thật | `the_scanned_tree_is_large_enough_to_be_real` | 101 |
| 5 | `use tauri::…` trong `core::store` | `core_store_does_not_depend_on_tauri` | 101 |
| 6 | **Bẫy 1** — đặt PRAGMA mà không đọc lại *(cả `set_and_verify_wal` lẫn `verify_wal`)* | `open_fails_when_wal_cannot_be_enabled` | 101 |
| 7 | **Bẫy 3** — quên `wal_autocheckpoint`/`busy_timeout` trên pool đọc | `the_three_pragmas_read_back_on_every_connection` | 101 |
| 8 | Bỏ `PRAGMA query_only = 1` | `writing_through_the_read_path_is_refused_by_sqlite` | 101 |
| 9 | **Bẫy 4** — đặt PRAGMA **trước** khi đọc `user_version` | `a_newer_schema_is_refused_without_touching_a_single_byte` | 101 |
| 10 | Mỗi job ghi chạy trên một luồng khác *(`thread::scope`)* | `writes_are_serialized` | 101 |
| 11 | Pool đọc thu về một kết nối | `reads_run_in_parallel_while_the_writer_works` | 101 |
| 12 | `shutdown()` không thả `Sender` ⇒ `write()` sau `close()` vẫn chạy được | `write_after_close_fails_instead_of_hanging` | 101 |
| 13 | Không TRUNCATE lúc đóng | `close_truncates_the_wal_to_nothing` | 101 |
| 14 | Bỏ điều kiện **(b)** *(vượt ngưỡng)* của luồng checkpoint | `the_wal_stops_growing_once_it_crosses_the_threshold` | 101 |
| 15 | Bước di trú chạy **ngoài** giao dịch | `a_failing_migration_rolls_back_and_leaves_the_version_alone` | 101 |
| 16 | Không sao lưu trước bước di trú | `one_step_runs_and_a_backup_is_written_first` | 101 |
| 17 | `RS_FLOOR` của `check-i18n.mjs` đặt trên số thật | `npm run check:i18n` | 1 |

⚠️ Ca 6 phải gỡ **hai** chỗ chứ không một: `verify_wal` trên pool đọc bắt lại đúng ca đó, nên một bản chỉ gỡ read-back của writer **vẫn xanh** — chính là hình dạng "hai cổng chồng nhau che nhau" mà nghiệm thu này tồn tại để phát hiện.

#### Bài học từ lượt chạy — ghi lại vì nó sẽ quay lại

🔴 **Ca AC5 đỏ ở lượt đầu với thông số đầu tiên** *(blob 8 KiB · ngưỡng 64 KiB · tick 10 ms · gap 3 ms)*: `.db-wal` 119 512 B sau đợt một, **288 432 B** sau đợt hai — dù `passive_runs = 56`, `passive_busy = 0`, `frames_checkpointed = 1035`. Checkpoint **có** chạy; WAL vẫn phình.

Nguyên nhân **không** phải cơ chế: SQLite chỉ quay `.db-wal` về đầu tệp trong `walRestartLog`, và hàm đó chỉ chạy khi một giao dịch ghi **bắt đầu** đúng lúc `nBackfill == mxFrame`. Với nhịp ghi dày hơn nhịp checkpoint, gần như mọi lượt ghi bắt đầu trên một WAL còn tồn đọng ⇒ không lượt nào quay đầu.

⛔ Đường sửa SAI mà rất dễ đi vào: đổi luồng nền sang TRUNCATE cho tệp co lại — test xanh, AC5 trông như đạt, và AD-12 bị phá ở đúng chỗ nó tồn tại để bảo vệ *(§Bẫy 8)*.
✅ Đường đã đi: chỉnh **nhịp của phép đo**, không chỉnh cơ chế — tick 3 ms **ngắn hơn hẳn** gap 10 ms, blob 32 KiB lớn hơn nửa ngưỡng 64 KiB. Lý do ghi ngay trong ca test. Cộng thêm một mệnh đề thứ hai ít phụ thuộc nhịp hơn: `.db-wal` phải nhỏ hơn **một phần tư** tổng lượng đã ghi.

⚠️ Ca `only_core_store_may_name_rusqlite` đỏ ở lượt đầu trên `core/dict/mod.rs:6` — một **doc-comment** ghi *"crate dành cho module này: `rusqlite` — dùng chung cài đặt với `core::store`"*. Đã sửa phép quét để bỏ qua dòng comment *(cùng luật `check-i18n.mjs` Kiểm A áp cho chuỗi tiếng Việt, và cùng lý do: một cổng đỏ trên câu giải thích chính luật nó canh là một cổng bị gỡ trong tuần)*. Comment đuôi dòng vẫn bị bắt.

#### Ổn định

`--test store_contract --test store_boundary` chạy **5 lượt liên tiếp**: 5 × (16 + 4) xanh, không dao động. ⚠️ Chưa chạy trên runner CI lần nào — xem `deferred-work.md`.

#### Tám lệnh trước khi báo xong — tất cả exit 0

| Lệnh | Exit |
|---|---|
| `npm run check:deps` | 0 |
| `npm run check:tokens` | 0 |
| `npm run check:i18n` | 0 |
| `npm run check:commands` | 0 |
| `npm run build` | 0 |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 0 — **40** test *(15 + 5 + 16 + 4)* |
| `npm run check:scope` | 0 — `VERDICT: PASS` |
| `npm run check:scope:bundled` | 0 — `VERDICT: PASS` |

Hai cổng cuối chạy **sau** khi sửa `lib.rs` *(Task 8)*, và đó là điều kiện của story: móc self-check gọi `app.exit(code)`, mà `AppHandle::exit` đi **qua vòng lặp sự kiện**, nên callback `RunEvent::Exit` mới **có** chạy trên đường đó — một `close()` treo ở đây làm hai cổng đỏ vì lý do không liên quan tới chúng.

### Completion Notes List

**Bảy AC đều có mã sản phẩm và ít nhất một ca dương + một ca âm.** 16 ca hành vi *(`store_contract.rs`)* + 4 ca ranh giới *(`store_boundary.rs`)*, cộng 17 ca nghiệm thu đỏ-rồi-xanh.

- **AC1** — `Store::write` đi qua **một** `Connection` sống trong **một** luồng, nhận việc qua `std::sync::mpsc`. `writes_are_serialized` chứng minh bằng **một bảng ghi vào chính database**: `COUNT(DISTINCT thread) = 1` *(đúng một luồng writer)*, `MAX(depth_seen) = 1` *(không job nào lồng nhau, đo bằng một bộ đếm trong DB)*, `open_before = 0` ở mọi hàng, và đúng 400/400 hàng. `reads_run_in_parallel_while_the_writer_works` đo **đỉnh số reader cùng lúc = 4** trong khi một luồng ghi chạy song song.
- **AC2** — **cả hai** cơ chế: *(1)* `rusqlite::Connection` không thoát khỏi module — đường đọc trả kết nối đã đặt `query_only = 1` nên **SQLite** từ chối lệnh ghi *(`writing_through_the_read_path_is_refused_by_sqlite` khẳng định lỗi mang chữ `readonly`, tức đến từ SQLite chứ không từ một phép kiểm tự viết)*; *(2)* `store_boundary.rs` quét cây nguồn với **sàn số tệp**, cấm mọi nhắc tới `rusqlite` ngoài `src/core/store/**`.
- **AC3** — ba PRAGMA **đặt rồi ĐỌC LẠI**, và đọc về sai thì `open()` **trả lỗi**. Nghiệm thu trên **cả hai** loại kết nối *(Bẫy 3)*. Ca âm dùng `:memory:` — một database WAL không dùng được.
- **AC4** — luồng nền có `Connection` **của riêng nó**; `(busy, log, checkpointed)` được **đọc và xét** *(`busy != 0` ⇒ ghi chẩn đoán và ⛔ không xoá cờ `dirty`)*; `RunEvent::Exit` chạy TRUNCATE với **trần thời gian**.
- **AC5** — điều kiện **(b)** *(vượt ngưỡng)* chạy **kể cả khi chưa rảnh**: ca test đặt `idle_before_passive` một giờ nên mọi lượt quan sát được đều là bằng chứng của vế (b). ⛔ Không ca nào đòi `.db-wal` co lại.
- **AC6** — `PRAGMA user_version`, mỗi bước một giao dịch, sao lưu bằng `wal_checkpoint(TRUNCATE)` → xác nhận `busy == 0` → `fs::copy` *(⛔ không copy tệp trần — Bẫy 5)*. Bước 1 của `global.db` là `schema_migration_log`.
- **AC7** — hợp đồng thứ tự trong `open()`. Ca test so **byte-for-byte** tệp `.db` trước/sau **cộng** khẳng định `.db-wal`/`.db-shm` không được tạo.

**Quyết định và ghi chú cần biết trước khi đọc mã:**

1. **`ReadHandle<'a>` là bí danh của `&rusqlite::Connection`, ⛔ không phải một kiểu bọc.** Task 2 cấm `rusqlite::Connection` xuất hiện trong chữ ký `pub`; Task 4 lại đòi `Store::read(|conn| …)`. Bí danh giữ được cả hai *(chỗ gọi không bao giờ gõ tên `rusqlite`)* mà không phải viết hàng trăm dòng chuyển tiếp **không thêm một phép cưỡng chế nào**. Cưỡng chế thật vẫn là `query_only` + `store_boundary.rs`. Lý do ghi ở doc-comment của `ReadHandle`. Cùng lý lẽ: `core::store` **tái xuất** `Transaction`, `SqlError`, `SqlResult`, `Row`.
2. **`StoreSpec.migrations` là một TRƯỜNG, không phải một hằng tra theo `kind`.** Đây là cách duy nhất nghiệm thu được ca 10 *(`user_version = target - 1` ⇒ đúng một bước + tệp `.bak`)* và ca 11 *(rollback)* mà **không** thêm mã sản phẩm chỉ để test gọi: `GLOBAL_MIGRATIONS` hôm nay có đúng **một** bước, nên `target - 1 = 0`, mà 0 là *"chưa có lược đồ"* — không có gì để sao lưu. Story 1.15 dùng đúng trường này cho `project.db`.
3. **Thêm khoá thứ NĂM ngoài bộ bốn story đề xuất: `err.store.read_failed`.** Bộ đề xuất là *"bộ đề xuất"*, và AC2 đòi một `INSERT` qua đường đọc phải **thất bại** — tức story này **ném thật** một lỗi đọc. Cho nó mượn `err.store.open_failed` là nói sai chuyện đang xảy ra. ⛔ Không khoá nào cho tính năng chưa tồn tại.
4. **`StoreError` có 7 biến thể nhưng 5 khoá.** `WriterGone`/`WriteFailed` chung một câu cho người dùng nhưng **khác `code`**; `PoolClosed`/`ReadFailed` cũng vậy. AD-21 cho phép đúng điều đó — `code` và `message_key` là hai trường, không phải một trường hai tên.
5. **`detail` (văn bản lỗi thô của SQLite) ⛔ KHÔNG đi vào `params`.** Nó là một câu, và AD-21 nói `params` mang **dữ liệu**. Nó ở lại trong `Debug`/`Display`/stderr. `every_store_error_converts_to_a_complete_ipc_error` khẳng định điều này bằng máy.
6. **Lỗi mở kho ⇒ ghi chẩn đoán rồi ĐI TIẾP, ⛔ không chặn khởi động** *(`lib.rs::open_global_store`)*. Hai lý do: chưa có bề mặt nào để **nói** với người dùng *(story này không dựng command nào)*, và một `setup()` trả `Err` làm hai cổng `check:scope*` đỏ. Đã ghi vào `deferred-work.md` và giao **Story 1.8**.
7. **⛔ SÁU con số `Tuning` CHƯA CÁI NÀO ĐƯỢC ĐO.** Khai là tạm trong `Default`, trong doc-comment của từng trường, trong doc-comment module, và trong một mục riêng của `deferred-work.md` giao **Story 2.4**. ⛔ Đừng đọc chúng như đã hiệu chỉnh.
8. **⛔ Không phụ thuộc mới, không feature mới, không bước CI mới, không `#[tauri::command]` nào.** Pool đọc là `Mutex<Vec<Connection>>` + `Condvar` + một guard trả kết nối trong `Drop`; hàng đợi là `std::sync::mpsc`; thư mục tạm trong test là `std::env::temp_dir()` + pid + bộ đếm nguyên tử. `check:deps` xanh.
9. **⛔ Không `unwrap()`/`expect()` nào trong `core::store`.** Mutex khoá qua `unwrap_or_else(|e| e.into_inner())`; nhánh *"không đạt tới được"* của `Lease::conn` trả **một giá trị lỗi** chứ không panic. Lý do là Bẫy 6, và nó áp cho cả những nhánh không thể xảy ra.
10. **`RS_FLOOR` nâng 14 → 18** cho quần thể mới **23** tệp `.rs` sau miễn trừ, giữ nguyên tỷ lệ dư địa cũ (~78%). Sàn tồn tại để bắt một cây bị **cắt mất**, ⛔ không phải để đếm tệp mới. ⛔ `VUE_FLOOR` không đụng tới *(ngoài phạm vi story này)*.

**Việc chưa làm được — ghi thẳng, ⛔ không đánh dấu đạt:**

- **NFR10 mới đóng nửa cơ chế.** Nửa *"xoá chỉ mục rồi dựng lại"* nghiệm thu ở **Epic 5** (AD-8).
- **Thoát cứng không có lần flush cuối** — `panic = "abort"`, `SIGKILL`, mất điện đều không đi qua `RunEvent::Exit`. Dữ liệu **không mất** *(WAL bảo đảm điều đó)*; thứ mất là lượt dọn dẹp. Đã cập nhật `deferred-work.md:22` và giao lại **Story 1.9 / 10.9**.
- **Ngưỡng 4 MiB chưa ai đo** — ca AC5 chạy trên ngưỡng thu nhỏ 64 KiB, chứng minh **cơ chế**, ⛔ không chứng minh **con số**.
- **Chưa lượt CI thật nào** — cùng danh sách với bốn phép nghiệm thu của Story 1.3 đang chờ runner.
- **Kho `project.db` và `library-index.db` chưa có mã khởi tạo** — có chủ ý *(Story 1.15 · Epic 5)*.

### File List

**Thêm mới**

- `src-tauri/src/core/store/pragmas.rs`
- `src-tauri/src/core/store/writer.rs`
- `src-tauri/src/core/store/reader.rs`
- `src-tauri/src/core/store/checkpoint.rs`
- `src-tauri/src/core/store/schema.rs`
- `src-tauri/tests/store_contract.rs`
- `src-tauri/tests/store_boundary.rs`

**Sửa**

- `src-tauri/src/core/store/mod.rs` — `Store` · `StoreSpec` · `StoreKind` · `Tuning` · `StoreError` · `open()` · `wal_path`/`wal_len`; doc-comment giao việc giữ nguyên, viết tiếp bên dưới
- `src-tauri/src/core/i18n/mod.rs` — 5 khoá `MessageKey` của tầng ghi dữ liệu kèm bảng tham số
- `src-tauri/src/lib.rs` — `build(ctx)?.run(callback)` · `setup()` mở `global.db` · `RunEvent::Exit` ⇒ `close()`
- `src/i18n/vi.json` — 5 chuỗi tương ứng *(11 → 16 khoá)*
- `scripts/check-i18n.mjs` — `RS_FLOOR` 14 → 18 kèm lý do
- `_bmad-output/implementation-artifacts/deferred-work.md` — cập nhật mục `panic = "abort"`; mở mục mới cho Story 1.7
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — trạng thái story

⛔ **Không đụng:** `src-tauri/Cargo.toml` *(gồm `[profile.release]`)* · `tauri.conf.json` · `capabilities/**` · `package.json` · `.github/workflows/ci.yml` · `src/selftest/**` · `src/tokens/**` · `src/commands/**` · `ports/mod.rs` · `_bmad-output/planning-artifacts/**`.

---

## Change Log

| Ngày | Ai | Việc |
|---|---|---|
| 2026-08-04 | Dev (Amelia) | Cài đặt trọn 13 task. `core::store` (writer nối tiếp · pool đọc `query_only` · ba PRAGMA đặt-rồi-đọc-lại · luồng checkpoint PASSIVE/TRUNCATE · lược đồ có phiên bản). 5 khoá lỗi mới. Nối `setup()` + `RunEvent::Exit`. 20 test mới (16 hành vi + 4 ranh giới) và 17 ca nghiệm thu đỏ-rồi-xanh. `RS_FLOOR` 14 → 18. Tám lệnh exit 0. Trạng thái: `review`. |
| 2026-08-04 | Scrum Master | Dựng story từ `epics.md#Story 1.7`, `ARCHITECTURE-SPINE.md` (AD-7, AD-8, AD-11, AD-12, AD-21, AD-30, AD-35), `deferred-work.md`, và lượt đọc nguồn `rusqlite-0.40.1` / `tauri-2.11.5` đã tải về máy. Trạng thái: `ready-for-dev`. |
