---
baseline_commit: 6a4e6b849fc6bf9fd57dccc04551bd5e9e522c7a
---
# Story 2.4: Mũi thăm dò — đo NFR18 và NFR2 đồng thời

Status: in-progress

> 🔴 **ĐÂY LÀ MỘT MŨI THĂM DÒ, KHÔNG PHẢI MỘT STORY TÍNH NĂNG.** Sản phẩm giao ra là **số đo** và
> **một báo cáo**, không phải một bề mặt mới. Mã sản phẩm story này được phép chạm là **các hằng
> số** *(sáu số `Tuning`, ba hằng nhịp flush)* và — chỉ khi số đo bắt buộc — ba đường nóng đã có
> tên ở Task 6. Mọi thứ khác là **ghi số rồi báo**, không phải vá.
>
> 🔵 Khuôn của mũi thăm dò trong dự án này đã có và nó là khuôn tốt: `phase-0-spike-results-2026-08-02.md`
> *(Giai đoạn 0)* và `font-spike-results-2026-08-03.md` *(Story 1.1)*. **Theo đúng khuôn đó, đừng
> phát minh khuôn mới** — xem AC6 và Task 8.

**Covers:** NFR2 (`epics.md:326`) · NFR18 (`epics.md:368`) · **AD-12** (`ARCHITECTURE-SPINE.md:159-163`) · **AD-35** (`:419-425`) · AD-11 (`:153-157`) · AD-31 (`:368-392`) · *mũi thăm dò bắt buộc của Epic 2* (`epics.md:830-836`)
**Epic:** 2 — Biên tập theo segment · story **thứ tư**, ngay sau 2.3
**Nguồn AC:** `epics.md:2118-2149`

**Hàng Deferred story này ĐÓNG:**
- `ARCHITECTURE-SPINE.md:894` — *"Ngưỡng kích thước WAL buộc checkpoint (AD-12) + nhịp flush cụ thể (AD-35)"* **(AC3)**
- `ARCHITECTURE-SPINE.md:897` — *"Thư viện editor cho panel Editor"* **(AC4)**

**Hàng Deferred story này ĐI QUA mà KHÔNG đóng:**
- `ARCHITECTURE-SPINE.md:899` — *"Chiến lược ảo hoá danh sách dài"*, **Giai đoạn 3**. 🔴 Story này
  **đo** trần của nó và **báo**, story này **không dựng** ảo hoá. Xem AC13.

**Nợ có chủ đích danh là story này, ĐÓNG hết ở đây** *(mười mục, mỗi mục một dòng — đừng gộp)*:

| `deferred-work.md` | Món | Đóng ở |
| --- | --- | --- |
| `:201-212` | sáu số `Tuning` chưa số nào được đo | Task 5 · AC7 |
| `:214` | `wal_threshold_bytes` mới chỉ nghiệm thu **cơ chế** ở 64 KiB thu nhỏ, chưa nghiệm thu **4 MiB** | Task 5 · AC3 |
| `:234` | hai luồng checkpoint của hai kho chạy chồng — chưa ai đo | Task 5 · AC10 |
| `:570` | một phiên chạy **HAI** kho (`global.db` + `project.db`), mỗi kho một luồng + pool 4 | Task 5 · AC10 |
| `:591` | đỉnh RSS thật của một lượt nhập 100 MB (`MAX_IMPORT_BYTES`) | Task 7b · AC18 |
| `:2084-2090` | trần NFR2 khi dựng 9.850 `<span>` — 300,1 ms Blink · 1.308,0 ms WebKit | Task 6 · AC13 |
| `:2167-2168` | `:data-caret` bắt Vue dựng lại cả trang mỗi lượt `selectionchange` | Task 6 · AC12 |
| `:2419` | ba ảnh bàn đo 2.2 phải chụp lại + lời khai NFR15 sai ở `2-2-ban-do-editor.html:11` | Task 9 · AC17 |
| `:2441` | `restoreEditedText()` quét **cả Chương** mỗi lượt dựng lại | Task 6 · AC12 |
| `:2449` | `nearestSentenceTo()` ép bố cục lại mỗi cú bấm hụt — **chỗ duy nhất của 2.3 không kèm số đo** | Task 6 · AC12 |

**Nợ ĐI QUA mà KHÔNG đóng, và mỗi cái đã có chủ khác:**
- `deferred-work.md:2290-2297` — ca e2e *gõ lần đầu vào một câu **chưa dịch*** còn đỏ. **Chủ: Story 2.3 (tiếp)**, chờ lượt gõ tay của Ice. Xem §Điều kiện khởi hành mục 2 — nó là **điều kiện đầu vào** của story này, không phải việc của story này.
- `deferred-work.md:2317-2340` — phán quyết **AD-34**. **Chủ: Ice.**
- `deferred-work.md:1954-1957` — mọi bằng chứng chỉ trên macOS. Story này **làm dày thêm** khoảng mù đó và phải nói ra bằng một dòng, không giấu. *(Sửa 2026-08-13: bản cũ trỏ `:145`, đó là nợ Blink/WKWebView của Story 1.6 — nhầm ~1.800 dòng. Xem AC20.)*

---

## Điều kiện khởi hành — ĐỌC TRƯỚC KHI CHẠY PHÉP ĐO ĐẦU TIÊN

### 1. Cây làm việc SẠCH, và đây là mốc gốc

`git status --porcelain` trả **0 dòng** lúc dựng story này (2026-08-13). `baseline_commit` ở
frontmatter là SHA thật của `HEAD`: `6a4e6b8` — *"feat: add frontend tests for editor typing zone
and status bar"*. Không có món vá cũ nào phải commit riêng trước.

### 2. 🔴 STORY 2.3 CÒN `in-progress`, VÀ STORY NÀY ĐO TRÊN ĐÚNG BỀ MẶT CỦA NÓ

Đây là điều kiện đầu vào nặng nhất, và nó phải được đọc hết trước khi lên lịch một phép đo nào.

**Cái đã đứng** — `sprint-status.yaml:130-131` và §Nghiệm thu cuối của 2.3: 9/9 cổng npm xanh ·
`build` xanh · `npm run test` **32/32** · `cargo test --locked` **319 xanh / 0 đỏ / 5 ignored**.
Và một **bằng chứng dương từ tay người**: Ice gõ tay 2026-08-13, thanh trạng thái ghi *"Đã lưu 4
giây trước"* và chữ đã vào `project.db` ⇒ **đường flush AD-35 chạy thật với người dùng thật trên
WKWebView** (`deferred-work.md:2234`).

**Cái còn đỏ** — đúng **một** ca: lượt gõ **đầu tiên** vào một câu **chưa dịch**. Câu đó là một
`<span>` **rỗng, rộng 0 px**, không text node để neo caret ⇒ `execCommand('insertText')` trả
`false`. `test:e2e` = **1 xanh / 1 đỏ**, đỏ ở `e2e/specs/editor-typing-flush.e2e.mjs:133`.

🔴 **Vì sao ca đó chạm thẳng vào AC1 của story này:** AC1 đòi *"một phiên gõ liên tục ít nhất 30
phút trên một Chương thật"*. Một Chương thật vừa nhập có **mọi** câu chưa dịch. Nếu ca đó còn hỏng
với tay người, phiên 30 phút **không bắt đầu được**.

⚠️ **Nhưng đừng đọc lượt đỏ đó thành *"tính năng hỏng"*** — `deferred-work.md:2290-2297` đã đo và
đã ghi rằng **bộ đo tự bóp méo vế này**: một lượt đo đọc `document.hasFocus() === false`, tức cửa
sổ không được hệ điều hành focus — trạng thái một người dùng thật không bao giờ ở trong. Và
`browser.keys()` của WebdriverIO **không gõ được chữ** (chỉ `keydown`, không `beforeinput`) — một
**giới hạn của bộ đo**, đã ghi tên, đừng chẩn đoán lại.

⇒ **Việc của story này, nói thẳng:** Task 1.0 chạy **một lượt kiểm đầu vào bằng tay** — mở app,
bấm vào một câu chưa dịch, gõ một chữ. Kết quả quyết định đường đi:

| Kết quả | Đường đi |
| --- | --- |
| **gõ được** | Ca e2e đỏ là một **giới hạn bộ đo** đã có tên. Ghi số vào §Debug Log, **đi tiếp**, và nói rõ trong báo cáo rằng phép đo NFR2/NFR18 đứng trên một bề mặt mà **e2e chưa lái được** |
| **không gõ được** | 🔴 **DỪNG. Không đo.** Đây là điều kiện khởi hành hỏng, không phải một số đo xấu. Báo cho Ice kèm nguyên văn, và story này quay về `backlog` cho tới khi 2.3 đóng. Đo hiệu năng của một bề mặt chưa gõ được là sản xuất một bảng số vô nghĩa |

### 3. Sáu con số `Tuning` — chủ sở hữu là story này, và chỗ khai là MỘT

`src-tauri/src/core/store/mod.rs:183-243`. Doc-comment của module (`:62-68`) nói thẳng: *"Không con
số nào ở `Tuning::default` được đo."*

| Trường | Giá trị tạm | Ràng buộc đã ghi tại chỗ |
| --- | --- | --- |
| `pool_size` | **4** | lớn hơn **không** nhanh hơn — một reader giữ ảnh chụp cũ làm `log > checkpointed`, tệp WAL vẫn lớn tiếp (Bẫy 8) |
| `busy_timeout` | **5 000 ms** | trạng thái **của từng kết nối**, không của database — writer, mỗi kết nối pool, và luồng checkpoint đều phải tự đặt |
| `checkpoint_tick` | **1 s** | độ phân giải của **cả hai** điều kiện kích hoạt ⇒ nó là **sàn** của mọi số đo về nhịp checkpoint |
| `idle_before_passive` | **5 s** | 🔴 *"**cố ý dài hơn** nhịp flush 2 s của AD-35"* (`:207-208`) — **con số nhạy nhất của cả sáu với cặp NFR2/NFR18** |
| `wal_threshold_bytes` | **4 MiB** | lấy đúng số autocheckpoint mặc định SQLite mà AD-12 vừa tắt *(1000 trang × 4096 B)* — tức **không đổi hành vi theo một hướng chưa ai đo** |
| `close_truncate_budget` | **2 s** | trần này canh `check-scope.mjs`/`check-scope-bundled.mjs` — một `close()` chờ hết `busy_timeout` làm **hai cổng của Story 1.2/1.3 đỏ vì tầng ghi dữ liệu** |

⚠️ `struct` + `Default` tồn tại **chính là** để story này hiệu chỉnh bằng **một** lượt sửa thay vì
một lượt đi săn (`:175-181`). Đừng chôn số trần vào `writer.rs`/`checkpoint.rs`.

### 4. Ba hằng nhịp flush — cũng là số của story này, và chúng khoá lẫn nhau

`src/panels/editorFlush.ts`: `EDITOR_IDLE_MS = 2000` (`:43`) · `EDITOR_HARD_CAP_MS = 5000` (`:56`) ·
`EDITOR_RETRY_FLOOR_MS = 2000` (`:78`). Cả ba doc-comment đã khai **TẠM — chủ là Story 2.4**.

🔴 **Ba ràng buộc chéo, cả ba đo được, và đổi một số mà quên một ràng buộc là hỏng im lặng:**

1. `Tuning::idle_before_passive = 5 s` được đặt *"cố ý dài hơn nhịp flush 2 s"*. **Nâng
   `EDITOR_IDLE_MS` lên ≥ 5 000 là làm luồng checkpoint đánh nhau với đường gõ.**
2. `EDITOR_HARD_CAP_MS = 5000` là con số **NFR18 đứng trên**. Nới nó là nới thẳng cửa sổ mất dữ
   liệu — và đó là **thay đổi tầng PRD** (AC5), không phải một lượt hiệu chỉnh.
3. `src/layout/writeSchedule.ts` khai cặp **`IDLE_MS = 500` / `HARD_CAP_MS = 5000`** (`:57`, `:67`)
   cho **bố cục**, và `check-layout.mjs` Kiểm B đứng trên hành vi của `createWriteSchedule`.
   ⚠️ **Đừng đụng cặp của bố cục** — nó thuộc Story 1.14, và bảng ở `writeSchedule.ts:32-33` đã ghi
   rõ hai chỗ dùng, hai cặp số, một hàm.

### 5. 🔴 NFR2 ĐÃ CÓ MỘT CHỖ VƯỢT TRẦN, ĐÃ ĐO, VÀ SỐ ĐÓ ĐƯỢC GIAO CHO STORY NÀY

Story 2.2 đo trên bàn đo hai engine, Chương lớn nhất **có thật** — **9.850 câu / 48.640 ký tự**
(`deferred-work.md:2076-2090`):

| | dựng DOM + bố cục | đo + vẽ **1** vạch *(ca thật)* | đo + vẽ **9.850** vạch *(ca trần)* |
|---|---|---|---|
| Blink (HeadlessChrome 151) | **300,1 ms** | 8,5 ms | 63,1 ms |
| WebKit (605.1.15 / Safari 26) | **1.308,0 ms** | 5,0 ms | 64,0 ms |

⇒ **6× trần NFR2 trên Blink, 26× trên WebKit.** Cơ chế đo vạch lề **không** phải chỗ đắt (5–9 ms ở
ca thật); chỗ đắt là **dựng 9.850 phần tử DOM**, tức đúng hàng Deferred *"ảo hoá danh sách dài"*.

⚠️ **Và số trên là số của HÔM QUA, khi mọi `target_text` còn RỖNG.** Chữ thật sẽ làm nó tăng.
Story này đo lại **với chữ thật trong Chương**.

⚠️ **Một lý lẽ đã có sẵn và nó phải được cân, không bỏ qua:** lượt dựng là **một lần mỗi Chương**,
không phải đường nóng NFR1 — cùng hạng với trần render kiểu song song ở Story 1.16, nơi Ice đã chốt
**1,4 s** là *"còn chấp nhận được"* cho một thao tác chạy một lần. 1,3 s nằm ngay dưới mốc đó.
⇒ Câu hỏi thật của AC1 **không** phải *"có frame nào vượt 50 ms trong cả vòng đời app không"* mà là
*"**trong lúc auto-save chạy** có frame nào vượt 50 ms không"* — nguyên văn NFR2. Lượt dựng Chương
**không nằm trong lúc auto-save chạy**. Xem Quyết định #3, và **đừng tự nới định nghĩa theo hướng
dễ hơn mà không ghi ra**.

### 6. BA ĐƯỜNG NÓNG chưa ai đo, cả ba có chủ là story này

Ba món này ra đời trong lượt vá 2026-08-13 của 2.3 và **chưa từng đi qua bàn đo hai engine**:

| Đường | Chỗ | Vì sao nó đắt |
| --- | --- | --- |
| `:data-caret` | `EditorPanel.vue:892` | `editorCaretSegmentId` là ref phản ứng đọc trong hàm render ⇒ **mỗi lượt `selectionchange`** chạy lại `v-for` trên tới 9.850 `<span>`. `selectionchange` bắn **liên tục** trong lúc kéo chọn. ⚠️ Đổi sang tra `Map` **KHÔNG** vá được — chi phí ở lượt duyệt danh sách của Vue |
| `restoreEditedText()` | `EditorPanel.vue:294`, `:300` | `querySelectorAll('[data-segment-id]')` trên **cả Chương** mỗi lượt dựng lại trang, trong khi `editedText` thường chỉ mang vài mục. O(cả Chương) thay cho O(số câu đang gõ dở) |
| `nearestSentenceTo()` | `EditorPanel.vue:565`, gọi ở `:602`, `:636` | mỗi cú bấm rơi vào khoảng trống `.doc` duyệt **từng** câu và gọi `getClientRects()`/`getBoundingClientRect()`. 🔴 Đây là **chỗ duy nhất** của cả lượt 2.3 không kèm số đo |

### 7. Đường checkpoint: trong một phiên gõ liên tục, CHỈ vế ngưỡng chạy được

Đọc thẳng từ `core/store/checkpoint.rs:286-315`:

```rust
let over_threshold = wal_len(&path)? > tuning.wal_threshold_bytes;
let idle = shared.dirty.load(…) && shared.since_last_write() >= tuning.idle_before_passive;
if !over_threshold && !idle { continue; }
```

🔴 **Hệ quả, và nó là trung tâm của AC3:** người dùng gõ liên tục ⇒ flush mỗi 2 s ⇒
`since_last_write()` **không bao giờ** chạm 5 s ⇒ **vế (a) rảnh không bao giờ chạy**. Trong suốt
một phiên 30 phút, thứ **duy nhất** giữ `.db-wal` khỏi phình là **vế (b) ngưỡng**. Đó đúng là câu
AD-12 đã viết: *"Phải có ngưỡng kích thước WAL buộc checkpoint để `.db-wal` không phình vô hạn khi
gõ liên tục hàng giờ."*

⇒ **Cặp đánh đổi thật của AC3 là `wal_threshold_bytes` ⟷ NFR2**, không phải ⟷ NFR18. Xem mục 8.

🔵 **Và bộ đếm đã có sẵn, không phải dựng:** `CheckpointStats` mang `idle_triggered` và
`threshold_triggered` (`checkpoint.rs:58-70`, `:163-170`), đọc qua `Store::checkpoint_stats()`
(`mod.rs:677`). ⚠️ **Nhưng nó chưa có đường ra IPC nào** — xem Quyết định #5 trước khi dựng một
đường mới.

### 8. 🔴 NFR18 KHÔNG treo trên ngưỡng WAL — và hiểu sai chỗ này là đi săn một mối ghép không tồn tại

`PRAGMA synchronous` đọc lại trên kết nối ghi = **2 (FULL)**, có lưới bằng máy
(`store_contract.rs::the_write_connection_fsyncs_the_wal_on_every_commit`, AC5 của Story 2.3). Ở
`FULL` + WAL, **mỗi lượt commit fsync WAL** ⇒ một flush đã trả `Ok` là **đã bền trên đĩa**, dù
checkpoint có chạy hay không.

⇒ Phân rã đúng, và nó phải được ghi vào báo cáo chứ không chỉ được biết:

| Ngưỡng | Bị chi phối bởi | KHÔNG bị chi phối bởi |
| --- | --- | --- |
| **NFR18** *(mất ≤ 5 s)* | `EDITOR_HARD_CAP_MS` — và **chỉ** nó, cộng độ trễ đường flush | `wal_threshold_bytes` *(dữ liệu đã ở WAL là đã bền)* |
| **NFR2** *(không frame > 50 ms)* | `wal_threshold_bytes` · `checkpoint_tick` · nhịp flush · ba đường nóng mục 6 | — |

⚠️ Đây là một **giả thuyết đọc từ mã, chưa phải một phép đo** — và luật của kho là *"đặt rồi ĐỌC
LẠI"*. AC8 buộc **đo** nó: kill cưỡng bức ở **hai** giá trị ngưỡng WAL cách xa nhau và cho thấy cửa
sổ mất dữ liệu **không đổi**. Nếu nó đổi, giả thuyết trên **sai** và đó là phát hiện lớn nhất của
story — báo, đừng vá.

### 9. 🔴 BẢN BUILD ĐƯỢC ĐO QUYẾT ĐỊNH SỐ ĐO CÓ NGHĨA HAY KHÔNG — và ở đây có một mâu thuẫn thật

Hai móc chuyển hướng dữ liệu thật (`AURATRANSLATE_E2E_DATA_DIR`, `AURATRANSLATE_E2E_LIBRARY_ROOT`)
bị gác bằng **đúng hai lớp** của AD-45: `#[cfg(all(debug_assertions, feature = "wdio"))]`
(`src-tauri/src/lib.rs:71-72`, `:94-96`). **Bản phát hành không có một dòng mã nào đọc chúng.**

Và `[profile.release]` của kho (`src-tauri/Cargo.toml`) là:

```toml
codegen-units = 1
lto = true
opt-level = "s"     # ⚠️ tối ưu KÍCH THƯỚC, không phải tốc độ
panic = "abort"
strip = true
```

⇒ Mâu thuẫn, nói thẳng:

- Đo trên bản **debug** ⇒ móc chuyển hướng chạy, dữ liệu thật an toàn — nhưng Rust debug trên đường
  ghi chậm hơn release **nhiều bậc**, và số NFR2/NFR18 đọc ra **không phải số của sản phẩm**.
- Đo trên bản **release như đang phát hành** ⇒ số đúng — nhưng app ghi vào `$APPDATA` **thật** và
  `~/Documents/AuraTranslate/` **thật** của Ice. `default_library_root` (`commands/project.rs:61-80`)
  **không** cho người dùng chọn thư mục; móc e2e đứng trước nó **chỉ trong bản debug+wdio**.

🔴 Và đây là chỗ story này khác mọi story trước: nó **cố ý giết tiến trình ≥ 20 lần giữa lúc đang
ghi**. Chạy việc đó lên dữ liệu thật của Ice là tái lập đúng lớp lỗi mà Story 1.22 đã tốn hai bề
mặt để đóng (`wdio.conf.mjs` §Giới hạn 1). **Quyết định #1 phải trả lời câu này trước phép đo đầu
tiên.**

⚠️ `opt-level = "s"` là một dữ kiện, không một lỗi: số phải đo trên **profile sẽ phát hành**, nên
số đọc ra là số của sản phẩm chứ không phải một ca tốt nhất. Nếu NFR2 trượt **chỉ vì** `"s"`, đó là
một đòn bẩy có thật để đưa vào §Cần Ice quyết — **không** phải một thứ dev tự đổi.

### 10. Bốn đường nghiệm thu vẫn đứng — nhưng một mũi thăm dò nghiệm thu bằng thứ khác

Bốn đường của `2-3`§8 *(vitest · cổng tĩnh · bàn đo · e2e)* giữ nguyên vai, và **AC25 của 2.3 —
một mệnh đề, một đường — vẫn áp**.

🔵 Nhưng Story 1.1 đã đặt tiền lệ cho hạng story này: *"Story này không sinh mã sản phẩm nên **không
có unit test**. Thứ thay thế cho test ở một mũi thăm dò là **tính lặp lại được**"* — lệnh chính xác,
phiên bản toolchain, checksum/tên tệp, ảnh chụp. Áp đúng chuẩn đó (AC6).

⚠️ **Hệ quả hai chiều:** story này **không** phải viết test cho các phép đo, **nhưng** mọi lượt đổi
hằng đều đi qua mã đang có lưới. `npm run test` **32/32** và `cargo test` **319/0/5** phải xanh lại
sau khi đổi số — xem AC15.

### 11. NFR15: cửa rà giấy phép VẪN ĐỨNG, và story này là chỗ nó dễ bị đọc sai nhất

Ice lật NFR15 vế *"không bộ chạy test frontend"* ngày 2026-08-12 và cấp phép cho **đúng ba gói được
gọi tên** — `vitest@4.1.10` · `@vue/test-utils@2.4.11` · `happy-dom@20.11.2`. Lượt lật đó **không**
cấp phép cho một thư viện **editor** (AC19 của 2.3, ký cùng ngày).

🔴 AC4 của story này là chỗ câu hỏi *"thêm một thư viện editor?"* được trả lời chính thức. Nếu câu
trả lời là **có**, gói đó là **gói thứ tư** và nó phải đi qua **trọn** cửa NFR15 **trước khi thêm**:
mở tệp giấy phép **trong nguồn đã tải** (`node_modules/`), ghi đường dẫn + dòng đầu, rồi vào **bảng
Stack** của `ARCHITECTURE-SPINE.md`. Khuôn đã có: §NFR15 của Story 2.3 *(và bài học ở đó — `LICENSE.md`
của `vitest` dài **811 dòng** và khai giấy phép của **27 gói vendor**, trong khi trường `license`
chỉ nói `"MIT"`)*.

⚠️ Cửa này **chưa có cổng máy nào** — `deferred-work.md` ghi đích danh rằng gói thứ tư sẽ gặp lại
đúng cửa này và *"lúc đó trí nhớ người là thứ duy nhất canh"*. **Chủ của việc có biến nó thành cổng
máy hay không là Ice**, không phải story này.

### 12. Dev KHÔNG sửa tài liệu quy hoạch — trừ hai ngoại lệ mà AC cấp tường minh

Tiền lệ giữ qua toàn Epic 1 và Epic 2: `epics.md`, `DESIGN.md`, `EXPERIENCE.md`, `prd.md` là lượt
riêng của Ice.

🔵 **Hai ngoại lệ, cả hai do AC của chính story này cấp, và cả hai có tiền lệ ở Story 1.1 Task 7:**
1. **Bảng Deferred** của `ARCHITECTURE-SPINE.md` — AC3 nguyên văn đòi *"giá trị đó ghi vào hàng
   Deferred tương ứng, đánh dấu đã đóng"*. Dùng **đúng khuôn** gạch ngang + `✅ Đã đóng <ngày>` mà
   hàng HVTĐTD và hàng FR115 đã dùng.
2. **Bảng Stack** — chỉ khi AC4 chốt thêm một thư viện. Bảng Stack là tài liệu Dev **được** đồng bộ
   (action item Epic 1, `sprint-status.yaml` §action_items).

⚠️ **Đừng chạm một AD nào.** `lint_spine.py` phải trả **0 findings** sau lượt sửa
(`.claude/skills/bmad-architecture/scripts/lint_spine.py`).

⚠️ **Số dòng của bảng Deferred đã TRÔI** kể từ story 2.3: hàng *"ngưỡng WAL"* nay ở **`:894`**
(2.3 ghi `:883`), *"thư viện editor"* ở **`:897`** (ghi `:886`), *"ảo hoá"* ở **`:899`** (ghi `:888`)
— bảng Stack lớn thêm ba hàng ở lượt 2.3. **Đọc lại số dòng trước khi trích, đừng chép từ story cũ.**

### 13. Mọi bằng chứng chỉ trên macOS

Ice chốt 2026-08-12: trọn phần Windows dời về **cuối dự án**. Story này **làm dày khoảng mù đó
nhiều hơn bất kỳ story nào trước**: NFR2 và NFR18 là hai ngưỡng **phi chức năng** và chúng phụ
thuộc thẳng vào engine webview *(WKWebView vs WebView2)*, vào bộ lập lịch của hệ điều hành, và vào
đường `fsync` của hệ tệp *(APFS vs NTFS)*.

⇒ AC20 đòi một dòng trong báo cáo, viết bằng chữ, rằng **cả hai ngưỡng chỉ được nghiệm thu trên
macOS** và số Windows là **một khoảng trống đã biết**, không phải một suy diễn.

---

## Story

As a chủ dự án,
I want biết chắc nhịp auto-save đạt được cả hai ngưỡng cùng lúc,
so that tôi không phát hiện ra chúng xung khắc sau khi đã xây tám story lên trên.

---

## Acceptance Criteria

Nguyên văn từ `epics.md:2126-2149`, đánh số để tham chiếu:

**AC1** — **Given** một phiên gõ liên tục ít nhất 30 phút trên một Chương thật · **When** đo ·
**Then** **không frame nào vượt 50 ms** trong lúc auto-save chạy (NFR2)

> 🔴 **AC1 chạy trên một THANG NHÂN TẠO, không phải một Chương thật — Ice chốt 2026-08-13.**
> Lý do đo được, không phải quên: thư viện thật của Ice **không có Chương nào cỡ thường nhật**.
> Khoảng **669 → 48.639 ký tự rỗng hoàn toàn**; chỉ có fixture nhỏ và hai tài liệu ngoại lệ *(một
> bảng tra từ điển, một văn bản thương hiệu)*. Chuyện này chỉ lộ ra **giữa lúc thi hành** — xem
> §Dev Agent Record Task 2 — tức khâu thẩm định khả thi trước khi mở story đã hụt.
>
> **Hệ quả bắt buộc, không được làm nhẹ đi:** báo cáo **cấm** viết *"đã đo trên một Chương thật"*.
> Nó phải ghi bằng một dòng rằng thư viện không có Chương cỡ đó, và số của AC1 đọc ra từ một
> **khoảng đo tổng hợp** cắt từ một tài liệu duy nhất. Không có dòng đó, số này sẽ được đọc lại
> sau như bằng chứng cho một thứ chưa từng được đo. **Cỡ mẫu phiên: n = 3** *(xem AC19)*.

**AC2** — **Given** ứng dụng bị kill cưỡng bức ở nhiều thời điểm ngẫu nhiên trong lúc gõ · **When**
mở lại · **Then** công việc mất **tối đa 5 giây** (NFR18) · **And** phép đo lặp lại ít nhất 20 lần
với kết quả nhất quán

> 📏 **"Nhất quán" được gán số, vì không có số thì nó không nghiệm thu được.** Nghiệm thu:
> ① **≥ 20 lượt kill HỢP LỆ** *(không phải 20 lượt bắn — xem AC9)*, **tại mỗi điểm lưới** của
> Task 5 *(xem AC8)*; ② báo cáo ghi **max của từng lượt chạy riêng**, 🔴 **CẤM gộp mẫu** — NFR18
> cũng như NFR2 là mệnh đề về **đuôi phân bố**, và gộp mẫu là đúng cơ chế nuốt mất một lượt xấu;
> ③ **dung sai**: nếu `max − trung vị > 2 s` **hoặc** có bất kỳ lượt nào > 5 s, phép đo bị gắn cờ
> **BẤT ỔN** và không được khai là "nhất quán" — kể cả khi mọi lượt đều ≤ 5 s.

**AC3** — **Given** ngưỡng kích thước WAL buộc checkpoint · **When** dò · **Then** chọn được một
giá trị đạt **cả hai** ngưỡng trên · **And** giá trị đó ghi vào hàng Deferred tương ứng của
`ARCHITECTURE-SPINE.md`, đánh dấu đã đóng

> 🔴 **Dòng đóng `:894` PHẢI khai phạm vi nó thật sự đóng.** Hàng Deferred gốc đóng khung bài toán
> là `wal_threshold_bytes` ⟷ nhịp flush **đánh đổi lẫn nhau** để đạt **cả** NFR18 **và** NFR2.
> Nếu số đo cho thấy NFR18 **không** treo trên `wal_threshold_bytes` *(§Điều kiện khởi hành mục
> 7-8)*, thì cặp đánh đổi thật hẹp hơn cặp gốc — và khuôn `✅ Đã đóng` sẽ đóng một **câu hỏi hẹp
> hơn câu hỏi đã đặt**. Nghiệm thu: dòng đóng ghi rõ **một câu** nói cặp nào thật sự được dò và
> cặp nào hoá ra không tồn tại, kèm số. Không có câu đó, người đọc SPINE sau này tin nhầm rằng
> trade-off gốc *(WAL ⟷ NFR18)* đã được giải triệt để.

**AC4** — **Given** thư viện editor cho Panel Editor · **When** chọn · **Then** lựa chọn được ghi
lại kèm lý do · **And** nó tuân hợp đồng trạng thái AD-31 nên không lan ra ngoài module

**AC5** — **Given** hai ngưỡng NFR2 và NFR18 không đạt được đồng thời · **When** xảy ra · **Then**
kết quả được báo cáo là **thay đổi tầng PRD cần chủ dự án quyết**, không phải một tối ưu kỹ thuật

> 🔴 **"Không đạt được đồng thời" đọc theo nghĩa HẸP — Ice chốt 2026-08-13.** Cụm này đọc được hai
> kiểu và hai kiểu cho hành vi trái ngược, nên nó được ghim:
>
> | Ca | Định nghĩa | Đường đi |
> | --- | --- | --- |
> | **AC5 đúng nghĩa** | **loại trừ nhau** — dò hết lưới Task 5 mà **không tồn tại** giá trị nào thoả cả hai | Task 10: dừng, báo Ice, Epic 2 dừng theo |
> | **Một ngưỡng trượt một mình** | tồn tại giá trị thoả ngưỡng này nhưng ngưỡng kia đỏ ở **mọi** điểm lưới | 🔵 **Nhánh mới, xem dưới** |
> | Chưa dò hết lưới | — | **chưa** được báo Ice; chạy nốt lưới trước |
>
> 🔵 **Nhánh một-ngưỡng-trượt:** phải dò **hết lưới đã ghim của Task 5** *(biên và bước nhảy là số
> cụ thể, không phải chữ "hết lưới")* trước đã. Hết lưới mà vẫn đỏ ⇒ **cũng** báo Ice theo khuôn
> Task 10, nhưng báo cáo phải nói rõ đây là ca **một** ngưỡng, kèm giá trị tốt nhất tìm được và
> khoảng cách còn lại tới ngưỡng. Chưa hết lưới ⇒ **không** được báo, và **không** được tự tối ưu
> ngoài phạm vi hằng số đã cho phép.

### AC bổ sung — dẫn xuất từ kiến trúc, từ nợ có chủ, và từ đo đạc mã nguồn

Năm AC trên nói *cái gì phải đúng*, không nói *phép đo phải trung thực thế nào*. Mười bảy AC dưới
đây **cùng hạng ràng buộc**, mỗi cái neo vào một nguồn kiểm chứng được.

**AC6 — báo cáo theo ĐÚNG khuôn mũi thăm dò đã có, ở ĐÚNG chỗ đã có.** Tệp:
`_bmad-output/planning-artifacts/research/editor-perf-spike-results-2026-08-XX.md` — **cùng thư
mục** với `phase-0-spike-results-2026-08-02.md` và `font-spike-results-2026-08-03.md`, **không** tạo
thư mục mới. Bảy mục bắt buộc, đúng khuôn Giai đoạn 0: ① frontmatter `title`/`status`/`created`/
`updated`/`relates_to`; ② **bảng tóm tắt ngay đầu** có dấu 🟢/🔴 cho từng phép đo — kết luận trước
số; ③ **môi trường đo** *(macOS, phiên bản Rust/Node/Tauri/WebKit, profile build, model máy)*;
④ từng phép đo kèm **lệnh chính xác** đã chạy; ⑤ **phỏng đoán bị bác** — không chỉ kết quả;
⑥ *"Việc chưa làm được ở giai đoạn này"*; ⑦ *"Cần Ice quyết"* tách riêng, có bảng **Được / Mất**.

**AC7 — SÁU số `Tuning` đều có một phán quyết, và không số nào đi qua bằng im lặng.** Mỗi trường ở
`core/store/mod.rs:183-243` nhận **đúng một** trong ba nhãn: **đã đo** *(kèm số và lệnh)* · **giữ
nguyên có lý lẽ đo được** *(kèm phép đo cho thấy đổi nó không đổi kết quả)* · **không đo được ở
story này** *(kèm lý do và **một chủ mới có tên**)*. 🔴 Nhãn thứ ba dùng quá **hai** lần là dấu hiệu
mũi thăm dò chưa chạy đủ — nói ra thay vì gom lại. Bảng sáu hàng vào báo cáo **và** vào §Completion
Notes.

> ⚖️ **Vượt ngưỡng hai thì có HỆ QUẢ, không chỉ có lời khai.** Nhãn ba dùng **≥ 3** lần ⇒ **DỪNG
> trước Task 8**, không viết báo cáo đóng, và báo Ice bằng một mục *"Cần Ice quyết"* nêu: số nào
> không đo được, vì sao, và cần gì để đo được. Lý do: mũi thăm dò này tồn tại để **thay im lặng
> bằng số**; giao ra bốn-năm-sáu hàng "không đo được" là giao lại đúng cái im lặng đã có, chỉ khác
> là lần này nó có chữ ký. Ice quyết chạy tiếp hay trả story về `backlog`.

**AC8 — mối ghép NFR18 ⟷ ngưỡng WAL được ĐO TẠI MỌI ĐIỂM LƯỚI, và giả thuyết bị BỎ.**

> 🔴 **Ice chốt 2026-08-13: bỏ giả thuyết, đo rộng.** Bản cũ của AC8 đo ở **hai** đầu *(512 KiB và
> 16 MiB)* để nghiệm một giả thuyết — rằng `synchronous = FULL` làm cửa sổ mất dữ liệu độc lập với
> `wal_threshold_bytes`. Hai vấn đề đã hạ bản đó: ① giả thuyết bị bác thì **AC3 không có đường
> về** — nó vẫn đòi "chọn được một giá trị" mà không còn tiêu chí nào; ② mô hình nhân quả của
> §mục 8 **bỏ sót một kênh có thật** *(xem dưới)*, nên xác suất bác không hề nhỏ. Bỏ giả thuyết
> thì không cần đường lui.

Nghiệm thu: bộ kill của AC2 chạy **tại mọi điểm** của lưới `wal_threshold_bytes` đã ghim ở Task 5
*(≥ 20 lượt kill hợp lệ mỗi điểm — xem AC2 và AC9)*, và cửa sổ mất dữ liệu **max của từng điểm**
được đặt cạnh nhau thành một bảng theo ngưỡng. Bảng đó **là** câu trả lời cho AC3: nó cho thấy
trực tiếp cặp đánh đổi thật, không cần suy từ một giả thuyết. Lưới phẳng ⇒ NFR18 không treo trên
ngưỡng WAL, ghi số và ghi kết luận đó vào dòng đóng `:894` *(AC3)*. Lưới dốc ⇒ cặp đánh đổi gốc
có thật, chọn giá trị từ chính bảng.

⚠️ **Kênh mà §mục 8 bỏ sót, và AC8 phải đo cho ra:** lập luận *"flush trả `Ok` là đã bền trên đĩa
nên `wal_threshold_bytes` không ảnh hưởng NFR18"* chỉ xét độ bền của dữ liệu **đã commit**. Nó
không xét đường vòng: ngưỡng nhỏ ⇒ checkpoint chạy dày ⇒ tranh `busy_timeout` với writer ⇒ **trễ
thời điểm commit KẾ TIẾP** ⇒ cửa sổ mất dữ liệu dài ra. Đo *"mất bao nhiêu giây"* không tách được
kênh này. Nghiệm thu bổ sung: mỗi lượt flush ghi **thời gian chờ khoá** riêng khỏi thời gian ghi;
một lưới dốc mà thời gian chờ khoá phẳng nói nguyên nhân khác hẳn một lưới dốc mà nó cũng dốc.

**AC9 — kill là kill CƯỠNG BỨC, và điều đó phải kiểm được từ số đo.** `SIGKILL` *(`kill -9`)*, không
đóng cửa sổ, không `Cmd+Q`. Lý do: đường thoát bình thường chạy `wire_exit_flush` (`lib.rs:350`) rồi
`close_open_work` → `Store::close()` → **TRUNCATE**; đo đường đó là đo một thứ khác hẳn và nó sẽ cho
một bảng xanh sai. Nghiệm thu: sau mỗi lượt kill, `.db-wal` **còn nội dung** *(kích thước > 0)* —
đó là dấu vân tay của một lượt thoát **không** đi qua TRUNCATE.

> 🔴 **`.db-wal` = 0 byte KHÔNG đồng nghĩa "kill trượt", và gộp hai thứ đó lại sẽ bẻ cong phân bố.**
> Bản cũ vứt thẳng mọi lượt 0 byte khỏi mẫu. Nhưng có **hai** nguyên nhân khác hẳn nhau cho cùng
> một dấu hiệu:
>
> | Nguyên nhân | Nghĩa thật | Xử lý |
> | --- | --- | --- |
> | Kill **trượt** — tiến trình đi qua đường thoát bình thường *(TRUNCATE)* | phép đo **hỏng** | bỏ khỏi mẫu, ghi ra |
> | Kill **trúng lúc app rảnh** — đã checkpoint xong, không còn gì chờ ghi | **THÀNH CÔNG**: mất **0 giây** công việc | 🔵 **giữ trong mẫu**, ghi là 0 s |
>
> Vứt cả hai là vứt đúng nhóm kết quả **tốt nhất**, đẩy phân bố đo được lệch về phía **xấu hơn
> thực tế** — một bảng đỏ nói sai nguyên nhân, đúng lớp lỗi Story 1.22 đã ghi tên hai lần.
>
> **Phân biệt bằng dấu hiệu khác kích thước WAL**, không bằng suy đoán: ① tiến trình có ghi mã
> thoát của `wire_exit_flush` vào nhật ký không *(đi qua = trượt)*; ② dòng gõ mang chỉ số đơn điệu
> tăng của Quyết định #4 — ký tự cuối **đã** nằm trong kho ⇒ trúng lúc rảnh, **chưa** nằm mà WAL
> vẫn rỗng ⇒ trượt. Không phân biệt được ⇒ bỏ mẫu **và ghi ra là không phân biệt được**.
>
> 📏 **Sàn đếm đúng cái cần đếm:** Task 4 phải bắn cho tới khi có **≥ 20 lượt HỢP LỆ**, không phải
> bắn đúng 20 lượt rồi thôi. Bắn 20 mà 8 lượt trượt thì còn 12 mẫu — dưới sàn AC2, dù Task 4 nhìn
> như đã xong. Ghi ra **cả hai** số: số lượt đã bắn và số mẫu hợp lệ.

⚠️ `panic = "abort"` nghĩa là kho **không** đi qua `close_global_store` ở một lần thoát cứng — món
nợ kế thừa đã ghi ở AC17 của Story 2.3. Story này **kế thừa** nó chứ không đóng; nói ra trong báo
cáo, đừng đánh dấu đạt.

**AC10 — phép đo chạy trong ĐÚNG kịch bản HAI KHO.** `deferred-work.md:570` ghi đích danh: một phiên
mở một Tác phẩm chạy **hai** kho — `global.db` + `project.db` — mỗi kho tự mang **một luồng
checkpoint + pool 4 kết nối**. Chưa có phép đo nào về tranh chấp CPU/I/O giữa hai luồng checkpoint
chạy song song trong cùng tiến trình (`:234`).

> 🔴 **Bản cũ của AC10 đóng `:234` + `:570` bằng CHỮ, không bằng nội dung — Ice chốt đo thật
> 2026-08-13.** Nó chép nguyên câu *"chưa có phép đo nào về tranh chấp CPU/I-O"* vào vế **Given**,
> rồi vế **Nghiệm thu** chỉ đòi *"ghi kích thước `.db-wal` của cả hai kho"*. Đường cong kích thước
> WAL là bằng chứng hai luồng checkpoint **có chạy** — nó **không** đo tranh chấp. Rà cả Task 4 và
> Task 5 của bản cũ: không bullet nào đo CPU hay I-O. Hai hàng nợ sẽ bị gạch trong khi câu hỏi
> chúng đặt ra chưa ai trả lời.

Nghiệm thu — **ba** vế, thiếu một vế là chưa đóng được `:234`:

1. **Ghi kích thước `.db-wal` của CẢ HAI** kho theo thời gian, không chỉ `project.db`. *(Vế này
   đóng `:570` — một phiên chạy hai kho, mỗi kho một luồng + pool 4.)*
2. **Đo tranh chấp thật:** thời gian CPU của **từng** luồng checkpoint và **độ trễ I-O** của từng
   lượt, chạy ở **hai chế độ** — hai kho **chạy chồng** so với **từng kho chạy riêng**. Chênh lệch
   giữa hai chế độ **là** con số mà `:234` hỏi. *(Vế này đóng `:234`.)*
3. **Ép trùng pha, đừng chờ may:** một kịch bản buộc hai luồng checkpoint kích hoạt **cùng thời
   điểm** *(nạp cả hai kho tới ngưỡng rồi thả cùng lúc)*. Không có vế này thì hai kho lệch pha tự
   nhiên suốt phiên vẫn cho AC10 xanh theo nghĩa đen mà chưa từng đo được tranh chấp lần nào.

⚠️ Vòng đo của vế 2 là **nguồn chi phí bàn đo thứ tư** — nó phải vào AC21 cùng ba nguồn kia, nếu
không chính phép đo tranh chấp sẽ làm nhiễu số NFR2 đang đo trên cùng cái máy.

**AC11 — "frame" và "mất bao nhiêu giây" đều có ĐỊNH NGHĨA ĐO ĐƯỢC, viết ra trước khi đo.** Hai
mệnh đề của AC1 và AC2 không tự mang đơn vị. Báo cáo phải ghi: phương pháp lấy mẫu frame *(và cách
loại các khoảng app **không** vẽ — một cửa sổ bị che hay một khoảng nghỉ cho `requestAnimationFrame`
một delta khổng lồ **không phải** một frame rớt)*; mốc 0 của phép trừ NFR18; và cách một ký tự đã gõ
được **truy ngược** về thời điểm gõ ra nó. Xem Quyết định #3 và #4.

> 🔴 **Biên 100% của bộ lọc phải có phán quyết, vì "không có số" khác "đạt".** Quyết định #3 đòi
> ghi số mẫu bị loại và cảnh báo *"lọc 90% là phép đo hỏng"* — nhưng bỏ trống đúng cái biên tệ
> nhất. Ghim ba mức:
>
> | Tỉ lệ mẫu bị loại trong cửa sổ auto-save | Phán quyết AC1 |
> | --- | --- |
> | ≤ 50 % | bình thường — ghi tỉ lệ, đọc số |
> | > 50 % và < 100 % | 🟡 **phép đo NGHI NGỜ** — ghi số kèm cờ, **cấm** khai 🟢 |
> | **= 100 %** *(0 delta hợp lệ)* | 🔴 **VÔ NGHĨA** — AC1 **không** xanh và **không** đỏ; ghi *"chưa đo được"*, đây là hỏng **bàn đo**, đi theo luật dừng của Task 0 |
>
> Ca 100% có tiền lệ trong chính story này: *"Lượt hỏng ③"* ở §Dev Agent Record. Không có ba mức
> trên, đường ra duy nhất là đo lại vô hạn.

**AC12 — ba đường nóng của §Điều kiện khởi hành mục 6 đều có SỐ.** `:data-caret` · `restoreEditedText()`
· `nearestSentenceTo()`. Mỗi cái: chi phí một lượt, ở Chương **thật** và ở Chương **9.850 câu**.
🔴 **Vá chỉ khi số nói cần**, và một bản vá ở đây phải kèm **đỏ-rồi-xanh** cộng một dòng nói vì sao
nó **không** phải thứ mà hàng Deferred *"ảo hoá danh sách dài"* sẽ làm lại từ đầu ở Giai đoạn 3 —
`deferred-work.md:2441` nói thẳng rằng vá lẻ `restoreEditedText()` là *"tối ưu một hằng số trước khi
biết bậc độ lớn có đổi không"*.

**AC13 — trần 9.850 span được đo lại VỚI CHỮ THẬT, và story này KHÔNG dựng ảo hoá.** Số 2.2 đo khi
mọi `target_text` còn rỗng. Đo lại sau khi Chương có bản dịch thật; số mới vào báo cáo cạnh số cũ.
🔴 Hàng Deferred `ARCHITECTURE-SPINE.md:899` là **Giai đoạn 3** và story này **không** mở nó — nó
ghi số, ghi hệ quả, và ghi **điều kiện mở lại** cho hàng đó nếu số mới đòi mở sớm.

> 📏 **"Đòi mở sớm" là một ngưỡng SỐ, không phải một cảm nhận.** Mốc so là trần **1,4 s** mà Story
> 1.16 đã chấp nhận cho một lượt dựng. Ghim:
>
> | Số mới ở trần 9.850 span *(engine chậm nhất)* | Phán quyết `:899` |
> | --- | --- |
> | < 1,4 s | giữ nguyên Giai đoạn 3, ghi số, **không** mở |
> | 1,4 s → 2,0 s | ghi **điều kiện mở lại** kèm số; Ice quyết, dev **không** tự mở |
> | > 2,0 s **hoặc** tăng > 50 % so với số 2.2 | 🔴 **đòi mở sớm** — vào mục *"Cần Ice quyết"* với bảng Được/Mất |
>
> 🔵 **Bàn đo được giữ nguyên — Ice chốt 2026-08-13.** Số cũ *(300,1 ms Blink · 1.308,0 ms
> WebKit)* đến từ bàn Playwright **hai engine, ngoài app**. Task 3 bản cũ đổi **cả hai** biến cùng
> lúc — bàn đo *(→ Tauri thật, chỉ WKWebView)* **và** dữ liệu *(→ chữ thật)* — nên chênh lệch
> không quy được cho biến nào. Ghim: đo lại trần 9.850 span **trên đúng bàn Playwright hai engine
> cũ**, chỉ đổi **một** biến là chữ thật. Số in-app WKWebView ghi thành **cột thứ ba, dán nhãn bàn
> đo khác** — 🔴 **cấm** đặt số in-app và số ngoài-app chung một cột. Cột ba lệch xa cột WebKit là
> một **phát hiện** *(bàn đo ngoài app không đại diện)* phải ghi ra, không phải chỗ hoà giải cho êm.

**AC14 — nếu đổi một hằng, đổi ở ĐÚNG một chỗ khai, và mọi thứ đứng trên nó phải xanh lại.**
`Tuning::default` (`mod.rs:229-243`) và ba hằng `editorFlush.ts` (`:43`, `:56`, `:78`) là **những
chỗ khai duy nhất**. ⚠️ **Cấm** đụng `IDLE_MS`/`HARD_CAP_MS` của `src/layout/writeSchedule.ts:57,67`
*(bố cục — Story 1.14)* và **cấm** đụng **hành vi** của `createWriteSchedule` *(`check-layout.mjs`
Kiểm B đứng trên nó)*. Mọi doc-comment ghi *"TẠM — chủ là Story 2.4"* phải được sửa thành số đã đo
kèm ngày và kèm lệnh đo lại.

> 🔧 **Sửa luôn SỐ DÒNG sai bên trong chính doc-comment đó.** `src/panels/editorFlush.ts` trỏ
> `ARCHITECTURE-SPINE.md:883` ở **hai** chỗ — `:35` và `:62` — nhưng hàng *"ngưỡng WAL"* nay ở
> **`:894`**, đúng như §Điều kiện khởi hành mục 12 tự xác nhận. Bản cũ của Task 5 chỉ đòi sửa
> *nội dung* doc-comment, nên `:883` sẽ sống sót qua lượt sửa và tiếp tục trỏ sai. Nghiệm thu:
> `grep -rn "ARCHITECTURE-SPINE.md:8[0-9][0-9]" src/ src-tauri/` trả về **0** dòng còn trỏ `:883`.

**AC15 — sau lượt đổi số, TOÀN BỘ lưới hiện có xanh lại.** **9/9** cổng npm · `npm run build` ·
`npm run test` **≥ 32/32** · `cargo test --locked` **≥ 319 xanh / 0 đỏ**. 🔴 Đặc biệt:
`store_contract.rs` lái cơ chế bằng `Tuning` **thu nhỏ** *(tick và idle tính bằng chục mili-giây)*
— một lượt đổi `Default` mà làm một ca đó đỏ là dấu hiệu ca đó đọc `Default` thay vì nhận tham số.
Sửa **ca test**, đừng lùi con số.

**AC16 — 0 dependency RUNTIME mới nếu AC4 chốt "không thư viện".** Ba dependency runtime giữ nguyên
và ghim chính xác: `@tauri-apps/api 2.11.1` · `dockview-vue 7.0.4` · `vue 3.5.40`. Nếu AC4 chốt
**có**: cửa NFR15 chạy **trọn vẹn trước khi thêm** *(§Điều kiện khởi hành mục 11)*, và lượt thêm đó
là một **quyết định của Ice**, không của dev — trình bảng Được/Mất, đừng tự cài.

⚠️ Công cụ đo *(Playwright, `hyperfine`, script `.mjs` dùng một lần…)* chạy từ bộ nhớ đệm `npx`
**ngoài kho**, cùng khuôn Story 2.2 và 2.3 — **không** vào `package.json`.

**AC17 — hai món nợ tài liệu đích danh của story này được đóng.** ① ba ảnh bàn đo của 2.2
(`2-2-ban-do/ban-do-blink-light.png` · `ban-do-webkit-dark.png` · `ban-do-webkit-light.png`) chụp
lại sau khi fixture thêm câu thứ sáu; ② `2-2-ban-do-editor.html:11` còn khai *"Dự án CỐ Ý không có
bộ chạy test frontend"* — lời khai đó **hết đúng** từ 2026-08-12, sửa nó *(đừng xoá trắng — xoá
trắng là đánh mất luôn cái cửa, đúng lý lẽ Task 0b.7 của 2.3)*.

**AC18 — đỉnh RSS của một lượt nhập 100 MB được đo, hoặc được TRẢ LẠI kèm chủ mới có tên.**
`deferred-work.md:591` gán món này cho story này *("cho con số")*. Nó **tách rời** khỏi cặp
NFR2/NFR18 và chỉ tốn một lượt nhập + một lượt lấy mẫu RSS. ⇒ Đo nó, **hoặc** ghi một hàng
`deferred-work.md` mới với **một chủ có tên** và lý do — bài học #7 của Epic 1: *"ghi nợ có chủ"*,
không hoãn bằng một câu chung chung.

> ⚖️ **Vế "trả lại" có ĐIỀU KIỆN, không phải một cửa mở sẵn.** Mọi Quyết định khác trong story đều
> buộc cân một bảng đánh đổi trước khi chọn nhánh; vế này bản cũ để trống, nên *"không đủ thời
> gian"* cũng thoả câu chữ dù chưa từng thử đo. Chỉ được chọn "trả lại" khi **một** trong hai đúng
> và điều đó **ghi ra kèm bằng chứng**: ① lượt nhập 100 MB **không chạy được** trên bàn đo đã dựng
> *(kèm lỗi cụ thể)*; ② lượt đo RSS **làm hỏng** số NFR2/NFR18 đang đo và không tách được ra phiên
> riêng *(kèm số cho thấy nhiễu)*. "Hết giờ" **không** phải điều kiện — món này tách rời khỏi cặp
> NFR2/NFR18 và chỉ tốn một lượt nhập.

**AC19 — mọi số đo LẶP LẠI ĐƯỢC, và điều đó được nghiệm thu bằng một lượt chạy lại.** Chuẩn Story
1.1: lệnh chính xác · phiên bản toolchain · tên tệp/checksum của mọi tạo tác đầu vào. 🔴 Tiền lệ
đắt: C3 của Story 1.22 kết luận *"ổn định"* trên **n=2** rồi bị chính bộ e2e bác lại ở lượt thứ
tám *(6 xanh / 2 đỏ)*. **Nói cỡ mẫu ra, đừng nói "ổn định".**

> 📏 **Sàn lặp lại được gán theo TỪNG phép đo — Ice chốt 2026-08-13.** Bản cũ đòi *"ít nhất một
> phép đo chạy hai lượt độc lập"*, tức chốt sàn đúng bằng cỡ mẫu mà tiền lệ nó vừa trích đã chứng
> minh là không đủ. Đo lại trước khi chốt: n=2 của AC1 **không** phải cỡ mẫu 2 — một phiên 30 phút
> ở 60 fps là ~216.000 frame, cỡ mẫu **trong phiên** đã khổng lồ. Cái n=2 thật sự mua là **độ tái
> lập giữa các phiên** *(nhiệt, tải nền)*. Nên:
>
> | Phép đo | Sàn | Lý lẽ |
> | --- | --- | --- |
> | **AC1** *(frame trong lúc auto-save)* | **n = 3 phiên** | +30 phút so với n=2; phá được ca hai phiên trùng nhau ngẫu nhiên |
> | **AC2** *(cửa sổ mất dữ liệu)* | **miễn nâng** | đã có ≥ 20 lượt kill hợp lệ **mỗi điểm lưới** *(AC8)* — vượt xa mối lo |
> | phép đo khác | n = 2 | giữ nguyên |
>
> 🔴 **Luật chống gộp mẫu — đây mới là chỗ 1.22 thật sự cháy.** 1.22 không cháy vì thiếu mẫu, nó
> cháy vì **một lời khai "ổn định"**. NFR2 là mệnh đề về **max** *("không frame nào vượt 50 ms")*,
> và thống kê max là loại tệ nhất khi bị gộp: một phiên xấu bị nuốt trọn. Nghiệm thu: báo cáo ghi
> **max của từng phiên riêng**, **cấm** pool; và mọi câu nói về độ ổn định phải kèm cỡ mẫu **ngay
> trong câu đó**, không để ở chú thích.

**AC20 — khoảng mù Windows được nói bằng chữ, không bị suy diễn lấp đi.** Một dòng trong báo cáo:
cả hai ngưỡng chỉ nghiệm thu trên macOS; hai chỗ lệch nền tảng đã biết là **engine webview**
*(WKWebView ⟷ WebView2)* và **đường `fsync`** *(APFS ⟷ NTFS)*. `deferred-work.md:1954-1957`.

> 🔧 **Trích dẫn đã sửa.** Bản cũ trỏ `deferred-work.md:145` ở cả AC20, §Điều kiện khởi hành mục 13
> và §References. `:145` **không** phải chỗ đó — nó là nợ của **Story 1.6**: *"Nghiệm thu DOM chạy
> trên Blink (Chrome), KHÔNG phải WKWebView"*, nói về **engine**, không về Windows. Câu được trích
> thật sự *("Trọn phần Windows dời về CUỐI dự án — Ice chốt 2026-08-12")* nằm ở **`:1954-1957`**.
> Lệch ~1.800 dòng.
>
> 🔴 **Khoảng mù áp cho CHÍNH CÁC CON SỐ, không chỉ cho khái niệm NFR.** Sản phẩm thật của story
> là những giá trị cụ thể ghi cứng vào **một** `Tuning::default` dùng chung cho **cả** macOS và
> Windows *(Task 5: "đổi số chỉ ở chỗ khai")*. Một dòng nói "NFR2/NFR18 chỉ nghiệm thu trên macOS"
> **không** che được việc sáu con số vừa chốt chưa từng chạy trên NTFS/WebView2 — đúng loại "khoá
> một giá trị sai cho một nền tảng chưa đo" mà AD-12 và NFR14 tồn tại để ngăn. Nghiệm thu bổ sung:
> mở **một hàng `deferred-work.md` riêng**, liệt kê **từng** giá trị đã chốt kèm cảnh báo chưa đo
> trên Windows, **chủ: Ice**, mở ở **cuối dự án** cùng lượt với phần Windows còn lại.

**AC21 — chi phí của CHÍNH BÀN ĐO được đo, và nó được trừ ra hoặc được nói ra.** 🔴 **BỐN** thứ
story này chạy đều ăn CPU trên **cùng một máy** với thứ nó đang đo: vòng lấy mẫu frame *(Quyết
định #3)*, lượt `stat()` `.db-wal` theo nhịp *(Quyết định #5 đường (a))*, bộ bơm phím *(Quyết định
#2 đường (c))*, và 🔵 **vòng đo tranh chấp CPU/I-O của AC10 vế 2** *(nguồn thứ tư, thêm 2026-08-13
theo lượt chốt AC10)*.

> 📏 **Nhịp `stat()` được GHIM bằng số trước khi chạy.** Mọi hằng đo khác trong story đều có giá
> trị cụ thể; riêng chu kỳ polling `.db-wal` bản cũ chỉ nói *"theo nhịp"*. Chọn 100 ms là tự làm
> nhiễu chính số NFR2 đang đo, và **không cổng nào bắt được** — AC21 bản cũ chỉ đòi *"trừ ra hoặc
> nói ra"* mà không ép định lượng **trước**. Ghim: **`stat()` mỗi 1000 ms**, và nếu phép đo cần
> nhịp dày hơn thì nhịp mới phải chạy qua lượt đối chứng dưới đây **trước** khi dùng cho số chính. Một bảng NFR2 đỏ vì bàn đo là một bảng nói **sai nguyên nhân** — đúng lớp lỗi mà Story 1.22
đã ghi tên hai lần *(`element.click()` cho một lượt đỏ đổ lỗi cho sản phẩm; `page.setContent()` cho
ba biến thể đo lại một cây DOM đã tháo)*.

Nghiệm thu: một lượt **đối chứng** chạy **cùng bàn đo, cùng thời lượng, KHÔNG có app** *(hoặc app
mở nhưng không gõ)*, và số đó vào báo cáo cạnh số chính. Cộng một lượt đổi **nhịp lấy mẫu** — nếu
frame max đổi theo nhịp lấy mẫu, thì thứ đang được đo là bàn đo. **Cả bốn** nguồn chi phí đều phải
có mặt trong lượt đối chứng, kể cả vòng đo tranh chấp của AC10.

> 🔴 **Mất tiêu điểm cửa sổ giữa phiên phải PHÁT HIỆN được, không chỉ được cảnh báo.** Quyết định
> #2(c) nêu đúng rủi ro *("cửa sổ phải giữ tiêu điểm OS suốt phiên")* nhưng không có hàng rào nào,
> khác hẳn bộ lọc rAF của Quyết định #3. Vỡ khi: một thông báo hệ thống hoặc screensaver cướp
> tiêu điểm ở phút thứ 12 của phiên 30 phút — phím bơm sau đó **không hạ cánh**, mà bộ lọc rAF
> *(vốn chỉ loại delta không có input/flush/checkpoint)* không phân biệt được với *"app đang
> nghỉ"*. Nghiệm thu: bàn đo ghi mốc thời gian mỗi lượt `blur`/`focus` của cửa sổ; mọi khoảng
> **không có tiêu điểm** bị loại khỏi mẫu **và ghi ra** như một hạng mục riêng, tách khỏi số mẫu
> bị bộ lọc rAF loại. Tổng thời gian mất tiêu điểm > **5 %** thời lượng phiên ⇒ phiên đó **bỏ**,
> chạy lại.

**AC22 — trạng thái MÁY lúc đo được ghi ra, vì một phiên 30 phút đủ dài để máy đổi trạng thái.**
Nguồn điện *(cắm sạc hay pin — macOS hạ xung trên pin)*, có tiết lưu nhiệt trong phiên không, và
những gì khác đang chạy. 🔴 AC1 nghiệm thu bằng **đuôi phân bố** *("không frame nào vượt")*, và đuôi
là chỗ một lượt tiết lưu nhiệt hiện ra trước tiên. Một số đuôi không kèm trạng thái máy là một số
**không lặp lại được**, tức nó trượt chuẩn AC19.

---

## Task 0 — BẢY QUYẾT ĐỊNH, chốt TRƯỚC phép đo đầu tiên

Khuôn cố định của mọi story lớn trong dự án (1.17 · 1.18 · 1.19 · 1.20 · 1.21 · 2.1 · 2.2 · 2.3).
Mỗi quyết định có **đề xuất mặc định kèm lý lẽ đo được**. Dev đọc, xác nhận hoặc phản biện **bằng
số** — không im lặng thi hành, và không tự đổi sau khi đã chạy phép đo. Phán quyết ghi vào §Dev
Agent Record.

🔵 **Không quyết định nào ở story này được Ice ký trước** — khác 2.3, nơi #1 và #2 đã có phán quyết.
Cả bảy đều mở.

### 🔴 Quyết định #1 — BẢN BUILD nào được đo, và dữ liệu thật của Ice được che bằng gì?

**Đây là quyết định chặn cả story** (§Điều kiện khởi hành mục 9). Bốn đường:

**(a) Bản debug + `--features wdio`.** Hai móc chuyển hướng chạy, dữ liệu thật an toàn tuyệt đối,
và fixture e2e đã có sẵn. **Cái giá, và nó giết phép đo:** Rust debug trên đường ghi chậm hơn
release nhiều bậc ⇒ mọi số NFR2/NFR18 là số của **một sản phẩm không tồn tại**. Một bảng đỏ đọc ra
từ đây không phân biệt được *"sản phẩm trượt"* với *"bản debug chậm"*.

**(b) Bản release đúng như phát hành, chạy trên dữ liệu thật.** Số đúng tuyệt đối. **Cái giá:** ≥ 20
lượt `SIGKILL` giữa lúc đang ghi, lên `global.db` thật và `~/Documents/AuraTranslate/` thật của Ice.
**Loại thẳng** — nó tái lập đúng lớp lỗi mà Story 1.22 đã tốn hai bề mặt để đóng.

**(c) `[profile.release]` + `debug-assertions = true` + `--features wdio`.** `cfg` là
`all(debug_assertions, feature = "wdio")` ⇒ **cả hai lớp bật** ⇒ móc chuyển hướng chạy **trên mã đã
tối ưu**. **Cái giá:** `debug-assertions = true` bật cả `debug_assert!` và *(theo mặc định)*
`overflow-checks` ⇒ codegen **khác** bản phát hành. Số đọc ra là số của một profile **gần** sản
phẩm, không **bằng** sản phẩm — và độ lệch đó phải được đo, không được đoán.

**(d) Bản release đúng như phát hành, cô lập bằng một `HOME` nháp.** Không đụng một dòng mã sản
phẩm nào. `dirs`/`dirs-sys` phân giải `$APPDATA` **và** `document_dir()` qua `$HOME` **trên macOS**
(đã đo ở Story 1.22 — chính phép đo đó là lý do hai móc kia đọc biến môi trường trong Rust thay vì
đổi `HOME`). **Cái giá phải nói ra:** đường này **chỉ đúng trên macOS** — trên Windows `dirs-sys`
đi Known Folder API và bỏ qua `%APPDATA%`. ⚠️ Nhưng cảnh báo đó được viết cho **mã sản phẩm**, còn
đây là **bàn đo**, và nửa Windows đã dời về cuối dự án. Hai điều kiện kèm theo, cả hai cưỡng chế
được: một **hàng rào chiều âm** kiểu `onComplete` của `wdio.conf.mjs` — `~/Documents/AuraTranslate/`
và `$APPDATA` thật phải **y nguyên từng byte** sau cả bộ đo; và một **hàng rào chiều dương** — mọi
`.atproj`/`global.db` sinh ra phải nằm trong `HOME` nháp.

**Đề xuất mặc định: (d) làm đường chính, (a) chạy MỘT lượt làm điểm quy chiếu.** Lý lẽ: (d) cho số
của **đúng profile sẽ phát hành** *(`opt-level = "s"`, `lto`, `panic = "abort"`)* mà không đổi một
dòng mã hay một dòng `Cargo.toml`; (a) chạy một lượt cho **hệ số debug↔release**, và hệ số đó có
giá trị riêng — **mọi số đo trước đây của dự án về đường ghi đều đi qua bản debug**, nên đây là lần
đầu ai đó biết chúng lệch bao nhiêu. (c) giữ làm đường lui nếu hàng rào chiều âm của (d) không dựng
được.

⚠️ Dù chọn đường nào: **hàng rào chiều âm phải chạy TRƯỚC lượt kill đầu tiên**, không sau. Bài học
Story 1.22 nguyên văn: bề mặt dữ liệu thứ hai được tìm ra *"bằng cách **đọc mã** lúc chuẩn bị
fixture, không bằng cách mất dữ liệu thêm một lần"*. Trước khi kill, **liệt kê mọi đường ghi** của
app và hỏi từng đường *"nó rơi vào đâu khi bàn đo chạy"*.

### 🔴 Quyết định #2 — Cái gì LÁI 30 phút gõ?

`browser.keys()` của WebdriverIO **đã bị loại bằng phép đo** — nó chỉ bắn `keydown`, không
`beforeinput`, nên chữ không hạ cánh (`deferred-work.md:2334-2337`). Ba đường còn lại:

**(a) Ice gõ tay 30 phút.** Trung thực tuyệt đối: IME thật, nhịp người thật, và nó là đường **duy
nhất** đo được vế *"gõ tiếng Việt có dấu qua bộ gõ"*. **Cái giá:** không lặp lại được, và AC2 đòi
**≥ 20** lượt kill + khởi động lại — nhân với tay người là nhiều giờ của Ice.

**(b) Bơm `execCommand('insertText')` theo nhịp, trong chính webview thật.** Đã chứng minh chạy ở
2.3 *(`beforeinput` → `input` → chữ hạ cánh)*. Rẻ, lặp lại được. **Cái giá, và nó thật:** nó **bỏ
qua** đường bàn phím của engine — không `keydown`, không lượt hợp thành IME, không lượt cuộn theo
caret của trình duyệt. ⇒ Một bảng xanh từ (b) **không** chứng minh NFR2 cho một người gõ thật.

**(c) Bơm phím ở tầng hệ điều hành** *(macOS `CGEventPost` / `osascript … keystroke`)*. Sự kiện
phím **thật** đi vào cửa sổ thật: engine chạy trọn đường `keydown` → `beforeinput` → `input`. Lặp
lại được, script hoá được, **không chạm một dòng mã sản phẩm nào**. **Cái giá:** cần quyền
Accessibility trên macOS; và cửa sổ phải giữ tiêu điểm hệ điều hành suốt phiên — chính là điều kiện
mà `document.hasFocus() === false` của bộ e2e đã **không** giữ được.

**Đề xuất mặc định: (c) làm đường chính, (a) làm một lượt xác nhận NGẮN.** Lý lẽ đo được: (c) là
đường duy nhất vừa **lặp lại được** *(AC2 đòi ≥ 20 lượt, AC19 đòi chạy lại)* vừa đi **trọn** đường
bàn phím của engine — tức nó đóng đúng khoảng cách mà (b) để hở và đúng khoảng cách mà bộ e2e đã tự
chứng minh là nó bóp méo. (a) chạy **một** phiên ngắn *(≈ 5 phút, có dấu tiếng Việt qua bộ gõ thật)*
để trả lời vế IME mà (c) không phủ, và để bảng số có **một** điểm neo từ tay người.

⚠️ **Nội dung gõ không được là `"aaaa"`.** Gõ tiếng Việt có dấu, độ dài câu thay đổi, và **có cả
lượt xoá** — `deferred-work.md` đã ghi rằng ca *xoá lùi qua đầu câu* là ca thủng cao nhất theo dự
đoán. Một dòng ký tự đồng nhất đo một sản phẩm không ai dùng.

### 🔴 Quyết định #3 — "frame" được lấy mẫu bằng gì, và cái gì bị loại khỏi mẫu?

**(a) Delta giữa hai lượt `requestAnimationFrame` liên tiếp**, đo trong trang. Đây là thứ bàn đo
của 2.2 và 2.3 đã dùng ⇒ số **so sánh được** với ba bảng đã có. **Cái giá:** rAF **không bắn** khi
trang không có gì để vẽ hoặc cửa sổ bị che ⇒ một khoảng nghỉ cho một delta khổng lồ **không phải**
một frame rớt. Phải có bộ lọc, và bộ lọc đó phải được ghi ra.

**(b) `PerformanceObserver` với `longtask`.** Đúng ngữ nghĩa *"tác vụ dài chặn luồng chính"*.
**Cái giá:** mức hỗ trợ `longtask` trong **WKWebView** phải **đo**, không được giả định — và nếu nó
vắng mặt thì phép đo im lặng trả về một bảng rỗng, đọc thành *"không có frame nào vượt"*. Đó đúng
lớp lỗi *"xanh rỗng"* mà AC15 của Story 2.3 đã đặt tên.

**(c) `performance.now()` quanh handler `input`.** Đo chi phí **handler**, không đo **frame**. Không
trả lời được NFR2 nguyên văn. Dùng làm số **bổ trợ**, không làm số nghiệm thu.

**Đề xuất mặc định: (a) làm số nghiệm thu, (c) làm số chẩn đoán, (b) chỉ sau khi đã đo là nó tồn
tại.** Bộ lọc bắt buộc của (a): loại mọi delta mà **không có** một lượt `input`/`flush`/checkpoint
nào trong cửa sổ đó *(tức app đang nghỉ)*, và **ghi ra số mẫu bị loại** — một bộ lọc bỏ 90% mẫu là
một phép đo hỏng, không phải một phép đo sạch. Cùng luật mà `check-deps.mjs` vừa học ở 2.3: in ra
số node đã bỏ *(82)* để con số không biến mất im lặng.

🔴 **Và một mệnh đề phạm vi phải chốt ở đây:** NFR2 nguyên văn là *"không frame nào vượt 50 ms
**trong lúc auto-save chạy**"*. Lượt **dựng Chương** *(300,1 ms Blink · 1.308,0 ms WebKit)* **không**
nằm trong lúc auto-save chạy. Đề xuất: đo và báo **cả hai** cửa sổ, tách bạch — *"trong lúc
auto-save"* là số **nghiệm thu AC1**, *"lúc dựng Chương"* là số **giao cho hàng Deferred ảo hoá**
(AC13). ⚠️ Đây là chỗ dễ tự nới định nghĩa theo hướng dễ hơn nhất trong cả story. **Ghi ranh giới
ra trước khi đo**, đừng vẽ nó sau khi thấy số.

### 🔴 Quyết định #4 — "mất tối đa 5 giây" đo từ mốc nào tới mốc nào?

NFR18 nói *"cửa sổ mất dữ liệu tối đa khi ứng dụng sập: ≤ 5 giây **công việc**"*. Ba cách hiểu:

**(a) Đồng hồ tường:** *(thời điểm `SIGKILL`)* − *(thời điểm gõ ra ký tự cuối cùng còn sống sót
trong `project.db`)*. Đây là *"công việc"* theo nghĩa người dùng cảm nhận.

**(b) Đếm ký tự mất.** Đơn vị sai — 5 giây gõ nhanh và 5 giây gõ chậm cho hai số khác nhau.

**(c) Số lượt flush bị mất.** Một proxy của cơ chế, không của trải nghiệm.

**Đề xuất mặc định: (a).** Và nó đòi một thứ mà bàn đo phải dựng **trước** lượt kill đầu tiên: mỗi
lượt gõ **mang một chỉ số đơn điệu tăng** *(ví dụ chèn một token `⟦n⟧` hoặc gõ một dãy số tăng dần)*
kèm **nhật ký thời điểm bơm** của chính bàn đo. Sau kill + mở lại, đọc `target_text` ra, lấy chỉ số
**lớn nhất còn sống**, tra ngược ra thời điểm bơm ra nó, rồi trừ khỏi thời điểm kill.

⚠️ Ba cái bẫy, cả ba đã thấy trước:
1. **Đọc `project.db` bằng đường nào** — mở app lại và đọc qua giao diện là đo **cả** đường nạp; mở
   `.db` bằng `sqlite3` **chỉ đọc** là đo đúng thứ NFR18 nói. Đề xuất: `sqlite3` chỉ đọc làm số
   nghiệm thu, một lượt mở app làm đối chứng *(và để bắt ca *"dữ liệu trên đĩa nhưng nạp lại không
   ra"*, thứ NFR18 không bắt được)*.
2. **`.db-wal` phải được đọc cùng** — sau `SIGKILL`, dữ liệu bền nằm trong WAL chứ chưa trong `.db`.
   Một lượt đọc bỏ qua `-wal` sẽ báo mất nhiều hơn thật. `sqlite3` mở đúng thư mục sẽ tự phục hồi
   WAL; **xác minh** điều đó ở lượt đo đầu tiên thay vì tin.
3. **Đồng hồ của bàn đo và của tiến trình bị kill là hai đồng hồ** — cùng máy nên cùng nguồn, nhưng
   ghi ra là mình đã kiểm.

### Quyết định #5 — Đọc trạng thái checkpoint bằng đường nào?

`CheckpointStats` *(`idle_triggered`, `threshold_triggered`, số lượt bị chặn)* đã có và `pub`
(`checkpoint.rs:58-70`, `mod.rs:677`) — nhưng **không có đường ra IPC**. Ba đường:

**(a) Đo từ NGOÀI tiến trình:** `stat()` `.db-wal` của **cả hai** kho theo nhịp, dựng đường răng
cưa. Không chạm một dòng mã sản phẩm nào, và nó chạy được trên **đúng bản build sẽ phát hành**
(Quyết định #1 đường (d)).

**(b) Thêm một lệnh IPC chỉ-debug**, gác đúng hai lớp AD-45 *(`debug_assertions` + `feature = wdio`)*
— tiền lệ đã có ở hai móc env. Cho số **chính xác** thay vì suy từ hình dạng tệp. **Cái giá:** nó
**không tồn tại** trong bản phát hành ⇒ nếu Quyết định #1 chốt (d), đường này **không dùng được ở
chính phép đo nghiệm thu**.

**(c) Đọc dòng chẩn đoán** mà `passive()`/`truncate()` đã ghi (`checkpoint.rs:333-390`).

**Đề xuất mặc định: (a) làm số nghiệm thu, (c) làm số đối chiếu.** Lý lẽ: (a) là đường **duy nhất**
tương thích với bản build mà Quyết định #1 đề xuất, và nó đo đúng thứ AD-12 quan tâm — *`.db-wal`
có phình vô hạn không*. (b) chỉ dựng nếu (a) **không** phân biệt được hai vế kích hoạt; và nếu dựng,
nó là mã sản phẩm mới ⇒ kéo theo sàn `*_FLOOR` và một dòng ở bảng Stack.

### Quyết định #6 — Chương thật nào được đo, và nó vào máy bằng đường nào?

**(a) Chương lớn nhất có thật** — 9.850 câu / 48.640 ký tự, chính Chương mà 2.2 đã đo. Cho số **ca
trần**, so sánh được với ba bảng đã có.

**(b) Một Chương cỡ trung bình thật** — cho số **ca thường nhật**, tức thứ NFR2 thật sự hứa với
người dùng.

**(c) Cả hai.**

**Đề xuất mặc định: (c), và (b) là số nghiệm thu AC1.** Lý lẽ: NFR2 hứa với **phiên làm việc bình
thường**; ca trần 9.850 câu là số cho hàng Deferred *"ảo hoá"* (AC13), không phải mẫu số của AC1.
Báo **cả hai**, nói rõ cái nào nghiệm thu cái gì. ⚠️ Cỡ của (b) phải lấy từ **dữ liệu thật của Ice**,
không phải một số tròn do dev nghĩ ra — và cỡ đó ghi vào báo cáo kèm nguồn.

⚠️ Đường vào: `create_work_from_text` qua IPC *(khuôn `e2e/support/workspace.mjs`)* hay đường nhập
của giao diện. Đường IPC rẻ và không giòn; **cái giá đã ghi ở chính fixture đó**: nó *"**không** đo
đường nhập của người dùng thật"*. Ở story này cái giá đó chấp nhận được — story đo **đường gõ**, không
đo đường nhập. Ghi ra là mình đã cân, đừng bỏ qua im lặng.

### Quyết định #7 — Thư viện editor: cái gì được cân, và cân theo tiêu chí nào? *(AC4)*

Hôm nay Panel Editor **không dùng thư viện nào**: vùng gõ là `contenteditable="true"` trên **đúng
một** `<span class="sent">` tại một thời điểm, cộng một **bộ lọc dán**. Mũi thăm dò của 2.3 đã đo
cơ chế đó trên **cả hai** engine và nó đứng — 6/6 thao tác giữ nguyên sổ sách `data-segment-id`,
lắp/tháo `contenteditable` tốn median **0,3 ms** Blink · **1 ms** WebKit, vạch lề lệch **0,00 px**.

Ba đường:

**(a) Không thư viện — ghi lại quyết định đó kèm số.** AC4 nói *"lựa chọn được ghi lại kèm lý do"*;
*"không dùng"* **là** một lựa chọn, và nó là lựa chọn duy nhất hiện có **số đo chống lưng**.

**(b) Thêm một thư viện editor.** Ứng viên và giấy phép *(tra 2026-08-13)*: **CodeMirror 6** — MIT ·
**ProseMirror** — MIT · **Lexical** — MIT · **Tiptap** *(nhân)* — MIT, và Tiptap đã mở mã 10 extension
Pro cũ về MIT năm 2025, phần thu phí dồn về nền tảng Cloud · **Quill** — BSD-3-Clause · **Slate** — MIT.
Cả sáu thuộc nhóm dễ dãi, tương thích GPL-3.0-or-later theo chiều đi vào. **Cái giá, và nó không
phải giấy phép:** cả sáu đều sở hữu **cây DOM của vùng soạn thảo**, còn sổ sách của dự án —
`data-segment-id`, vạch lề đo bằng `getClientRects()`, `⏐` là pseudo-element — **sống trong đúng cây
DOM đó**. Thêm một thư viện là **giao lại quyền sở hữu text node** cho nó, tức mở lại đúng câu hỏi
mà đường (c) của Quyết định #1 ở Story 2.3 đã làm **biến mất**, và lật một doctrine vừa được ký bằng
máu: *"DOM sở hữu văn bản bản dịch, Vue không"* (`deferred-work.md:2261`).

**(c) Hoãn tiếp.** 🔴 **Loại thẳng** — AC4 là AC của story này, và hàng Deferred `:897` ghi *Giai
đoạn 2*, tức **bây giờ**. Hoãn lần nữa là để một hàng Deferred trôi qua đúng cửa sổ nó được hẹn.

**Đề xuất mặc định: (a).** Điều kiện để phán quyết đó **có giá trị** chứ không phải một lượt hợp
thức hoá hiện trạng: nó phải đứng trên **số của story này** *(AC1 + AC12 + AC13)*, không chỉ số của
2.3. 🔴 **Và một điều kiện lật ghi ra trước:** nếu AC1 hoặc AC12 cho thấy cơ chế tự viết **không**
đạt NFR2 ở Chương cỡ thường, thì (a) **sai** và đường đi là **trình bảng Được/Mất cho Ice**, không
phải dev tự cài một gói thứ tư *(§Điều kiện khởi hành mục 11)*.

⚠️ Vế thứ hai của AC4 — *"tuân hợp đồng trạng thái AD-31 nên không lan ra ngoài module"* — phải được
trả lời **kể cả khi phán quyết là (a)**: ghi ra rằng cơ chế hiện tại giữ AD-31 ở đâu *(auto-save
không đổi trạng thái, không tạo `SegmentVersion` — doc-comment tại chỗ trên hàm ghi, AC15 của 2.3)*
và **cái gì sẽ vỡ** nếu một story sau thay nó bằng một thư viện. Đó là thứ làm hàng Deferred đóng
được, chứ không phải câu *"chúng tôi không dùng thư viện"*.

---

## Tasks / Subtasks

- [ ] **Task 0 — Bảy quyết định** (AC: mọi AC)
  - [ ] Đọc §Task 0, ghi phán quyết từng cái vào §Dev Agent Record **kèm số đo hoặc kèm lý lẽ đo được**
  - [ ] 🔴 Quyết định #1 chốt xong **trước** khi build bất cứ thứ gì
  - [ ] 🔴 **LUẬT DỪNG cho lớp lỗi "bàn đo dựng không nổi"** — song song với luật dừng của Task 1.0, và nó thiếu ở bản cũ. Đếm **vòng chẩn đoán**: một vòng = một giả thuyết + một lượt build/chạy để nghiệm nó. **Ba** vòng bị bác liên tiếp mà chưa dựng được bàn đo ⇒ **DỪNG**, ghi cả ba giả thuyết và cách chúng bị bác vào §Dev Agent Record, báo Ice. **Không** đoán tiếp. Bằng chứng vì sao luật này cần: §Dev Agent Record của chính story đã chạy **bốn** vòng *(`strings` · CSP · cache `build.rs` · `document.title`)* và quyết định *"không đoán tiếp"* là suy luận tại chỗ, **không** phải luật viết sẵn — lượt thi hành sau sẽ không dừng đúng lúc

- [ ] **Task 1 — Điều kiện khởi hành và hàng rào dữ liệu thật** (AC: 1, 2)
  - [ ] **1.0 — kiểm đầu vào bằng TAY**: mở app, bấm vào một câu **chưa dịch**, gõ một chữ. Ghi kết quả. 🔴 Không gõ được ⇒ **DỪNG**, báo Ice, story về `backlog` (§Điều kiện khởi hành mục 2)
  - [ ] Dựng bàn đo theo phán quyết Quyết định #1 · #2 · #3 · #4
  - [ ] 🔴 **Hàng rào chiều âm chạy TRƯỚC lượt kill đầu tiên**: chụp trạng thái `~/Documents/AuraTranslate/` và `$APPDATA` thật *(danh sách tệp + kích thước + mtime)*; đối chiếu lại sau cả bộ đo
  - [ ] **Hàng rào chiều dương**: mọi `.atproj`/`global.db` sinh ra phải nằm trong vùng nháp — kiểm bằng máy, không bằng mắt
  - [ ] Liệt kê **mọi** đường ghi của app và trả lời từng đường *"nó rơi vào đâu khi bàn đo chạy"* (bài học Story 1.22)
  - [ ] Ghi phiên bản: macOS · Rust · Node · `@tauri-apps/cli` · WebKit · model máy · profile build
  - [ ] **Lượt đối chứng chi phí bàn đo** — cùng bàn đo, cùng thời lượng, không gõ (AC21)
  - [ ] Đổi **nhịp lấy mẫu** một lượt; frame max đổi theo ⇒ đang đo bàn đo, không đo sản phẩm (AC21)
  - [ ] Ghi trạng thái máy: nguồn điện · tiết lưu nhiệt · tải nền (AC22)

- [ ] **Task 2 — Thang đo vào máy** (AC: 1, 2, 13)
  - [ ] 🔴 **Thư viện thật KHÔNG có Chương cỡ thường nhật** — khoảng 669 → 48.639 ký tự rỗng hoàn toàn. Ice chốt 2026-08-13: dùng **thang nhân tạo** cắt từ một tài liệu duy nhất, **không** giả vờ đó là Chương thật (AC1)
  - [ ] Ghi cỡ, nguồn, và **cách cắt** của từng bậc thang — lệnh chính xác, để lượt sau dựng lại được (AC19)
  - [ ] Dựng lại Chương **9.850 câu / 48.640 ký tự** của 2.2 để so sánh được với ba bảng đã có
  - [ ] Cả hai vào máy qua đường đã chốt ở Quyết định #6; ghi lệnh chính xác

- [ ] **Task 3 — Đo NFR2** (AC: 1, 11, 12, 13)
  - [ ] Phiên gõ liên tục **≥ 30 phút** trên thang nhân tạo, lái theo Quyết định #2 · **n = 3 phiên** (AC19)
  - [ ] Lấy mẫu frame theo Quyết định #3; **ghi số mẫu bị bộ lọc loại**, và áp ba mức phán quyết ≤50% / >50% / **=100%** của AC11
  - [ ] 🔵 Ghi mốc `blur`/`focus` cửa sổ; loại khoảng mất tiêu điểm thành **hạng mục riêng**; mất > **5 %** thời lượng ⇒ **bỏ phiên, chạy lại** (AC21)
  - [ ] Tách **hai** cửa sổ: *"trong lúc auto-save chạy"* (nghiệm thu AC1) và *"lúc dựng Chương"* (giao cho AC13)
  - [ ] Đo ba đường nóng của §mục 6 — `:data-caret` · `restoreEditedText()` · `nearestSentenceTo()` — ở **cả hai** cỡ (AC12)
  - [ ] 🔴 Đo lại trần dựng 9.850 span **với chữ thật, TRÊN ĐÚNG BÀN PLAYWRIGHT HAI ENGINE CŨ** — đổi **một** biến duy nhất là chữ. Đặt cạnh số cũ *(300,1 ms Blink · 1.308,0 ms WebKit)* (AC13)
  - [ ] 🔴 Số in-app WKWebView ghi thành **cột thứ ba, dán nhãn bàn đo khác** — cấm chung cột với số ngoài-app. Cột ba lệch xa cột WebKit ⇒ ghi ra như một **phát hiện** (AC13)
  - [ ] Ghi **max của từng phiên riêng**; 🔴 **cấm gộp mẫu**; mọi câu về độ ổn định kèm cỡ mẫu ngay trong câu (AC19)

- [ ] **Task 4 — Đo NFR18** (AC: 2, 9, 10, 11)
  - [ ] Bơm dòng gõ **mang chỉ số đơn điệu tăng** kèm nhật ký thời điểm bơm (Quyết định #4)
  - [ ] 🔴 **Tự động hoá vòng kill** — sau lượt chốt AC8, tổng số lượt là **N điểm lưới × ≥ 20 mẫu hợp lệ**; làm tay ở cỡ đó là chỗ đẻ ra lỗi phương pháp
  - [ ] `SIGKILL` ở thời điểm **ngẫu nhiên** trong lúc gõ, bắn tới khi đủ **≥ 20 mẫu HỢP LỆ** *(không phải 20 lượt bắn)*; ghi **cả hai** số: đã bắn / hợp lệ (AC9)
  - [ ] 🔵 Sau mỗi lượt, phân **ba** loại theo AC9 — ① `.db-wal` > 0 ⇒ **hợp lệ**; ② `.db-wal` = 0 **và** ký tự cuối đã vào kho / không thấy dấu `wire_exit_flush` ⇒ **trúng lúc rảnh, GIỮ trong mẫu, ghi 0 s**; ③ `.db-wal` = 0 **và** có dấu đi qua đường thoát bình thường ⇒ **trượt, bỏ và ghi ra**. Không phân biệt được ⇒ bỏ **và ghi là không phân biệt được**
  - [ ] Đọc `target_text` bằng `sqlite3` **chỉ đọc**; xác minh WAL được phục hồi ở lượt đầu tiên
  - [ ] Một lượt đối chứng: mở app lại và đọc qua giao diện — bắt ca *"trên đĩa nhưng nạp lại không ra"*
  - [ ] Báo cáo ghi **max của từng lượt chạy riêng**, 🔴 **cấm gộp mẫu**; áp dung sai của AC2 *(`max − trung vị > 2 s` ⇒ gắn cờ **BẤT ỔN**)*
  - [ ] Ghi kích thước `.db-wal` của **CẢ HAI** kho theo thời gian — vế ① của AC10, đóng `:570`
  - [ ] 🔴 **Đo tranh chấp thật** — thời gian CPU từng luồng checkpoint + độ trễ I-O từng lượt, ở **hai chế độ**: hai kho **chạy chồng** vs **từng kho chạy riêng**. Chênh lệch hai chế độ là con số `:234` hỏi — vế ② của AC10
  - [ ] 🔴 **Ép trùng pha** — nạp cả hai kho tới ngưỡng rồi thả cùng lúc, buộc hai luồng checkpoint kích hoạt cùng thời điểm. Vế ③ của AC10; thiếu nó thì lệch pha tự nhiên vẫn cho AC10 xanh mà chưa đo được gì
  - [ ] Vòng đo tranh chấp vào lượt đối chứng chi phí bàn đo như **nguồn thứ tư** (AC21)

- [ ] **Task 5 — Dò ngưỡng WAL và sáu số `Tuning`** (AC: 3, 7, 8, 10, 14)
  - [ ] 🔴 **LƯỚI ĐƯỢC GHIM BẰNG SỐ — sáu điểm, không phải chữ "theo lưới"**: `512 KiB · 1 MiB · 2 MiB · 4 MiB · 8 MiB · 16 MiB`. Sáu điểm phủ **năm** bậc gấp đôi và ôm hai đầu mà AC8 bản cũ dùng. Bản cũ không có biên, bước nhảy, hay điều kiện dừng — dev thử 2 điểm rồi coi là xong, hoặc dò hàng chục điểm không có đáy, cả hai đều thoả câu chữ
  - [ ] **Điều kiện dừng:** chạy **hết** sáu điểm. Đây cũng là cái mà nhánh một-ngưỡng-trượt của **AC5** treo lên — chưa hết lưới thì **chưa** được báo Ice
  - [ ] **Luật tinh chỉnh, tối đa 2 lượt:** nếu hai điểm liền nhau nằm hai bên một lằn ngưỡng *(một đạt, một trượt)*, thêm **một** điểm giữa. Quá 2 lượt tinh chỉnh ⇒ dừng, ghi số, để Ice quyết
  - [ ] Chạy bộ kill của Task 4 **tại MỌI điểm lưới** *(≥ 20 mẫu hợp lệ mỗi điểm)*; dựng bảng cửa sổ mất dữ liệu **max theo ngưỡng** — bảng đó **là** câu trả lời cho AC3, không cần giả thuyết (AC8)
  - [ ] Mỗi điểm ghi thêm: frame max · số lượt checkpoint theo vế **ngưỡng** · đỉnh `.db-wal` · 🔵 **thời gian chờ khoá của mỗi lượt flush, tách riêng khỏi thời gian ghi** — kênh `busy_timeout` mà §mục 8 bỏ sót (AC8)
  - [ ] Sáu hàng `Tuning` — mỗi hàng **đúng một** nhãn trong ba nhãn của AC7, kèm số hoặc kèm chủ mới có tên. 🔴 Nhãn ba dùng **≥ 3** lần ⇒ **DỪNG trước Task 8**, báo Ice (AC7)
  - [ ] 🔴 Kiểm chéo `idle_before_passive` ⟷ `EDITOR_IDLE_MS`: đổi nhịp flush mà quên ràng buộc *"cố ý dài hơn"* là làm luồng checkpoint đánh nhau với đường gõ (§mục 4)
  - [ ] 🔵 **Dựng CỔNG MÁY cho ràng buộc đó**, không để nó sống bằng câu chữ: một `debug_assert!`/ca test giữ bất biến `idle_before_passive > EDITOR_IDLE_MS` sau khi **cả hai** số đã hiệu chỉnh. Bản cũ chỉ có một dòng nhắc trong Task 5 — hai lượt sửa riêng biệt *(hạ cái này, nâng cái kia)* lọt hết mọi cổng cho tới khi Task 11 chạy xong
  - [ ] Đổi số **chỉ ở chỗ khai** — `Tuning::default` (`mod.rs:229-243`) và ba hằng `editorFlush.ts` (`:43`, `:56`, `:78`). **Không** đụng `writeSchedule.ts:57,67` (AC14)
  - [ ] Sửa mọi doc-comment *"TẠM — chủ là Story 2.4"* thành số đã đo + ngày + lệnh đo lại
  - [ ] 🔧 **Sửa luôn số dòng sai TRONG doc-comment**: `editorFlush.ts:35` và `:62` trỏ `ARCHITECTURE-SPINE.md:883`, vị trí thật là **`:894`**. Nghiệm thu: `grep -rn "ARCHITECTURE-SPINE.md:883" src/ src-tauri/` trả **0** dòng (AC14)

- [ ] **Task 6 — Ba đường nóng: số trước, vá sau (nếu có)** (AC: 12, 13)
  - [ ] Ghi số cho cả ba trước khi cân nhắc một dòng vá nào
  - [ ] 🔴 Vá **chỉ khi** số nói cần; mỗi bản vá kèm **đỏ-rồi-xanh** + một dòng vì sao nó không phải thứ Giai đoạn 3 sẽ làm lại (`deferred-work.md:2441`)
  - [ ] Không vá ⇒ ghi lại **kèm chủ mới có tên**, không hoãn bằng câu chung chung

- [ ] **Task 7 — Phán quyết thư viện editor** (AC: 4, 16)
  - [ ] Viết phán quyết kèm lý do, đứng trên **số của story này**, không chỉ số của 2.3
  - [ ] Trả lời vế AD-31 *(không lan ra ngoài module)* **kể cả khi phán quyết là "không thư viện"* — ghi cả *cái gì sẽ vỡ* nếu một story sau thay nó
  - [ ] Nếu phán quyết là **có**: cửa NFR15 chạy **trọn** trước khi thêm, và lượt thêm là quyết định của **Ice** — trình bảng Được/Mất

- [ ] **Task 7b — Đỉnh RSS của một lượt nhập 100 MB** (AC: 18)
  - [ ] Đo, **hoặc** trả lại bằng một hàng `deferred-work.md` kèm **chủ có tên** và lý do (`:591`)
  - [ ] 🔵 **Vế "trả lại" chỉ mở khi một trong hai đúng, và phải kèm bằng chứng**: ① lượt nhập 100 MB **không chạy được** trên bàn đo đã dựng *(kèm lỗi cụ thể)*; ② lượt đo RSS **làm hỏng** số NFR2/NFR18 và không tách được ra phiên riêng *(kèm số cho thấy nhiễu)*. 🔴 *"Hết giờ"* **không** phải điều kiện (AC18)

- [ ] **Task 8 — Báo cáo và đóng hai hàng Deferred** (AC: 3, 4, 5, 6, 19, 20)
  - [ ] Viết `research/editor-perf-spike-results-2026-08-XX.md` theo **đúng bảy mục** của AC6
  - [ ] Mục *"Phỏng đoán bị bác"* — bắt buộc có, kể cả khi ngắn (tiền lệ Giai đoạn 0)
  - [ ] Mục *"Cần Ice quyết"* tách riêng, bảng **Được / Mất**
  - [ ] Một dòng về khoảng mù Windows — trích **`deferred-work.md:1954-1957`**, không phải `:145` (AC20)
  - [ ] 🔵 **Mở một hàng `deferred-work.md` riêng cho CÁC CON SỐ vừa chốt** — liệt kê **từng** giá trị đã ghi vào `Tuning::default`, cảnh báo chưa đo trên NTFS/WebView2, **chủ: Ice**, mở ở **cuối dự án**. Một dòng "NFR chỉ nghiệm thu trên macOS" không che được việc sáu con số dùng chung cho cả hai nền tảng (AC20)
  - [ ] `ARCHITECTURE-SPINE.md:894` — đóng hàng *"Ngưỡng WAL + nhịp flush"*, khuôn gạch ngang + `✅ Đã đóng`
  - [ ] 🔴 **Dòng đóng `:894` phải khai PHẠM VI nó thật sự đóng** — một câu nói cặp đánh đổi nào được dò thật và cặp nào hoá ra không tồn tại, kèm số từ bảng lưới của AC8. Thiếu câu đó, người đọc SPINE sau này tin nhầm trade-off gốc *(WAL ⟷ NFR18)* đã giải triệt để (AC3)
  - [ ] `ARCHITECTURE-SPINE.md:897` — đóng hàng *"Thư viện editor"*, cùng khuôn
  - [ ] 🔴 **Không** đóng `:899` *(ảo hoá)*; ghi **điều kiện mở lại** nếu số mới đòi mở sớm
  - [ ] `lint_spine.py` trả **0 findings**
  - [ ] `.memlog.md` của architecture: một dòng `(version)` ghi số đo, một dòng `(decision)` ghi phán quyết thư viện editor

- [ ] **Task 9 — Hai món nợ tài liệu đích danh** (AC: 17)
  - [ ] Chụp lại ba ảnh bàn đo 2.2 sau khi fixture có câu thứ sáu
  - [ ] Sửa lời khai NFR15 sai ở `2-2-ban-do-editor.html:11` — **sửa**, không xoá trắng

- [ ] **Task 10 — Nếu hai ngưỡng KHÔNG đạt đồng thời** (AC: 5)
  - [ ] 🔴 **Xác định ĐANG ở ca nào TRƯỚC đã** — bảng ba ca của AC5: ① **loại trừ nhau** *(hết lưới, không giá trị nào thoả cả hai)* ⇒ khuôn Task 10 đầy đủ; ② **một ngưỡng trượt một mình** *(hết lưới, ngưỡng kia đỏ ở mọi điểm)* ⇒ cũng báo Ice theo khuôn này, **nhưng** báo cáo nói rõ là ca một ngưỡng, kèm giá trị tốt nhất tìm được và khoảng cách còn lại tới ngưỡng; ③ **chưa hết lưới** ⇒ 🔴 **chưa** được báo Ice, chạy nốt sáu điểm của Task 5 trước, và **không** tự tối ưu ngoài phạm vi hằng số đã cho phép
  - [ ] 🔴 **Không** tự nới `EDITOR_HARD_CAP_MS`, **không** tự đổi `opt-level`, **không** tự thêm một thư viện, **không** tự sửa `prd.md`/`epics.md`
  - [ ] Viết mục *"Cần Ice quyết"* nêu **đòn bẩy có thật kèm cái giá của từng đòn bẩy** (khuôn Story 1.1 §Đòn bẩy nếu vượt trần)
  - [ ] Nói thẳng trong báo cáo: đây là **thay đổi tầng PRD**, và Epic 2 dừng ở đây cho tới khi Ice quyết

- [ ] **Task 11 — Nghiệm thu cuối** (AC: 14, 15, 16)
  - [ ] **9/9** cổng npm · `npm run build` · `npm run test` ≥ 32/32 · `cargo test --locked` ≥ 319/0
  - [ ] Sàn `*_FLOOR` — chỉ rà **nếu** story thêm tệp sản phẩm; số thật hôm nay ở §Dev Notes
  - [ ] `npm run check:deps` — xác nhận **0** runtime mới, và không gói thứ tư nào lọt vào `devDependencies`

### Review Findings

> Rà soát đối kháng ba tầng ngày 2026-08-13, mốc gốc `6a4e6b8`. 24 phát hiện còn lại sau gộp trùng,
> 1 loại làm nhiễu. Mức nặng chấm theo hệ quả cho **người cầm story này đi tiếp**, không theo mức
> tầng con tự gán. `[Decision]` phải giải trước `[Patch]`.
>
> ✅ **ĐÃ XỬ XONG 2026-08-13.** Sáu `[Decision]` Ice ký hết, mười bảy `[Patch]` đã vá vào chính
> story này. Phán quyết của Ice: ① **thang nhân tạo** cho AC1 *(kèm lệnh cấm khai "Chương thật")* ·
> ② **đo tranh chấp CPU/I-O thật** cho AC10 *(không hạ tuyên bố)* · ③ AC5 **nghĩa hẹp + nhánh
> một-ngưỡng-trượt** · ④ **bỏ giả thuyết**, AC8 đo mọi điểm lưới · ⑤ AC19 **n=3 cho AC1** + luật
> **cấm gộp mẫu** · ⑥ giữ **bàn Playwright hai engine**, số in-app thành cột thứ ba dán nhãn riêng.
>
> ⚠️ **Ngân sách đo đã nở ra theo ba lượt chốt ②④ và mục ⑤** — lưới **6 điểm × ≥ 20 mẫu hợp lệ** =
> **≥ 120 lượt kill**, cộng hai chế độ đo tranh chấp, cộng 3 phiên gõ 30 phút. Hai việc vì thế
> thành **bắt buộc**, không còn tuỳ chọn: lưới phải **ghim số** *(đã ghim ở Task 5)* và vòng kill
> phải **tự động hoá** *(đã ghi vào Task 4)*. Tất cả chạy trên một bộ đo **hiện chưa dựng được**.

**Loại làm nhiễu (1):** AC4 bị cho là dùng tiêu chí phủ quyết rộng hơn AD-31 khi loại mọi thư viện
editor bằng doctrine `deferred-work.md:2261` *(không phải AD đánh số)*. Bác: AC4 chỉ đòi *"lựa chọn
được ghi lại kèm lý do"* — phủ quyết có lý lẽ ghi rõ **là** một lựa chọn hợp lệ.

- [x] [Review][Decision] **🔴 Tiền đề của AC1 không tồn tại — thư viện thật không có Chương cỡ trung bình** — `epics.md:2128-2130` đòi đo trên *"một Chương thật"*. Khoảng 669 → 48.639 ký tự trong thư viện của Ice **rỗng hoàn toàn**; chỉ có fixture nhỏ và hai tài liệu ngoại lệ. "Thang" cắt nhân tạo mà Quyết định #6(b) dùng là khoảng đo tổng hợp, **không phải** một Chương thật theo nghĩa đen của AC. Điều này chỉ lộ ra giữa lúc thi hành, tức khâu thẩm định khả thi trước khi mở story đã hụt. Ba đường: ① chấp nhận thang nhân tạo và ghi rõ sự thay thế vào AC1 + báo cáo; ② Ice dựng/nhập một Chương thật cỡ thường nhật trước khi đo; ③ 2.4 về `backlog` tới khi có dữ liệu thật.
- [x] [Review][Decision] **🔴 AC10 đóng `:234` + `:570` về CHỮ, không về NỘI DUNG** — cả hai món nợ ghi đích danh *"chưa có phép đo nào về **tranh chấp CPU/I-O** giữa hai luồng checkpoint chạy song song"*. AC10 chép nguyên câu đó vào vế **Given**, nhưng vế **Nghiệm thu** chỉ đòi *"ghi kích thước `.db-wal` của cả hai kho theo thời gian"*. Đường cong kích thước WAL là bằng chứng về **hoạt động checkpoint**, không phải phép đo tranh chấp CPU/I-O. Rà cả Task 4 và Task 5 không có bullet nào đo CPU/I-O thật. Kèm theo: AC10 cũng không có cơ chế **ép** hai luồng trùng thời điểm — hai kho lệch pha tự nhiên suốt phiên thì AC10 vẫn xanh theo nghĩa đen mà câu hỏi gốc vẫn chưa ai trả lời. Hai đường: ① thêm phép đo tranh chấp thật *(thời gian CPU + độ trễ I-O, chạy chồng so với chạy riêng)* + kịch bản ép trùng pha; ② hạ tuyên bố xuống "đi qua, không đóng" và trả `:234`/`:570` về `deferred-work.md` kèm chủ có tên.
- [x] [Review][Decision] **AC5 mơ hồ ở đúng chỗ nó kích hoạt leo thang tầng PRD** — *"hai ngưỡng NFR2 và NFR18 không đạt được đồng thời"* đọc được hai kiểu: (i) **ít nhất một** ngưỡng trượt, hay (ii) **cả hai** trượt khi đo cùng lúc. Task 10 lặp lại y nguyên cụm đó chứ không giải. Vỡ khi: NFR2 xanh hoàn toàn nhưng NFR18 thỉnh thoảng vượt 5 s một mình — dev không biết đây là "dừng, báo Ice, đừng tự vá" hay "chỉ một cái trượt, tự tối ưu tiếp". Đây là cửa duy nhất đưa kết quả lên tầng PRD; để mơ hồ là để dev tự quyết một việc của chủ dự án.
- [x] [Review][Decision] **AC8 bị bác thì AC3 không có đường về** — AC8 nói rõ *"lệch ⇒ giả thuyết sai… báo, đừng vá"*, nhưng AC3 vẫn buộc *"chọn được một giá trị đạt cả hai ngưỡng"*. Nếu giả thuyết ở §Điều kiện khởi hành mục 7-8 *(`wal_threshold_bytes` chỉ đánh đổi với NFR2, không với NFR18)* sai, toàn bộ mô hình phân rã sụp và spec không nói dựa vào tiêu chí nào để chọn giá trị nữa.
- [x] [Review][Decision] **AC19 chốt ngưỡng lặp lại đúng bằng cỡ mẫu mà chính nó dẫn ra là đã bị bác** — AC19 đòi *"ít nhất một phép đo chạy **hai** lượt độc lập"*, đồng thời trích bài học C3 của Story 1.22: kết luận "ổn định" trên **n=2** rồi bị chính bộ e2e bác ở lượt thứ tám. Cảnh báo *"nói cỡ mẫu ra, đừng nói ổn định"* có, nhưng ngưỡng nghiệm thu vẫn dừng ở đúng n=2. Ice chốt: giữ n=2 kèm cấm mọi lời khai "ổn định", hay nâng sàn?
- [x] [Review][Decision] **Hai bàn đo cho trần 9.850 span chưa được nối khớp** — bảng số cũ *(300,1 ms Blink · 1.308,0 ms WebKit)* đến từ bàn đo **hai engine ngoài app** (Playwright, kiểu 2.2/2.3). Nhưng ĐÍNH CHÍNH của Quyết định #1 chốt đo **trong app Tauri thật** — tức **chỉ WKWebView**. Task 3 đòi *"đo lại… đặt cạnh số cũ"* mà không nói dùng bàn nào. Vỡ khi: dev chỉ đo lại WKWebView rồi đặt cạnh một số Blink không được đo lại — bảng so sánh lẫn số cũ *(synthetic, text rỗng)* với số mới *(text thật)* và không ai kiểm phép so sánh còn hợp lệ không.
- [x] [Review][Patch] 🔴 `sprint-status.yaml` khai sai trạng thái thật của story [_bmad-output/implementation-artifacts/sprint-status.yaml:9] — dòng log ghi *"chuyen sang **ready-for-dev** (create-story)"* trong khi giá trị thật ở `:34` là `in-progress`, và story ghi `Status: in-progress`. Nặng hơn: lý do chặn ghi mơ hồ là *"co dieu kien chan"*, nhưng §Dev Agent Record *(`:952`, chặn ở `:1277`)* cho thấy đây **không** còn là "điều kiện khởi hành chưa mở" — mà là **bản thân bộ đo đang bế tắc**: bốn lượt build hỏng, bốn giả thuyết bị bác, chưa tiêm được `bench.js` vào webview bản release. Người đọc sprint-status sẽ đánh giá sai hoàn toàn mức sẵn sàng.
- [x] [Review][Patch] 🔴 AC3 thu hẹp câu hỏi của hàng Deferred `:894` mà không có chỗ ghi sự thu hẹp [ARCHITECTURE-SPINE.md:894] — hàng Deferred đóng khung bài toán là `wal_threshold_bytes` ⟷ nhịp flush **đánh đổi lẫn nhau** để đạt **cả** NFR18 và NFR2. §Điều kiện khởi hành mục 7-8 tự kết luận NFR18 **không** treo trên `wal_threshold_bytes` *(vì `synchronous=FULL` đã đảm bảo bền)*, nên *"cặp đánh đổi thật của AC3 là `wal_threshold_bytes` ⟷ NFR2"*. Nếu AC8 xác nhận, khuôn `✅ Đã đóng` sẽ đóng một câu hỏi **hẹp hơn** câu hỏi gốc — và không cơ chế nào buộc ghi sự thu hẹp vào chính dòng đóng. Người đọc SPINE sau này tin nhầm trade-off gốc đã giải triệt để.
- [x] [Review][Patch] Trích dẫn `deferred-work.md:145` sai ~1.800 dòng [2-4-mui-tham-do-do-nfr18-va-nfr2-dong-thoi.md:AC20] — story trích `:145` cho luận điểm *"trọn phần Windows dời về cuối dự án — Ice chốt 2026-08-12"* ở cả §Điều kiện khởi hành mục 13, AC20 và §References. Nhưng `:145` thật là nợ của **Story 1.6**: *"Nghiệm thu DOM chạy trên Blink (Chrome), KHÔNG phải WKWebView"* — nói về **engine**, không về Windows. Câu được trích thật sự nằm ở **`:1954-1957`**. *(Đã kiểm tay.)*
- [x] [Review][Patch] AC9 đếm nhầm cái cần đếm, và tiêu chí loại mẫu có thể bẻ cong phân bố [2-4-...md:AC9] — ① Task 4 đòi *"≥ 20 lượt SIGKILL"* còn AC9 lại vứt lượt nào để lại `.db-wal` = 0 byte; bắn 20 lượt mà 8 lượt trúng ngay sau checkpoint thì còn **12 mẫu hợp lệ** nhưng Task 4 vẫn coi là xong — phải đòi ≥ 20 lượt **hợp lệ**. ② Một `SIGKILL` rơi đúng lúc app rảnh *(đã checkpoint xong, không còn gì chờ ghi)* để lại WAL gần rỗng — đó là một kết quả **THÀNH CÔNG** *(mất 0 giây công việc)*, không phải "kill trượt". Loại thẳng nhóm này đẩy phân bố đo được lệch về phía **xấu hơn** thực tế; cần tách "kill trượt" khỏi "kill trúng lúc rảnh" bằng dấu hiệu khác kích thước WAL.
- [x] [Review][Patch] AC2 đòi *"kết quả nhất quán"* mà không cho dung sai số [2-4-...md:AC2] — 20 lượt cho ra 19 lượt ≤ 2 s và 1 lượt = 4,9 s là "nhất quán" *(vì vẫn ≤ 5 s)* hay là bất ổn phải gắn cờ? Ghi **max** thay vì trung bình là đúng hướng nhưng chưa đủ — thiếu ngưỡng độ lệch cho phép.
- [x] [Review][Patch] AC13 đòi ghi *"điều kiện mở lại"* ảo hoá mà không cho ngưỡng số kích hoạt [2-4-...md:AC13] — đo lại ra 350 ms Blink / 1.400 ms WebKit *(nhích lên so với 300,1 / 1.308,0)*: dev không có căn cứ quyết đây là "vẫn dưới trần 1,4 s của Story 1.16" hay "đủ để mở Giai đoạn 3 sớm".
- [x] [Review][Patch] Task 5 *"dò `wal_threshold_bytes` theo lưới"* không có biên trên/dưới, bước nhảy, hay điều kiện dừng [2-4-...md:Task 5] — dev thử 2 điểm *(512 KiB, 16 MiB — vốn là hai điểm của AC8)* rồi coi là dò xong, hoặc dò hàng chục điểm tốn nhiều giờ. Cả hai đều thoả câu chữ.
- [x] [Review][Patch] Nhịp poll `stat()` của `.db-wal` không được gán số, trong khi mọi hằng đo khác đều bị ghim [2-4-...md:Quyết định #5a] — AC21 liệt kê ba nguồn chi phí bàn đo *(rAF · `stat()` · bộ bơm phím)* phải trừ ra hoặc nói ra, nhưng không ép định lượng nhịp poll **trước** khi chạy. Chọn 100 ms là tự làm nhiễu chính số NFR2 đang đo, và không phép kiểm nào bắt được.
- [x] [Review][Patch] Ràng buộc `idle_before_passive` > `EDITOR_IDLE_MS` chỉ là ghi chú câu chữ, không có cổng máy [2-4-...md:Task 5] — Task 5 nhắc *"đổi nhịp flush mà quên ràng buộc là làm luồng checkpoint đánh nhau"*, nhưng không có assertion/test nào giữ bất biến đó sau khi **cả hai** số được hiệu chỉnh theo số đo. Hai lượt sửa riêng biệt trong cùng Task 5 *(hạ `idle_before_passive` xuống 2,5 s, nâng `EDITOR_IDLE_MS` lên 2,2 s)* lọt hết cổng.
- [x] [Review][Patch] Không có luật dừng cho lớp lỗi *"bản thân bộ đo dựng không nổi"* [2-4-...md:Task 0] — Task 1.0 có luật dừng tường minh *(không gõ được ⇒ DỪNG, story về `backlog`)*, nhưng chỗ tiêm `bench.js` thất bại liên tiếp thì không. §Dev Agent Record chứng minh khoảng trống này bằng thực tế: 4 lượt rebuild, 4 giả thuyết bị bác *(`strings` · CSP · cache `build.rs` · `document.title`)*, và quyết định *"không đoán tiếp"* là suy luận tại chỗ chứ **không** phải luật viết sẵn. Lượt thi hành sau sẽ không dừng đúng lúc.
- [x] [Review][Patch] Khoảng mù Windows áp cho khái niệm NFR nhưng không áp cho chính các hằng số sắp bị đóng băng [2-4-...md:AC20] — sản phẩm thật của story là những con số cụ thể ghi cứng vào **một** `Tuning::default` dùng chung cho **cả** macOS và Windows *(Task 5: "đổi số chỉ ở chỗ khai")*. AC20 chỉ đòi một dòng nói NFR2/NFR18 nghiệm thu trên macOS. Không task nào mở một hàng Deferred cảnh báo rằng **các giá trị số vừa chốt** chưa được đo trên NTFS/WebView2 — đúng loại "khoá một giá trị sai cho một nền tảng chưa đo" mà AD-12/NFR14 tồn tại để ngăn.
- [x] [Review][Patch] Bộ lọc rAF loại 100% mẫu thì AC1 không có nhánh phán quyết [2-4-...md:Quyết định #3] — Quyết định #3 đòi ghi số mẫu bị loại và cảnh báo *"lọc 90% là phép đo hỏng"*, nhưng bỏ trống biên 100%: 0 delta hợp lệ trong toàn cửa sổ auto-save thì AC1 không xanh, cũng không hẳn đỏ — chỉ là **vô nghĩa**, và không có đường ra nào ngoài đo lại vô hạn.
- [x] [Review][Patch] Mô hình nhân quả §8 bỏ sót kênh tranh chấp khoá, làm yếu AC8 [2-4-...md:§Điều kiện khởi hành mục 8] — lập luận *"flush trả `Ok` là đã bền nên `wal_threshold_bytes` không ảnh hưởng NFR18"* chỉ xét độ bền dữ liệu **đã commit**. Nó không xét: ngưỡng nhỏ ⇒ checkpoint chạy dày ⇒ tranh `busy_timeout` với writer ⇒ **trễ thời điểm commit kế tiếp** — một kênh gián tiếp đẩy độ trễ flush về phía `EDITOR_HARD_CAP_MS`. AC8 chỉ nhìn *"mất bao nhiêu giây"*, không tách riêng "trễ do chờ khoá".
- [x] [Review][Patch] AC18 có cửa thoát không khoá [2-4-...md:AC18] — *"Đo, **hoặc** TRẢ LẠI kèm chủ mới có tên"*. Mọi Quyết định khác trong story đều có bảng đánh đổi bắt buộc cân trước khi chọn nhánh; riêng đây không nêu **khi nào** được phép chọn vế trả lại. Dev bỏ đo RSS 100 MB vì "không đủ thời gian", ghi một hàng nợ có chủ — thoả câu chữ, dù chưa từng thử đo. *(Nhẹ hơn báo cáo gốc: AC vẫn đòi chủ có tên + lý do.)*
- [x] [Review][Patch] Hai tham chiếu dòng lỗi thời sống ngay trong mã mà Task 5 sắp sửa [src/panels/editorFlush.ts:35] — cả `:35` và `:62` trỏ `ARCHITECTURE-SPINE.md:883`; vị trí thật hôm nay là `:894`, đúng như §Điều kiện khởi hành mục 12 tự xác nhận. Task 5 chỉ đòi *"sửa mọi doc-comment 'TẠM — chủ là Story 2.4' thành số đã đo + ngày + lệnh đo lại"* — không nói phải sửa luôn **số dòng sai bên trong** doc-comment đó. `:883` sẽ sống sót qua lượt sửa. *(Kiểm tay: hai chỗ, không phải một.)*
- [x] [Review][Patch] Mất tiêu điểm cửa sổ giữa phiên 30 phút không có cơ chế phát hiện [2-4-...md:Quyết định #2c] — rủi ro đã nêu đúng *("cửa sổ phải giữ tiêu điểm OS suốt phiên")* nhưng không có hàng rào phát hiện và loại phần mẫu hỏng, khác hẳn bộ lọc rAF của Quyết định #3. Thông báo hệ thống cướp focus ở phút 12: phím sau đó không hạ cánh, mà bộ lọc rAF *(chỉ loại delta không có input/flush/checkpoint)* không phân biệt được với "app đang nghỉ".
- [x] [Review][Patch] AC7 đặt ngưỡng *"nhãn thứ ba dùng quá hai lần"* mà không gán hệ quả bắt buộc [2-4-...md:AC7] — *"dùng quá hai lần là dấu hiệu mũi thăm dò chưa chạy đủ — nói ra thay vì gom lại"* không định nghĩa hành động khi ngưỡng bị vượt. Cả 6 số `Tuning` cùng nhận nhãn *"không đo được ở story này"* *(vượt xa 2)*: phải quay lại Task 0 / báo Ice, hay cứ ghi vào báo cáo rồi đóng story?

---

## Dev Notes

### Cái đã có, cái chưa có — đo ngày 2026-08-13

| Thứ | Trạng thái | Nguồn |
| --- | --- | --- |
| Vùng gõ Editor *(một câu tại một thời điểm)* | **ĐÃ CÓ** — đo 6/6 trên hai engine | `EditorPanel.vue` · 2.3 §Task 0.1 |
| Đường flush AD-35 đủ năm vế (a)…(e) | **ĐÃ CÓ**, vế *"xác nhận segment"* chờ Story 2.5 | `editorFlush.ts` · `lib.rs:350` |
| Thanh trạng thái *"Đã lưu N giây trước"* | **ĐÃ CÓ** | `src/StatusBar.vue` |
| `PRAGMA synchronous = FULL`, có lưới | **ĐÃ ĐO** = `2` | `store_contract.rs` |
| Bộ đếm checkpoint theo **vế kích hoạt** | **ĐÃ CÓ**, `pub`, **chưa có đường ra IPC** | `checkpoint.rs:58-70` · `mod.rs:677` |
| Móc chuyển hướng `$APPDATA` + Library root | **ĐÃ CÓ**, chỉ `debug + feature wdio` | `lib.rs:71-72`, `:94-96` |
| Bộ chạy test frontend | **ĐÃ CÓ** — `vitest` 4.1.10, 32 ca | `package.json` · `tests/frontend/**` |
| Bộ chạy trong WKWebView thật | **ĐÃ CÓ**, **chập chờn** (8 lượt: 6 xanh/2 đỏ), **không gõ được chữ** | `e2e/**` · `wdio.conf.mjs` |
| Sáu số `Tuning` đã hiệu chỉnh | **CHƯA** — story này | `mod.rs:229-243` |
| Ba hằng nhịp flush đã hiệu chỉnh | **CHƯA** — story này | `editorFlush.ts:43,56,78` |
| Ảo hoá danh sách dài | **CHƯA CÓ**, và **không** thuộc story này | SPINE`:899` |
| Cột `segment.status` · bảng `segment_version` | **CHƯA CÓ** — Story 2.5 · 2.6 | AD-31 |

### Ranh giới AD — cái story này được phép và không được phép

**AD-12** (`SPINE:159-163`) — *"Phải có ngưỡng kích thước WAL buộc checkpoint để `.db-wal` không
phình vô hạn khi gõ liên tục hàng giờ."* 🔴 Story này là chỗ *"hàng giờ"* được đo lần đầu. Và
`wal_autocheckpoint = 0` là một **quyết định** — đừng bật lại nó để "cho gọn"; đó là lật AD-12.

**AD-35** (`SPINE:419-425`) — cả năm vế, cộng hai mệnh đề dễ bỏ sót: flush *"đi qua **đúng
`store::Writer` nối tiếp** của AD-11"*, và *"chỉ được coi là xong **sau khi đã ghi vào WAL**"*.
⚠️ Story này **hiệu chỉnh con số** của AD-35, nó **không** đổi hình dạng. Đổi *"idle + trần cứng
không reset"* thành một hình dạng khác là lật một AD.

**AD-11** (`SPINE:153-157`) — một writer nối tiếp. Bàn đo **không** được mở một kết nối ghi thứ hai
vào `project.db` trong lúc app đang chạy. Đọc thì được *(WAL cho phép đọc song song)*; **ghi thì
không**.

**AD-31** (`SPINE:368-392`) — auto-save: trạng thái **không đổi**, `SegmentVersion` **không** tạo.
Đây là hợp đồng mà AC4 nói một thư viện editor phải tuân.

**AD-45** — hai lớp gác cho mọi móc chỉ-e2e. Nếu Quyết định #5 chốt dựng một lệnh IPC chẩn đoán, nó
đi **đúng** hai lớp đó, không một lớp.

**AD-1** (`SPINE:75-79`) — ngoại lệ *"văn bản đang gõ là state cục bộ frontend"* dừng ở **văn bản
đang gõ**. Bàn đo không được sinh ra một quy tắc nghiệp vụ nào ở TS.

### Chuẩn của một mũi thăm dò trong kho này

- **Mã dùng một lần KHÔNG vào repo.** Tiền lệ Story 1.1 (app thăm dò trong scratchpad) và Story 2.3
  (`probe-typing.mjs` · `probe-paste.mjs` · `probe-plaintext3.mjs` — scratchpad, ghi tên trong
  §Debug Log, không phải tạo tác của story).
- **Công cụ đo chạy từ `npx`, ngoài `package.json`** — khuôn Story 2.2 và 2.3 với Playwright.
- **Ghi cả lượt đo HỎNG.** Story 2.3 ghi **ba** lượt đo hỏng trước khi có một số đúng, và bài học
  nằm ở đó: `page.setContent()` không thực thi lại `<script>` nội tuyến ⇒ ba biến thể đo lại một cây
  DOM **đã tháo**. Nếu tin số đó, story đã đi vá một khuyết tật **không tồn tại**.
- **Một kết quả không tự nhất quán là dấu hiệu phép đo hỏng**, không phải engine hành xử vậy — điều
  tra thay vì ghi.
- **Nói cỡ mẫu.** Không viết *"ổn định"* trên n=2 (bài học C3 của Story 1.22).

### Bài học Epic 1 · Story 2.1 · 2.2 · 2.3 áp thẳng vào story này

1. **Đo trước khi tin** (retro §7.1) — cả story này *là* bài học đó.
2. **Kiểm điều kiện đo trước khi lật một quyết định** — số đọc theo trạng thái hiện tại có thể bác
   oan một quyết định đúng. Ở đây: một bảng NFR2 đỏ đọc từ bản **debug** sẽ bác oan cả AD-35.
3. **Cổng mới phải vào CI** (retro §4) — chỉ áp nếu story này sinh ra một cổng; một mũi thăm dò
   thường không sinh.
4. **Một luật ngoài đơn hàng phải ghi ra và lật được** — nếu bàn đo sinh ra một quy ước mới *(ví dụ
   định nghĩa "frame")*, ghi `deferred-work.md` với chủ là Ice và *"chỗ lật là một dòng"*.
5. **Dev không sửa tài liệu quy hoạch** — trừ hai ngoại lệ AC cấp (§mục 12).
6. **`in-progress` không phải chỗ đậu** (retro §8.2) — để dở thì ghi **nguyên nhân cụ thể**.
7. **Năng lực chưa dựng ≠ lệch spec** — vế *"xác nhận segment"* của AD-35 vẫn thuộc Story 2.5. Ghi
   nợ có chủ, đừng sửa `epics.md`.
8. **Ký hiệu cấm** — emoji "biển cấm" `U+26D4` đã gỡ khỏi toàn kho, 0 còn lại. Viết `không`/`KHÔNG`
   thẳng.
9. **Gặp một lượt đỏ không tái lập được thì BẮT NGUYÊN VĂN TRƯỚC** — action item còn `open` của
   Epic 1. Áp cho cả bộ e2e lẫn bàn đo của story này.
10. **Lật một quyết định thì lật đúng ĐIỀU KIỆN của nó** — cửa NFR15 chặn bằng một **quy trình**,
    không bằng một mệnh đề *"không được dùng thư viện"*. AC4 đi qua cửa, không phá cửa.
11. 🔵 **Một bản vá chữa triệu chứng không trung tính** — bài học đắt nhất của 2.3: bản vá cho ①
    *(gõ ngược)* đẻ ra ④ *(chữ biến mất khỏi màn hình)*, vì nó giữ chuỗi đứng yên mà không chữa
    nguyên nhân *(hai chủ sở hữu cho một text node)*. Áp thẳng vào Task 6: **số trước, vá sau**.

### Git intelligence — 5 commit gần nhất

`6a4e6b8` *(HEAD)* Story 2.3 phần test frontend — `editorTypingZone` + `statusBar` · `6a9777b` Story
2.2 hạ cánh *(`editorGutter.ts` + `editorSegments.ts` + `editorPanelState.ts`, `EditorPanel.vue`
39 → 519 dòng, cột `target_text` bước 6, Kiểm I + Kiểm J)* · `c86c2fb` Story 2.1 *(bảng `segment`,
bước 5, `core/segment/split.rs`)* · `f950332` mở lại `push` + `pull_request` trong CI · `8ae61cd`
thoát chuỗi PowerShell trong step đo `.msi`.

Đọc được từ đó: ba lượt gần nhất đi **cùng một cặp** — `src-tauri/src/core/**` + `commands/segment.rs`
và `src/panels/**`. Story này đi **khác hẳn**: nó chạm rất ít mã và sinh chủ yếu **tài liệu** —
đúng hình dạng commit của Story 1.1 (`research/font-spike-results-*.md` + hai bảng của SPINE).
Khuôn thông điệp commit: `<type>(<scope>): <câu tiếng Việt mô tả điều đã thay đổi>`.

### Phụ thuộc — con số phải giữ

**Runtime: 0 gói mới** *(trừ khi AC4 chốt ngược và Ice ký)*. Ba gói giữ nguyên, ghim chính xác:
`@tauri-apps/api 2.11.1` · `dockview-vue 7.0.4` · `vue 3.5.40`.

**Dev: 0 gói mới.** Bộ hiện tại đáng nhắc: `vitest 4.1.10` · `@vue/test-utils 2.4.11` ·
`happy-dom 20.11.2` · TypeScript 5.9.3 · Vite 8.2.0 · `@wdio/* 9.30.1` · toolchain Rust **1.97.1**
*(ghim đúng số máy Ice đang chạy — `@stable` sẽ trôi và làm số đo hết so sánh được; ở một story đo
hiệu năng thì đó không phải chi tiết)*.

Số thật sau lượt sửa cổng của 2.3: **522** gói npm đã cài · **326** crate Rust.

### Thông tin kỹ thuật mới nhất — và chỗ duy nhất nó có giá trị

Story này **không thêm phụ thuộc nào** và không chạm API bên ngoài nào, nên phần này ngắn — trừ AC4.

**Ứng viên thư viện editor, tra 2026-08-13** *(dùng cho Quyết định #7, không phải một khuyến nghị
cài)*: **CodeMirror 6** — MIT, dòng 6.x *(6.0.2 ở gói gộp `codemirror`; các gói `@codemirror/*` phát
hành độc lập)* · **ProseMirror** — MIT · **Lexical** — MIT · **Tiptap** *(nhân)* — MIT; 2025 Tiptap
mở mã 10 extension Pro cũ về MIT và dồn phần thu phí về nền tảng Cloud · **Quill** — BSD-3-Clause ·
**Slate** — MIT. Cả sáu tương thích GPL-3.0-or-later theo chiều đi vào.

🔴 **Nhưng giấy phép không phải trục quyết định ở đây** — trục là **quyền sở hữu text node**
(Quyết định #7). Và nếu một gói được chọn, **lời khai `license` trong `package.json` không đủ**:
NFR15 đòi mở tệp giấy phép **trong nguồn đã tải**. Bài học 2.3: `LICENSE.md` của `vitest` dài **811**
dòng và khai giấy phép của **27 gói vendor** *(24 MIT · 2 BSD-3-Clause · 1 ISC)* mà trường `license`
không nói một chữ nào.

⚠️ **Một ràng buộc nền tảng không tra được từ tài liệu, chỉ đo được**, và nó là trung tâm của cả
story: `requestAnimationFrame`, `PerformanceObserver`/`longtask`, và nhịp `fsync` **không** giống
nhau giữa WKWebView/APFS và WebView2/NTFS. Dự án chạy trên WKWebView hôm nay. ⇒ mọi kết luận của
story này mang một chú thích nền tảng, và AC20 buộc viết nó ra.

---

### Project Structure Notes

Tạo tác **mới** story này dự kiến sinh:

```
_bmad-output/planning-artifacts/research/editor-perf-spike-results-2026-08-XX.md   # báo cáo (AC6)
_bmad-output/implementation-artifacts/2-2-ban-do/*.png                              # chụp lại 3 ảnh (AC17)
```

Tệp **sửa** *(dự kiến — hình dạng cuối theo phán quyết Task 0 và theo số đo)*:

```
src-tauri/src/core/store/mod.rs        # Tuning::default + doc-comment "TẠM" → số đã đo
src/panels/editorFlush.ts              # ba hằng + doc-comment "TẠM" → số đã đo
ARCHITECTURE-SPINE.md                  # bảng Deferred: đóng :894 và :897 (+ bảng Stack nếu AC4 chốt "có")
architecture/.../.memlog.md            # một dòng (version) + một dòng (decision)
_bmad-output/implementation-artifacts/2-2-ban-do-editor.html   # sửa lời khai NFR15 ở dòng 11
_bmad-output/implementation-artifacts/deferred-work.md         # đóng mười món, mở món mới KÈM CHỦ
src/panels/EditorPanel.vue             # CHỈ nếu Task 6 kết luận cần vá, và chỉ ba đường đã nêu tên
```

🔴 **Không** đụng: `src/layout/writeSchedule.ts` *(hằng của bố cục — Story 1.14; hành vi —
`check-layout.mjs` Kiểm B)* · `core/store/checkpoint.rs` **hành vi** *(chỉ số ở `Tuning` được đổi)* ·
`core/segment/split.rs` *(AD-4)* · `capabilities/main.json` *(`config_invariants.rs:333` khoá ở đúng
ba quyền)* · các phép kiểm hành vi trong `check-layout.mjs` Kiểm B và `check-commands.mjs` Kiểm
C/D/E · `epics.md` · `DESIGN.md` · `EXPERIENCE.md` · `prd.md` *(lượt riêng của Ice)*.

**Sàn `*_FLOOR` — số THẬT đo 2026-08-13** *(chỉ phải rà nếu story thêm tệp sản phẩm; một mũi thăm dò
thường không thêm)*:

| Cổng · sàn | Giá trị | Quần thể thật hôm nay |
| --- | --- | --- |
| `check-commands.mjs` `TS_FLOOR` (`:219`) | 30 | **36** tệp `.ts` trong `src/**` |
| `check-commands.mjs` `VUE_FLOOR` (`:211`) · `check-i18n.mjs` (`:289`) | 13 | **16** tệp `.vue` |
| `check-commands.mjs` `COMMAND_FLOOR` (`:226`) | 29 | — |
| `check-commands.mjs` `CLICK_FLOOR` (`:244`) · `DISPATCH_FLOOR` (`:245`) | 17 · 23 | — |
| `check-commands.mjs` `SELECTION_SURFACE_FLOOR` (`:1908`) | 7 | ⚠️ **đừng gỡ** lời gọi ở `EditorPanel.vue:67` |
| `check-i18n.mjs` `RS_FLOOR` (`:279`) | 36 | **42** tệp `.rs` *(chú thích tại chỗ còn ghi 43 — số cũ, cổng đếm lúc chạy nên nó không sai lệch phép kiểm)* |
| `check-layout.mjs` `FILE_FLOOR` (`:97`) | 43 | **52** tệp `src/**` |
| `check-tokens.mjs` `FILE_FLOOR` (`:91`) · `COMPONENT_FILE_FLOOR` (`:92`) | 45 · 43 | — |

🔵 **`tests/frontend/**` thêm 0 vào quần thể cả bốn cổng** — Quyết định #6 của Story 2.3, đã xác
nhận bằng số. Tệp bàn đo của story này sống **ngoài kho** nên cùng tính chất.

Quy ước đặt tên đã đo: Rust `snake_case` · Vue `PascalCase.vue` · state của panel là
`<tênPanel>State.ts` cùng thư mục · khoá i18n phẳng theo dấu chấm có tiền tố miền · command trên dây
`snake_case`, tham số `camelCase` · tên hàm test Rust là một câu mô tả hành vi, `snake_case`, **không**
tiền tố `test_`.

---

### References

- AC nguyên văn — `_bmad-output/planning-artifacts/epics.md:2118-2149`
- NFR2 — `epics.md:326` · NFR18 — `epics.md:368` · hợp đồng flush dạng bảng — `:415` · hàng *"Ngưỡng WAL + nhịp flush"* trong bảng NFR — `:454` · ghi chú cài đặt Epic 2 *(mũi thăm dò bắt buộc)* — `:830-836`
- AD-1 — `ARCHITECTURE-SPINE.md:75-79` · AD-4 — `:95-101` · **AD-11 — `:153-157`** · **AD-12 — `:159-163`** · AD-21 — `:302-306` · AD-31 — `:368-392` · AD-34 — `:406-417` · **AD-35 — `:419-425`**
- **Hàng Deferred phải đóng** — `ARCHITECTURE-SPINE.md:894` *(ngưỡng WAL + nhịp flush)* · `:897` *(thư viện editor)* · **hàng KHÔNG đóng** — `:899` *(ảo hoá danh sách dài)*
- Khuôn đóng một hàng Deferred *(gạch ngang + ✅ Đã đóng)* — hàng HVTĐTD và hàng FR115, cùng bảng
- `Tuning` sáu số tạm + doc-comment *"chủ sở hữu là Story 2.4"* — `src-tauri/src/core/store/mod.rs:62-68`, `:175-243` · `Store::write` — `:612-618` · `Store::checkpoint_stats` — `:677`
- Luồng checkpoint, hai vế kích hoạt — `core/store/checkpoint.rs:286-315` · `CheckpointStats` — `:58-70`, `:163-170` · PASSIVE/TRUNCATE — `:329-390` · vì sao PASSIVE ở nền và TRUNCATE chỉ lúc đóng — `:1-34`
- Ba PRAGMA + luật *"đặt rồi ĐỌC LẠI"* — `core/store/pragmas.rs` · `synchronous = FULL` có lưới — `src-tauri/tests/store_contract.rs::the_write_connection_fsyncs_the_wal_on_every_commit` · ngưỡng WAL thu nhỏ 64 KiB — `store_contract.rs:589-615`
- Ba hằng nhịp flush — `src/panels/editorFlush.ts:43`, `:56`, `:78` · hình dạng AD-35 — `src/layout/writeSchedule.ts:31-97` · **cặp hằng của bố cục, KHÔNG đụng** — `:57`, `:67` · bảng hai chỗ dùng — `:32-33` · cổng đứng trên nó — `scripts/check-layout.mjs:288`
- Ba đường nóng — `src/panels/EditorPanel.vue:892` *(`:data-caret`)* · `:294`, `:300` *(`restoreEditedText`)* · `:565`, `:602`, `:636` *(`nearestSentenceTo`)*
- Móc e2e hai lớp AD-45 — `src-tauri/src/lib.rs:60-144` · `default_library_root` — `src-tauri/src/commands/project.rs:61-80` · `wire_exit_flush` — `lib.rs:343-350` · `RunEvent::Exit` — `:272-278`
- `[profile.release]` *(`opt-level = "s"`, `lto`, `panic = "abort"`, `strip`)* — `src-tauri/Cargo.toml`
- Fixture workspace + hai lựa chọn có chủ ý — `e2e/support/workspace.mjs` · giới hạn bộ e2e *(chập chờn, `element.click()` không trung thực, `$APPDATA`/Library root)* — `e2e/wdio.conf.mjs` §Giới hạn · chuột thật — `e2e/support/pointer.mjs`
- **Khuôn báo cáo mũi thăm dò** — `_bmad-output/planning-artifacts/research/phase-0-spike-results-2026-08-02.md` · `font-spike-results-2026-08-03.md` · §Tiền lệ cần theo của Story 1.1 — `1-1-mui-tham-do-font-do-dung-luong-that-va-ra-giay-phep.md:254-273`
- **Story trước** — `2-3-hop-dong-flush-va-trang-thai-da-luu.md` *(đặc biệt §ĐÍNH CHÍNH đầu tệp · §Debug Log Task 0.1 · §Bảng AC25 · §Nghiệm thu cuối)* · `2-2-panel-editor-lien-mach.md` · bàn đo — `2-2-ban-do-editor.html` · `2-3-ban-do-vung-go.html`
- **Nợ có chủ là story này** — `deferred-work.md:201-212` · `:214` · `:234` · `:570` · `:591` · `:2084-2090` · `:2167-2168` · `:2419` · `:2441` · `:2449`
- **Nợ đi qua, chủ khác** — `deferred-work.md:2290-2297` *(ca e2e còn đỏ — Story 2.3 tiếp)* · `:2317-2340` *(phán quyết AD-34 — Ice)* · `:145` *(mọi bằng chứng chỉ macOS)*
- Bài học Epic 1 — `epic-1-retro-2026-08-11.md` §4, §5, §7.1, §8.1, §8.2 · action item còn `open` — `sprint-status.yaml` §action_items
- [Web 2026-08-13] npm `codemirror` — MIT, dòng 6.x · [Web 2026-08-13] `tiptap.dev` release notes + `@tiptap/core` npm — MIT, 10 extension Pro mở mã năm 2025

---

## Dev Agent Record

### Agent Model Used

*(điền lúc thi hành)*

### Phán quyết Task 0 — ghi TRƯỚC phép đo đầu tiên

| # | Quyết định | Phán quyết | Số đo chống lưng |
| --- | --- | --- | --- |
| #1 | bản build được đo + hàng rào dữ liệu thật | **(d) có ĐÍNH CHÍNH** — Rust release nguyên vẹn + `$HOME` nháp; frontend là bản Vite production **cộng một module bàn đo không commit**; (a) chạy một lượt lấy hệ số debug↔release | đề xuất (d) **thiếu một đường đọc số** — xem §ĐÍNH CHÍNH #1 dưới |
| #2 | cái gì lái 30 phút gõ | **(c) `CGEventPost`/`osascript` làm đường chính, (a) Ice gõ tay một lượt ngắn** — giữ nguyên đề xuất | `browser.keys()` đã bị loại bằng đo (`deferred-work.md:2334-2337`); (b) bỏ qua đường bàn phím engine |
| #3 | "frame" lấy mẫu bằng gì | **(a) delta rAF làm số nghiệm thu · (c) `performance.now()` quanh handler `input` làm số chẩn đoán · (b) `longtask` chỉ sau khi đo là nó tồn tại trong WKWebView** | giữ nguyên đề xuất; bộ lọc + **số mẫu bị loại** bắt buộc in ra |
| #4 | "mất 5 giây" đo từ mốc nào | **(a) đồng hồ tường** — mỗi lượt bơm mang chỉ số đơn điệu tăng + nhật ký thời điểm bơm; đọc lại bằng `sqlite3` chỉ đọc | giữ nguyên đề xuất |
| #5 | đọc trạng thái checkpoint bằng đường nào | **(a) `stat()` `.db-wal` CẢ HAI kho theo nhịp làm số nghiệm thu · (c) dòng chẩn đoán làm số đối chiếu** | (b) **không dùng được**: nó sống sau `cfg(debug_assertions)`, mà #1 chốt đo trên release |
| #6 | Chương thật nào được đo | **(c) cả hai; (b) Chương cỡ thường nhật là số nghiệm thu AC1** | giữ nguyên đề xuất; cỡ của (b) lấy từ dữ liệu thật của Ice, ghi nguồn |
| #7 | thư viện editor | *(chốt ở Task 7, SAU khi có số AC1 + AC12 + AC13 — đề xuất mặc định (a) "không thư viện", điều kiện lật đã ghi)* | chưa có số ⇒ **chưa ký**; ký sớm là hợp thức hoá hiện trạng |

#### 🔴 ĐÍNH CHÍNH Quyết định #1 — đường (d) như story mô tả KHÔNG đọc ra được số NFR2

Đề xuất mặc định của story chọn (d) *(release đúng như phát hành)* làm đường chính, và lý lẽ về
**dữ liệu** của nó đứng vững. Nhưng nó bỏ sót một vế, và vế đó chặn: **trên bản release không có
đường nào tiêm bộ lấy mẫu frame vào trang, cũng không có đường nào đọc mẫu ra.** Ba lớp khoá, cả
ba đọc được từ cấu hình chứ không phải suy đoán:

| Lớp | Chỗ khai | Hệ quả |
| --- | --- | --- |
| Không có feature `devtools` | `Cargo.toml:33` — `tauri = { version = "=2.11.5", features = ["protocol-asset"] }` | Web Inspector **không** gắn được vào WKWebView |
| Không có plugin WebDriver | `Cargo.toml` §`[features]` — `wdio` **không** trong `default`, và `default` cố ý rỗng | không `browser.execute` |
| CSP chặn mọi đường mạng | `tauri.conf.json` — `connect-src 'self' ipc: http://ipc.localhost` | bàn đo **không** POST được số ra một cổng localhost |

⇒ **Đường (d) nguyên bản tự mâu thuẫn với Quyết định #3(a).** Đây là một phát hiện của lượt thi
hành, không một lượt đổi ý: nó được tìm ra bằng cách **đọc cấu hình lúc chuẩn bị bàn đo** — đúng
bài học Story 1.22 mà chính §Điều kiện khởi hành mục 9 trích *(«liệt kê mọi đường ghi TRƯỚC khi
kill»)*, áp sang đường **đọc**.

**Phán quyết — (d) giữ nguyên ở vế đắt nhất, và vế còn lại được nói ra thay vì giấu:**

1. **Rust: release nguyên vẹn.** `[profile.release]` **không** đổi một dòng *(`opt-level = "s"`,
   `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`)*, `Cargo.toml` **không**
   đổi một dòng, **không** bật `wdio`, **không** bật `debug-assertions`. Đây là vế mà **NFR18 đứng
   trên** — cửa sổ mất dữ liệu là chuyện của đường ghi Rust, và nó được đo trên đúng nhị phân sẽ
   phát hành.
2. **Frontend: bản Vite production, cộng một module bàn đo KHÔNG commit.** Lý lẽ đo được: NFR2 là
   một mệnh đề về **luồng chính của webview**, và mã chạy ở đó do **Vite** biên dịch, không do
   `rustc`. Bộ lấy mẫu sống trong `scratchpad`, tiêm vào `dist/` sau `npm run build`, đúng tiền lệ
   *"mã dùng một lần KHÔNG vào repo"* (Story 1.1 · `probe-typing.mjs` của 2.3).
3. **Đường đọc số ra: `put_config` — một lệnh IPC ĐÃ CÓ, không một lệnh mới.** Bàn đo gọi nó
   **đúng một lần, SAU khi đã ngừng lấy mẫu**, rồi mình đọc lại bằng `sqlite3` chỉ đọc từ
   `global.db` trong `$HOME` nháp. ⇒ **0 dòng mã sản phẩm mới**, và Quyết định #5(b) *(dựng một
   lệnh IPC chẩn đoán)* **không cần dựng** — đó là lý do #5 chốt (a).
4. **Cái giá, ghi ra chứ không nuốt:** trang được đo mang thêm một vòng rAF mà bản phát hành không
   có. Đó **chính là** thứ AC21 buộc đo — lượt đối chứng *(cùng bàn đo, cùng thời lượng, không gõ)*
   và lượt đổi nhịp lấy mẫu là phép trừ cho vế này, không phải một thủ tục cho đủ.

⚠️ Đường (c) *(release + `debug-assertions` + `wdio`)* giữ làm đường lui **chỉ** nếu hàng rào chiều
âm của (d) không dựng được — và nếu phải dùng, độ lệch codegen của nó phải được **đo**, không đoán.

**Hàng rào `$HOME` nháp — đọc mã trước, nhưng nghiệm thu bằng máy:** trên macOS cả
`app.path().app_data_dir()` lẫn `app.path().document_dir()` đi qua `dirs::…` →
`dirs_sys-0.5.0::home_dir()`, và hàm đó đọc `$HOME` **trước tiên** *(`dirs-6.0.0/src/mac.rs` —
`data_dir()` = `home_dir()/Library/Application Support`, `document_dir()` = `home_dir()/Documents`;
`tauri-2.11.5/src/path/desktop.rs:75`, `:108`)*. 🔴 Đó là **đọc mã, chưa phải phép đo** — luật của
kho là *"đặt rồi ĐỌC LẠI"*, nên hàng rào chiều dương **và** chiều âm chạy **trước lượt kill đầu
tiên**, không sau.

### Debug Log References

#### Task 1.0 — kiểm đầu vào bằng TAY (2026-08-13) · 🟢 CỬA MỞ

**Ice gõ tay trong `npm run tauri dev`, bấm vào một câu CHƯA DỊCH, chữ HẠ CÁNH.**

⇒ Theo bảng đường đi của §Điều kiện khởi hành mục 2: ca e2e còn đỏ ở
`e2e/specs/editor-typing-flush.e2e.mjs:133` là một **giới hạn của bộ đo** đã có tên, **không** một
khuyết tật sản phẩm. Story 2.4 **đi tiếp**.

🔴 **Và điều đó phải đi vào báo cáo, không được nuốt:** mọi phép đo NFR2/NFR18 của story này đứng
trên một bề mặt mà **bộ e2e chưa lái được**. Hệ quả cụ thể: bàn đo **không** dùng được
`browser.keys()` *(chỉ `keydown`, không `beforeinput` — `deferred-work.md:2334-2337`)*, nên
Quyết định #2 phải tìm một đường lái khác, và lượt xác nhận cuối cùng vẫn là tay người.

⚠️ Lượt kiểm này **không** đóng ca e2e đỏ. Chủ của nó vẫn là **Story 2.3 (tiếp)**
(`deferred-work.md:2290-2297`) — story 2.4 đi qua, không nhận.

#### Task 1 — hàng rào dữ liệu thật · 🟢 CẢ HAI CHIỀU ĐỨNG (2026-08-13)

Chạy **trước** lượt kill đầu tiên, đúng thứ tự §Điều kiện khởi hành mục 9 đòi.

| Phép đo | Lệnh | Kết quả |
| --- | --- | --- |
| Ảnh chụp gốc dữ liệu thật | `fence.sh snap before-everything` | **116** dòng *(SHA-256 + cỡ + mtime từng tệp của `~/Documents/AuraTranslate/` và `$APPDATA` thật)* |
| 🔴 **`$HOME` nháp có chuyển hướng bản RELEASE không** | `HOME=<nháp> AuraTranslate.app/Contents/MacOS/auratranslate` | 🟢 `global.db` + `-shm` + `-wal` hạ cánh trong `$DRAFT/Library/Application Support/com.auratranslate.desktop/` |
| Hàng rào chiều âm sau lượt khởi động | `fence.sh diff before-everything after-boot` | 🟢 **y nguyên từng byte** |
| **Hàng rào tự kiểm — ca ĐỎ cố ý** | thêm một dòng giả vào ảnh chụp rồi `diff` | 🔴 **bắt được**, mã thoát 1 |

🔵 **Vế thứ ba mới là vế đáng ghi:** hàng rào được nghiệm thu **đỏ-rồi-xanh**, không chỉ xanh. Một
hàng rào chỉ từng xanh là một hàng rào chưa ai biết nó có bắt được gì không — đúng lớp lỗi *"xanh
rỗng"* mà AC15 của Story 2.3 đặt tên. Và nó **đã bắt một lỗi thật ngay lượt đầu**: bộ lọc
`grep -v '^# snapshot'` không khớp dòng `diff` *(chúng bắt đầu bằng `-#`/`+#`)* ⇒ hàng rào báo vỡ
oan. Sửa thành `grep -vE '^[+-]# snapshot'`.

⚠️ Bề mặt **thứ hai** *(`document_dir()` → `~/Documents/AuraTranslate/`)* **chưa** được nghiệm thu
ở lượt này, và lý do là một dữ kiện chứ không một thiếu sót: app **không** chạm `document_dir()`
lúc khởi động, nó chỉ chạm khi có một Tác phẩm được tạo. Nghiệm thu vế đó đi cùng lượt bàn đo tạo
Tác phẩm đầu tiên, **vẫn trước lượt kill đầu tiên**.

#### Quyết định #2 — cửa Accessibility ĐÃ MỞ

`osascript -e 'tell application "System Events" to keystroke "a"'` ⇒ **mã thoát 0**, không hộp xin
quyền. ⇒ đường (c) *(bơm phím ở tầng hệ điều hành)* chạy được, không phải xin Ice cấp quyền.

#### 🔴 Task 2 — dữ liệu thật của Ice KHÔNG CÓ "Chương cỡ thường nhật", và đó là một phát hiện

Quyết định #6 đường (b) đòi *"cỡ lấy từ dữ liệu thật của Ice, không phải một số tròn do dev nghĩ
ra"*. Đo trọn thư viện thật — **30** `.atproj`, đọc bằng `sqlite3 -readonly`:

| Hạng | Cỡ `chapter.source_text` | Là cái gì |
| --- | --- | --- |
| `Thieu Chuu.atproj` | **72.861** ký tự | một **bảng tra** Hán-Việt, mỗi dòng một chữ — không phải văn xuôi |
| `Epochtime (2).atproj` | **48.639** ký tự | tài liệu thương hiệu tiếng Việt · **đúng Chương của bảng 9.850 câu ở 2.2** |
| 5 tệp tin tức | 668–669 ký tự | fixture thử |
| phần còn lại | 237–350 ký tự | fixture thử |

🔴 **Khoảng trống nằm ở giữa: từ 669 tới 48.639 ký tự KHÔNG có một Chương thật nào.** Thư viện của
Ice hôm nay là *fixture thử* + **hai** tài liệu lớn, không phải một kho tác phẩm dịch.

⇒ **Hệ quả cho AC1, nói thẳng:** mẫu nghiệm thu *"Chương cỡ thường nhật"* **không xác định được từ
dữ liệu của Ice**, và dev **không** được phép bịa một số tròn. Đường đi đã chọn, ghi ra trước khi
đo: đo **một thang** cắt từ **chính văn bản thật** *(tiền tố của `Epochtime (2)`)* thay vì một điểm
— và để Ice đọc **đường cong** rồi chọn điểm nghiệm thu, thay vì để dev chọn hộ. Việc chọn điểm là
một hàng của §Cần Ice quyết (AC5 · AC6 mục ⑦).

#### Task 1 — bàn đo: HAI lượt đo HỎNG trước khi có một cổng đúng

Ghi cả lượt hỏng, đúng §Chuẩn của một mũi thăm dò *(Story 2.3 ghi ba lượt hỏng trước khi có một
số đúng, và bài học nằm ở đó)*.

**Lượt hỏng ① — cổng `strings` là một cổng ÂM TÍNH GIẢ.** Cổng nghiệm thu đầu tiên của lượt tiêm
là `strings <nhị phân> | grep '__bench__'`. Nó báo **đỏ**. Nhưng nó **không thể** xanh: `[profile.
release]` khai `strip = true`, và Tauri **nén** bộ tài nguyên nhúng ⇒ chuỗi không bao giờ nằm dạng
thô trong nhị phân. Một cổng chỉ biết báo đỏ là một cổng **không nói gì**.

**Lượt hỏng ② — và cổng hỏng che mất một lỗi THẬT nằm ngay dưới nó.** Chạy app rồi chụp màn hình:
ô bàn đo **không** hiện. Nguyên nhân: một lượt đổi **chỉ trong `dist/`** không làm `cargo` coi cây
là bẩn ⇒ nó **không dựng lại** ⇒ nhị phân giữ nguyên bộ tài nguyên nhúng **cũ**, và lượt tiêm biến
mất **im lặng**. Vá: `touch src-tauri/src/lib.rs` trước `tauri build`.

**Lượt hỏng ③ — một tệp script RIÊNG cùng origin vẫn KHÔNG chạy, và nguyên nhân CHƯA được đặt tên.**
Sau khi ép dựng lại: nhị phân **mới hơn** `dist/` *(13:09 so với 13:06 ⇒ nó đã nhúng lại)*, thẻ
`<script src="/bench.js">` **có mặt** trong `dist/index.html`, `dist/bench.js` **có mặt** — và dấu
sống vẫn **vắng**, trong khi `config_value` của chính lượt chạy đó có `app_config|mode` và
`app_config|workspace_layout` *(tức đường `put_config` chạy tốt)*.

🔴 **Giả thuyết hàng đầu là CSP** *(`script-src 'self'`, `tauri.conf.json`)*, **nhưng nó CHƯA được
đo** — `'self'` theo chuẩn thì cho phép một script cùng origin, nên nếu CSP là thủ phạm thì Tauri
đã viết lại chỉ thị đó theo một cách chưa ai ở đây kiểm. **Không đặt tên cho một nguyên nhân chưa
đo** *(luật của kho, và action item còn `open` của Epic 1: gặp một lượt đỏ thì bắt nguyên văn
trước)*. Ghi lại là một khoảng chưa chẩn đoán.

⇒ **Đường đi:** tiêm vào **đuôi chính bundle mà app vốn đã nạp** *(`dist/assets/index-*.js`)*.
Không thêm một **nguồn** script nào ⇒ không cửa CSP nào phải qua. Vá `__TAURI_INTERNALS__.invoke`
vẫn chạy vì `@tauri-apps/api` tra global đó **lúc gọi**, không giữ tham chiếu lúc nạp.

🔵 **Và lượt hỏng ③ có một mặt DƯƠNG cho sản phẩm, phải nói ra:** một script lạ thả vào thư mục
tài nguyên **không** thực thi được trong bản phát hành. Dù cơ chế chưa được đặt tên, **kết quả**
là thứ AD-15 và CSP tồn tại để cho. Đó là một dữ kiện có lợi, không phải một trở ngại — bàn đo
phải đi vòng, còn sản phẩm thì đứng.

⇒ **Cổng mới, bằng máy chứ không bằng mắt:** bench ghi một **dấu sống** qua `put_config` ngay khi
`__TAURI_INTERNALS__` sẵn sàng; script dựng chạy app trong một `$HOME` nháp riêng rồi hỏi
`global.db` bằng `sqlite3`. Có dấu ⇒ bench **đã chạy trong webview thật của bản release**. Đây là
một mệnh đề kiểm được, không phải một lượt nhìn ảnh.

🔵 Và lượt hỏng ② tự nó là **hàng rào chiều dương cho bề mặt THỨ HAI**, thu được miễn phí: ảnh chụp
cho thấy app khai *"Library chưa có Tác phẩm nào"* trong khi thư viện thật của Ice có **30**
`.atproj`. ⇒ `document_dir()` **đã** bị `$HOME` nháp chuyển hướng, đo trên **bản release**, không
suy từ `dirs-6.0.0/src/mac.rs`.

#### Mốc gốc AC15 — đo TRƯỚC khi đổi một hằng nào

`npm run test` ⇒ **40/40 ca xanh / 6 tệp** *(vitest 4.1.10, 1,70 s)*.

⚠️ **Dev Notes của story ghi 32, và số đó đã TRÔI.** Sàn của AC15 *(«≥ 32/32»)* vẫn đứng, nhưng
quần thể thật hôm nay là **40** — mọi lượt nghiệm thu sau khi đổi hằng phải đối chiếu với **40**,
không với 32, nếu không một lượt mất 8 ca sẽ đi qua cổng mà không ai thấy.

#### Hình dạng bàn đo — 0 dòng mã sản phẩm

| Mảnh | Chỗ | Vai |
| --- | --- | --- |
| `bench.js` | scratchpad → tiêm vào `dist/index.html` | đếm delta rAF · đóng dấu `input` + `save_segment_targets` · đổ số qua `put_config` |
| `fence.sh` | scratchpad | hàng rào hai chiều, tự kiểm đỏ-rồi-xanh |
| `type-driver.sh` | scratchpad | bơm phím **tầng hệ điều hành** (Quyết định #2c), tiếng Việt có dấu + lượt xoá + chỉ số `⟦n⟧` đơn điệu tăng (Quyết định #4) |
| `kill-campaign.sh` | scratchpad | `SIGKILL` · kiểm `.db-wal > 0` (AC9) · lấy mẫu `.db-wal` **cả hai kho** (AC10) |
| `build-bench.sh` | scratchpad | dựng bản đo + cổng dấu sống |

⚠️ `bench.js` là script **cổ điển, cùng origin** — `script-src 'self'` của CSP cho phép; một script
**nội tuyến** thì không. Nó nằm trong `<head>` **trước** bundle module *(module luôn `defer`)*, tức
nó vá được `__TAURI_INTERNALS__.invoke` trước khi mã sản phẩm giữ tham chiếu.

⚠️ Thêm một dữ kiện cho AC13: **48.639 ký tự / 9.850 câu ≈ 4,9 ký tự một câu.** Đó là hình dạng của
một tài liệu markdown nhiều dòng ngắn, **không** phải văn xuôi dịch. Trần 9.850 span là một số
**thật** và **lặp lại được**, nhưng nó là trần của *một hình dạng văn bản*, không của *một Chương
tiểu thuyết cỡ đó*. Ghi ra để không ai đọc bảng NFR2 rộng hơn thứ nó đo.

### Completion Notes List

#### 🔴 TRẠNG THÁI DỪNG 2026-08-13 — `in-progress`, và đây là nguyên nhân CỤ THỂ

Bài học #6 của Epic 1: *"`in-progress` không phải chỗ đậu — để dở thì ghi **nguyên nhân cụ thể**"*.

**Cái đã đứng, đã nghiệm thu, lặp lại được:**

| Task | Trạng thái | Bằng chứng |
| --- | --- | --- |
| 1.0 cửa khởi hành | 🟢 **MỞ** | Ice gõ tay, chữ hạ cánh vào câu chưa dịch |
| 0 — bảy quyết định | 🟢 **CHỐT** *(trừ #7, cố ý chờ số)* | bảng §Phán quyết Task 0 + §ĐÍNH CHÍNH #1 |
| 1 — hàng rào dữ liệu thật | 🟢 **ĐỨNG hai chiều**, tự kiểm đỏ-rồi-xanh | `fence.sh`, ảnh chụp 116 dòng, Library rỗng trên bản release |
| 1 — môi trường + trạng thái máy (AC22) | 🟢 **GHI** | `env.txt` — cắm sạc, `CPU_Speed_Limit = 100` |
| 2 — Chương thật vào máy | 🟢 **RÚT XONG** thang 2.000/6.000/16.000/48.639 ký tự, cắt từ chính văn bản thật | `chapters/ladder-*.txt` |
| AC15 mốc gốc | 🟢 **40/40** | `npm run test` |

#### 🔴 HÀNG RÀO CHIỀU ÂM BÁO VỠ Ở LƯỢT ĐỐI CHIẾU CUỐI — chưa chẩn đoán, KHÔNG được gạt đi

`fence.sh diff before-everything final` (2026-08-13T06:28:46Z) trả **đỏ**. Hai nhóm, ngữ nghĩa
khác hẳn nhau:

**Nhóm ① — vô hại, và giải thích được:** ba tệp `project.db-shm` trong `~/Documents/AuraTranslate/`
*(`Epochtime (2)`, `Thieu Chuu`, `test tieng trung`)* đổi **mtime**, **SHA-256 y nguyên**, cỡ y
nguyên. Đây là dấu vết của chính lượt `sqlite3 -readonly` mà story này chạy lúc 12:58 để đo cỡ
Chương — mở một kho WAL chạm `-shm` kể cả khi chỉ đọc. **Nội dung không đổi một byte.**

**Nhóm ② — 🔴 CHƯA CHẨN ĐOÁN, và nó là loại phải báo:**

| Tệp | Trước | Sau |
| --- | --- | --- |
| `$APPDATA/global.db` | `43790b68…` · 28.672 B · mtime 1786597522 | **`82501d75…`** · 28.672 B · mtime **1786601884** |
| `$APPDATA/global.db-shm` | `9ec1b930…` · 32.768 B | **biến mất** |
| `$APPDATA/global.db-wal` | `be617613…` · 12.392 B | **biến mất** |

⇒ **`global.db` THẬT của Ice đã đổi nội dung** *(băm khác)* lúc ~13:18 giờ máy.

**Hình dạng của lượt đổi khớp với một lượt ĐÓNG BÌNH THƯỜNG:** nội dung WAL được gấp vào tệp chính
rồi `-wal`/`-shm` bị gỡ — đúng đường `Store::close()` → TRUNCATE. Một `SIGKILL` thì **để lại**
`-wal`, nên đây **không** phải dấu vết của bộ kill *(bộ kill của Task 4 chưa từng chạy)*.

🔴 **Nhưng “khớp hình dạng” KHÔNG phải “đã chẩn đoán”, và mình không đặt tên cho nguyên nhân.**
Mọi lượt chạy app của story này đều đi qua `HOME=<nháp>` và đã được đo là chuyển hướng đúng; không
lượt nào của story chạm `$APPDATA` thật. Giả thuyết còn lại chưa kiểm được từ đây: bản `tauri dev`
mà **Ice** mở cho lượt kiểm tay Task 1.0 vẫn còn chạy trên dữ liệu thật và đã thoát lúc đó.
**Chủ của việc xác nhận: Ice** — chỉ Ice biết app đó có còn mở tới 13:18 không.

🔵 Hai bản sao lưu `global.db.bak-v1` và `.bak-v2` **y nguyên từng byte** trong cả hai ảnh chụp.

✅ **ĐÓNG 2026-08-13 — Ice xác nhận: bản `tauri dev` của lượt kiểm tay Task 1.0 CÒN MỞ tới 13:18.**
⇒ Lượt đổi là app **của Ice** thoát bình thường trên dữ liệu thật, **không** phải bàn đo của story.
Hàng rào đã làm đúng việc: nó bắt một lượt đổi thật, và nó **không** báo oan bàn đo.
⚠️ Bài học đi vào bàn đo, không vào tài liệu: một phiên đo phải kiểm **không còn instance nào của
app chạy trên dữ liệu thật** trước khi chụp ảnh gốc. Thêm vào `fence.sh snap` như một điều kiện.

#### ✅ CHỖ CHẶN ĐÃ CÓ TÊN — `build.rs` không bao giờ chạy lại khi `dist/` đổi

Ba lượt tiêm biến mất, và nguyên nhân **không** phải CSP *(giả thuyết ở lượt hỏng ③ SAI — bác bỏ)*.

`src-tauri/build.rs:66` khai `println!("cargo:rerun-if-changed=windows-app-manifest.xml")`. Luật
của Cargo: một khi build script phát **bất kỳ** `rerun-if-changed` nào, cargo **thôi** theo dõi cả
gói và chỉ theo dõi **đúng danh sách đã khai**. Mà `build.rs` chính là chỗ `tauri-build` **nhúng**
`dist/` vào nhị phân.

⇒ Lượt `touch src-tauri/src/lib.rs` làm `rustc` **liên kết lại** nhị phân *(mtime mới, 4.912.272
byte)* nhưng **giữ nguyên bộ tài nguyên nhúng CŨ**. Bench nằm trong `dist/` thật, `grep` thấy nó
thật, nhị phân mới thật — và trang được phục vụ vẫn là trang **trước lượt tiêm**. Đây cũng là lý do
`dist/bench.js` + thẻ `<script>` ở lượt hỏng ③ *"không chạy"*: blob nhúng chưa bao giờ có chúng.

**Vá:** `touch src-tauri/windows-app-manifest.xml src-tauri/build.rs` — chạm **đúng tệp mà build.rs
khai là đầu vào**, thay vì chạm một tệp cargo không hề theo dõi cho mục đích này.

🔵 Ba giả thuyết bị **bác**, ghi ra theo mục ⑤ của khuôn báo cáo: ① `strings` chứng minh được lượt
tiêm *(sai — `strip` + nén)*; ② CSP chặn script cùng origin *(sai — mã ở trong chính bundle app vẫn
không chạy)*; ③ `document.title` là kênh đọc được từ ngoài *(sai — Tauri không đồng bộ nó sang tiêu
đề cửa sổ native)*.

⚠️ Ghi vào đây thay vì gạt đi, vì đây **chính xác** là loại sự kiện hàng rào tồn tại để bắt, và vì
một hàng rào báo đỏ mà bị giải thích cho qua thì lần sau nó là một hàng rào chết.

#### 🔴 GIẢ THUYẾT `build.rs` CŨNG BỊ BÁC — và ở đây mình DỪNG vòng đoán

Sau lượt vá: `build.rs` **đã** chạy lại *(thư mục `build/auratranslate-33ad…` mtime 13:37)*, lượt
tiêm **còn** trong `dist/assets/index-*.js` *(grep = 1)*, `bench.js` **hợp lệ cú pháp**
*(`node --check` OK)*, và app **dựng bình thường** *(ảnh chụp: Vue mount, màn Library đầy đủ)*.
Vẫn **không** ô bàn đo, `config_value` **rỗng**.

⇒ **Bốn giả thuyết, bốn lần bác.** Nguyên nhân thật vẫn **chưa có tên**, và bốn lượt rebuild cho
một lượt tiêm là đã vượt ngưỡng mà chính §Bài học #9 đặt ra. **Không đoán tiếp.**

🔵 **Và đây là chỗ nhìn lại thì thấy đường vòng RẺ HƠN, đáng lẽ phải thấy sớm hơn bốn lượt build:**
**NFR18 KHÔNG CẦN bàn đo tiêm.** Nó cần đúng ba thứ, cả ba đã có và cả ba đi qua đường sản phẩm
thật: ① dựng Tác phẩm **qua chính giao diện** *(màn Library có ô «Hoặc nhập đường dẫn tệp» + nút
«Tạo Tác phẩm từ tệp» — thấy trong ảnh chụp)*; ② gõ bằng `osascript` tầng hệ điều hành; ③ đọc lại
bằng `sqlite3` chỉ đọc. **0 dòng tiêm, 0 rebuild.**

⇒ **Đổi đường, và nó còn TRUNG THỰC HƠN đường cũ:** lượt nhập đi qua giao diện thật thay vì qua
IPC, tức nó đóng luôn cái giá mà Quyết định #6 phải ghi nợ *(«fixture IPC không đo đường nhập của
người dùng thật»)*. Bàn đo tiêm chỉ còn cần cho **NFR2** *(lấy mẫu rAF)* — một chỗ chặn **hẹp hơn
nhiều** so với «cả story».

**Cái CHẶN, nói thẳng:** bàn đo **chưa chạy được trong webview của bản release**. Mã đã nằm trong
chính bundle app *(grep = 1)*, nhị phân đã dựng lại **sau** `dist/` *(13:22 so với 13:20)*, nhưng
dấu sống vẫn vắng. **Nguyên nhân CHƯA được đặt tên** — và lượt chẩn đoán đang chạy đúng cách: một
kênh **không đi qua IPC** *(`document.title`, đọc từ ngoài bằng `osascript`)* tách hai câu hỏi mà
lượt trước bó làm một — *"mã có chạy không"* và *"`put_config` có ghi được không"*. Lượt trước
`.catch(function(){})` **nuốt** vế thứ hai; đó là lỗi của bàn đo, đã sửa.

**Lượt hỏng ④ — và kênh chẩn đoán THỨ HAI cũng vô giá trị, phải nói ra.** `document.title` được
chọn làm kênh không-qua-IPC. Đo: `osascript` trả về `AuraTranslate`. 🔴 **Nhưng đó là tiêu đề cửa
sổ native mà Tauri đặt từ `tauri.conf.json`, không phải `document.title` của trang** — Tauri v2
**không** đồng bộ hai thứ đó. ⇒ Phép đo này **không phân biệt được** *"bench không chạy"* với
*"bench chạy và đặt title nhưng title không ra tới cửa sổ"*. Nó **không** là bằng chứng cho kết
luận nào, và ghi nó như một bằng chứng sẽ là đúng cái lỗi mà Story 1.22 đã ghi tên hai lần.

⇒ **Kênh chẩn đoán tiếp theo phải là kênh đọc được từ ngoài mà KHÔNG qua Tauri:** đề xuất —
bench vẽ một khối màu đặc kích thước cố định ở một góc rồi đọc bằng `screencapture` + so pixel
*(đường này đã dùng được ở lượt hỏng ②)*, hoặc `localStorage` rồi đọc tệp SQLite của WKWebView
trong `$HOME` nháp. **Chưa thử cái nào** — đây là mũi tiếp theo, không phải một kết luận.

🔴 **Vì sao KHÔNG được đi tiếp khi chưa gỡ chỗ này:** mọi số của AC1/AC2/AC12/AC13 đọc ra từ một
bàn đo chưa chứng minh được là nó đang chạy sẽ là một **bảng xanh rỗng** — đúng lớp lỗi mà AC15 của
Story 2.3 đặt tên và mà chính story này cấm ở AC21.

**Còn nguyên, chưa chạy một phép đo nào:** Task 3 *(NFR2)* · Task 4 *(NFR18, ≥ 20 `SIGKILL`)* ·
Task 5 *(dò ngưỡng WAL + sáu số `Tuning`)* · Task 6 *(ba đường nóng)* · Task 7 *(phán quyết thư
viện editor — cố ý chờ số)* · Task 7b *(RSS 100 MB)* · Task 8 *(báo cáo + hai hàng Deferred)* ·
Task 9 *(hai món nợ tài liệu)* · Task 10 · Task 11.

⚠️ **Chưa một hằng nào bị đổi**, và cây làm việc **0 dòng** mã sản phẩm — `git status` chỉ có
`sprint-status.yaml` và chính tệp story này. Lượt `touch src-tauri/src/lib.rs` của script dựng chỉ
chạm **mtime**, không chạm nội dung.


#### Bảng sáu số `Tuning` (AC7)

| Trường | Nhãn | Số đo / chủ mới |
| --- | --- | --- |
| `pool_size` | | |
| `busy_timeout` | | |
| `checkpoint_tick` | | |
| `idle_before_passive` | | |
| `wal_threshold_bytes` | | |
| `close_truncate_budget` | | |

#### Bảng ba hằng nhịp flush (AC14)

| Hằng | Cũ | Mới | Số đo |
| --- | --- | --- | --- |
| `EDITOR_IDLE_MS` | 2000 | | |
| `EDITOR_HARD_CAP_MS` | 5000 | | |
| `EDITOR_RETRY_FLOOR_MS` | 2000 | | |

### File List

### Review Findings

### Change Log
