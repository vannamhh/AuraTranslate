---
baseline_commit: 6a9777bac1add41aaed79590901d8d337380cab4
---
# Story 2.3: Hợp đồng flush và trạng thái đã lưu

Status: in-progress

> 🔵 **ĐÍNH CHÍNH 2026-08-13 — chẩn đoán "AD-34 giành tiêu điểm" là SAI, và bản vá Ice ký KHÔNG được thi hành.**
>
> Ice ký đường ① *(cho `section.mode`/`section.panel` thôi giành tiêu điểm)*. Phép đo tiếp theo cho thấy **không ai giành cả**: `focus.ts::enter()` chỉ chạy lúc đổi chế độ, chốt chống-rơi-`body` của nó **chỉ ghi console**, và `PanelFrame` chỉ **nghe** `focusin`/`focusout`. Thứ đặt tiêu điểm lên `section.panel` là **hành vi mặc định của trình duyệt** — nó đi ngược cây tìm tổ tiên focus được gần nhất, và nó chọn như vậy **chỉ vì `<span>` chưa `contenteditable`** tại thời điểm engine xử lý cú bấm *(Vue vá DOM ở một microtask sau)*.
>
> ⇒ **Nguyên nhân thật là một lượt đặt thuộc tính bất đồng bộ**, đã vá: `setAttribute('contenteditable','true')` **đồng bộ ngay trong `mousedown`**. **AD-34 và `focus.ts` không bị chạm một dòng.** Đo lại trong WKWebView thật, trên một câu **đã có chữ**: `caretRangeFromPoint` → `#text@18` *(trong câu)* · `getSelection().type` → **`"Caret"`** · `activeElement` → **`SPAN.sent"`**. Trước đó cả ba là `null` / `"None"` / `SECTION.panel`.
>
> 🔴 **VÌ SAO VẪN KHÔNG PHẢI `review`** — bài học #6: *"`in-progress` không phải chỗ đậu — phải để dở thì ghi **nguyên nhân cụ thể**"*.
>
> **Còn đúng một ca chưa chạy: lượt gõ ĐẦU TIÊN vào một câu CHƯA DỊCH.** Câu đó là một `<span>` **rỗng**, rộng **0 pixel**, không text node để neo caret ⇒ `execCommand('insertText', …)` trả `false`. Đây là ca **thường nhất** của tính năng, không phải một ca biên. Ba hướng đã thử và chưa đủ, ba hướng chưa thử — ghi ở `deferred-work.md` §*CÒN LẠI SAU ĐÍNH CHÍNH*.
>
> **Mọi phần còn lại xong và xanh:** 9/9 cổng npm · `build` · `npm run test` **32/32** · `cargo test` **319/0** · e2e **1/2** *(ca `Enter` xanh; ca gõ đỏ ở đúng vế trên)*. Bốn lượt đỏ-rồi-xanh đã tái lập.

**Covers:** FR100 (`epics.md:290`) · **AD-35** (`ARCHITECTURE-SPINE.md:419-425`) · **UX-DR30** (`epics.md:563`) · NFR18 (`epics.md:368`) · NFR2 (`epics.md:326`)
**Epic:** 2 — Biên tập theo segment · story **thứ ba**, ngay sau 2.2 *(đã `done` 2026-08-12)*
**Nguồn:** `epics.md:2075-2115` · AD-1 (`:75-79`) · AD-11 (`:153-157`) · AD-12 (`:159-163`) · AD-21 (`:302-306`) · AD-31 (`:368-392`) · AD-35 (`:419-425`) · AD-37 · UX-DR30 (`epics.md:563`) · `EXPERIENCE.md:127`
**Nợ ĐÓNG ở đây:** `deferred-work.md:180-182` *(`isTypingZone` — chủ đã chuyển sang story này đích danh ở Task 10.1 của 2.2)* · `deferred-work.md:2135-2139` *(cổng **Kiểm J** của `check-commands.mjs` hết hạn ở story này và phải được gỡ **đúng lúc**)*
**🔵 BỐN PHÁN QUYẾT CỦA ICE, ký 2026-08-12 lúc dựng story — dev KHÔNG mở lại:** ① **NFR15 vế *"không bộ chạy test frontend"* được LẬT** — story này dựng `vitest` + `@vue/test-utils` + `happy-dom` *(§Điều kiện khởi hành mục 8)*; ② **Quyết định #1 = đường (c)** — vùng gõ là **một câu tại một thời điểm**; ③ **Quyết định #2 = đường (a)** — tái dùng `createWriteSchedule`, không dựng lịch thứ hai; ④ ba mục trên **không** kéo theo giấy phép cho một thư viện **editor** — hàng Deferred đó vẫn thuộc Story 2.4 (AC19).

**Nợ ĐI QUA mà KHÔNG đóng:** hàng Deferred *"ngưỡng WAL + nhịp flush cụ thể"* (`ARCHITECTURE-SPINE.md:883`) — **chủ là Story 2.4**, story này dựng **cơ chế**, 2.4 hiệu chỉnh **con số** · hàng Deferred *"thư viện editor"* (`:886`) — chủ là 2.4 · hàng Deferred *"ảo hoá danh sách dài"* (`:888`) — chủ là 2.4, và 2.2 đã đo và đã báo trần · `deferred-work.md:2120-2126` *(vế `Selection.anchorNode` phải xét lại khi caret thật xuất hiện — **story này là lượt đó**, xem AC22)* · `:875-882` *(không bộ chạy test frontend)* · `:145` *(mọi bằng chứng chỉ trên macOS)*

---

## Điều kiện khởi hành — ĐỌC TRƯỚC KHI GÕ MỘT DÒNG

### 1. Cây làm việc SẠCH, và đây là mốc gốc

`git status --porcelain` trả **0 dòng** lúc dựng story này (2026-08-12). `baseline_commit` ở frontmatter là SHA thật của `HEAD`: `6a9777b` *(“feat: add editor gutter measurement and segment state management” — lượt commit của Story 2.2)*. Không có món vá cũ nào phải commit riêng trước.

### 2. 🔴 CÓ MỘT CỔNG ĐANG CHẶN CHÍNH VIỆC CỦA STORY NÀY, VÀ NÓ PHẢI ĐƯỢC GỠ **CÙNG LƯỢT**

`scripts/check-commands.mjs:2068-2135` — **Kiểm J**. Nó khẳng định `EditorPanel.vue` **không** mang năm thứ: `contenteditable` · `<textarea>` · `<input>` · `v-model` · `@input`/`@beforeinput`/`@paste`/`@cut`. Đó là AC18 của Story 2.2, hệ quả phán quyết Quyết định #1 đường (b) do Ice ký 2026-08-12.

`deferred-work.md:2135-2139` ghi hạn của nó bằng chữ:

> *"Cổng này **hết hạn ở Story 2.3**, và nó phải được gỡ **ĐÚNG LÚC**. Story 2.3 dựng vùng gõ, nên nó phải gỡ cổng **cùng lượt** với hợp đồng flush AD-35 — **không sớm hơn**. Gỡ sớm là mở lại đúng cửa sổ mất dữ liệu im lặng mà cổng tồn tại để đóng. **Chủ: Story 2.3.**"*

⚠️ *"Cùng lượt"* là một mệnh đề về **thứ tự làm việc**, không phải về một commit: đường flush phải chạy được **trước** khi `contenteditable` chạm nhánh chính. Xem AC8 — nó là mệnh đề nghiệm thu, không phải một lời khuyên.

⚠️ Kiểm J đọc bản **đã che** (bỏ chú thích và chuỗi). Gỡ nó nghĩa là gỡ cả khối `TYPING_BANS`, cả sàn nội dung `data-segment-id`, và cả tiêu đề in ra — **không** để lại một cổng xanh rỗng. `@keydown` **chưa từng** nằm trong danh sách cấm (làm rõ ở code review 2026-08-12), nên story này không phải "mở khoá" nó.

### 3. Cột `target_text` ĐÃ CÓ. Cột `status` thì KHÔNG — và story này **không cần** nó

Lược đồ `project.db` hôm nay ở **phiên bản 6** (`schema.rs:431-455`): bước 5 = `SEGMENT_DDL`, bước 6 = `SEGMENT_TARGET_TEXT_DDL` (`ALTER TABLE segment ADD COLUMN target_text TEXT NOT NULL DEFAULT ''`). **Bước di trú kế tiếp phải đánh số 7.** Số **4** là số **đã cháy** — `segment_contract.rs::the_project_migration_set_never_reuses_the_burned_number_four` canh việc đó bằng máy.

Chín cột thật của `segment` hôm nay:

```sql
id · chapter_id · ord · source_text · is_paragraph_end · retired_at · created_at · updated_at · target_text
```

🔴 **`status` chưa tồn tại, và đó KHÔNG phải một khối cho story này** — AD-31 (`ARCHITECTURE-SPINE.md:376`) nói bằng một hàng bảng: *"Auto-save (FR100) | trạng thái **không đổi** | **không** tạo `SegmentVersion`"*. Một auto-save không đổi trạng thái thì nó không cần một cột trạng thái để không đổi. Chủ của `status` vẫn là **Story 2.5**; chủ của `SegmentVersion` vẫn là **Story 2.6**.

⇒ Bước di trú 7 chỉ tồn tại **nếu** Quyết định #3 kết luận cần một cột mới. Mặc định là **không có bước di trú nào** ở story này — xem Quyết định #3.

### 4. 🔴 THANH TRẠNG THÁI **CHƯA TỒN TẠI** TRONG ỨNG DỤNG. AC7 buộc dựng nó.

Đo 2026-08-12: `src/App.vue` có `main.shell` chứa `header.titlebar` · `div.modeport` · `pre.selftest` · dải báo lỗi cấu hình. **Không một phần tử nào** là thanh trạng thái, và không một khoá `vi.json` nào cho nó.

Token thì **đã có**: `spacing.status-height = "34px"` (`src/tokens/tokens.json:480`), và `DESIGN.md:283` + `:316` ghim đúng số đó *(⚠️ `DESIGN.md:132` còn ghi `status-height: 32px` ở một khối bảng cũ, và mockup `key-screen-workspace.html:73` dựng `.status{height:32px}` — **34px thắng**, nó là số trong bảng token và trong `tokens.json`; lệch này ghi vào `deferred-work.md` với chủ là Ice, **đừng sửa `DESIGN.md`**)*.

⇒ AC7 của story này là lượt **đầu tiên** thanh trạng thái tồn tại. Đó là phạm vi thật, không phải một dòng chữ thêm vào chỗ có sẵn. Xem Quyết định #5.

### 5. NHỊP FLUSH ĐÚNG HÌNH DẠNG AD-35 **ĐÃ ĐƯỢC VIẾT RỒI** — đừng dựng lại từ đầu

`src/layout/writeSchedule.ts` (Story 1.14 · AC4) là một module **thuần**, không `import` gì, và nó cài **chính xác** hình dạng mà AD-35 đòi:

```ts
due = Math.min(now + idleMs, firstChangeAt + hardCapMs)
```

`firstChangeAt` **không** được gán lại cho tới khi `onWrite` chạy ⇒ trần **không bị reset bởi sự kiện kế tiếp**. Đó đúng là mệnh đề *"đồng hồ trần này không được reset bởi phím gõ"* của AC2. Hằng hiện tại: `IDLE_MS = 500` · `HARD_CAP_MS = 5000`.

⚠️ Doc-comment của chính tệp đó (`:22-28`) đã rào trước:

> *"Đây là **mượn hình dạng** của AD-35, KHÔNG phải 'áp AD-35 cho bố cục'. […] Cái được mượn là **hình dạng** (idle cộng một trần cứng), không phải các bảo đảm."*

⇒ Câu hỏi *"tái dùng hay dựng riêng"* là **Quyết định #1**, và nó phải được trả lời bằng lý lẽ ghi ra, không bằng một lượt `import` cho tiện. Cùng tệp đó cũng để lại một tài sản thứ hai đáng giá hơn: `simulateWrites()` — một mô phỏng **không đồng hồ, không timer** mà `scripts/check-layout.mjs` Kiểm B (`:288`) đã dùng để khẳng định một mệnh đề **định lượng** thay cho một lượt đếm tay. Cổng của AC11 đi đúng đường đó.

### 6. Đường ghi xuống đĩa: **một** writer nối tiếp, và nó đã chặn kể cả khi luồng chết

- `Store::write(job)` (`core/store/mod.rs:612-618`) **chặn** tới khi job chạy xong. Mỗi job là **một giao dịch**: `Ok` ⇒ commit, `Err` ⇒ rollback (`writer.rs:142-159`).
- Không có kết nối ghi thứ hai để mở: `Connection` ghi được `move` vào luồng writer và `rusqlite::Connection` là `Send` nhưng **không `Sync`** ⇒ trình biên dịch cưỡng chế phần còn lại (`writer.rs:1-11`). Nửa kia là `tests/store_boundary.rs`.
- Gọi **lồng** (một job ghi gọi lại `Store::write`) bị bắt bằng `ON_WRITER_THREAD` và trả `WriteFailed`, **không treo** (`writer.rs:47-55`, `:131-138`).

🔴 **Vế *"chỉ xong sau khi đã ghi vào WAL"* của AC5 — ĐÃ ĐO MỘT NỬA, dev phải đo nốt.** `core/store/pragmas.rs` đặt **ba** PRAGMA: `journal_mode = WAL` · `wal_autocheckpoint = 0` · `busy_timeout`. Nó **không** đặt `PRAGMA synchronous`. Mặc định biên dịch của SQLite là `FULL (2)`, và ở `FULL` + WAL mỗi lượt commit **fsync WAL** — tức `Store::write` trả `Ok` **là** bằng chứng đã ghi vào WAL, và AC5 thoả **mà không cần thêm một dòng nào**. ⚠️ Nhưng đó là một mệnh đề về **giá trị mặc định**, không phải một lời khai trong mã. Task 4.4 đòi **đọc lại `PRAGMA synchronous` bằng một test** và ghi số đo — cùng luật *"đặt rồi ĐỌC LẠI"* mà `WalUnavailable` (`mod.rs:308-321`) tồn tại để dạy. Nếu đọc ra `1 (NORMAL)`, AC5 **chưa thoả** và đó là một phát hiện phải báo, không phải một lượt vá thầm.

### 7. Sáu con số `Tuning` vẫn TẠM, và story này **không** hiệu chỉnh chúng

`core/store/mod.rs:62-68` nói thẳng: không con số nào ở `Tuning::default` được đo, chủ sở hữu là **Story 2.4**, vì `wal_threshold_bytes` và nhịp flush **đánh đổi lẫn nhau**. `idle_before_passive = 5 s` được đặt *"cố ý dài hơn nhịp flush 2 s của AD-35"* (`:207-208`) — tức tầng kho **đã** được viết với giả định nhịp flush của story này là 2 s. Giữ đúng giả định đó; đổi nó là đổi một con số mà story khác sở hữu.

### 8. 🔵 DỰ ÁN NAY **CÓ** BỘ CHẠY TEST FRONTEND — Ice lật quyết định 2026-08-12

Luật cũ, giữ qua mười hai story, ghi tại chỗ ở `src/commands/registry.ts:10-13` · `src/commands/README.md:20` · `src/i18n/README.md:101`:

> *"Dự án không có bộ chạy test frontend, và thêm một (`vitest`) là thêm một phụ thuộc phải rà tương thích GPLv3 bằng cách **mở tệp giấy phép trong nguồn đã tải**, rồi vào **bảng Stack** trước khi thêm. Đó là quyết định của Ice."*

🔴 **Đọc kỹ câu trên trước khi ăn mừng: thứ nó chặn là một QUY TRÌNH, không phải một năng lực.** Nó chưa bao giờ nói *"không chạy được test"* — nó nói *"đi qua cửa rà giấy phép trước"*. ⇒ Lượt lật này **không** bác bỏ luật cũ; nó **đi qua cửa** mà luật cũ dựng, và cửa đó là Task 0b. Ghi ra để không ai sau này đọc lượt này thành *"NFR15 đã hết hiệu lực"* — **nó không hết**: gói thứ tư vẫn phải đi qua đúng cửa đó.

**Ba gói, đo trên npm registry ngày 2026-08-12, và cả ba tương thích ngay:**

| Gói | Bản | `license` **khai** | Ràng buộc, đã đối chiếu với kho |
| --- | --- | --- | --- |
| `vitest` | **4.1.10** | MIT | peer `vite: ^6.0.0 \|\| ^7.0.0 \|\| ^8.0.0` — kho ghim **8.2.0** ✅ · `engines.node ^20 \|\| ^22 \|\| >=24` — máy **22.22.2** ✅ |
| `@vue/test-utils` | **2.4.11** | MIT | peer `vue: 3.x` — kho ghim **3.5.40** ✅ |
| `happy-dom` | **20.11.2** | MIT | `engines.node >=20` ✅ · là peer **tuỳ chọn** của `vitest` 4 |

⚠️ **Cột `license` ở trên là một LỜI KHAI trong `package.json` của gói, KHÔNG phải tệp giấy phép.** NFR15 đòi *"mở tệp giấy phép **trong nguồn đã tải**"* — cùng hình dạng với luật *"đặt PRAGMA rồi ĐỌC LẠI"* của `core/store/pragmas.rs`, và cùng lý do: một trường siêu dữ liệu và một tệp thật là hai thứ, và chỉ một trong hai có hiệu lực pháp lý. Task 0b đọc **ba tệp thật** trong `node_modules/`, ghi đường dẫn và dòng đầu vào §Completion Notes. Ba gói MIT ⇒ tương thích GPL-3.0-or-later theo diện gộp gói, cùng khuôn ba font SIL OFL 1.1 mà Story 1.1 đã ghi ba hàng vào bảng Stack.

**Bốn đường nghiệm thu từ hôm nay, và mỗi đường có một vai KHÔNG chồng lên nhau:**

1. **Test đơn vị/component — `vitest`.** Vế **hành vi** của module thuần *(`editorFlush`, `editorSegments`)*, của mã đụng DOM *(`editorGutter` đo `getClientRects`)*, và của `.vue` *(qua `@vue/test-utils` trên `happy-dom`)*. Chạy bằng mili-giây, chạy được ở mỗi lượt sửa. **Đây là đường mới, và nó là đường mặc định cho mọi mệnh đề hành vi frontend của story này.**
2. **Cổng tĩnh `scripts/check-*.mjs`** — vế **khai báo trên toàn cây**: không màu viết thẳng, mọi text node qua `t()`, **đúng năm** giá trị vạch, sàn quần thể. 🔴 **Vitest KHÔNG thay được vai này** và không được thử: một cổng quét cả `src/**` để tìm thứ *không được phép tồn tại ở bất kỳ đâu* là một phép kiểm khác hạng với một test khẳng định một hàm trả đúng giá trị. Cổng ở lại.
3. **Bàn đo chạy tay** — vế **thị giác** và vế **đo số trên engine thật**. `2-2-ban-do-editor.html` và ba giới hạn đã ghi của nó (`deferred-work.md:2113-2119`).
4. **e2e WebdriverIO** — vế **hành vi trong WKWebView thật**. ⚠️ ESLint **cấm** `.click()` trong `e2e/**` từ Story 1.22; dùng `realClick()` của `e2e/support/pointer.mjs`. Bộ e2e **chập chờn** (8 lượt gần nhất 6 xanh / 2 đỏ): gặp đỏ không tái lập được thì **bắt nguyên văn lỗi TRƯỚC**, đừng chạy lại ngay. `happy-dom` **không** thay được vai này — nó là một bản mô phỏng DOM trong Node, không phải WebKit.

🔴 **Luật chống hai nguồn sự thật, áp từ hôm nay:** một mệnh đề được nghiệm thu ở **đúng một** trong bốn đường trên. Story này **không** được vừa dựng một cổng vừa viết một test cho cùng một điều — xem AC25. Và nó **không** di chuyển các phép kiểm hành vi đang sống trong cổng *(`check-layout.mjs` Kiểm B · `check-commands.mjs` Kiểm C/D/E)* sang vitest: đó là một lượt tái cấu trúc có rủi ro riêng và phải có story riêng.

### 9. Nợ `isTypingZone` có chủ là story này, và nó là nợ **hai chiều**

`deferred-work.md:181-182`. `src/commands/keys.ts:434-439`:

```ts
function isTypingZone(target: unknown): boolean {
  if (typeof target !== 'object' || target === null) return false
  const el = target as { tagName?: unknown; isContentEditable?: unknown }
  if (el.isContentEditable === true) return true
  return el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT'
}
```

Luật gọi nó: `if (lacksPrimaryMod(entry.mods) && isTypingZone(event.target)) return false` (`keys.ts:510`), với `lacksPrimaryMod = (m) => !m.meta && !m.ctrl` (`:415`).

🔴 **Hệ quả đo được, và nó cắt cả hai chiều** — story này là lượt **đầu tiên** trong dự án có một `isContentEditable === true` thật:

- **Chiều tốt:** hợp âm trần *(và mọi hợp âm chỉ có `Shift`)* thôi dispatch khi con trỏ ở trong Editor ⇒ gõ chữ `b` không bật chế độ song ngữ. Đó đúng là điều luật này được viết ra để làm (`keys.ts:418-424`).
- **Chiều phải đo:** bốn command `selection.extend_*` của Story 1.18 dùng `Shift+Mũi tên`, và `Shift` **không** phải phím bổ trợ chính ⇒ chúng cũng **thôi dispatch** trên bề mặt Editor. Hành vi native của `contenteditable` vẫn mở rộng vùng chọn, nên câu hỏi thật là: **Auto-Lookup còn chạy trên Panel Editor sau khi nó thành vùng gõ không?** Đó là AC23 — một phép **đo**, không một giả định theo chiều nào.

### 10. Mọi bằng chứng chỉ xanh trên macOS

Ice chốt 2026-08-12: trọn phần Windows dời về **cuối dự án**. Mọi thứ Epic 2 → Epic 9 thêm vào chạy **chỉ trên macOS** cho tới lượt đó, và *"khoảng mù không đứng yên — nó dày lên theo từng epic"*. CI nay tự chạy lại mỗi lượt push (`f950332`); bài học §8.1 của retro vẫn đứng: **push xong thì đọc kết quả**.

⚠️ Story này chạm đúng hai chỗ có tiền sử lệch giữa hai engine: **hành vi `contenteditable`** *(chuẩn hoá DOM khi gõ, `beforeinput`, IME)* và **copy/selection quanh ký tự chèn** *(vết sẹo `WORD_JOINER` của 1.18b — `deferred-work.md:839-848`)*. Nghiệm thu trên Chrome rồi viết *"tương đương"* là đúng lỗi mà `deferred-work.md:145` đã ghi tên.

---

## Story

As a người dịch,
I want không bao giờ mất quá năm giây công việc dù ứng dụng có sập giữa lúc tôi đang gõ,
So that tôi không phải bận tâm về việc lưu.

---

## Acceptance Criteria

Nguyên văn từ `epics.md:2083-2114`, đánh số để tham chiếu:

**AC1** — **Given** người dùng ngừng gõ khoảng 2 giây · **When** xảy ra · **Then** văn bản Editor flush xuống Rust

**AC2** — **Given** người dùng gõ **liên tục không nghỉ** · **When** 5 giây trôi qua kể từ lần flush trước · **Then** flush vẫn xảy ra · **And** đồng hồ trần này **không được reset bởi phím gõ**

**AC3** — **Given** người dùng xác nhận segment, rời segment, đóng Tác phẩm, hoặc thoát ứng dụng · **When** xảy ra · **Then** flush xảy ra ngay

**AC4** — **Given** một flush · **When** thực hiện · **Then** đi qua **đúng `store::Writer` nối tiếp** · **And** không mở kết nối riêng

**AC5** — **Given** một flush · **When** được coi là hoàn tất · **Then** chỉ sau khi **đã ghi vào WAL**, không phải khi mới vào hàng đợi trong bộ nhớ

**AC6** — **Given** một flush do auto-save · **When** hoàn tất · **Then** **không** tạo `SegmentVersion` và **không** đổi trạng thái segment

**AC7** — **Given** lần flush gần nhất · **When** hiển thị · **Then** thanh trạng thái ghi *"Đã lưu N giây trước"* · **And** không có hộp thoại và không có dấu chấm *"chưa lưu"*

### AC bổ sung — dẫn xuất từ kiến trúc, từ UX, và từ đo đạc mã nguồn

Bảy AC trên không nói hết thứ phải đúng để tính năng chạy được trong hệ thống đang có. Mười tám AC dưới đây **cùng hạng ràng buộc**, mỗi cái neo vào một nguồn kiểm chứng được.

**AC8 — bề mặt Editor gõ được, và cổng Kiểm J được GỠ đúng lúc, không sớm hơn.** `check-commands.mjs:2068-2135` cấm năm thứ; story này gỡ **cả khối** *(`TYPING_BANS`, sàn nội dung `data-segment-id`, tiêu đề in ra)* — **không** để lại một cổng xanh rỗng. 🔴 Thứ tự làm việc là mệnh đề: đường flush của AC1 → AC5 phải **chạy được và nghiệm thu được** trước khi `contenteditable` chạm nhánh chính. Nghiệm thu: `npm run check:commands` xanh sau khi gỡ, **và** §Completion Notes ghi bằng chứng rằng lượt gõ đầu tiên trên nhánh chính đã có đường lưu *(một ca `segment_contract.rs` round-trip xanh — AC16 — chạy trước lượt gỡ cổng)*.

**AC9 — thanh trạng thái là một bề mặt MỚI, và nó tuân đủ ba luật đã có.** Chưa tồn tại (§Điều kiện khởi hành mục 4). (a) chiều cao đọc `var(--space-status-height)` — **34px**, không `32px` của mockup; (b) typography `ui` *(`DESIGN.md:226`, `:316` — giãn dòng 1.4 cho nhãn một dòng)*, qua token, **không** số viết thẳng ⇒ Kiểm B/B2 của `check-tokens.mjs`; (c) **mọi text node đi qua `t()`** ⇒ Kiểm A2 của `check-i18n.mjs:900`, và khoá mới vào `vi.json` theo văn phạm chấm có tiền tố miền (Kiểm B, `:1018`). Nghiệm thu: `npm run check:tokens` + `check:i18n` xanh.

**AC10 — *"N giây trước"* tính và định dạng **ở frontend**, Rust không trả về một chữ nào.** AD-21 (`ARCHITECTURE-SPINE.md:302-306`) — Rust không bao giờ trả văn bản hiển thị; §Consistency Conventions — **định dạng số và ngày giờ chỉ ở frontend**, `params` của `IpcError` là `chuỗi → chuỗi`. ⇒ mốc flush cuối là một `number` trong state TS; `N` là một phép trừ ở TS; câu là một khoá `vi.json` có placeholder. 🔴 **Cấm** một command IPC trả `"Đã lưu 3 giây trước"`.

**AC11 — nhịp flush sống ở một module THUẦN, và một bộ TEST chạy hành vi thật của nó.** 🔵 Từ lượt lật NFR15, đường nghiệm thu là **vitest**, không phải một cổng mới — và story này **không** được dựng cả hai cho cùng một mệnh đề (AC25). Ba mệnh đề định lượng phải xanh:
  1. một dòng phím **liên tục 30 giây** *(sự kiện mỗi 100 ms)* cho **≥ 6** lượt flush ⇒ trần 5 s thật sự nổ;
  2. giữa hai lượt flush liên tiếp trong dòng đó, khoảng cách **≤ 5.000 ms**;
  3. một dòng phím **thưa** *(một sự kiện, rồi im)* cho **đúng một** lượt flush ở **≈ 2.000 ms** ⇒ vế idle thật sự chạy.
  Nghiệm thu **đỏ-rồi-xanh**: đổi `Math.min(...)` thành `now + idleMs` *(tức biến nó thành debounce thuần)* ⇒ mệnh đề 1 và 2 **đỏ**. ⚠️ Module này **không được đọc `Date.now()` bên trong** — mọi thời điểm đi vào qua tham số, cùng luật `writeSchedule.ts:18-20` đã ghi. 🔴 Luật đó **không hết hiệu lực** chỉ vì nay có `vi.useFakeTimers()`: một hàm nhận thời điểm qua tham số kiểm được **tất định và tức thời**; một hàm bị đồng hồ giả bao quanh kiểm được đúng chừng nào không ai quên bật đồng hồ giả. Đường rẻ hơn không phải đường đúng hơn.

**AC12 — flush đi qua đúng `Store::write`, và không có đường ghi thứ hai.** `Store::write` là bề mặt duy nhất; `tests/store_boundary.rs` đã cấm `rusqlite` xuất hiện ngoài `src/core/store/**` và phép cấm đó **đã** phủ mọi mã mới của story này. Nghiệm thu: một ca ở `store_boundary.rs` **hoặc** `segment_boundary.rs` khẳng định lệnh ghi mới không mở kết nối nào và không nhận `&Store` ở tầng dưới *(cùng khuôn `insert_segments` nhận `&Transaction` — `commands/segment.rs:95-99`)*.

**AC13 — lệnh ghi nhận một LÔ, không một lượt IPC mỗi câu.** Một flush có thể mang **nhiều** segment đã đổi *(người dùng gõ xuyên qua ba câu trong 5 giây là chuyện thường)*. N lượt `invoke` ⇒ N giao dịch ⇒ N lượt xếp hàng trên writer **duy nhất, nối tiếp** của AD-11 — đúng điểm nghẽn mà Story 2.2 vừa đo và vừa vá (`insert_segments`: `prepare_cached` cắt **57–64 ms** trên 9.850 hàng). ⇒ **một** lệnh, **một** giao dịch, `prepare_cached` **một lần** cho câu `UPDATE`. Nghiệm thu: `segment_contract.rs` có một ca flush **nhiều** segment trong một lượt gọi.

**AC14 — câu `UPDATE` chạm ĐÚNG hai cột: `target_text` và `updated_at`.** Không `created_at` *(nó là mốc tạo)*, không `ord`, không `source_text` *(AD-4 đóng băng ranh giới)*, không `retired_at`, không `chapter_id`. `updated_at` sinh **ở tầng SQL** bằng `strftime('%Y-%m-%dT%H:%M:%fZ','now')` — cùng khuôn `insert_segments` (`commands/segment.rs:100-105`), **không** truyền từ Rust. Nghiệm thu: một ca đọc lại cả chín cột sau flush và khẳng định bảy cột kia **y nguyên từng byte**.

**AC15 — auto-save KHÔNG tạo `SegmentVersion` và KHÔNG đổi trạng thái — và điều đó phải đọc được từ MÃ, không từ sự vắng mặt.** AD-31 hàng 1 (`ARCHITECTURE-SPINE.md:376`). Hôm nay cả hai thứ đó **chưa tồn tại** *(bảng `segment_version` → Story 2.6; cột `status` → Story 2.5)*, nên một test *"không có `SegmentVersion` nào"* là một test **xanh rỗng**. ⇒ mệnh đề được giao bằng **một doc-comment tại chỗ trên chính hàm ghi**, gọi tên AD-31 và gọi tên hai story chủ — cùng khuôn `editorSegments.ts:132-135` đã đặt cho ba nhánh vạch chưa có nguồn. Kèm AC14, thứ cưỡng chế thật là *"chỉ hai cột được chạm"*.

**AC16 — round-trip: gõ → flush → nạp lại cho ĐÚNG chữ đã gõ, và ranh giới segment KHÔNG đổi.** AD-4 (`ARCHITECTURE-SPINE.md:95-101`) đóng băng ranh giới; gõ **không bao giờ** là một lượt tách lại. Nghiệm thu ở `segment_contract.rs`: ghi `target_text` cho k segment → đọc qua `read_open_chapter_segments` → khớp từng chuỗi, `id`/`ord`/`is_paragraph_end` **y nguyên**, số hàng **không đổi**. ⚠️ **Gộp ngầm khi gõ đè lên ranh giới là UX-DR32 và thuộc Story 2.9** — story này **không** cài nó, và cũng **không** được cài một nửa của nó.

**AC17 — flush lúc THOÁT ứng dụng thật sự chạy TRƯỚC khi kho đóng.** AD-35 vế (e). Đường thoát hôm nay: `app.run(|handle, event| { if RunEvent::Exit { close_global_store · close_dict_layers · close_open_work } })` (`src-tauri/src/lib.rs:272-278`), và `close_open_work` (`:483-491`) `take()` `OpenWork` rồi `store.close()`. 🔴 **Không một hook nào ở webview hôm nay chạy trước điểm đó** — một `beforeunload` không được nối, và `WindowEvent::CloseRequested` không được nghe *(chỉ `DragDrop` được nghe — `lib.rs:521-522`)*. ⇒ vế (e) là một **lỗ thật**, không phải một dòng chữ. Xem Quyết định #4. ⚠️ `panic = "abort"` nghĩa là một lần thoát cứng **không** đi qua đây — đó là món nợ đã ghi cho `close_global_store`, và story này **kế thừa** nó chứ không đóng nó; nói ra trong §Completion Notes, đừng đánh dấu đạt.

**AC18 — *"rời segment"* có một định nghĩa ĐO ĐƯỢC trên một trang liền mạch.** AC1 của Story 2.2 cấm ô, bảng, khối — nên không có một widget nào để "rời". Định nghĩa duy nhất còn lại và nó đã có sẵn nguồn dữ liệu: **`editorCaretSegmentId` đổi giá trị** (`editorPanelState.ts:49`, `:117-119`). ⇒ caret đi từ câu A sang câu B **là** một lượt rời segment ⇒ flush ngay cho A. Nghiệm thu: mệnh đề này được cài ở **đúng một** chỗ và có doc-comment nói vì sao nó là định nghĩa.

**AC19 — 0 dependency RUNTIME mới, và ĐÚNG BA dev-dependency mới, không một gói thứ tư.** `package.json` hôm nay có **đúng ba** dependency runtime: `@tauri-apps/api 2.11.1` · `dockview-vue 7.0.4` · `vue 3.5.40` — con số đó **không đổi** ở story này. Ba gói được cấp phép, ghim chính xác, **không** dải `^`: `vitest@4.1.10` · `@vue/test-utils@2.4.11` · `happy-dom@20.11.2`, cả ba vào `devDependencies` và vào **bảng Stack** của `ARCHITECTURE-SPINE.md` *(cùng khuôn lượt +10 phụ thuộc mà correct-course 2026-08-11 đã ghi — bảng Stack là tài liệu Dev **được** đồng bộ, khác `epics.md`/`DESIGN.md` vốn là lượt riêng của Ice)*.

🔴 **Đây vẫn là story mà một dev sẽ muốn một thư viện EDITOR nhất, và lượt lật hôm nay KHÔNG cấp phép cho nó.** Hàng Deferred *"thư viện editor cho panel Editor"* (`ARCHITECTURE-SPINE.md:886`) có chủ là **Story 2.4** (`epics.md:2142-2145` — *"lựa chọn được ghi lại kèm lý do"*). ⇒ nếu dev kết luận story này **cần** một thư viện editor: **dừng lại và báo**, kèm số đo của mũi thăm dò Quyết định #1. Đừng gõ vào giữa story một quyết định mà story khác sở hữu, và đừng đọc *"NFR15 vừa được lật một lần"* thành *"NFR15 đã hết hiệu lực"* — cửa rà giấy phép vẫn đứng, cho **mọi** gói thứ tư.

**AC20 — mọi sàn `*_FLOOR` bị vượt được nâng theo SỐ THẬT, đo chứ không ước.** Quần thể đo 2026-08-12 *(sau Story 2.2)*: `src/**` = **35** `.ts` + **15** `.vue` = 50 tệp; `src-tauri/src/**` = **43** `.rs`. Sàn phải rà nếu story thêm tệp: `VUE_FLOOR = 13` · `TS_FLOOR = 28` · `COMMAND_FLOOR = 29` · `CLICK_FLOOR = 17` · `DISPATCH_FLOOR = 23` · `SELECTION_SURFACE_FLOOR = 7` (`check-commands.mjs:211,219,226,244,245,1838`) · `RS_FLOOR = 36` · `VUE_FLOOR = 13` (`check-i18n.mjs:279,289`) · `FILE_FLOOR = 43` · `COMPONENT_FILE_FLOOR = 40` (`check-tokens.mjs:91,92`) · `FILE_FLOOR = 40` (`check-layout.mjs:97`). Số thật đo được ghi vào §Completion Notes.

**AC21 — nợ `isTypingZone` được ĐÓNG hoặc được ghi lại kèm SỐ, không đi qua lần thứ ba.** `deferred-work.md:181-182` đã chuyển chủ sang story này đích danh. Hai chiều hỏng của nó: **(1)** mù shadow DOM *(`composedPath()[0]` không bao giờ được hỏi)*; **(2)** chặn nhầm input phi văn bản *(`checkbox`/`radio`/`button`/`range`, và input `disabled`/`readonly`)*. ⚠️ Sản phẩm hôm nay **vẫn** không có shadow DOM lẫn input phi văn bản nào *(ô phím của `ShortcutsOverlay.vue:159` là `<button>` có chủ ý)* — nên đóng chiều (2) hôm nay là viết mã cho một nhánh **không chỗ gọi nào đi qua**, đúng thứ mà danh mục `MessageKey` của `core::i18n` đã cấm bằng chữ. ⇒ Việc của story này: **đóng chiều thật sự chạm tới** *(vùng gõ Editor được `isTypingZone` nhận đúng — nghiệm thu **đỏ-rồi-xanh** bằng Kiểm D của `check-commands.mjs:1036`, vốn đã lái được cả hai nhánh bằng một object giả)*, và **ghi lại chiều còn lại kèm lý do đo được**, không hoãn bằng một câu chung chung.

**AC22 — đường `Selection.anchorNode` của Story 2.2 được XÉT LẠI, không được giả định là còn đúng.** `deferred-work.md:2124-2125` ghi đích danh: *"Chữ ký này phủ **cơ chế hôm nay**, không phủ Story 2.3: khi caret thật xuất hiện cùng `contenteditable`, **Story 2.3** vẫn phải xét lại toàn bộ đường `Selection.anchorNode` này."* Hai thứ đổi: caret **thật** tồn tại, và DOM **biến động khi gõ**. Nghiệm thu: §Dev Agent Record ghi phán quyết — giữ nguyên, hay đổi sang `beforeinput`/`selectionchange` với một cơ chế neo khác — **kèm bằng chứng chạy thật**, không kèm một lý lẽ.

**AC23 — đo xem Auto-Lookup còn chạy trên bề mặt Editor sau khi nó thành vùng gõ.** `lacksPrimaryMod && isTypingZone` (`keys.ts:510`, `:415`) làm bốn command `selection.extend_*` của Story 1.18 *(`Shift+Mũi tên`)* thôi dispatch trong Editor. `epics.md:1762` hứa Editor *"nhận được cùng hành vi khi chúng có nội dung ở các epic sau, **không cần cài lại**"*, và `useSelectionSurface(surface, 'source')` vẫn đang cắm ở `EditorPanel.vue:67`. ⇒ **Một phép đo, hai kết quả đều hợp lệ:** còn chạy *(hành vi native của `contenteditable` đủ)* ⇒ ghi số và đóng; không còn chạy ⇒ đó là một **khuyết tật sản phẩm mới lộ ra**, ghi `deferred-work.md` **kèm chủ**, đừng vá mù trong story này. ⚠️ `SELECTION_SURFACE_FLOOR = 7` đang canh lời gọi đó — **đừng gỡ nó**.

**AC24 — bộ chạy test frontend là một CỔNG CI, không một lệnh chạy tay.** 🔴 Bài học §4 của retro Epic 1 nguyên văn: *"cổng mới phải vào CI, không chỉ chạy tay"* — `check:lint` từng sống **một ngày** ngoài CI. Và correct-course 2026-08-11 tìm ra **danh sách thứ ba không ai canh**: `.githooks/pre-push`. ⇒ `npm test` phải có mặt ở **cả ba** — `package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push` — và `npm run check:gates` *(vốn đối chiếu `package.json` với `ci.yml`)* phải xanh **sau khi biết về nó**. ⚠️ Vị trí trong chuỗi CI: **sau** 11 cổng `check:*`, **trước** `npm run build` — test frontend không cần `dist/`, và đặt nó sau `cargo test` là bắt người ta chờ vài phút để biết một mệnh đề mili-giây đã đỏ. Nghiệm thu **đỏ-rồi-xanh**: một test cố tình sai ⇒ **CI đỏ**, không phải một dòng cảnh báo rồi exit 0 *(đúng ba đường mà `check-deps.mjs:9-11` ghi lại là đã từng làm sai)*.

**AC25 — MỘT mệnh đề, MỘT đường nghiệm thu. Không hai nguồn sự thật.** Bốn đường *(vitest · cổng tĩnh · bàn đo · e2e)* có bốn vai không chồng nhau (§Điều kiện khởi hành mục 8). Story này **không** được vừa dựng một cổng vừa viết một test cho cùng một điều — hai bản khai của cùng một mệnh đề sẽ rẽ nhau ở lượt sửa thứ hai, và lúc đó không ai biết bản nào đúng *(cùng lớp lỗi mà AD-33 tồn tại để chặn ở tầng dữ liệu, và mà `check-tokens.mjs:85-91` vừa phải đi dọn ở tầng sổ sách)*. Cưỡng chế: §Completion Notes mang một bảng **mệnh đề → đường nghiệm thu → tệp**, và **không dòng nào có hai đường**. 🔴 Story này cũng **không** di chuyển các phép kiểm hành vi đang sống trong cổng — `check-layout.mjs` Kiểm B (`:288`, chạy `simulateWrites`) và `check-commands.mjs` Kiểm C (`:777`) / D (`:1036`) / E (`:1436`) **ở nguyên chỗ**. Đó là một lượt tái cấu trúc có rủi ro riêng; ghi thành một hàng `deferred-work.md` **kèm chủ**, đừng làm ở đây.

---

## Task 0 — SÁU QUYẾT ĐỊNH, chốt TRƯỚC dòng mã đầu tiên

Khuôn cố định của mọi story lớn trong dự án (1.17 · 1.18 · 1.19 · 1.20 · 1.21 · 2.1 · 2.2). Mỗi quyết định có **đề xuất mặc định kèm lý lẽ đo được**. Dev đọc, xác nhận hoặc phản biện **bằng số** — không im lặng thi hành, và không tự đổi sau khi đã gõ mã. Phán quyết ghi vào §Dev Agent Record.

🔵 **Quyết định #1 và #2 đã được Ice CHỐT lúc dựng story (2026-08-12) — dev KHÔNG mở lại chúng, chỉ thi hành.** Bốn quyết định còn lại (#3 → #6) vẫn mở và vẫn theo đúng khuôn trên: xác nhận hoặc phản biện **bằng số**.

### 🔴 Quyết định #1 — CƠ CHẾ GÕ trên một trang liền mạch, khi mỗi câu là một `<span>` mang `data-segment-id`

**Đây là quyết định chặn cả story, và nó phải được quyết bằng một MŨI THĂM DÒ, không bằng lý lẽ.**

Hình dạng hôm nay (`EditorPanel.vue:304-315`): một `.doc` chứa `<span class="sent" :data-segment-id="s.id">{{ s.target_text }}</span>` chảy **inline**, xen `<br v-if="s.is_paragraph_end">`. Vạch lề đo hình học **từ chính các `<span>` đó** (`editorGutter.ts:79-105`), và caret đọc từ `anchorNode.closest('[data-segment-id]')` (`EditorPanel.vue:238-248`). ⇒ **`data-segment-id` là sổ sách của toàn bộ story 2.2, và nó sống trong DOM.**

🔴 Rủi ro trung tâm, nói thẳng: đặt `contenteditable` lên `.doc` thì **trình duyệt được quyền sửa cây DOM đó** — gộp span khi xoá qua ranh giới, tách span khi gõ giữa, **xoá sạch span** khi `⌘A` rồi gõ đè, và chèn `<div>`/`<br>` của riêng nó khi `Enter`. Một `segment.id` biến mất khỏi DOM là một câu **không còn đường về hàng `segment` của nó** — và AD-3 nói id đã về hưu **không bao giờ** được tái dùng, nên hỏng ở đây là hỏng vĩnh viễn.

Bốn đường, và mỗi đường có một cái giá phải nói ra:

**(a) `contenteditable="true"` trên `.doc`, đọc lại văn bản bằng cách duyệt các `<span>`.** Gần mockup nhất, một vùng gõ liền thật. Cái giá: mọi thứ ở đoạn trên. Cần một **lượt chuẩn hoá sau mỗi `input`** để dựng lại đúng tập span — tức tự viết một nửa engine editor, đúng thứ hàng Deferred *"thư viện editor"* tồn tại vì nó.

**(b) `contenteditable="plaintext-only"` trên `.doc`.** Cắt được nhánh chèn markup của trình duyệt, và WebKit hỗ trợ giá trị này từ lâu *(nó là giá trị **gốc** của WebKit)*. Nhưng nó **không** cắt được nhánh gộp/xoá span — nó chỉ hứa về markup, không hứa về cấu trúc phần tử. Rẻ hơn (a) một bậc, cùng lớp rủi ro.

**(c) Vùng gõ là MỘT câu tại một thời điểm.** `.doc` giữ nguyên chỉ-đọc; câu mà caret đang chạm được thay bằng một bề mặt gõ được **tại chỗ** *(cùng `<span>`, cùng dòng chảy inline, chỉ thêm `contenteditable` lên đúng nó)*. Sổ sách `data-segment-id` **không bao giờ** bị trình duyệt đụng vì mỗi vùng gõ chỉ chứa **một** id và không có ranh giới nào bên trong nó để mà gộp. Cái giá: *"rời segment"* trở thành một lượt tháo/lắp `contenteditable`, và phải giữ caret không nhảy khi lắp; `⌘A` chỉ chọn một câu, không cả trang.

**(d) Một `<textarea>`/`<div contenteditable>` phủ toàn Chương, bỏ `<span>` từng câu.** **Loại thẳng** — nó xoá sạch AC2 của Story 2.2 *(vạch lề cao đúng bằng câu)* và AC4/AC5 *(ranh giới `⏐`)*, tức lật một story vừa `done` mà không ai ký.

🔵 **PHÁN QUYẾT của Ice 2026-08-12: đường (c).** Dev **không** mở lại. Lý lẽ đã cân trước khi ký, và nó là lý lẽ về **rủi ro có giá cố định**: (a) và (b) đặt cược sổ sách `segment.id` vào hành vi chuẩn hoá DOM của **hai** engine mà dự án chưa đo cái nào; (c) làm câu hỏi đó **biến mất** thay vì trả lời nó. Và (c) không phá một mệnh đề nào của Story 2.2: `<span>` vẫn chảy inline, vẫn `position: relative`, `::after` vẫn neo được, `getClientRects()` vẫn đo được. Cái giá của (c) là **đường tiêu điểm**, mà đó lại đúng chỗ dự án đã có hạ tầng *(`declareFocus`/`FOCUS_OWNERS`, và `editorCaretSegmentId` đã theo dõi caret từ 2.2)*.

**Năm hệ quả cưỡng chế được, mỗi cái là một mệnh đề nghiệm thu chứ không phải một lời khuyên:**

1. **`contenteditable` sống trên đúng MỘT `<span class="sent">` tại một thời điểm** — chính câu mà caret đang chạm. `.doc` **không** mang `contenteditable`. ⇒ trong vùng gõ **không tồn tại một ranh giới câu nào** để trình duyệt gộp qua, và `data-segment-id` của mọi câu khác nằm ngoài tầm với của lượt gõ.
2. **Lắp và tháo là hai thao tác đối xứng, và caret KHÔNG được nhảy.** Đặt `contenteditable` lên câu B rồi gỡ khỏi câu A phải giữ đúng vị trí caret người dùng vừa bấm tới — một lượt nhảy về đầu câu ở đây là thứ người dùng đọc thành *"ứng dụng ăn mất chỗ tôi đang gõ"*. Đây là ca test đắt nhất của story, và nó **phải** có test (AC24).
3. **`Enter` bên trong một câu vẫn có thể tách `<span>` đó** — (c) đóng ca *gộp qua ranh giới*, nó **không** tự đóng ca *tách trong lòng một câu*. Mũi thăm dò phải đo đúng ca này, và lời giải mặc định là chặn `Enter` trong vùng gõ *(xuống dòng là cấu trúc đoạn — AD-37 nói cấu trúc đoạn là **dữ liệu đã lưu**, không phải thứ gõ ra)*.
4. **`⌘A` chỉ chọn một câu, không cả trang** — hệ quả trực tiếp, và nó là **đúng** chứ không phải một cái giá: một `⌘A` + `Delete` trên cả Chương là một lượt mất dữ liệu mà `⌘Z` phải gánh, còn ở đây nó chỉ chạm một câu.
5. **AC18 (*"rời segment"*) và cơ chế lắp/tháo là CÙNG MỘT sự kiện** — `editorCaretSegmentId` đổi giá trị **vừa** kích hoạt flush cho câu cũ **vừa** dời `contenteditable`. Cài ở **đúng một** chỗ; hai đường riêng sẽ lệch nhau ở lượt sửa thứ hai.

🔴 **Mũi thăm dò VẪN BẮT BUỘC — phán quyết chọn đường, không miễn phép đo.** Chạy **trước** dòng mã sản phẩm đầu tiên *(cùng khuôn Task 7/Task 8 của Story 2.2 — "đo trước khi tin", retro §7.1)*, trên **cả Blink và WebKit**, ba mệnh đề đã thu hẹp theo đường (c):

1. **Sổ sách sống sót không, ở ca (c)?** Trên một câu đang `contenteditable`: gõ · xoá lùi tới **đầu** câu rồi xoá tiếp · `⌘A` + gõ đè · `Enter` giữa câu · dán một khối nhiều dòng. Sau mỗi thao tác, **đếm số phần tử `[data-segment-id]`** trên cả trang và đối chiếu số ban đầu. 🔴 Hai ca có khả năng thủng cao nhất là **xoá lùi qua đầu câu** *(caret ra khỏi vùng gõ — trình duyệt có thể gộp vào `<span>` liền trước)* và **dán nhiều dòng**.
2. **Frame nào vượt 50 ms không?** Gõ liên tục trên fixture, đo frame budget. Story 2.2 đã đo trần **dựng** DOM *(300,1 ms Blink · 1.308,0 ms WebKit cho 9.850 câu)* và giao số đó cho 2.4; số cần ở đây là chi phí **mỗi lượt gõ** cộng chi phí **lắp/tháo `contenteditable`** khi caret đi qua ranh giới — khác hẳn, và ca thứ hai chưa ai đo.
3. **`⏐` có rò ra lúc copy không?** 2.2 đã đo *"`innerText` không rò trên cả hai engine"* — nhưng đó là đo trên bề mặt **chỉ-đọc**. Đo lại trên bề mặt gõ được, và đo thêm chiều **dán vào**.

⚠️ Nếu phép đo cho thấy **cả (c) cũng không đạt**, đó là *"cần một thư viện editor"* — và khi đó **dừng lại và báo** (AC19), vì đó là hàng Deferred của **Story 2.4**, không phải một quyết định gõ vào giữa story này. Lượt lật NFR15 hôm nay cấp phép cho **ba gói test đã nêu đích danh**, không cấp phép cho một gói thứ tư.

### Quyết định #2 — Tái dùng `createWriteSchedule` hay dựng một lịch riêng cho Editor?

`src/layout/writeSchedule.ts` cài **đúng** hình dạng AD-35 và đã có một cổng chạy hành vi thật của nó (`check-layout.mjs:288`). Ba đường:

**(a) `import { createWriteSchedule }` với `idleMs = 2000`, `hardCapMs = 5000`.** Rẻ nhất, không một dòng logic mới. Cái giá: `writeSchedule.ts` sống ở `src/layout/` và doc-comment của nó khai vai *"nhịp ghi bố cục"*; một `import` từ `src/panels/` làm tên module nói dối về phạm vi của nó, và `check-layout.mjs` sẽ đang canh một tệp mà **hai** tính năng đứng lên.

**(b) Chép hàm sang `src/panels/editorFlush.ts`.** Hai bản cài đặt của cùng một `Math.min` — đúng thứ *"tái dùng, đừng viết lại"* cấm, và chúng sẽ rẽ nhau ở lượt sửa thứ hai.

**(c) Nâng `createWriteSchedule` lên một chỗ trung lập** *(ví dụ `src/config/` hoặc một tệp mới không thuộc `layout/`)*, để **cả hai** tính năng đứng lên nó, và mỗi tính năng mang hằng của riêng nó.

🔵 **PHÁN QUYẾT của Ice 2026-08-12: đường (a).** Dev **không** mở lại. Lý lẽ đo được: đường (c) là một lượt **di chuyển tệp** chạm `check-layout.mjs:295-296` *(nó `import()` tệp đó bằng **đường dẫn viết thẳng** `join(SRC_ROOT, 'layout', 'writeSchedule.ts')`, và `abort()` nếu không nạp được ⇒ đổi chỗ tệp làm Kiểm B **đỏ ngay**)*, chạm `FILE_FLOOR` của bốn cổng, và đổi lấy đúng **một** thứ — một cái tên đọc thuận hơn. Đường (a) tốn **0** dòng và giữ nguyên **một** nguồn sự thật cho một hàm mười dòng.

**Ba điều kiện kèm theo, cả ba cưỡng chế được:**

1. **Doc-comment của `writeSchedule.ts:22-28` phải được sửa cho đúng.** Nó đang nói *"đây là **mượn hình dạng** của AD-35, KHÔNG phải áp AD-35 cho bố cục"* — từ story này trở đi tệp đó **là** đường AD-35 thật, và một comment nói ngược sẽ đánh lừa đúng người đọc kỹ nhất. Sửa thành: **một hàm, hai chỗ dùng, hai cặp hằng, và chỉ chỗ dùng của Editor mang các bảo đảm của AD-35.**
2. **Hằng của Editor khai ở tệp của Editor, KHÔNG đè lên `IDLE_MS`/`HARD_CAP_MS`.** Bố cục giữ `500/5000`; Editor mang `2000/5000` khai ở `editorFlush.ts`, kèm chú thích *"TẠM — chủ là Story 2.4"* trỏ `ARCHITECTURE-SPINE.md:883`. Đổi hằng của bố cục để "cho gọn" là đổi một con số mà Story 1.14 sở hữu.
3. **`createWriteSchedule` KHÔNG được sửa hành vi ở story này.** `check-layout.mjs` Kiểm B đang đứng trên nó; một lượt sửa để "hợp Editor hơn" làm đỏ một cổng của story khác, hoặc tệ hơn là **không** làm đỏ và lặng lẽ đổi nhịp ghi bố cục. Cần một hành vi khác ⇒ đó là một tham số mới có giá trị mặc định giữ nguyên hành vi cũ, và một test cho **cả hai** giá trị.

⚠️ **Hằng của Editor không được viết đè lên `IDLE_MS`/`HARD_CAP_MS`.** Hai tính năng, hai cặp số: bố cục giữ `500/5000`, Editor mang `2000/5000` khai ở tệp của nó, kèm chú thích rằng **cả hai con số là TẠM và chủ là Story 2.4** (`ARCHITECTURE-SPINE.md:883`).

### Quyết định #3 — Hình dạng lệnh ghi: cái gì đi trên dây, và bao nhiêu lượt?

**(a) `save_segment_targets(chapterId, [{ id, target_text }])`** — một lệnh, một lô, một giao dịch, `prepare_cached` một lần.

**(b) `save_segment_target(id, text)`** — một lệnh mỗi câu.

**(c) `save_chapter_targets(chapterId, [text; N])`** — gửi cả Chương theo `ord`.

**Đề xuất mặc định: (a).** (b) bị loại bằng số: mỗi lượt là một giao dịch trên writer **duy nhất, nối tiếp** của AD-11, và Story 2.2 vừa đo rằng ngay cả chi phí **parse** trên đường đó cũng đáng 57–64 ms. (c) bị loại vì nó gửi lại **nguyên khối** những câu không đổi — Chương lớn nhất có thật là **9.850** câu / **48.640** ký tự, và AD-35 nói flush chạy **mỗi 2 giây**.

⚠️ Ba mệnh đề kèm theo, mỗi cái neo vào một tiền lệ đã có:
- Khoá theo **`segment.id`**, không theo `ord` — Story 2.8 sắp lại `ord` mà giữ nguyên `id` (AD-3), và `commands/segment.rs:125-127` đã ghi đúng luật đó cho khoá `v-for`.
- Struct qua biên IPC **không** đặt `#[serde(rename_all)]`; tham số của `invoke()` đi trên dây dưới dạng **camelCase** *(`chapterId`)* — `src/config/segment.ts:19-20` là chỗ duy nhất gõ cái tên đó.
- Lệnh giữ `MutexGuard` của `OpenWorkState` **xuyên suốt**, cùng lý do và cùng đường hỏng mà `split_chapter_into_segments` đã ghi ở `commands/segment.rs:194-215` — nhả sớm để "tối ưu" là mở lại một cuộc đua ghi.

🔴 **Mặc định là KHÔNG có bước di trú 7.** Nếu dev kết luận cần một cột mới, đó là một phát hiện phải báo kèm lý do — và số phải là **7**, không bao giờ là 4 (`schema.rs` §vết sẹo).

### Quyết định #4 — Đường flush lúc thoát ứng dụng (AC17, AD-35 vế e)

Lỗ đã đo (§AC17): không hook nào ở webview chạy trước `RunEvent::Exit`. Ba đường:

**(a) `beforeunload`/`pagehide` ở webview gọi flush.** Rẻ. Cái giá: `invoke()` là **bất đồng bộ**, và không có gì bảo đảm lượt gọi kịp về trước khi tiến trình đi — đây đúng là lớp "trông như đã lưu".

**(b) Nghe `WindowEvent::CloseRequested` ở Rust, phát một event xuống webview, chờ webview trả lời rồi mới cho đóng.** Đúng ngữ nghĩa, và hạ tầng đã có: `wire_drag_drop` (`lib.rs:492-525`) là tiền lệ **nguyên vẹn** của việc nối một `on_window_event` gốc tới webview, và nó cần **0** permission ACL *(`tests/config_invariants.rs:333` khoá `capabilities/main.json` ở đúng ba quyền — **đừng nới**)*. Cái giá: một `api.prevent_close()` cộng một đường chờ có **trần thời gian** — không có trần thì một webview treo làm ứng dụng không đóng được.

**(c) Không làm gì, dựa vào trần 5 giây.** Mất tối đa 5 giây khi thoát. **Loại** — AD-35 liệt kê vế (e) tách riêng khỏi trần chính vì trần không đủ, và *"thoát ứng dụng"* là thao tác người dùng chắc chắn nhất trong danh sách.

**Đề xuất mặc định: (b), với một trần thời gian ghi ra thành hằng.** Kèm ba điều kiện: trần phải **ngắn hơn** `close_truncate_budget = 2 s` cộng lại để `check:scope`/`check:scope:bundled` không đỏ vì tầng này *(`Tuning::close_truncate_budget` doc-comment, `core/store/mod.rs:219-226`)*; hết trần ⇒ **ghi chẩn đoán rồi đóng**, không treo; và **nói thẳng trong §Completion Notes** rằng `panic = "abort"` khiến một lần thoát cứng vẫn không đi qua đây — món nợ kế thừa, không phải món nợ đóng.

⚠️ *"Đóng Tác phẩm"* của AC3 **chưa có đường nào tồn tại** hôm nay: không có lệnh đóng `.atproj`, và `resetEditorPanel()` chỉ được gọi từ `modes/libraryImport.ts::finishSubmit` khi Tác phẩm đang mở **bị thay** (`editorPanelState.ts:129`). ⇒ **đó** là chỗ duy nhất vế "đóng Tác phẩm" chạm tới được hôm nay, và nó phải flush **trước** `resetEditorPanel()` — nếu không, một lượt tạo Tác phẩm mới ăn mất bản dịch chưa flush của Tác phẩm cũ, im lặng.

### Quyết định #5 — Thanh trạng thái sống ở đâu, và *"N giây trước"* cập nhật thế nào?

**Chỗ sống — (a) trong `App.vue::main.shell`, dưới `.modeport`** *(đúng mockup `key-screen-workspace.html:182`, và đúng `EXPERIENCE.md:417` vốn đã tính chiều cao vùng làm việc = *"chiều cao cửa sổ trừ thanh tiêu đề 38px và thanh trạng thái 32px"*)*; **(b) trong `WorkspaceMode.vue`** — chỉ hiện ở Workspace.

**Đề xuất: (a).** Nó là **vỏ ứng dụng**, không phải nội thất một chế độ: `EXPERIENCE.md:417` đã dùng nó để tính ngưỡng bố cục cho **cả** ba chế độ, và UX-DR15 hứa Panel Lookup *"rút về thanh trạng thái"* ở màn hình hẹp (Story 4.12) — một thanh chỉ tồn tại trong Workspace sẽ phải bị chuyển chỗ lần nữa ở đó. ⚠️ Story này dựng **cái vỏ + đúng một thông điệp**; **không** dựng khung mở rộng cho các thông điệp tương lai — chưa story nào đặt hàng chúng.

**Nhịp cập nhật — (a) một bộ đếm 1 giây;** **(b) chỉ cập nhật khi có flush;** **(c) đếm thô: 1 giây trong 60 giây đầu rồi thưa dần.**

**Đề xuất: (a), với một `setInterval` 1 giây DUY NHẤT sống cùng thanh trạng thái**, dừng khi chưa có lượt flush nào. Lý lẽ: (b) làm câu đứng yên ở *"Đã lưu 0 giây trước"* trong suốt 5 giây — nó **nói dối theo hướng an tâm**, đúng thứ UX-DR30 tồn tại để tránh. Cái giá của (a) đã cân: một lượt cập nhật `ref` mỗi giây, chạm **một** text node — khác hẳn hạng với lượt `v-for` trên 9.850 `<span>` mà `deferred-work.md:2143-2152` đã ghi.

⚠️ **Cấm dấu chấm *"chưa lưu"*, cấm hộp thoại** — AC7 nguyên văn, UX-DR30 nguyên văn, `EXPERIENCE.md:127` nguyên văn. Và cấm luôn một biến thể đội lốt: một câu *"đang lưu…"* nhấp nháy mỗi 2 giây là cùng một tiếng ồn dưới một cái tên khác.

### Quyết định #6 — Tệp test sống Ở ĐÂU, và cái gì được test

Hệ quả trực tiếp của lượt lật NFR15. Hai câu hỏi, và câu thứ nhất có một cái bẫy **đo được**.

**Chỗ ở — (a) đồng vị trí trong `src/**`** *(`src/panels/editorFlush.test.ts` — mặc định của Vitest)*; **(b) một cây riêng ở gốc kho, `tests/frontend/**`** *(soi gương `src-tauri/tests/`)*.

🔴 **Bẫy của (a), đo ngày 2026-08-12 — nó không làm cổng đỏ, nó làm cổng MÙ.** Bốn cổng đếm quần thể `src/**` và `abort()` khi số tệp **dưới sàn**: `TS_FLOOR = 28` trên 35 tệp `.ts` · `FILE_FLOOR = 43` trên 53 · `COMPONENT_FILE_FLOOR = 40` trên 50 · `FILE_FLOOR = 40` trên 50 (`check-commands.mjs:219` · `check-tokens.mjs:91,92` · `check-layout.mjs:97`). Sàn là **cận dưới**, nên thêm tệp **không** làm đỏ — nó **thổi phồng mẫu số**. Doc-comment của chính các cổng đó đặt ra một doctrine bằng số: sàn phải nằm ở **~80–85%** của số thật, và `check-tokens.mjs:85-91` ghi lại một lượt *"bắt kịp"* vì ba story đã để sàn tụt xuống **69,8%** — *"đúng trạng thái **canh không được gì** mà chính nó cảnh báo"*. Hai chục tệp test đổ vào `src/**` đẩy mẫu số lên mà không thêm một dòng mã sản phẩm nào ⇒ mọi story sau phải nâng sàn vì một lý do **giả**, và cái doctrine 80% mất nghĩa.

Cộng hai va chạm cụ thể hơn: `check-i18n.mjs` Kiểm A đỏ với **chữ tiếng Việt ở vị trí mã**, mà một tệp test viết cho người Việt đọc thì đầy chuỗi tiếng Việt; và `check-tokens.mjs` Kiểm B đỏ với **màu viết thẳng trong component**, mà một fixture test màu vạch cần đúng thứ đó.

**Đề xuất mặc định: (b) — `tests/frontend/**` ở gốc kho.** Tiền lệ đã có và nó đã được cấp miễn trừ **có tên**: `check-i18n.mjs` Kiểm A quét `src-tauri/**/*.rs` nhưng **`src-tauri/tests/**` được miễn trừ có tên** *(ghi ở `core/store/mod.rs:52-59`)*. Cây frontend đi đúng khuôn đó: một thư mục, một miễn trừ có tên, **0** tệp thêm vào quần thể của bốn cổng. ⚠️ Kèm điều kiện: `tsconfig` phải nhìn thấy cây mới, và `npm run build` *(`vue-tsc --noEmit` hai lượt)* phải **vẫn xanh** — một cây test không được kiểm kiểu là một cây test sẽ mục.

**Cái gì được test ở story này — đề xuất: đúng bốn nhóm, không hơn.** ① nhịp flush *(AC11)*; ② lắp/tháo `contenteditable` và caret không nhảy *(hệ quả 2 của Quyết định #1)*; ③ *"rời segment"* kích hoạt đúng một lần *(AC18)*; ④ thanh trạng thái đếm đúng *N* và dừng khi chưa có flush *(AC7, AC10)*.

🔴 **KHÔNG viết test hồi tố cho mã của Story 1.x/2.1/2.2 trong story này.** Cám dỗ sẽ rất mạnh — một bộ chạy test mới luôn mời gọi phủ ngược. Nhưng đó là phạm vi của một story riêng, và trộn nó vào đây làm diff của 2.3 không đọc được một mình *(tiền lệ Ice: cây bẩn trước story đi commit riêng)*. Ghi thành một hàng `deferred-work.md` **kèm chủ**, đừng viết.

- [x] **Task 0 — Chốt BỐN quyết định còn mở (#3 → #6).** Ghi phán quyết vào §Dev Agent Record **trước** dòng mã sản phẩm đầu tiên. Xác nhận hoặc phản biện **bằng số**. ⚠️ **#1 và #2 đã chốt bởi Ice** — không mở lại, chỉ thi hành.
  - [x] 0.1 Chạy **mũi thăm dò của Quyết định #1** trên bàn đo, **cả Blink và WebKit**, ba mệnh đề đã thu hẹp theo đường (c). Ghi bảng số vào §Debug Log References. 🔴 Đây là điều kiện để Task 3 bắt đầu.
- [x] **Task 0b — Dựng bộ chạy test frontend** (AC19, AC24; hệ quả lượt lật NFR15 do Ice ký 2026-08-12)
  - [x] 0b.1 🔴 **Rà giấy phép TRƯỚC khi thêm** — NFR15. Cài ba gói, rồi **mở tệp giấy phép thật** trong `node_modules/vitest/`, `node_modules/@vue/test-utils/`, `node_modules/happy-dom/`. Ghi **đường dẫn tệp + dòng đầu** của cả ba vào §Completion Notes. ⚠️ Trường `license` trong `package.json` của gói là một **lời khai**, không phải tệp giấy phép — cùng luật *"đặt rồi ĐỌC LẠI"*.
  - [x] 0b.2 Ghim **chính xác**, không dải `^`: `vitest@4.1.10` · `@vue/test-utils@2.4.11` · `happy-dom@20.11.2`. Cả ba vào `devDependencies`.
  - [x] 0b.3 Thêm **ba hàng** vào bảng Stack của `ARCHITECTURE-SPINE.md` *(Dev **được** đồng bộ bảng Stack — tiền lệ correct-course 2026-08-11, "+10 phụ thuộc"; `epics.md` và `DESIGN.md` thì **không**)*.
  - [x] 0b.4 `vitest.config.ts`: môi trường `happy-dom`, phạm vi quét đúng cây đã chốt ở Quyết định #6. Xác nhận `npm run check:deps` vẫn xanh *(sàn là cận dưới — cây lớn thêm không làm đỏ, nhưng đọc lại số thật)*.
  - [x] 0b.5 `npm test` vào **cả ba** danh sách: `package.json` · `ci.yml` *(sau 11 cổng `check:*`, **trước** `npm run build`)* · `.githooks/pre-push`. `npm run check:gates` xanh.
  - [x] 0b.6 Nghiệm thu **đỏ-rồi-xanh**: một test cố tình sai ⇒ **CI đỏ** và `pre-push` chặn. Không phải một cảnh báo rồi exit 0.
  - [x] 0b.7 Cập nhật ba chỗ đang khai luật cũ: `src/commands/registry.ts:10-13` · `src/commands/README.md:20` · `src/i18n/README.md:101`. Viết đúng thứ đã đổi: **có** bộ chạy test frontend từ 2026-08-12, và cửa rà giấy phép của NFR15 **vẫn đứng** cho gói tiếp theo.
  - [x] 0b.8 `npm run build` *(`vue-tsc --noEmit` hai lượt)* vẫn xanh với cây test mới trong tầm `tsconfig`.
- [x] **Task 1 — Lịch flush, module thuần** (AC1, AC2, AC11; Quyết định #2 **đã chốt = (a)**)
  - [x] 1.1 `editorFlush.ts` `import { createWriteSchedule }` từ `src/layout/writeSchedule.ts` với `idleMs = 2000`, `hardCapMs = 5000`. **Không** đọc `Date.now()` bên trong; mọi thời điểm đi vào qua tham số.
  - [x] 1.2 Hai hằng khai ở `editorFlush.ts`, kèm chú thích *"TẠM — chủ là Story 2.4"* trỏ `ARCHITECTURE-SPINE.md:883`. **Không** đè `IDLE_MS`/`HARD_CAP_MS` của bố cục.
  - [x] 1.3 **Test vitest** — ba mệnh đề định lượng của AC11. **Không** dựng thêm một cổng cho cùng ba mệnh đề (AC25).
  - [x] 1.4 Nghiệm thu **đỏ-rồi-xanh**: biến `Math.min` thành debounce thuần ⇒ mệnh đề 1 và 2 đỏ.
  - [x] 1.5 Sửa doc-comment `writeSchedule.ts:22-28` cho đúng vai mới — **một hàm, hai chỗ dùng, hai cặp hằng**; chỉ chỗ dùng của Editor mang các bảo đảm AD-35. ⚠️ **Không** đổi hành vi `createWriteSchedule`; `check-layout.mjs` Kiểm B đang đứng trên nó.
- [x] **Task 2 — Lệnh IPC ghi bản dịch** (AC4, AC12, AC13, AC14, AC15, AC16; phụ thuộc Quyết định #3)
  - [x] 2.1 Hàm **thuần** trước, `#[tauri::command]` chỉ là vỏ trong `wire` — khuôn `commands/segment.rs:216-259`, `:329-372`.
  - [x] 2.2 Một lô, **một** giao dịch, `prepare_cached` **một lần** cho câu `UPDATE`.
  - [x] 2.3 `UPDATE` chạm **đúng hai cột**: `target_text` + `updated_at` (`strftime` ở tầng SQL).
  - [x] 2.4 Giữ `MutexGuard` của `OpenWorkState` xuyên suốt — `commands/segment.rs:194-215`.
  - [x] 2.5 Lỗi theo AD-21 `{ code, message_key, params, retryable }`; `MessageKey` mới ở `core/i18n/mod.rs`; khoá vào `vi.json`; **không chữ tiếng Việt có dấu ở vị trí mã `.rs`**.
  - [x] 2.6 Đăng ký lệnh ở `lib.rs::invoke_handler` *(khối `generate_handler!`, `:228-250`)*. **Không** thêm mục ACL vào `capabilities/main.json`.
  - [x] 2.7 Adapter TS ở `src/config/segment.ts` — khuôn `{ outcome, error }`, `hasIpcBridge()`, `isIpcError()`, **không** ném.
  - [x] 2.8 Ca ở `segment_contract.rs`: lô nhiều segment (AC13) · bảy cột kia y nguyên (AC14) · round-trip đúng chữ, ranh giới không đổi (AC16).
  - [x] 2.9 Ca biên: `chapter_id` không thuộc Tác phẩm đang mở · `segment.id` không tồn tại · chưa Tác phẩm nào mở ⇒ `project.no_work_open`.
  - [x] 2.10 Doc-comment tại chỗ nêu AD-31 hàng 1 và gọi tên hai story chủ (`status` → 2.5 · `SegmentVersion` → 2.6) — AC15.
- [ ] 🔴 **Task 3 — Vùng gõ: MỘT câu tại một thời điểm** (AC8) — **KHÔNG đánh dấu xong.** Sáu subtask dưới đây **đã làm và đã nghiệm thu trên Blink** *(bàn đo hai engine · vitest · e2e vế vùng gõ xanh)*, và Kiểm J đã gỡ đúng lúc. Vế còn đỏ, **đo lại 2026-08-13 bằng một lượt `npm run test:e2e` thật**: `editor-typing-flush.e2e.mjs` = **1 xanh / 1 đỏ**, đỏ ở `:133` — **lượt gõ ĐẦU TIÊN vào một câu CHƯA DỊCH** cho `execCommand('insertText')` trả `false`, vì câu đó là một `<span>` **rỗng, rộng 0 px**, không text node để neo caret. Đây là ca **thường nhất** của tính năng: mọi Chương mới mở ra đều toàn câu rỗng.

  🔵 **ĐÍNH CHÍNH 2026-08-13 — chẩn đoán cũ của chính dòng này đã bị BÁC.** Bản trước ghi *"bấm chuột vào một câu không đặt được caret, vì một lượt dời tiêu điểm của hợp đồng **AD-34 §2** xoá vùng chọn"*. Phép đo cho thấy **không ai giành tiêu điểm cả**, và nguyên nhân thật là một lượt đặt thuộc tính **bất đồng bộ** — đã vá bằng `setAttribute` đồng bộ trong `mousedown`. **AD-34 và `focus.ts` không bị chạm một dòng**, và vế *"dừng và báo vì đụng AD Story 1.6 sở hữu"* **không còn áp dụng**. Ba hướng đã thử và ba hướng chưa thử cho ca `<span>` rỗng ghi ở `deferred-work.md` §*CÒN LẠI SAU ĐÍNH CHÍNH*.
  - [x] 3.1 `contenteditable` sống trên **đúng một** `<span class="sent">` — câu caret đang chạm. `.doc` **không** mang `contenteditable`. **Không** phá AC1/AC2/AC4/AC5 của Story 2.2 — `<span>` chảy inline, `data-segment-id` còn nguyên, `::after` còn neo được, `getClientRects()` còn đo được.
  - [x] 3.2 Lắp/tháo đối xứng, **caret không nhảy** *(hệ quả 2)*. **Test vitest** với `@vue/test-utils` — đây là ca test đắt nhất của story.
  - [x] 3.3 Chặn `Enter` trong vùng gõ *(hệ quả 3 — cấu trúc đoạn là dữ liệu đã lưu, AD-37)*; xử lý ca **xoá lùi qua đầu câu** và ca **dán nhiều dòng** theo số đo của Task 0.1.
  - [x] 3.4 Nối đường gõ vào lịch flush của Task 1 và lệnh của Task 2.
  - [x] 3.5 🔴 **Gỡ Kiểm J** khỏi `check-commands.mjs` — cả khối, không để lại cổng xanh rỗng. Làm **sau** khi Task 2.8 xanh.
  - [x] 3.6 IME: `event.isComposing` đã được `keys.ts:504` tôn trọng ở tầng hợp âm — kiểm rằng một lượt commit composition tiếng Việt **không** bị đường flush hay lượt tháo `contenteditable` cắt giữa chừng.
- [x] **Task 4 — Ba đường flush còn lại** (AC3, AC5, AC17, AC18; phụ thuộc Quyết định #4)
  - [x] 4.1 **Rời segment** = `editorCaretSegmentId` đổi giá trị. Cài ở **đúng một** chỗ, kèm doc-comment nói vì sao đó là định nghĩa (AC18). ⚠️ Cùng sự kiện đó dời `contenteditable` *(hệ quả 5 của Quyết định #1)* — **một** đường, không hai. **Test vitest**: kích hoạt **đúng một lần** cho mỗi lượt đổi.
  - [x] 4.2 **Đóng Tác phẩm**: flush **trước** `resetEditorPanel()` trong `modes/libraryImport.ts::finishSubmit`.
  - [x] 4.3 **Thoát ứng dụng**: theo phán quyết Quyết định #4, có **trần thời gian** khai thành hằng; hết trần ⇒ chẩn đoán rồi đóng, **không treo**.
  - [x] 4.4 **AC5**: một test đọc lại `PRAGMA synchronous` trên kết nối ghi và khẳng định giá trị. Đọc ra `NORMAL` ⇒ AC5 **chưa thoả**, báo — đừng vá thầm. *(**Xác nhận segment** của AC3 thuộc Story 2.5; ghi rõ trong §Completion Notes rằng vế đó chưa có đường nào chạm tới, đừng đánh dấu đạt.)*
- [x] **Task 5 — Thanh trạng thái** (AC7, AC9, AC10; phụ thuộc Quyết định #5)
  - [x] 5.1 Dựng vỏ ở chỗ đã chốt; chiều cao `var(--space-status-height)`, typography `ui` qua token.
  - [x] 5.2 Khoá `vi.json` mới có placeholder cho `N`; mọi text node đi qua `t()` (Kiểm A2).
  - [x] 5.3 `N` tính ở TS từ mốc flush cuối; **Rust không trả một chữ nào** (AC10).
  - [x] 5.4 Một `setInterval` 1 giây duy nhất, dừng khi chưa có flush nào; nhả ở `onBeforeUnmount`. **Test vitest** *(`vi.useFakeTimers()`)*: `N` đếm đúng, câu **không** hiện trước lượt flush đầu tiên, timer **được nhả** khi unmount.
  - [x] 5.5 **Không** dấu chấm *"chưa lưu"*, **không** hộp thoại, **không** *"đang lưu…"* nhấp nháy.
  - [x] 5.6 Ghi lệch `32px` (mockup + `DESIGN.md:132`) vs `34px` (token + `DESIGN.md:283`) vào `deferred-work.md`, **chủ: Ice**. Dev **không** sửa `DESIGN.md`.
- [x] **Task 6 — `isTypingZone` và hợp đồng vùng chọn** (AC21, AC22, AC23)
  - [x] 6.1 Nghiệm thu **đỏ-rồi-xanh** qua Kiểm D (`check-commands.mjs:1036`): một hợp âm trần **không** dispatch khi `event.target` mang `isContentEditable === true`.
  - [x] 6.2 Đo và ghi kết quả AC23 — Auto-Lookup trên bề mặt Editor còn chạy hay không, **kèm số**. Không còn ⇒ ghi `deferred-work.md` kèm chủ, **đừng vá mù**.
  - [x] 6.3 Xét lại đường `Selection.anchorNode` (AC22) — phán quyết kèm bằng chứng chạy thật.
  - [x] 6.4 Chiều shadow DOM / input phi văn bản của nợ `deferred-work.md:181`: đóng **hoặc** ghi lại kèm lý do đo được. Không hoãn bằng một câu chung chung.
- [x] **Task 7 — Bàn đo và e2e**
  - [x] 7.1 Bàn đo của story này *(mở rộng từ `2-2-ban-do-editor.html`, hoặc một tệp mới)* — ghi thẳng ba giới hạn kế thừa: chép DOM chứ không mount · ba font nhúng vắng mặt · `⏐` là pseudo-element nên không hiện.
  - [ ] 7.2 Nghiệm thu trong **WKWebView thật** — **một nửa đã làm, một nửa KHÔNG chạm tới được.** Spec `e2e/specs/editor-typing-flush.e2e.mjs` chạy trong cửa sổ Tauri thật và **2/2 xanh** cho vế *vùng gõ lên đúng một câu* + *`Enter` không tách câu*. Vế *"đóng app → mở lại → chữ còn đó"* **không cài được**: **không tồn tại đường mở lại một `.atproj`** *(`OpenWorkState` khởi tạo `None`; màn hình mở lại thuộc Epic 5)*. Vế *"chữ còn đó sau khi nạp lại"* nghiệm thu ở `segment_contract.rs::typed_text_round_trips_through_the_flush_and_the_load_command`. Ghi nợ kèm chủ **Epic 5**.
  - [x] 7.3 Đo NFR2 lúc gõ trên fixture; ghi số vào §Completion Notes. Vượt ⇒ **báo**, số thuộc Story 2.4.
- [x] **Task 8 — Cổng, test và sàn** (AC19, AC20, AC24, AC25)
  - [x] 8.1 `npm run check:gates` xanh; `npm test` và mọi cổng mới có mặt ở **cả ba** danh sách — `package.json`, `ci.yml`, `.githooks/pre-push` *(bài học correct-course 2026-08-11: danh sách thứ ba không ai canh)*.
  - [x] 8.2 Nâng mọi `*_FLOOR` bị vượt theo **số thật đo được**; ghi số vào §Completion Notes. ⚠️ Nếu Quyết định #6 chốt (b) — test ở `tests/frontend/**` — thì **0** tệp test vào quần thể của bốn cổng, và các sàn chỉ đổi theo tệp **sản phẩm** thật sự thêm.
  - [x] 8.3 Xác nhận **0** dependency **runtime** mới và **đúng ba** dev-dep mới, ghim chính xác. Cần một gói thứ tư ⇒ **dừng và báo** (AC19).
  - [x] 8.4 Thứ tự CI bắt buộc: 11 cổng `check:*` → **`npm test`** → `npm run build` **trước** `cargo test` *(vì `tauri::generate_context!` nhúng `dist/` lúc biên dịch)* → `cargo test` → build ứng dụng thật → `check:scope`/`check:scope:bundled`.
  - [x] 8.5 Bảng **mệnh đề → đường nghiệm thu → tệp** vào §Completion Notes; **không dòng nào hai đường** (AC25).
- [x] **Task 9 — Ghi nợ có chủ.** Mỗi món ghi `deferred-work.md` **kèm chủ**, đừng gom thành một câu *(retro §5 — nợ nghiệm thu thị giác có hệ số nhân)*.
  - [x] 9.1 Đánh dấu **ĐÓNG** cho `deferred-work.md:2135-2139` *(Kiểm J)* và cho phần đã đóng của `:180-182` *(`isTypingZone`)*, kèm bằng chứng.
  - [x] 9.2 🔵 Món **mới sinh ra từ lượt lật NFR15**: *"phủ test hồi tố cho mã Story 1.x / 2.1 / 2.2"* — cố ý **không** làm ở đây (Quyết định #6), ghi thành một hàng **kèm chủ**. Và *"di chuyển các phép kiểm hành vi từ cổng sang vitest"* (AC25) — cũng một hàng, cũng kèm chủ.
  - [x] 9.3 Món kế thừa **không** đóng: `panic = "abort"` bỏ qua đường thoát (AC17) · vế *"xác nhận segment"* của AC3 thuộc 2.5 · lệch `32px`/`34px` (chủ: Ice) · nợ WKWebView-thật của 2.2 nếu Task 7.2 không chạy tới.

---

## Dev Notes

### Cái đã có, cái chưa có — đo ngày 2026-08-12

| Thứ | Trạng thái | Nguồn |
| --- | --- | --- |
| Cột `segment.target_text` | **ĐÃ CÓ** — `NOT NULL DEFAULT ''`, bước di trú 6 | `schema.rs:368-369`, `:451-454` |
| Lệnh IPC **đọc** segment | **ĐÃ CÓ** — `read_open_chapter_segments`, không tham số | `commands/segment.rs:290-326` |
| Lệnh IPC **ghi** segment | **CHƯA CÓ** — story này là lượt đầu tiên | `lib.rs:228-250` |
| `store::Writer` nối tiếp, `Store::write` chặn | **ĐÃ CÓ** | `core/store/writer.rs` · `mod.rs:612-618` |
| `PRAGMA synchronous` khai tường minh | **CHƯA KHAI** — mặc định SQLite là `FULL`; phải **đo lại** | `core/store/pragmas.rs` |
| Lịch `idle + trần cứng không reset` | **ĐÃ CÓ** (`500/5000`, vai *"bố cục"*) | `src/layout/writeSchedule.ts:61-97` |
| Cổng chạy hành vi thật của lịch đó | **ĐÃ CÓ** — khuôn để chép | `scripts/check-layout.mjs:288` |
| `editorCaretSegmentId` *(nguồn của "rời segment")* | **ĐÃ CÓ** | `editorPanelState.ts:49`, `:117-119` |
| `resetEditorPanel()` *(chỗ duy nhất "đóng Tác phẩm" chạm tới)* | **ĐÃ CÓ**, một chỗ gọi | `editorPanelState.ts:131-141` · `modes/libraryImport.ts:117` |
| **Thanh trạng thái** | **CHƯA CÓ** — token `status-height: 34px` có, phần tử không | `tokens.json:480` · `src/App.vue:139-251` |
| Cổng cấm gõ (**Kiểm J**) | **ĐANG CHẶN** — story này gỡ | `check-commands.mjs:2068-2135` |
| Cột `segment.status` · bảng `segment_version` | **CHƯA CÓ**, và **không cần** ở story này | `schema.rs:295` · AD-31 hàng 1 |
| Hook webview chạy trước `RunEvent::Exit` | **CHƯA CÓ** — chỉ `DragDrop` được nghe | `lib.rs:272-278`, `:521-522` |
| **Bộ chạy test frontend** | **CHƯA CÓ — story này dựng**. Ice lật NFR15 ngày 2026-08-12 | `package.json` · §Điều kiện khởi hành mục 8 |
| Bộ chạy test **trong webview thật** | **ĐÃ CÓ** — WebdriverIO 9.30.1, chập chờn 6/8 | `package.json` · `e2e/**` |
| Thư viện **editor** · ảo hoá danh sách dài | **CHƯA CÓ**, và cả hai **không** thuộc story này | SPINE:886,888 · AC19 |

### Ranh giới AD — cái story này được phép và không được phép

**AD-1** (`SPINE:75-79`) — *"frontend chỉ render và giữ state UI… Ngoại lệ **duy nhất, tường minh**: văn bản đang gõ trong Editor là state cục bộ frontend, chỉ qua IPC khi auto-save, xác nhận segment, hoặc rời segment."* 🔴 **Story này LÀ ngoại lệ đó** — và ngoại lệ dừng ở *"văn bản đang gõ"*. Không quy tắc nghiệp vụ nào khác được sinh ra ở TS: không tách câu lại *(`Intl.Segmenter` **đã có mặt** trong kho ở `wordBoundary.ts` cấp **TỪ** — đừng nhìn thấy nó rồi dùng cho câu)*, không suy trạng thái, không suy đoạn.

**AD-35** (`SPINE:419-425`) — cả năm vế (a)…(e), cộng hai mệnh đề dễ bỏ sót: flush *"đi qua **đúng `store::Writer` nối tiếp** của AD-11, không mở kết nối riêng"*, và *"chỉ được coi là xong **sau khi đã ghi vào WAL** — nếu chỉ vào hàng đợi trong bộ nhớ thì ngưỡng 5 giây của NFR18 không bảo đảm gì"*. Vế cuối của AD-35 *(“thao tác rời rạc ghi ngay, không đi qua bộ đệm gõ” — FR94, FR58)* thuộc **Epic 7/Epic 8**; story này không cài nó, nhưng **đừng** dựng một đường ghi mà chúng không dùng lại được.

**AD-11** (`SPINE:153-157`) — *"không module nào được tự mở kết nối ghi"*. Đã cưỡng chế bằng trình biên dịch + `store_boundary.rs`.

**AD-12** (`SPINE:159-163`) — `idle_before_passive = 5 s` được đặt **cố ý dài hơn** nhịp flush 2 s. Đổi nhịp flush là đổi giả định của tầng kho.

**AD-31** (`SPINE:368-392`) — auto-save: trạng thái **không đổi**, `SegmentVersion` **không** tạo. Và một mệnh đề mà story này phải **không phá**: xuất xứ (FR117, Story 2.7) so *"văn bản đích **hiện tại** với bản **lúc nạp segment**"*, **không** dùng cờ dirty. ⇒ Story này được phép dùng một tập "đã đổi" để **quyết định gửi gì**, nhưng nó **không được huỷ** bản gốc lúc nạp — `editorSegments` giữ đúng vai đó hôm nay và phải giữ tiếp.

**AD-4** (`SPINE:95-101`) — ranh giới tính **một lần lúc nhập**, không bao giờ tính lại. Gõ không phải một lượt tách. **Gộp ngầm (UX-DR32) là Story 2.9.**

**AD-21** (`SPINE:302-306`) — Rust không bao giờ trả văn bản hiển thị. `params` là `chuỗi → chuỗi`; số đi trên dây dưới dạng **chuỗi**.

**AD-34 §2** (`SPINE:406-417`) — mỗi panel khai điểm vào focus. Đã thoả từ 2.2 (`PanelFrame` → `declareFocus`); nếu Quyết định #1 chốt đường (c), điểm vào có thể cần chi tiết hơn *(câu đang dở)* — khai **tường minh**, đừng dựa vào focus mặc định của trình duyệt.

### Chuẩn kiểm thử của kho

- **Hai loại tệp test Rust**, phân theo hậu tố: `*_contract.rs` = hành vi lúc chạy · `*_boundary.rs` = kiểm tĩnh trên cây nguồn.
- **Tên hàm test** là một câu mô tả hành vi, `snake_case`, **không** tiền tố `test_`. Ví dụ có thật: `a_retired_chapter_id_is_never_handed_out_again` · `the_language_branch_comes_from_source_lang_not_from_the_content` · `the_project_migration_set_never_reuses_the_burned_number_four`.
- **Không** script `test` trong `package.json` — Rust test chạy `cargo test --locked --manifest-path src-tauri/Cargo.toml`.
- Cổng cưỡng chế hành vi ở TS chạy bằng cách **`import()` thẳng hàm thật** từ `scripts/check-*.mjs`; nó chỉ áp cho **module thuần**, không cho `.vue`.
- Test lái cơ chế bằng `Tuning` **thu nhỏ** *(tick và idle tính bằng chục mili-giây)* thay vì chờ 5 giây thật — nhân với hai nền tảng CI thì đó là phút, và §Testing standards cấm. Áp đúng luật đó cho lịch flush: **tham số hoá**, đừng `sleep`.

### Bài học Epic 1 · Story 2.1 · Story 2.2 áp thẳng vào story này

1. **Đo trước khi tin** (retro §7.1) — Quyết định #1 **không** được quyết bằng lý lẽ. Story 2.1 quyết Quyết định #1 bằng một cây thăm dò chạy thật và tái lập 5/5 hàng; Story 2.2 vá `insert_segments` **sau** ba lượt đo. Cùng chuẩn.
2. **Kiểm điều kiện đo trước khi lật một quyết định** — số đọc được theo trạng thái hiện tại có thể bác oan một quyết định đúng. Với AC5, đừng kết luận từ *"không thấy `PRAGMA synchronous` trong mã"*; đọc giá trị thật ra.
3. **Cổng mới phải vào CI, không chỉ chạy tay** (retro §4) — và nay là **ba** danh sách: `package.json`, `ci.yml`, `.githooks/pre-push`.
4. **Một luật ngoài đơn hàng phải được ghi ra và lật được** — Story 2.1 thêm luật *"một câu phải có ít nhất một chữ"*; Story 2.2 thêm dòng *"chưa câu nào có bản dịch"*. Cả hai ghi `deferred-work.md` với chủ là Ice và *"chỗ lật là một dòng"*. Cùng chuẩn nếu story này sinh ra một luật hiển thị ngoài AC.
5. **Dev không sửa tài liệu quy hoạch** — tiền lệ quyết định #3 của Ice ở Story 1.3, giữ qua toàn Epic 1 và Epic 2. Lệch `32px`/`34px` đi vào `deferred-work.md`, **không** vào `DESIGN.md`.
6. **`in-progress` không phải chỗ đậu** (retro §8.2) — phải để dở thì ghi **nguyên nhân cụ thể** trong story file.
7. **Năng lực chưa dựng ≠ lệch spec** — vế *"xác nhận segment"* của AC3 chưa có đường nào chạm tới (Story 2.5). Ghi nợ có chủ; **đừng** sửa `epics.md`, và **đừng** đánh dấu AC3 đạt trọn vẹn.
8. **Ký hiệu cấm** — emoji "biển cấm" `U+26D4` đã gỡ khỏi toàn kho (8.298 ca, 0 còn lại). Viết `không`/`KHÔNG` thẳng.
9. **Gặp một lượt e2e đỏ không tái lập được thì BẮT NGUYÊN VĂN TRƯỚC** — action item còn `open` của Epic 1.
10. 🔵 **Lật một quyết định thì lật đúng ĐIỀU KIỆN của nó** — lượt lật NFR15 hôm nay là ví dụ sống: luật cũ chặn bằng một **quy trình rà giấy phép**, không bằng một mệnh đề *"không chạy được test"*. Một lượt lật đọc sai điều kiện sẽ hoặc bác oan một luật còn đúng, hoặc gỡ luôn cái cửa mà luật đó dựng. Ở đây: cửa **ở lại** (AC19), chỉ lời khai được sửa (Task 0b.7).

### Git intelligence — 5 commit gần nhất

`6a9777b` Story 2.2 hạ cánh *(`editorGutter.ts` + `editorSegments.ts` + `editorPanelState.ts` mới, `EditorPanel.vue` từ 39 → 519 dòng, cột `target_text` bước 6, lệnh `read_open_chapter_segments`, Kiểm I + Kiểm J)* · `c86c2fb` Story 2.1 *(bảng `segment`, bước 5, `core/segment/split.rs`)* · `f950332` mở lại `push` + `pull_request` trong CI · `8ae61cd` thoát chuỗi PowerShell trong step đo `.msi` · `788a4ae` story 2.1 sẵn sàng cho dev.

Đọc được từ đó: hai lượt gần nhất đi **cùng một cặp** — một nửa `src-tauri/src/core/**` + `commands/segment.rs` + `schema.rs`, một nửa `src/panels/**`. Story này đi **đúng cặp đó lần thứ ba**, cộng hai vùng mới chưa story nào chạm ở Epic 2: `src/App.vue` *(thanh trạng thái)* và `src-tauri/src/lib.rs` *(đường thoát)*. Khuôn thông điệp commit của kho: `<type>(<scope>): <câu tiếng Việt mô tả điều đã thay đổi>`.

### Phụ thuộc mới — ba gói, và ranh giới của giấy phép đó

**Runtime: 0 gói mới.** Ba dependency runtime giữ nguyên, ghim chính xác: Vue **3.5.40** · `dockview-vue` **7.0.4** · `@tauri-apps/api` **2.11.1**.

**Dev: đúng ba gói mới**, ghim chính xác, đã đo tương thích (§Điều kiện khởi hành mục 8): `vitest@4.1.10` · `@vue/test-utils@2.4.11` · `happy-dom@20.11.2` — cả ba **MIT**. Dev-dep đã có, đáng nhắc: TypeScript **5.9.3** · Vite **8.2.0** · `@wdio/*` **9.30.1** · toolchain Rust **1.97.1** *(ghim đúng số máy Ice đang chạy — `@stable` sẽ trôi và làm số đo hết so sánh được)*.

🔴 **Giấy phép này hẹp, và hẹp có chủ ý.** Nó cấp cho **ba gói được gọi tên**, không cấp cho *"phụ thuộc nào phục vụ việc test"* và tuyệt đối không cấp cho một thư viện **editor** (AC19). Cửa NFR15 — mở tệp giấy phép thật, rồi vào bảng Stack, **trước khi** thêm — vẫn đứng nguyên cho gói thứ tư. Đó là lý do Task 0b.7 đi sửa **lời khai** ở ba tệp thay vì xoá nó: một dòng *"dự án không có bộ chạy test frontend"* để lại sau lượt này là sổ sách nói dối, nhưng xoá trắng nó là đánh mất luôn cái cửa.

### Thông tin kỹ thuật mới nhất — vì sao phần này ngắn, và chỗ duy nhất nó không ngắn

Story này **không thêm phụ thuộc nào** và không chạm API bên ngoài nào: bề mặt là DOM tiêu chuẩn cộng SQLite qua `rusqlite` đã ghim. Không có phiên bản thư viện nào phải tra, không breaking change nào áp vào.

⚠️ **Một ràng buộc nền tảng không tra được từ tài liệu, chỉ đo được** — và nó là trung tâm của Quyết định #1: hành vi **chuẩn hoá DOM khi gõ trong `contenteditable`** không được chuẩn hoá giữa các engine, và `contenteditable="plaintext-only"` có lịch sử hỗ trợ lệch *(nó là giá trị **gốc của WebKit**, và Chromium hỗ trợ muộn hơn nhiều)*. Dự án chạy trên **WKWebView của macOS** hôm nay và sẽ phải chạy trên **WebView2/Chromium của Windows** ở lượt Ice mở lại phần Windows. ⇒ mọi kết luận của Task 0.1 phải có **hai cột**, một cho mỗi engine. Đây đúng là lớp nợ mà `deferred-work.md:145` đã đặt tên, và story này là chỗ nó đắt nhất.

---

### Project Structure Notes

Tệp **mới** story này dự kiến tạo *(tên là gợi ý, không phải mệnh lệnh — hình dạng cuối theo phán quyết Task 0)*:

```
src/panels/editorFlush.ts           # nhịp flush + tập segment đã đổi (module THUẦN)
src/StatusBar.vue                   # thanh trạng thái, vỏ ứng dụng (Quyết định #5)
vitest.config.ts                    # môi trường happy-dom, phạm vi quét (Task 0b.4)
tests/frontend/**                   # cây test — CHỖ Ở theo Quyết định #6, mặc định NGOÀI `src/**`
```

⚠️ **Vì sao cây test nằm ngoài `src/**`** — xem Quyết định #6: bốn cổng đếm quần thể `src/**` với doctrine sàn ~80–85%, và tệp test đổ vào đó **thổi phồng mẫu số** mà không thêm một dòng mã sản phẩm nào. Cộng hai va chạm cụ thể: Kiểm A của `check-i18n` đỏ với chữ tiếng Việt ở vị trí mã, Kiểm B của `check-tokens` đỏ với màu viết thẳng — cả hai đều là thứ một tệp test bình thường sẽ có. Tiền lệ miễn trừ có tên đã tồn tại cho `src-tauri/tests/**`.

Tệp **sửa**:

```
src/panels/EditorPanel.vue                # vùng gõ, nối lịch flush (Quyết định #1)
src/panels/editorPanelState.ts            # mốc flush cuối, tập đã đổi, hook "rời segment"
src/config/segment.ts                     # + adapter lệnh ghi
src/App.vue                               # + thanh trạng thái vào `main.shell`
src/modes/libraryImport.ts                # flush TRƯỚC `resetEditorPanel()`
src/i18n/vi.json                          # khoá thanh trạng thái + khoá lỗi mới
package.json                              # + 3 devDependencies ghim chính xác, + script `test`
.github/workflows/ci.yml                  # + bước `npm test`
.githooks/pre-push                        # + `npm test` (danh sách thứ BA)
src/commands/registry.ts                  # sửa lời khai "không bộ chạy test frontend" (Task 0b.7)
src/commands/README.md · src/i18n/README.md  # cùng lời khai đó, hai chỗ còn lại
src-tauri/src/commands/segment.rs         # + lệnh ghi (hàm thuần + vỏ `wire`)
src-tauri/src/core/i18n/mod.rs            # + MessageKey mới
src-tauri/src/lib.rs                      # + đăng ký lệnh, + đường flush lúc thoát (Quyết định #4)
src-tauri/tests/segment_contract.rs       # ca mới
src-tauri/tests/store_contract.rs         # ca `PRAGMA synchronous` (AC5 · Task 4.4)
scripts/check-commands.mjs                # GỠ Kiểm J; sàn
scripts/check-layout.mjs  (hoặc mới)      # cổng ba mệnh đề định lượng của nhịp flush (AC11)
```

**Không** đụng: `src/layout/workspaceLayout.ts` · `WorkspaceDock.vue` · `PanelFrame.vue` *(hợp đồng focus đã thoả)* · `src/panels/SourcePanel.vue` · `core/segment/split.rs` *(AD-4 đóng băng ranh giới)* · `capabilities/main.json` *(`config_invariants.rs:333` khoá ở đúng ba quyền)* · **hành vi** của `createWriteSchedule` *(Kiểm B đang đứng trên nó)* · các phép kiểm hành vi trong `check-layout.mjs` Kiểm B và `check-commands.mjs` Kiểm C/D/E *(AC25 — không di chuyển sang vitest ở story này)* · `epics.md` và `DESIGN.md` *(lượt riêng của Ice)*.

⚠️ **`ARCHITECTURE-SPINE.md` thì Dev ĐƯỢC sửa** — và chỉ **bảng Stack**, chỉ để thêm ba hàng phụ thuộc mới. Tiền lệ: correct-course 2026-08-11 đã đồng bộ bảng đó *(+10 phụ thuộc, `lint_spine.py` 0 findings)* với chủ là Dev. Đừng chạm AD nào, đừng chạm hàng Deferred nào.

Quy ước đặt tên đã đo: Rust `snake_case` · Vue `PascalCase.vue` · state của panel là `<tênPanel>State.ts` cùng thư mục · khoá i18n phẳng theo dấu chấm có tiền tố miền · command trên dây `snake_case`, tham số `camelCase` *(do `invoke()` tự chuyển)* · struct qua biên IPC **không** đặt `#[serde(rename_all)]` · index đặt tên `idx_<bảng>_<cột theo thứ tự>`.

---

### References

- AC nguyên văn — `_bmad-output/planning-artifacts/epics.md:2075-2115`
- FR100 — `epics.md:290` · NFR2 — `:326` · NFR18 — `:368` · UX-DR30 — `:563` · UX-DR32 *(gộp ngầm, Story 2.9)* — `:569`
- Hợp đồng flush ở dạng bảng — `epics.md:415`
- AD-1 — `ARCHITECTURE-SPINE.md:75-79` · AD-4 — `:95-101` · AD-11 — `:153-157` · AD-12 — `:159-163` · AD-21 — `:302-306` · AD-31 — `:368-392` · **AD-35 — `:419-425`** · AD-34 — `:406-417`
- Hàng Deferred *ngưỡng WAL + nhịp flush* — `ARCHITECTURE-SPINE.md:883` · *thư viện editor* — `:886` · *ảo hoá danh sách dài* — `:888`
- *"Đã lưu N giây trước"* — `ux-designs/.../EXPERIENCE.md:127` · thanh trạng thái trong phép tính bố cục — `:417` · typography `ui` — `DESIGN.md:226`, `:316` · bảng token spacing — `DESIGN.md:283`
- Token `status-height` — `src/tokens/tokens.json:480`
- `Store::write` / `Store::read` — `src-tauri/src/core/store/mod.rs:605-637` · `Tuning` sáu số tạm — `:175-240` · `StoreError` + `From<StoreError> for IpcError` — `:288-496`
- Writer nối tiếp, chống gọi lồng — `core/store/writer.rs:47-55`, `:120-182` · PASSIVE/TRUNCATE — `core/store/checkpoint.rs:1-34`
- Ba PRAGMA, luật *"đặt rồi ĐỌC LẠI"* — `core/store/pragmas.rs:140-226` · `WalUnavailable` — `core/store/mod.rs:308-321`
- `insert_segments` + bảng đo `prepare_cached` — `commands/segment.rs:70-116` · khoá `OpenWorkState` chắn cuộc đua ghi — `:194-215` · lệnh đọc segment — `:290-326` · vỏ `wire` — `:329-372`
- `SEGMENT_DDL` + index — `core/store/schema.rs:329-343` · `SEGMENT_TARGET_TEXT_DDL` — `:345-369` · `PROJECT_MIGRATIONS` + vết sẹo số 4 — `:371-455`
- Đăng ký command — `src-tauri/src/lib.rs:228-250` · `RunEvent::Exit` — `:272-278` · `close_open_work` — `:483-491` · tiền lệ `on_window_event` — `:492-525`
- Lịch `idle + trần cứng` — `src/layout/writeSchedule.ts:31-97` · mô phỏng không đồng hồ — `:110-131` · cổng của nó — `scripts/check-layout.mjs:288`
- `EditorPanel.vue` *(bề mặt 2.2 dựng)* — `src/panels/EditorPanel.vue` · state panel — `editorPanelState.ts` · năm giá trị vạch — `editorSegments.ts:42`, `:108-116` · hình học — `editorGutter.ts:71-108`
- `isTypingZone` + luật vùng gõ — `src/commands/keys.ts:415`, `:434-439`, `:510`
- Hợp đồng vùng chọn — `src/panels/selectionContract.ts` · lời gọi ở Editor — `EditorPanel.vue:67` · sàn — `check-commands.mjs:1838`
- **Kiểm J** *(cổng phải gỡ)* — `scripts/check-commands.mjs:2068-2135` · Kiểm I *(năm giá trị vạch)* — `:1918` · Kiểm D *(hai nền tảng, lái được bằng object giả)* — `:1036`
- Kiểm A2 *(mọi text node qua `t()`)* — `scripts/check-i18n.mjs:900` · Kiểm B *(văn phạm khoá)* — `:1018` · Kiểm C *(placeholder)* — `:1122`
- Kiểm B *(màu viết thẳng)* — `scripts/check-tokens.mjs:813` · Kiểm E *(sàn giãn dòng)* — `:1370` · Kiểm H *(focus ring)* — `:1457`
- Sàn cổng — `check-commands.mjs:211,219,226,244,245,1838` · `check-i18n.mjs:279,289` · `check-tokens.mjs:91,92` · `check-layout.mjs:97`
- **Lời khai NFR15 cũ, phải sửa** *(Task 0b.7)* — `src/commands/registry.ts:10-13` · `src/commands/README.md:20` · `src/i18n/README.md:101`
- **Doctrine sàn ~80–85%** *(lý lẽ của Quyết định #6)* — `check-commands.mjs:200-245` · `check-tokens.mjs:85-92` · `check-layout.mjs:90-97`
- **Miễn trừ có tên cho một cây test** *(tiền lệ của `tests/frontend/**`)* — `core/store/mod.rs:52-59`
- Cổng phụ thuộc, sàn cây, và luật *"exit khác 0 khi thất bại"* — `scripts/check-deps.mjs:1-52`
- **Bảng Stack là tài liệu Dev ĐƯỢC đồng bộ** — action item Epic 1 *(“`ARCHITECTURE-SPINE.md` lỗi thời so với mã — 8 chỗ… bảng Stack +10 phụ thuộc… `lint_spine.py` 0 findings”)*, `sprint-status.yaml` §action_items
- Nợ `isTypingZone` — `deferred-work.md:181-182` · nợ **Kiểm J hết hạn** — `:2135-2139` · nợ *"xét lại `Selection.anchorNode` ở 2.3"* — `:2120-2126` · nợ WKWebView thật — `:2127-2134` · giới hạn bàn đo 2.2 — `:2113-2119` · trần NFR2 dựng 9.850 span — `:2100-2112` · nợ *"mọi bằng chứng trên Blink"* — `:145` · vết sẹo rò ký tự chèn — `:839-848` · không bộ chạy test frontend — `:875-882`
- Bài học Epic 1 — `_bmad-output/implementation-artifacts/epic-1-retro-2026-08-11.md` §4, §5, §7.1, §8.1, §8.2
- Story trước — `_bmad-output/implementation-artifacts/2-2-panel-editor-lien-mach.md` · bàn đo — `2-2-ban-do-editor.html`

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, BMad `dev-story` workflow) — 2026-08-12.

### Phán quyết Task 0 — ghi TRƯỚC dòng mã đầu tiên

🔵 **Quyết định #1 = (c)** và **#2 = (a)** do Ice ký 2026-08-12. Không mở lại; hai mục dưới chỉ ghi lại để bảng đọc được một mình. Bốn phán quyết còn lại kèm số đo, không kèm lý lẽ suông.

| # | Phán quyết | Số đo chống lưng |
| --- | --- | --- |
| #1 | **(c)** — vùng gõ là MỘT câu tại một thời điểm | Ice ký. Mũi thăm dò vẫn chạy (Task 0.1) |
| #2 | **(a)** — `import { createWriteSchedule }`, hằng riêng ở `editorFlush.ts` | Ice ký |
| #3 | **(a)** — `save_segment_targets(chapterId, [{id, target_text}])`, **KHÔNG bước di trú 7** | dưới |
| #4 | **(b)** — `WindowEvent::CloseRequested` + `prevent_close` + trần thời gian | dưới |
| #5 | **(a)** + **(a)** — thanh trạng thái ở `App.vue`, `setInterval` 1 giây | dưới |
| #6 | **(b)** — cây test ở `tests/frontend/**` | dưới |

#### Quyết định #3 — hình dạng lệnh ghi: **(a)**, và **KHÔNG có bước di trú 7**

Xác nhận đề xuất mặc định, và hai vế bị loại đều bị loại **bằng số đã có trong kho**, không bằng linh cảm:

- **(b) một lệnh mỗi câu — loại.** `insert_segments` vừa đo 2026-08-12: riêng chi phí **parse** trên đường ghi đáng **57,15 – 64,19 ms** cho 9.850 hàng (`commands/segment.rs:80-84`). Mỗi lượt `invoke` là **một giao dịch** trên writer duy nhất, nối tiếp của AD-11, và `Store::write` **chặn** (`core/store/mod.rs:612-618`) ⇒ N câu đã đổi trong một nhịp flush cho N lượt chặn xếp hàng.
- **(c) gửi cả Chương theo `ord` — loại.** Chương lớn nhất có thật: **9.850** câu / **48.640** ký tự. AD-35 nói flush chạy mỗi 2 giây ⇒ đó là 48.640 ký tự qua dây **mỗi 2 giây** để ghi lại phần lớn là những câu không đổi.

🔴 **Không bước di trú 7.** Đo lại `schema.rs:431-455`: `PROJECT_MIGRATIONS` = `[1, 2, 3, 5, 6]`, target **6**; bước 6 = `SEGMENT_TARGET_TEXT_DDL`. Chín cột của `segment` hôm nay **đã đủ** cho story này — `target_text` để ghi, `updated_at` để đóng dấu. AD-31 hàng 1 nói auto-save **không đổi trạng thái**, và một auto-save không đổi trạng thái thì không cần một cột trạng thái để mà không đổi. `status` giữ chủ là Story 2.5, `segment_version` giữ chủ là Story 2.6. ⇒ story này **không chạm `schema.rs`**, và số 7 không được tiêu.

#### Quyết định #4 — đường flush lúc thoát: **(b)**, trần **1.200 ms**

Xác nhận đề xuất mặc định. Lý lẽ đo được cho **(b)** thay vì **(a)**: `invoke()` là bất đồng bộ và `RunEvent::Exit` **không chờ** ai — `lib.rs:272-278` gọi `close_open_work` ngay, và `close_open_work` `take()` `OpenWork` rồi `store.close()`. Một `beforeunload` gọi `invoke` ở đó là đúng lớp *"trông như đã lưu"*: lượt ghi có thể vẫn đang bay khi kho đóng. Hạ tầng của **(b)** đã có nguyên vẹn: `wire_drag_drop` (`lib.rs:492-525`) là tiền lệ nối `on_window_event` gốc tới webview, và nó cần **0** permission ACL.

**Trần = 1.200 ms**, và con số này chọn bằng phép trừ chứ không bằng cảm giác:

- `Tuning::close_truncate_budget = 2.000 ms` (`core/store/mod.rs:219-226`), và doc-comment của nó ghi thẳng rủi ro: `check:scope`/`check:scope:bundled` chạy nhị phân **với timeout cứng** rồi đọc dòng `VERDICT:`, nên một đường đóng chậm làm **hai cổng của Story 1.2/1.3 đỏ vì tầng ghi**, không vì phạm vi mà chúng canh.
- Đường thoát sau lượt chờ vẫn phải trả tiếp `close_truncate_budget` cho TRUNCATE ⇒ tổng xấu nhất = trần flush + 2.000 ms. Chọn 1.200 ms cho tổng **3,2 s**, dưới mọi timeout của hai cổng đó.
- Hết trần ⇒ **ghi chẩn đoán ra `stderr` rồi đóng**, không treo. Một webview treo không được quyền làm ứng dụng không đóng được.

⚠️ `panic = "abort"` khiến một lần thoát cứng **không** đi qua đường này. Món nợ **kế thừa** từ `close_global_store`, story này không đóng nó — ghi lại ở §Completion Notes, không đánh dấu đạt.

#### Quyết định #5 — thanh trạng thái: **(a)** chỗ ở, **(a)** nhịp cập nhật

Xác nhận cả hai đề xuất. Chỗ ở **(a)** vì `EXPERIENCE.md:417` đã dùng thanh trạng thái để tính chiều cao vùng làm việc cho **cả ba** chế độ, và UX-DR15 hứa Panel Lookup *"rút về thanh trạng thái"* ở màn hình hẹp (Story 4.12) — một thanh chỉ sống trong `WorkspaceMode.vue` sẽ phải chuyển chỗ lần nữa ở đó. Nhịp **(a)** vì **(b)** làm câu đứng yên ở *"Đã lưu 0 giây trước"* suốt 5 giây, tức nói dối theo hướng an tâm — đúng thứ UX-DR30 tồn tại để tránh.

Cái giá của **(a)** đã cân bằng số: một lượt gán `ref` mỗi giây chạm **một** text node, khác hẳn hạng với lượt `v-for` trên 9.850 `<span>` mà `deferred-work.md:2143-2152` đã ghi. Chiều cao đọc `var(--space-status-height)` = **34px** (`tokens.json:480`), **không** `32px` của mockup.

#### Quyết định #6 — cây test: **(b)** `tests/frontend/**`

Xác nhận đề xuất mặc định. Đo lại quần thể hôm nay, 2026-08-12, **trước** một dòng mã nào của story:

| Cổng | Sàn | Quần thể THẬT | Tỷ lệ |
| --- | --- | --- | --- |
| `check-commands.mjs:219` `TS_FLOOR` | 28 | **35** tệp `.ts` trong `src/**` | 80,0 % |
| `check-commands.mjs:211` `VUE_FLOOR` | 13 | **15** tệp `.vue` | 86,7 % |
| `check-tokens.mjs:91` `FILE_FLOOR` | 43 | **53** *(50 `src/**` + 3 tệp token)* | 81,1 % |
| `check-tokens.mjs:92` `COMPONENT_FILE_FLOOR` | 40 | **50** tệp component | 80,0 % |
| `check-layout.mjs:97` `FILE_FLOOR` | 40 | **50** tệp `src/**` | 80,0 % |
| `check-i18n.mjs:279` `RS_FLOOR` | 36 | **43** *(42 tệp `src-tauri/src/**` + `build.rs`)* | 83,7 % |

Cả sáu sàn đang nằm trong doctrine 80–85 %. Sàn là **cận dưới**, nên tệp test đổ vào `src/**` **không** làm cổng đỏ — nó thổi phồng mẫu số, và mọi story sau phải nâng sàn vì một lý do **giả**. `check-tokens.mjs:85-91` ghi lại đúng một lượt *"bắt kịp"* sau khi ba story để sàn tụt xuống **69,8 %**; đường **(a)** dựng lại chính lượt tụt đó, chỉ nhanh hơn. Tiền lệ miễn trừ có tên đã có sẵn: `src-tauri/tests/**` được miễn trừ Kiểm A của `check-i18n` (`core/store/mod.rs:52-59`).

**Cái gì được test — đúng bốn nhóm**, không hơn: ① nhịp flush (AC11) · ② lắp/tháo `contenteditable`, caret không nhảy · ③ *"rời segment"* kích hoạt đúng một lần (AC18) · ④ thanh trạng thái đếm đúng *N* và im trước lượt flush đầu (AC7, AC10). **Không** phủ test hồi tố cho mã 1.x/2.1/2.2 — ghi nợ kèm chủ (Task 9.2).

### Debug Log References

#### Task 0.1 — mũi thăm dò Quyết định #1, đường (c), CẢ HAI engine · 2026-08-12

**Bàn đo:** `_bmad-output/implementation-artifacts/2-3-ban-do-vung-go.html` *(bản chép cơ chế (c): `.doc` chỉ-đọc, `contenteditable` trên đúng một `<span class="sent">`)*.
**Cách lái:** Playwright `playwright-core@1.62.1` — **chuột thật** (`mouse.click`) và **phím thật** (`keyboard.press`/`type`), không `element.click()`, không `dispatchEvent`. Bài học Story 1.22 C2: một bàn đo bấm bằng `element.click()` xanh nhờ một hành vi mà chuột thật không có.
**Engine:** Blink **HeadlessChrome/151.0.7922.34** · WebKit **605.1.15 / Version 26.5**.
⚠️ Playwright **không** vào `package.json` — nó chạy từ bộ nhớ đệm `npx` ngoài kho, cùng khuôn Story 2.2. AC19 giữ nguyên: **0** runtime, **3** dev-dep.

**① SỔ SÁCH `data-segment-id` — 6/6 sống sót, cả hai engine, cả sáu thao tác**

| Thao tác | Blink | WebKit |
| --- | --- | --- |
| gõ 12 ký tự giữa câu 2 | 6→6, id 1-6, 1 vùng gõ | 6→6 |
| **xoá lùi QUA đầu câu** *(Home + 8×Backspace)* | 6→6 | 6→6 |
| `⌘A` rồi gõ đè | 6→6 | 6→6 |
| `Enter` giữa câu *(chặn)* | 6→6, chặn 2/2 lượt | 6→6, chặn 2/2 |
| dán khối 3 dòng | 6→6 | 6→6 |
| dán THẬT *(`⌘C` từ trang → `⌘V`)* | 6→6 | 6→6 |

⇒ Vế trung tâm của đường (c) **đứng**: không câu nào mất, không câu nào nhân đôi, và ca *"xoá lùi qua đầu câu"* — ca thủng cao nhất theo dự đoán của story — **không thủng**, vì ranh giới `<span>` **là** ranh giới editing host, nên caret không đi ra được để mà gộp.

**🔴 ① vế hai — SỔ SÁCH SỐNG, NHƯNG MỘT LỖ KHÁC MỞ RA, và story chưa gọi tên nó**

Đếm id chỉ trả lời *"có câu nào mất không"*. Nó **không** trả lời *"engine có tiêm cấu trúc vào TRONG một câu không"*. Đo tiếp bằng `innerHTML` + `textContent`, bốn biến thể, hai engine — sau khi sửa một phép đo hỏng *(xem §Ba lượt đo hỏng dưới đây)*:

| Biến thể | Blink: phần tử trong câu · có `\n` | WebKit: phần tử trong câu · có `\n` |
| --- | --- | --- |
| **A** `contenteditable="true"`, không lọc | **1** *(`<pre id="clipsrc">`)* · **có** | **2** *(`<span style>` + `<pre style>`)* · **có** |
| **B** `plaintext-only`, không lọc | 0 · **CÓ** | **2** *(`<div>BBB</div><div>CCC</div>`)* · không |
| **C** `true` + lọc dán | **0** · không | **0** · không |
| **D** `plaintext-only` + lọc dán | **0** · không | **0** · không |

Ba kết luận, mỗi cái là một số:

1. **`contenteditable="true"` một mình THỦNG trên cả hai engine.** Một lượt dán bơm `<pre>`, `<span style>` — và trên Blink còn bơm cả một `id="clipsrc"` **trùng** vào tài liệu. Cộng một `\n` **thật** vào `textContent`, tức một ngắt đoạn nằm trong `target_text` của **một** câu — trái AD-37 *(cấu trúc đoạn là dữ liệu ĐÃ LƯU, `segment.is_paragraph_end`)*.
2. **`plaintext-only` KHÔNG phải lời giải, và nó hỏng KHÁC NHAU trên hai engine.** Blink chặn markup nhưng **giữ `\n`**; WebKit **đổi `\n` thành `<div>`** — tức bơm phần tử **khối** vào trong một `<span>` inline, tệ hơn hẳn ca nó định chữa. Story §Quyết định #1 (b) phỏng đoán *"nó chỉ hứa về markup, không hứa về cấu trúc phần tử"* — phép đo xác nhận, và còn chỉ ra rằng trên WebKit nó **tự tạo** cấu trúc.
3. **Cái lọc là đòn bẩy, không phải giá trị thuộc tính.** Với một handler chặn `insertFromPaste` rồi chèn text đã làm phẳng, **C và D giống nhau từng chữ** trên cả hai engine: 0 phần tử, 0 `\n`.

⇒ **Chọn `contenteditable="true"` + lọc.** Lý lẽ chọn `true` thay vì `plaintext-only`: khi đã có lọc, hai giá trị cho kết quả **y hệt**, nên `plaintext-only` đổi lấy con số **không** — trong khi nó mang hai hành vi khác nhau giữa hai engine và một lịch sử hỗ trợ lệch. Một giá trị có hai hành vi và không lợi ích đo được là một món nợ, không phải một lớp phòng thủ.

⚠️ Đây **không** phải mở lại Quyết định #1. Ice ký đường **(c)** — *"vùng gõ là MỘT câu tại một thời điểm"* — và (c) không khai **giá trị** của thuộc tính (`epics`-story `:235`: *"chỉ thêm `contenteditable` lên đúng nó"*). Mũi thăm dò chọn giá trị đó bằng số, đúng trong lòng (c).

**🔵 Một dữ kiện LẬT được một lo ngại, chứ không lật một quyết định:** `isContentEditable` đọc ra **`true`** cho **cả hai** giá trị trên **cả hai** engine. ⇒ `isTypingZone` (`keys.ts:434-439`) nhận đúng vùng gõ mới **mà không cần sửa một dòng** — vế *"chiều thật sự chạm tới"* của AC21 được thoả bằng mã đã có, và việc của story chỉ là **chứng minh** nó (Task 6.1, đỏ-rồi-xanh qua Kiểm D). ⚠️ Lượt đo thứ ba từng đọc ra `false` trên WebKit dưới `plaintext-only` và đó là một **số đọc từ một cây DOM đã tháo** — xem §Ba lượt đo hỏng. Nếu tin nó, story đã đi vá một khuyết tật không tồn tại.

**② FRAME BUDGET — trần NFR2 = 50 ms, KHÔNG frame nào vượt**

| | Blink | WebKit |
| --- | --- | --- |
| gõ liên tục *(một câu tiếng Việt đầy dấu, delay 12 ms)* | 56 frame · max **17,60 ms** · vượt 50 ms: **0** | 62 frame · max **18,00 ms** · vượt 50 ms: **0** |
| **lắp/tháo `contenteditable`** *(n=40, kèm một lượt đọc `getClientRects` để buộc bố cục lại)* | median **0,3 ms** · max **0,6 ms** | median **1 ms** · max **2 ms** |
| vạch lề sau khi lắp | 6→6 vạch · lệch `top` tối đa **0,00 px** | 6→6 · **0,00 px** |

⇒ Chi phí **lắp/tháo** — con số mà story ghi là *"ca thứ hai chưa ai đo"* — nhỏ hơn trần một frame **hai bậc độ lớn**. Cái giá của đường (c) không nằm ở hiệu năng. Và vạch lề của Story 2.2 **không lệch một pixel** sau khi câu thành vùng gõ: `getClientRects()` vẫn đo được, `::after` vẫn neo được, `<span>` vẫn chảy inline — AC1/AC2/AC4/AC5 của 2.2 không bị đường (c) chạm tới.

**③ `⏐` RÒ RA LÚC COPY? — KHÔNG, trên cả hai engine, cả trên bề mặt GÕ ĐƯỢC**

| Đường copy | Blink | WebKit |
| --- | --- | --- |
| `⌘A` trong **một câu đang gõ** | không rò | không rò |
| bôi đen **cả trang** chỉ-đọc *(350 ký tự)* | không rò | không rò |
| `doc.innerText` | không rò | không rò |

⇒ Quyết định #3 của Story 2.2 *(`⏐` là pseudo-element)* giữ được mệnh đề của nó **sau khi bề mặt thành gõ được** — vết sẹo `WORD_JOINER` của 1.18b (`deferred-work.md:839-848`) không tái diễn. Chiều **dán vào** đo ở bảng ① vế hai: chuỗi dán vào **không** mang `⏐`.

**caret KHÔNG nhảy khi lắp/tháo qua ranh giới** — ba lượt bấm chuột thật câu 1 → câu 3 → câu 1, cả hai engine cho `{segmentId:1,offset:2}` → `{segmentId:3,offset:3}` → `{segmentId:1,offset:2}`, lắp/tháo **3/2** *(lượt lắp đầu không có gì để tháo — đối xứng đúng)*.

#### 🔴 BA LƯỢT ĐO HỎNG TRƯỚC KHI CÓ MỘT SỐ ĐÚNG — ghi ra vì bài học nằm ở đây

Ghi lại thay vì chỉ trình bày kết quả cuối, đúng luật *"đo trước khi tin"* — và vì cả ba lượt đều **xanh trông có lý**:

1. **Lượt một** dựng một `ClipboardEvent` bằng tay để mô phỏng dán. Một `ClipboardEvent` tự dựng **không** mang hành vi mặc định của engine, nên nó không chứng minh gì về lượt dán thật. Sửa: nạp clipboard hệ thống bằng một lượt `⌘C` **thật** từ chính trang, rồi bấm `⌘V`.
2. **Lượt hai** cho **ba biến thể ra cùng một HTML**, kể cả biến thể gọi `preventDefault()`. Một kết quả không tự nhất quán là dấu hiệu **phép đo hỏng**, không phải engine hành xử vậy — nên nó được điều tra thay vì được ghi.
3. **Lượt ba** tìm ra nguyên nhân, và nó đáng một dòng: **`page.setContent()` KHÔNG thực thi lại `<script>` nội tuyến.** Cây DOM mới, nhưng `window.__mount` của biến thể **A** vẫn sống và đóng gói phần tử `#doc` **đã tháo** ⇒ ba biến thể sau đo lại chính biến thể A. Bằng chứng: log tích luỹ `mount:args` của cả bốn lượt trong khi các sự kiện `beforeinput`/`paste` chỉ xuất hiện **một** lần. Sửa: ghi trang ra tệp và `page.goto()` với **một trang mới cho mỗi biến thể**.

⇒ Số `isContentEditable === false` của lượt ba là số của một cây đã tháo. Đó đúng là món *"kiểm điều kiện đo trước khi lật một quyết định"* mà bài học §2 của story ghi tên: nếu tin nó, story đã đi vá `isTypingZone` cho một khuyết tật **không tồn tại**.

**Tệp mũi thăm dò** *(scratchpad, ngoài kho — không phải tạo tác của story)*: `probe-typing.mjs` · `probe-paste.mjs` · `probe-plaintext3.mjs`.

### Completion Notes List

#### 🔴 ĐỌC MỤC NÀY TRƯỚC — một khuyết tật sản phẩm đã ĐO, chưa vá, và AC8 KHÔNG đạt trọn vẹn

> 🔵 **ĐÍNH CHÍNH 2026-08-13 (code review) — đoạn ngay dưới đây viết 2026-08-12 và đã BỊ LẬT một
> phần.** Vế *"chưa gõ được **bằng chuột**"* **không còn đúng**: §ĐÍNH CHÍNH ở đầu tệp đo lại và
> tìm ra nguyên nhân thật *(một lượt đặt thuộc tính bất đồng bộ, không phải AD-34 giành tiêu
> điểm)*, rồi vá bằng `setAttribute` đồng bộ trong `mousedown`. Chuột **đặt được** caret.
>
> Thứ **còn đỏ** là một ca khác và hẹp hơn: **lượt gõ đầu tiên vào một câu CHƯA DỊCH** —
> `execCommand('insertText')` trả `false` trên một `<span>` rỗng 0 px. Đo lại 2026-08-13:
> `test:e2e` = **1 xanh / 1 đỏ**, đỏ ở `editor-typing-flush.e2e.mjs:133`.
>
> ⇒ **AC8 vẫn KHÔNG đạt trọn vẹn**, nhưng vì lý do khác với lý do đoạn dưới ghi. Giữ đoạn dưới
> làm dấu vết, đừng đọc nó như trạng thái hôm nay.

Bề mặt Editor **gõ được trên Blink** *(bàn đo hai engine · 32 ca vitest · e2e vế vùng gõ, tất cả xanh)* nhưng **chưa gõ được bằng chuột trên macOS/WKWebView** — tức trên đúng nền tảng duy nhất dự án đang chạy.

Phát hiện này chỉ lộ ra vì story dựng một spec e2e chạy trong **cửa sổ Tauri thật**. Chuỗi sáu phép đo, nguyên nhân, và lý do story **không tự vá** *(lời giải chạm hợp đồng **AD-34** mà Story 1.6 sở hữu — AC19 nói gặp một quyết định story khác sở hữu thì **dừng và báo**)* ghi đầy đủ ở `deferred-work.md` §*Deferred from: 2-3-hop-dong-flush-va-trang-thai-da-luu*, mục đầu của khối *MÓN MỚI*. **Chủ: Ice** *(phán quyết AD-34)*.

⚠️ Vế *"engine gõ được"* **không** hỏng: `execCommand('insertText', …)` cho `beforeinput` → `input` → chữ hạ cánh. Và `browser.keys()` của bộ e2e **không** gõ được chữ *(chỉ `keydown`)* — một **giới hạn của bộ đo**, không một khuyết tật sản phẩm.

#### Ba AC KHÔNG đánh dấu đạt trọn vẹn, và lý do từng cái

| AC | Trạng thái | Vì sao |
| --- | --- | --- |
| **AC8** | **một phần** | vùng gõ lên đúng một câu · Kiểm J đã gỡ đúng lúc · nhưng **chuột không đặt được caret trên WKWebView** *(mục trên)* |
| **AC3** | **một phần** | ba đường *(nhịp 2 s · rời segment · đóng Tác phẩm)* + vế **thoát ứng dụng** đã dựng. Vế ***"xác nhận segment"*** chưa có đường nào chạm tới — nó cần cột `segment.status` và một máy trạng thái, **chủ: Story 2.5** |
| **AC17** | **đạt, kèm nợ kế thừa** | `wire_exit_flush` phủ lượt đóng **bình thường**. `panic = "abort"` khiến một lần thoát **cứng** không đi qua đây — món nợ kế thừa từ `close_global_store`, story này **không** đóng nó |

#### NFR15 — ba tệp giấy phép THẬT, mở trong `node_modules/` (Task 0b.1)

Rà **TRƯỚC khi thêm**, khác ba lượt trước của dự án *(cả ba là lượt đuổi theo)*:

| Gói | Đường dẫn tệp giấy phép | Dòng đầu |
| --- | --- | --- |
| `vitest@4.1.10` | `node_modules/vitest/LICENSE.md` | `# Vitest core license` |
| `@vue/test-utils@2.4.11` | `node_modules/@vue/test-utils/LICENSE` | `The MIT License (MIT)` |
| `happy-dom@20.11.2` | `node_modules/happy-dom/LICENSE` | `MIT License` |

🔴 **Và đây là chỗ đọc tệp thật hơn đọc trường `license`:** `LICENSE.md` của `vitest` dài **811** dòng — phần đầu là MIT của chính nó, phần sau khai giấy phép của **27 gói nó vendor**: **24 MIT · 2 BSD-3-Clause · 1 ISC**. Trường `license` trong `package.json` chỉ nói `"MIT"` và không nói một chữ nào về 27 gói kia. Cả ba nhóm thuộc nhóm dễ dãi, tương thích GPL-3.0-or-later theo chiều đi vào.

⚠️ Cửa NFR15 **vẫn đứng** cho gói thứ tư. Task 0b.7 sửa **lời khai** ở ba tệp *(`registry.ts` · `commands/README.md` · `i18n/README.md`)* thay vì xoá nó — xoá trắng là đánh mất luôn cái cửa.

#### Một khuyết tật của CỔNG phụ thuộc, tìm ra và vá cùng lượt

`vitest` khai `@opentelemetry/api` làm **peer tuỳ chọn chưa cài**, và `npm ls --all --json` xếp một node **rỗng** cho nó vào `dependencies`. Bản trước của `check-deps.mjs` đếm node đó là thành viên cây rồi báo *"cây npm có thư viện thu thập dữ liệu"* — trong khi **không một byte** của gói đó có trên đĩa. Đó là lượt **thứ tư** cùng họ với ba lượt mà doc-comment của chính tệp đó đã ghi *(đọc một lời khai thành một sự thật)*.

Nay cổng chỉ đếm node **có `version`**, và **in ra** số node chỉ-lời-khai đã bỏ *(**82**)* để con số không biến mất im lặng. Nghiệm thu **đỏ-rồi-xanh** trên chính cổng: cài thật `@opentelemetry/api@1.9.0` ⇒ Kiểm 2 **đỏ**; gỡ ⇒ **xanh**. Số thật sau lượt sửa: **522** gói đã cài · **326** crate Rust.

#### Bảng AC25 — MỘT mệnh đề, MỘT đường nghiệm thu, MỘT tệp

Không dòng nào có hai đường.

| Mệnh đề | Đường | Tệp |
| --- | --- | --- |
| trần cứng 5 s thật sự nổ trên dòng phím liên tục 30 s | vitest | `tests/frontend/editorFlush.test.ts` |
| khoảng cách giữa hai lượt flush ≤ 5 000 ms | vitest | `tests/frontend/editorFlush.test.ts` |
| dòng phím thưa ⇒ đúng một flush ở 2 000 ms | vitest | `tests/frontend/editorFlush.test.ts` |
| gõ tiếp trong lúc lô bay không bị nuốt | vitest | `tests/frontend/editorFlush.test.ts` |
| một lượt ghi thành công không reset trần cứng | vitest | `tests/frontend/editorFlush.test.ts` |
| `contenteditable` trên đúng một câu, theo caret | vitest | `tests/frontend/editorTypingZone.test.ts` |
| caret không nhảy khi lắp vùng gõ | vitest | `tests/frontend/editorTypingZone.test.ts` |
| `Enter` bị chặn *(phím + `insertParagraph`)* | vitest | `tests/frontend/editorTypingZone.test.ts` |
| dán nhiều dòng: 0 `\n`, 0 phần tử tiêm | vitest | `tests/frontend/editorTypingZone.test.ts` |
| commit IME không bị cắt | vitest | `tests/frontend/editorTypingZone.test.ts` |
| văn bản đang gõ sống sót lượt tháo panel | vitest | `tests/frontend/editorTypingZone.test.ts` |
| rời segment ⇒ đúng một flush, đúng câu cũ | vitest | `tests/frontend/editorLeaveSegment.test.ts` |
| caret về `null` / cùng câu ⇒ không flush | vitest | `tests/frontend/editorLeaveSegment.test.ts` |
| một lô mang nhiều câu (AC13, phía TS) | vitest | `tests/frontend/editorLeaveSegment.test.ts` |
| ghi trượt ⇒ giữ tập chờ, không dựng mốc *"Đã lưu"* | vitest | `tests/frontend/editorLeaveSegment.test.ts` |
| thanh trạng thái im trước flush đầu, đếm đúng *N*, nhả timer | vitest | `tests/frontend/statusBar.test.ts` |
| Auto-Lookup còn chạy trên bề mặt Editor (AC23, vế DOM) | vitest | `tests/frontend/editorAutoLookup.test.ts` |
| luật vùng gõ **không** `preventDefault` (AC23, vế phím) | cổng tĩnh | `check-commands.mjs` Kiểm D |
| `isTypingZone` nhận vùng gõ `contenteditable` (AC21) | cổng tĩnh | `check-commands.mjs` Kiểm D |
| `npm run test` có ở cả ba danh sách (AC24) | cổng tĩnh | `check-gates.mjs` Kiểm F |
| chỉ đếm gói ĐÃ CÀI trong cây npm | cổng tĩnh | `check-deps.mjs` §④ |
| một lô, một giao dịch, `prepare_cached` một lần (AC13) | cargo test | `segment_contract.rs` |
| `UPDATE` chạm đúng hai cột, bảy cột kia y nguyên (AC14) | cargo test | `segment_contract.rs` |
| round-trip gõ → flush → nạp lại, ranh giới không đổi (AC16) | cargo test | `segment_contract.rs` |
| lô mang một id lạ ⇒ từ chối TRỌN, không ghi một phần | cargo test | `segment_contract.rs` |
| id thuộc Chương khác không bao giờ ghi lẫn | cargo test | `segment_contract.rs` |
| `PRAGMA synchronous = FULL` ⇒ commit fsync WAL (AC5) | cargo test | `store_contract.rs` |
| sổ sách `data-segment-id` sống sót 6 thao tác gõ, 2 engine | bàn đo | `2-3-ban-do-vung-go.html` |
| frame budget khi gõ + chi phí lắp/tháo (NFR2) | bàn đo | `2-3-ban-do-vung-go.html` |
| `⏐` không rò lúc copy trên bề mặt gõ được | bàn đo | `2-3-ban-do-vung-go.html` |
| vùng gõ lên đúng một câu trong **WKWebView thật** | e2e | `editor-typing-flush.e2e.mjs` |
| `Enter` không tách câu trong **WKWebView thật** | e2e | `editor-typing-flush.e2e.mjs` |

#### AC5 — số đo, không một mệnh đề về giá trị mặc định

`PRAGMA synchronous` đọc lại trên kết nối ghi = **2 (FULL)** ⇒ mỗi commit trên WAL **fsync** WAL ⇒ `Store::write` trả `Ok` **là** bằng chứng đã ghi vào WAL. **AC5 thoả mà không thêm một dòng mã nào** — nhưng nay nó là một lời khai **có lưới** (`store_contract.rs::the_write_connection_fsyncs_the_wal_on_every_commit`), không một mệnh đề về mặc định biên dịch của SQLite.

#### AC20 — sàn, đo chứ không ước

Bốn sàn tụt dưới doctrine 80–85 % sau khi story thêm **2** tệp sản phẩm, đã nâng theo **số thật**:

| Cổng · sàn | Cũ | Mới | Quần thể thật |
| --- | --- | --- | --- |
| `check-commands.mjs` `TS_FLOOR` | 28 *(77,8 %)* | **30** *(83,3 %)* | 36 tệp `.ts` |
| `check-layout.mjs` `FILE_FLOOR` | 40 *(76,9 %)* | **43** *(82,7 %)* | 52 tệp `src/**` |
| `check-tokens.mjs` `FILE_FLOOR` | 43 *(78,2 %)* | **45** *(81,8 %)* | 55 tệp |
| `check-tokens.mjs` `COMPONENT_FILE_FLOOR` | 40 *(76,9 %)* | **43** *(82,7 %)* | 52 tệp component |

Giữ nguyên vì còn trong doctrine: `VUE_FLOOR` 13/16 = 81,3 % *(hai cổng)* · `RS_FLOOR` 36/43 = 83,7 % · `COMMAND_FLOOR` 29/34 · `CLICK_FLOOR` 17/21 · `DISPATCH_FLOOR` 23/28 · `SELECTION_SURFACE_FLOOR` 7/7.

🔵 **Quyết định #6 được xác nhận bằng số:** **7** tệp ở `tests/frontend/**` thêm **0** vào quần thể của cả bốn cổng.

#### AC19 — phụ thuộc

**0** dependency runtime mới *(giữ đúng ba: `@tauri-apps/api 2.11.1` · `dockview-vue 7.0.4` · `vue 3.5.40`)*. **Đúng ba** dev-dep mới, ghim chính xác, không dải `^`. Ba hàng đã vào bảng Stack của `ARCHITECTURE-SPINE.md` kèm một khối *"Rà NFR15 lượt bốn"*; `lint_spine.py` **0 findings**.
⚠️ Playwright **không** vào `package.json` — nó chạy từ bộ nhớ đệm `npx` ngoài kho, cùng khuôn Story 2.2.

#### Nghiệm thu cuối

- **9/9** cổng npm xanh · `npm run build` xanh *(`vue-tsc` hai lượt, nay **có** cây test trong tầm `tsconfig`)* · **`npm run test` 32/32** · **`cargo test --locked` 319 xanh / 0 đỏ / 5 ignored** *(+9 ca so với 2.2)*.
- ~~**e2e** `editor-typing-flush.e2e.mjs` **2/2 xanh**~~; `attribution-focus.e2e.mjs` xanh làm đối chứng lượt khởi động.

  🔴 **ĐÍNH CHÍNH 2026-08-13 (code review) — dòng gạch trên SAI, và nó là lời khai thứ ba lệch
  nhau về cùng một trạng thái.** Chạy lại `npm run test:e2e` để lấy số thay vì chọn giữa hai
  lời khai: `editor-typing-flush.e2e.mjs` = **1 xanh / 1 đỏ**. Ca đỏ dừng ở
  `e2e/specs/editor-typing-flush.e2e.mjs:133` — `expect(inserted.ok).toBe(true)`, tức
  `execCommand('insertText')` trả `false` trên một câu **chưa dịch** *(`<span>` rỗng, rộng 0 px,
  không text node để neo caret)*. Đúng ca mà §ĐÍNH CHÍNH đầu tệp đã công bố.
  Bộ e2e tổng: **4 spec xanh / 1 spec đỏ**, 5 spec, 04:35.

  ⚠️ Bài học nằm ở chỗ **vì sao** ba lời khai lệch nhau: mục này viết 2026-08-12, §ĐÍNH CHÍNH
  viết 2026-08-13, và không ai quay lại sửa mục cũ. Một story file mang ba bản của cùng một sự
  thật thì bài học #6 *("`in-progress` phải ghi **nguyên nhân cụ thể**")* không còn chỗ đứng.
- **Bốn** nghiệm thu **đỏ-rồi-xanh** đã tái lập: ① `Math.min` → debounce thuần ⇒ mệnh đề 1+2 của AC11 đỏ · ② gỡ nhánh `isContentEditable` ⇒ Kiểm D đỏ · ③ một test cố tình sai ⇒ `npm run test` **exit 1** và `pre-push` **chặn** · ④ cài thật `@opentelemetry/api` ⇒ `check:deps` Kiểm 2 đỏ.
- ⚠️ **Mọi bằng chứng chỉ trên macOS** — nửa Windows không có đường nghiệm thu nào cho tới lượt Ice mở lại (`deferred-work.md:145`).

#### Một lượt đo hỏng phải ghi ra, vì bài học nằm ở đó

`page.setContent()` của Playwright **không thực thi lại `<script>` nội tuyến**. Ba biến thể của mũi thăm dò vì thế đo lại một cây DOM **đã tháo**, và một trong các số đọc ra (`isContentEditable === false` trên WebKit dưới `plaintext-only`) là **sai**. Nếu tin nó, story đã đi vá `isTypingZone` cho một khuyết tật không tồn tại. Chi tiết ba lượt ở §Debug Log References.

### File List

**Mới**

```
src/panels/editorFlush.ts                                  # nhịp flush + tập đã đổi (module THUẦN)
src/StatusBar.vue                                          # thanh trạng thái, vỏ ứng dụng
vitest.config.ts                                           # happy-dom, phạm vi tests/frontend/**
tests/frontend/editorFlush.test.ts                         # AC11 — ba mệnh đề định lượng
tests/frontend/editorTypingZone.test.ts                     # vùng gõ, caret, dán, IME
tests/frontend/editorLeaveSegment.test.ts                   # AC18 — rời segment
tests/frontend/statusBar.test.ts                            # AC7 · AC10
tests/frontend/editorAutoLookup.test.ts                     # AC23 vế DOM
tests/frontend/support/segmentFixture.ts                    # fixture + bộ ghi lượt gọi
tests/frontend/support/setup.ts                             # vá khoảng thiếu của happy-dom
e2e/specs/editor-typing-flush.e2e.mjs                       # WKWebView thật
_bmad-output/implementation-artifacts/2-3-ban-do-vung-go.html
_bmad-output/implementation-artifacts/2-3-ban-do/2-3-vung-go-blink-light.png
_bmad-output/implementation-artifacts/2-3-ban-do/2-3-vung-go-blink-dark.png
_bmad-output/implementation-artifacts/2-3-ban-do/2-3-vung-go-webkit-light.png
_bmad-output/implementation-artifacts/2-3-ban-do/2-3-vung-go-webkit-dark.png
```

**Sửa**

```
src/panels/EditorPanel.vue                 # vùng gõ, ba handler, đường chuột, khối đầu tệp viết lại
src/panels/editorPanelState.ts             # nhịp flush, tập đã sửa, mốc "Đã lưu", đường thoát
src/panels/editorSegments.ts               # (không đổi — giữ vai bản-lúc-nạp cho FR117)
src/layout/writeSchedule.ts                # doc-comment vai mới: một hàm, hai chỗ dùng
src/config/segment.ts                      # + adapter `saveSegmentTargets`
src/App.vue                                # + `<StatusBar />`
src/main.ts                                # + `wireExitFlush()` trước `mount()`
src/modes/libraryImport.ts                 # flush TRƯỚC `replace_open_work`, không trước reset
src/i18n/vi.json                           # + `status.saved_seconds_ago`, `err.segment.unknown_ids`
src/commands/registry.ts                   # sửa lời khai NFR15
src/commands/README.md                     # sửa lời khai NFR15
src/i18n/README.md                         # sửa lời khai NFR15
src-tauri/src/commands/segment.rs          # + `save_segment_targets` (hàm thuần + vỏ `wire`)
src-tauri/src/core/i18n/mod.rs             # + `MessageKey::SegmentUnknownIds`
src-tauri/src/lib.rs                       # + đăng ký lệnh, `ExitFlush`, `wire_exit_flush`, `confirm_exit_flush`
src-tauri/tests/segment_contract.rs        # + 8 ca đường flush
src-tauri/tests/store_contract.rs          # + ca `PRAGMA synchronous` (AC5)
scripts/check-commands.mjs                 # GỠ Kiểm J; + 5 ca Kiểm D (AC21, AC23); TS_FLOOR
scripts/check-deps.mjs                     # §④ chỉ đếm gói ĐÃ CÀI
scripts/check-gates.mjs                    # + Kiểm F; + 5 ca tự kiểm
scripts/check-layout.mjs                   # + 3 API vào allowlist; FILE_FLOOR
scripts/check-tokens.mjs                   # FILE_FLOOR · COMPONENT_FILE_FLOOR
package.json                               # + 3 devDependencies ghim; + `test`; `check:lint` quét tests
package-lock.json                          # hệ quả lượt cài
tsconfig.json                              # + tests/frontend/** và vitest.config.ts
eslint.config.js                           # + tests/frontend/** vào hai khối có kiểu
.github/workflows/ci.yml                   # + bước `npm run test`
.githooks/pre-push                         # + `npm run test` (danh sách thứ BA)
_bmad-output/planning-artifacts/architecture/.../ARCHITECTURE-SPINE.md   # bảng Stack + 3 hàng
_bmad-output/implementation-artifacts/deferred-work.md                   # đóng 2 món, ghi 10 món
_bmad-output/implementation-artifacts/sprint-status.yaml                 # in-progress → review
```

**Không đụng, có chủ ý:** `epics.md` · `DESIGN.md` · `capabilities/main.json` · `schema.rs` *(không bước di trú 7)* · hành vi `createWriteSchedule` · `check-layout.mjs` Kiểm B · `check-commands.mjs` Kiểm C/E · `core/segment/split.rs` · `PanelFrame.vue` · `SourcePanel.vue`.

### Review Findings

Lượt code review **2026-08-13**, ba tầng song song *(Blind Hunter · Edge Case Hunter · Acceptance Auditor)* trên `git diff HEAD` + tệp mới, mốc gốc `6a9777b`. 18 phát hiện sau khi gộp trùng; 2 loại làm nhiễu.

🔴 **Ba phát hiện đầu tiên đều nằm trên đường flush lúc THOÁT và lúc GHI TRƯỢT — tức đúng vế mà story tồn tại để đóng.** Không phát hiện nào bác một phán quyết Ice đã ký, và không phát hiện nào chạm Quyết định #1/#2.

#### Cần Ice quyết — 🔵 CẢ BA ĐÃ CÓ PHÁN QUYẾT 2026-08-13, cả ba thành mục vá

| # | Phán quyết của Ice | Hệ quả |
| --- | --- | --- |
| D1 | **Log + sàn thử lại; CHƯA báo ra màn hình** | ① log `error` vô điều kiện; ② một hằng sàn thử lại khai ở `editorFlush.ts` kèm nhãn *"TẠM — chủ 2.4"*. Vế báo lỗi lên `StatusBar` **không** làm ở story này ⇒ UX-DR30 **không bị thu hẹp**, và nó vẫn là một câu hỏi mở có chủ là Ice |
| D2 | **Chặn lượt tạo Tác phẩm khi flush trượt** | `flushEditorNow()` trả về kết quả; `beginSubmit` dừng và đặt `lastError` nếu trượt. Người dùng bị cản một lượt nhưng **không mất chữ** — cùng hướng *"nhánh chính luôn trung thực"* đã ký ở 1.22 |
| D3 | **Chạy `npm run test:e2e` lấy số thật** | Ba chỗ lệch *(§ĐÍNH CHÍNH `:18` · §Completion Notes `:825` · checkbox Task 3 `:376`)* được viết lại theo **một** bản, sau khi có số |

- [x] [Review][Patch] ✅ *D1 — đã quyết.* **Flush trượt ⇒ vòng lặp thử lại 0 ms, im lặng tuyệt đối** — `onFlushed()` chỉ gọi `schedule.onWrite()` khi tập chờ đã sạch (`editorFlush.ts:137`), nên một lượt ghi trượt để `due` đứng nguyên ở một mốc **đã quá hạn**; `armFlushTimer()` tính `Math.max(0, due - Date.now())` = **0** ⇒ `setTimeout(…, 0)` ⇒ gọi lại ngay ⇒ trượt lại. Vòng lặp chặt, không backoff, không trần số lần. Và `flushEditorNow()` destructure `{ outcome }` **bỏ hẳn `error`** (`editorPanelState.ts:250`) nên không một dòng log nào. Đường tái lập chắc chắn, không cần lỗi kho: `npm run dev` trong trình duyệt thường ⇒ không cầu IPC ⇒ `outcome === null` mọi lượt ⇒ **một ký tự là đủ**. Người dùng chỉ thấy con số trên `StatusBar` bò lên quá trần 5 giây — `StatusBar.vue` cố ý không có trạng thái lỗi theo UX-DR30. **Quyết định thuộc Ice vì lời giải là một CON SỐ (nhịp thử lại) và một hợp đồng UX (có báo lỗi ghi hay không) — nhịp flush có chủ là Story 2.4, UX-DR30 có chủ là Ice.** Đề xuất: ① log `error` ngay *(vô điều kiện, rẻ, không chạm số nào)*; ② một sàn thử lại ghi thành hằng ở `editorFlush.ts` kèm nhãn *"TẠM — chủ 2.4"*; ③ vế báo lỗi ra màn hình để Ice chốt riêng.
- [x] [Review][Patch] ✅ *D2 — đã quyết.* **`beginSubmit()` không kiểm kết quả flush trước khi `resetEditorPanel()` vứt tập chờ** — `flushEditorNow()` **không ném** theo thiết kế, nên `beginSubmit` (`libraryImport.ts:120`) chạy tiếp bất kể lượt flush có chạm WAL hay không; nếu `created !== null` thì `finishSubmit` gọi `resetEditorPanel()` → `flush.reset()` (`editorPanelState.ts:292`) **vứt vô điều kiện** mọi mục còn lại. Doc-comment tại chỗ khai *"flush TRƯỚC reset"* là điều kiện bắt buộc, nhưng **không mã nào kiểm điều kiện đó đã thoả**. ⚠️ Việc dời lời gọi sang `beginSubmit` là **đúng** và số đo chống lưng nó vẫn đứng — phát hiện này không bác lượt dời đó, nó chỉ nói lượt dời chưa đóng vế *"flush trượt"*. **Quyết định thuộc Ice:** chặn lượt tạo Tác phẩm khi flush trượt *(người dùng bị cản, nhưng không mất chữ)*, hay giữ tập chờ sống qua `reset()` *(không cản, nhưng tập chờ mang `chapter_id` của Tác phẩm đã đóng)*, hay báo rồi vẫn đi tiếp.
- [x] [Review][Patch] ✅ *D3 — đã quyết: chạy e2e lấy số.* **Story file mang BA lời khai lệch nhau về cùng một trạng thái** — §ĐÍNH CHÍNH 2026-08-13 *(`:18`)* ghi **e2e 1/2**, ca gõ **đỏ**; §Completion Notes → Nghiệm thu cuối *(`:825`)* ghi **e2e 2/2 xanh**; và checkbox **Task 3** *(`:376`)* vẫn mang chẩn đoán *"AD-34 §2 xoá vùng chọn"* mà chính §ĐÍNH CHÍNH đã bác là **SAI**. Thêm nữa `e2e/specs/editor-typing-flush.e2e.mjs` khẳng định thẳng `expect(inserted.ok).toBe(true)` **không** skip/xfail, nên người đọc chỉ mã không thể biết ca này đang được kỳ vọng đỏ. ⚠️ Đây không phải bắt lỗi văn phong — bài học #6 của chính story *("`in-progress` không phải chỗ đậu — phải ghi **nguyên nhân cụ thể**")* đứng hay đổ ở đúng chỗ này. **Cần Ice xác lập trạng thái thật hôm nay** *(hoặc cho phép chạy `npm run test:e2e` để lấy số)*, rồi ba chỗ trên được viết lại theo **một** bản.

#### Vá được, không cần hỏi

- [x] [Review][Patch] `inFlight` là cờ boolean chứ không phải một promise chờ được — `await flushEditorNow()` trở thành **no-op tức thời** khi một lô đang bay, nên cả đường thoát ứng dụng lẫn `beginSubmit` mất đúng cái bảo đảm mà `await` ở đó hứa; ký tự gõ **trong lúc** lô bay không bao giờ lên dây [src/panels/editorPanelState.ts:239]
- [x] [Review][Patch] Lượt `CloseRequested` **thứ hai** không bị chốt `ready` chắn — `ready` chỉ bật *sau* khi webview trả lời hoặc hết trần, nên một lệnh đóng thứ hai tới trước mốc đó vẫn `prevent_close()` + `emit` lần nữa và đẻ **luồng chờ thứ hai**; webview lượt hai trúng cửa `inFlight` ⇒ `confirm_exit_flush` ngay ⇒ `release()` đánh thức **cả hai** luồng ⇒ `destroy()` khi lô ghi gốc có thể chưa chạm WAL [src-tauri/src/lib.rs:674]
- [x] [Review][Patch] Dán/kéo-thả với vùng chọn tràn qua ranh giới câu bị **nuốt hoàn toàn** — `preventDefault()` chạy ở `:724` **trước** khi guard `!sent.contains(range.startContainer)` thoát hàm ở `:736`, nên nội dung clipboard biến mất, không ký tự nào hạ cánh, không một tín hiệu nào. Vá bằng cách kiểm containment **trước** `preventDefault()` [src/panels/EditorPanel.vue:724]
- [x] [Review][Patch] `PRAGMA synchronous` chỉ được **ĐỌC LẠI**, chưa bao giờ được **ĐẶT** — AC5 vì thế đứng trên một **mặc định biên dịch** của `libsqlite3-sys`. Đó mới là nửa sau của chính luật *"đặt rồi ĐỌC LẠI"* mà `pragmas.rs` tồn tại để dạy; một lượt nâng phụ thuộc hạ nó xuống `NORMAL` là AD-35 mất bảo đảm bền vững [src-tauri/src/core/store/pragmas.rs]
- [x] [Review][Patch] Cơ chế đồng thời **mới và phức tạp nhất** của story không có một ca test nào — `ExitFlush` (`Mutex` + `Condvar`, `:185-220`), `wire_exit_flush`, `confirm_exit_flush` chỉ được nghiệm thu bằng doc-comment. Cả hai ca ở trên *(lượt đóng thứ hai; `release()` sớm)* lẽ ra bị một lưới máy bắt [src-tauri/src/lib.rs:185]
- [x] [Review][Patch] AC12 đòi *"một ca ở `store_boundary.rs` **hoặc** `segment_boundary.rs`"* — cả hai tệp **không có trong diff**; mệnh đề chỉ được phủ **gián tiếp** bởi cổng tĩnh sẵn có, và bảng AC25 không có dòng nào ứng với nó [src-tauri/tests/segment_boundary.rs]
- [x] [Review][Patch] `runsScript` dùng `\b(?!:)` nên chỉ loại được hậu tố `:` — `npm run test-something` vẫn khớp `test` *(ký tự sau là `-`, không phải `:`)*, tức một cổng tự nhận *"không đỏ oan"* còn một cửa đọc nhầm chưa đóng; bộ `REQUIRED_CASES` cũng chỉ phủ ca `test:e2e` [scripts/check-gates.mjs:184]
- [x] [Review][Patch] Chú thích gỡ Kiểm J ghi *"**sáu** ca mới"* — đếm thật trong `segment_contract.rs` là **tám** `#[test]`, và §Change Log của chính story cũng ghi **8**. Sai số nằm ngay trong một cổng cưỡng chế, trong đúng kho lấy *"đo chứ không ước"* làm luật [scripts/check-commands.mjs:2152]

#### Hoãn, đã ghi `deferred-work.md`

- [x] [Review][Defer] `restoreEditedText()` quét **toàn bộ** `[data-segment-id]` trong `.doc` thay vì duyệt theo key của `editedText` — O(cả Chương) thay vì O(số câu đang gõ dở), mỗi lượt dựng lại trang [src/panels/EditorPanel.vue:294] — hoãn, cùng hàng nợ *"ảo hoá danh sách dài"*, chủ là Story 2.4
- [x] [Review][Defer] `nearestSentenceTo()` gọi `getClientRects()` cho **từng** câu trên mỗi cú bấm hụt ⇒ layout thrash trên Chương 9 850 câu, và là hàm hiếm hoi của lượt này **không** kèm số đo [src/panels/EditorPanel.vue:565] — hoãn, cùng chủ Story 2.4
- [x] [Review][Defer] Cửa rà giấy phép NFR15 chỉ tồn tại dưới dạng **lời khai trong comment** ở bốn chỗ, không một cổng máy nào xác minh lượt rà đã thật sự xảy ra [src/commands/registry.ts] — hoãn, có sẵn từ trước lượt này; NFR15 xưa nay là một quy trình người, chủ là Ice
- [x] [Review][Defer] Tiêu điểm/caret **không** được khôi phục sau một lượt dựng lại component *(đổi preset bố cục)*: `restoreEditedText()` chép đúng chữ nhưng watcher caret không chạy vì **giá trị** `editorCaretSegmentId` không đổi, và `savedCaret` đã về `null` [src/panels/EditorPanel.vue:363] — hoãn, vì lời giải là *giành* tiêu điểm lúc mount, đi ngược doctrine chống-giành-tiêu-điểm của `PanelFrame`; cần Ice
- [x] [Review][Defer] Bỏ `min-width` cho câu rỗng chưa kèm một phép đo nào rằng **caret còn nhìn thấy** trên một `<span>` rỗng 0 px trước khi gõ — e2e chỉ khẳng định chữ *hạ cánh được*, không khẳng định caret *hiện ra* [src/panels/EditorPanel.vue] — hoãn, cùng hàng với ca đỏ đã công bố *(gõ lần đầu vào câu chưa dịch)*

#### Nghiệm thu sau khi vá — 2026-08-13

**9/9 cổng npm xanh** · `npm run build` xanh · **`npm run test` 40/40** *(+6 ca mới, tệp
`editorFlushRetry.test.ts`)* · **`cargo test --locked` 324 xanh / 0 đỏ** *(+5 ca: bốn cho
`ExitFlush`, một cho AC12)* · **`npm run test:e2e` 4 spec xanh / 1 spec đỏ** *(ca đỏ là ca đã
công bố, xem D3)*.

🔴 **Hai nghiệm thu đỏ-rồi-xanh, và số đo là thứ đáng giữ lại:**

| Bản vá | Lượt ĐỎ tái lập bằng | Số đo |
| --- | --- | --- |
| Sàn thử lại | dựng lại `armFlushTimer(0)` | **8 001** lượt gọi IPC trong 10 giây mô phỏng, so với trần **6** sau khi vá |
| `inFlight` là promise | dựng lại `if (inFlight !== null) return` | câu B gõ trong lúc lô bay **không có mặt** trên dây: `expected [ { id: 11, … } ] to deep equally contain { id: 12, … }` |

⚠️ **Hai lượt lệch khỏi bản vá đã ghi ở trên, nói ra thay vì để người đọc tự phát hiện:**

1. **Dán tràn ranh giới** — bản ghi ban đầu là *"kiểm containment **trước** `preventDefault()`"*.
   Đọc kỹ thì đó là bản vá **sai hướng**: không chặn nghĩa là thả engine chạy hành vi dán mặc
   định trên một vùng chọn tràn ranh giới — đúng đường tiêm markup và gộp `<span>` mà Task 0.1
   đo được, và mất một `data-segment-id` là hỏng **vĩnh viễn** theo AD-3. ⇒ giữ `preventDefault()`,
   chỉ đóng vế **im lặng** bằng một dòng `console.warn`.
2. **`PRAGMA synchronous`** — bản vá này **đè lên một quyết định đã ghi thành chữ**:
   doc-comment của `store_contract.rs` viết *"Ca này **đọc**, nó không **đặt** […] không phải một
   chỗ để lặng lẽ chèn một `PRAGMA synchronous = FULL` — chủ là **Story 2.4**"*. Lý lẽ giữ bản
   vá: giá trị hiện tại **đã là** `FULL`, nên đặt tường minh đổi **0** về hành vi và **0** về
   hiệu năng, và 2.4 vẫn tự do hiệu chỉnh; cái đổi là AD-35 thôi đứng trên một **mặc định biên
   dịch** của một thư viện C được ghim. **Nếu Ice thấy ngược lại, một lượt `git checkout` trên
   `pragmas.rs` + `store_contract.rs` là gỡ xong.**

**Loại làm nhiễu (2):** ① văn bản composition IME trung gian vào tập chờ — `insertCompositionText` **không** nằm trong `FROM_OUTSIDE` nên bộ gõ không gãy, và trạng thái bền vững hội tụ ở lượt commit; lưu *"thứ người dùng đang nhìn thấy"* là chủ ý ghi rõ ở `:776-778`. ② khoảng hở *"hai vùng gõ trong cùng một tick"* — không có hệ quả quan sát được, engine không xử lý một lượt nhập thứ hai trong cùng tick đó.

### Change Log

| Ngày | Việc |
| --- | --- |
| 2026-08-12 | **Task 0** — bốn phán quyết (#3 → #6) kèm số đo. Không bước di trú 7; trần thoát **1 200 ms** chọn bằng phép trừ từ `close_truncate_budget`; thanh trạng thái ở vỏ ứng dụng + `setInterval` 1 s; cây test ở `tests/frontend/**` |
| 2026-08-12 | **Task 0.1** — mũi thăm dò Quyết định #1 trên **cả hai** engine. Sổ sách 6/6 sống sót qua 6 thao tác; frame max **17,60 / 18,00 ms** *(0 vượt 50 ms)*; lắp/tháo median **0,3 / 1 ms**; `⏐` **không rò**. 🔴 Lật một giả định: `contenteditable="true"` một mình **thủng** trên cả hai engine *(tiêm markup + `\n`)*, và `plaintext-only` **không** phải lời giải — **cái lọc dán** là đòn bẩy. Ba lượt đo hỏng ghi ra kèm nguyên nhân |
| 2026-08-12 | **Task 0b** — bộ chạy test frontend. Ba tệp giấy phép thật đã mở; 3 hàng vào bảng Stack *(`lint_spine.py` 0 findings)*; `npm run test` vào **cả ba** danh sách; **Kiểm F** của `check-gates` ra đời vì A/D chỉ duyệt `check:*`. Vá một khuyết tật của `check-deps.mjs` *(đếm lời khai thành thành viên cây)*, đỏ-rồi-xanh trên chính cổng |
| 2026-08-12 | **Task 1** — `editorFlush.ts` tái dùng `createWriteSchedule` *(2000/5000)*. Ba mệnh đề định lượng AC11 xanh; đỏ-rồi-xanh bằng debounce thuần. `writeSchedule.ts` nhận doc-comment vai mới |
| 2026-08-12 | **Task 2** — `save_segment_targets`: một lô, một giao dịch, `prepare_cached` một lần, chạm **đúng hai cột**. Lý do từ chối ra khỏi closure bằng một **ô có kiểu** *(`BatchReject`)*, không đoán lại từ chuỗi lỗi — nên một lỗi kho thật không bị chẩn đoán thành một id lạ. 8 ca `segment_contract.rs` xanh |
| 2026-08-12 | **Task 3** — vùng gõ MỘT câu. `contenteditable="true"` + lọc `beforeinput` ba nhóm. **Kiểm J gỡ SAU** khi Task 2.8 xanh |
| 2026-08-12 | **Task 4** — rời segment = `editorCaretSegmentId` đổi, cài ở **đúng một** chỗ. 🔴 Vế *"đóng Tác phẩm"* chuyển từ `finishSubmit` sang **`beginSubmit`**: đo được rằng flush ở `finishSubmit` là **SAI** — `replace_open_work` đã chạy, nên lô mang `chapter_id` cũ vào `project.db` **mới** và bị từ chối trọn, mất im lặng. `wire_exit_flush` đóng lỗ AD-35 vế (e). `PRAGMA synchronous` = **2 (FULL)** ⇒ AC5 thoả |
| 2026-08-12 | **Task 5** — `StatusBar.vue`, chiều cao **34px** *(token thắng mockup)*, `N` tính ở TS, không dấu chấm *"chưa lưu"*, không *"đang lưu…"* |
| 2026-08-12 | **Task 6** — AC21 đóng nửa chạm tới được, đỏ-rồi-xanh qua Kiểm D. **AC23 đo được: Auto-Lookup CÒN CHẠY** trên cả hai đường *(chuột và bàn phím)*, vì luật vùng gõ `return false` **trước** `preventDefault()` nên hành vi native vẫn mở rộng vùng chọn |
| 2026-08-12 | **Task 7** — bàn đo `2-3-ban-do-vung-go.html` + 4 ảnh *(2 engine × 2 theme)*. 🔴 Spec e2e trong **WKWebView thật** lộ ra một **khuyết tật sản phẩm**: bấm chuột không đặt được caret. Sáu phép đo, nguyên nhân trỏ vào **AD-34**, **dừng và báo** thay vì vá |
| 2026-08-12 | **Task 8** — 4 sàn nâng theo số thật; bảng AC25 *(32 dòng, không dòng nào hai đường)* |
| 2026-08-13 | 🔴 **Ba khuyết tật Ice bắt bằng mắt, ba nguyên nhân khác nhau.** ① *gõ ngược* — Vue so **vnode cũ với vnode mới** (không so với DOM) nên nó ghi đè text node ở mỗi phím, caret rơi về 0; ② *chữ đã dịch biến mất* — **do bản vá cho ① đẻ ra** (một biến đóng băng **dùng chung cho mọi câu**); ③ *khung viền + chữ lệch* — focus ring mặc định. ⇒ ① và ② cùng một nguyên nhân gốc: **hai chủ sở hữu cho một text node**. Lời giải là một **doctrine**, không một bản vá: **DOM sở hữu văn bản bản dịch**, template render bản **lúc nạp**, `restoreEditedText()` chép ngược một lần sau mỗi lượt dựng lại. Hai ca hồi quy khoá lại, ca *gõ ngược* khẳng định **danh tính text node** chứ không phải chuỗi cuối |
| 2026-08-13 | 🔵 **ĐÍNH CHÍNH — chẩn đoán "AD-34 giành tiêu điểm" là SAI, và bản vá Ice đã ký KHÔNG được thi hành.** Không ai giành cả: `focus.ts::enter()` chỉ chạy lúc đổi chế độ, chốt chống-rơi-`body` chỉ ghi console, `PanelFrame` chỉ nghe. Thứ đặt tiêu điểm lên `section.panel` là **hành vi mặc định của trình duyệt**, và nó chọn vậy chỉ vì `<span>` **chưa** `contenteditable` lúc engine xử lý cú bấm. ⇒ vá bằng `setAttribute` **đồng bộ**; **AD-34 không bị chạm một dòng** |
| 2026-08-13 | Ba nguyên nhân thật của *"khó click để focus"*, cả ba cần: `setAttribute` đồng bộ · `nearestSentenceTo()` *(bấm vào khoảng trống cũng ăn)* · **`Selection.setPosition()` thay `addRange()`** — đo được: trên một phần tử soạn thảo được **rỗng**, WebKit **bỏ qua** `addRange` nhưng **nhận** `setPosition` |
| 2026-08-13 | **Ice ký** thu hẹp UX-DR20: `.sent:empty::after { content: none }`. Đo trước/sau trên hai engine: **9,05 → 0,00 px/câu**; 40 câu chưa dịch từng đẩy chữ **433,9 px**. Phép đo nay **sống** trong bàn đo |
| 2026-08-12 | **Task 9** — 2 món nợ ĐÓNG tại chỗ, 10 món ghi mới, mỗi món một chủ. Ghi rõ rằng lý do *"không có bộ chạy test frontend"* của bốn hàng nợ cũ nay **đã sai** |
| 2026-08-13 | 🔴 **Code review ba tầng** — 18 phát hiện sau gộp trùng, **11 vá · 5 hoãn · 2 nhiễu**. Ba khuyết tật nặng đều nằm trên đường flush lúc **ghi trượt** và lúc **thoát**: ① một lượt ghi trượt quay vòng ở `setTimeout(…, 0)` — đo được **8 001** lượt IPC trong 10 giây, và `error` bị nuốt nên không một dòng log nào; ② `inFlight` là `boolean` khiến `await flushEditorNow()` là một **lời hứa rỗng**, nên chữ gõ trong lúc lô bay chết cùng lượt thoát; ③ lệnh `CloseRequested` **thứ hai** đẻ một luồng chờ thứ hai và `release()` sớm ⇒ `destroy()` khi lô ghi gốc còn bay. Cả ba nay có lưới máy. Tầng Acceptance Auditor **không** tìm được vi phạm AC nào về hành vi — thứ lọt lưới là những đường mà **không AC nào mô tả** |
| 2026-08-13 | 🔴 **Một lượt điều tra ca đỏ `<span>` rỗng — KHÔNG kết luận được, và một khẳng định đã được RÚT LẠI.** Đo được và tái lập được: chuỗi `pointer` của WebDriver để cửa sổ mất tiêu điểm **hệ điều hành** *(`hasFocus: false` · `selType: "None"` · `activeElement: SECTION.mode`)*, và WebKit không giữ vùng chọn trong một cửa sổ như vậy ⇒ ca đỏ **không phân biệt được** *"sản phẩm hỏng"* với *"bộ đo hỏng"*. Nhưng lượt xanh trọn vẹn *(`SPAN.sent` · `Caret` · `execCommand` `true` · 2/2)* chỉ xảy ra **một lần trên tám**, và câu *"sản phẩm không hỏng"* viết ra dựa trên nó **đã bị rút**: một mẫu bằng 1 trên một bộ đo đã ghi là chập chờn không phải một phép đo. Mã dựng trên kết luận đó đã **gỡ**. Ba việc kế tiếp và số đo đầy đủ ở `deferred-work.md` §*Đo thêm về ca đỏ `<span>` rỗng*. **AC8 vẫn KHÔNG đạt** |
| 2026-08-13 | 🔵 **Ba lời khai lệch nhau về e2e được hợp nhất bằng một PHÉP ĐO, không bằng một lượt chọn.** `npm run test:e2e` chạy thật: `editor-typing-flush.e2e.mjs` = **1 xanh / 1 đỏ**, đỏ ở `:133`. ⇒ §ĐÍNH CHÍNH đúng; *"2/2 xanh"* ở §Completion Notes sai; checkbox Task 3 còn mang chẩn đoán **AD-34** đã bị bác. Cả ba chỗ viết lại theo **một** bản, và chính tệp e2e nay mang một khối chú thích nói rằng ca đó đỏ **có chủ** — cấm `skip`, cấm đổi mệnh đề, cấm né bằng fixture có sẵn chữ |
