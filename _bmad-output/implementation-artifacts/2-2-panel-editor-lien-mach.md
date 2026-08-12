---
baseline_commit: c86c2fbe18d541a020703a46a5317580a53895ea
---
# Story 2.2: Panel Editor liền mạch

Status: done

**Covers:** UX-DR19 · UX-DR20 · AD-1 *(nhãn `Covers: UX-DR13` ở `epics.md:2036` trỏ sai nguồn — xem §Điều kiện khởi hành mục 6)*
**Epic:** 2 — Biên tập theo segment · story **thứ hai**, ngay sau 2.1 *(đã `done`)*
**Nguồn:** `epics.md:2034-2073` · UX-DR2 · UX-DR3 · UX-DR5 · UX-DR6 · UX-DR7 · UX-DR8 · UX-DR12 · UX-DR16 · UX-DR19 · UX-DR20 (`epics.md:495-539`) · AD-1 · AD-21 · AD-31 · AD-34 · AD-37 (`ARCHITECTURE-SPINE.md`)
**Nợ đóng ở đây:** `deferred-work.md:2012-2024` *(chi phí `insert_segments` chuẩn bị lại statement mỗi hàng — chủ đích danh là Story 2.2)* · nửa **Editor** của nợ *"bề mặt đọc phải tự khai token, không kế thừa `ui-md` của `body`"* (`deferred-work.md:130-133`)
**Nợ ĐI QUA đây mà KHÔNG đóng:** `deferred-work.md:180` *(`isTypingZone` mù shadow DOM — ghi *"nhặt lại ở Epic 2"*, nhưng Editor của story này **chưa gõ được** theo phán quyết Quyết định #1 ⇒ chuyển chủ sang **Story 2.3**, Task 10.1)* · `:140-146` *(vế DOM không có phép kiểm tự động)* · `:875-882` *(không bộ chạy test frontend)* · hàng Deferred **thư viện editor** và **ảo hoá danh sách dài** của `ARCHITECTURE-SPINE.md:886,888`

---

## Điều kiện khởi hành — ĐỌC TRƯỚC KHI GÕ MỘT DÒNG

### 1. Cây làm việc SẠCH, và đây là mốc gốc

`git status --porcelain` trả **0 dòng** lúc dựng story này (2026-08-12). `baseline_commit` ở frontmatter là SHA thật của `HEAD`: `c86c2fb` *(“feat(segment): implement split chapter functionality…” — lượt commit của Story 2.1)*. Không có món vá cũ nào phải commit riêng trước.

### 2. Bảng `segment` KHÔNG có cột nào chứa bản dịch, và cũng KHÔNG có cột trạng thái

Tám cột thật hôm nay (`schema.rs:322-333`):

```sql
CREATE TABLE segment (
  id INTEGER PRIMARY KEY AUTOINCREMENT, chapter_id INTEGER NOT NULL, ord INTEGER NOT NULL,
  source_text TEXT NOT NULL, is_paragraph_end INTEGER NOT NULL, retired_at TEXT,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE INDEX idx_segment_chapter_ord ON segment (chapter_id, ord);
```

Doc-comment ngay trên nó (`schema.rs:293-296`) đã gọi tên ba cột vắng mặt và giao chủ cho từng cột:

> *"**Ba cột CỐ Ý không có, và mỗi cột có chủ:** `target_text` (bản dịch) → **Story 2.2/2.3**, đi kèm **bước di trú 6** — thêm hôm nay là đoán trước hợp đồng flush của AD-35 mà 2.3 chưa chốt. `status` (máy trạng thái AD-31) → **Story 2.5**. `role` → Story 6.13."*

🔴 **Hệ quả không được đi vòng:** trong **năm** giá trị vạch lề mà AC3 đòi, hôm nay chỉ **hai** có nguồn dữ liệu thật — *không vạch* (chưa dịch) và `primary` (đang sửa, suy từ tiêu điểm). `confirmed` chờ 2.5 · `tm-rule` chờ Epic 7 · `ornament` có **chỗ đọc** (`retired_at`, dựng sẵn ở 2.1 đúng vì lý do này — `schema.rs:287-289`) nhưng **chưa đường nào cho segment về hưu** cho tới 2.8. Đây là **năng lực chưa dựng, không phải lệch spec**: AC mô tả đích đến. Ghi thành nợ có chủ, **đừng sửa `epics.md`**.

Bước di trú kế tiếp của `project.db` là **6** — hôm nay bộ có **bốn** bước tới đích **5** (`schema.rs:335-339`; số **4** là số đã cháy, và `pinned_contract.rs` canh việc đó).

### 3. Dự án KHÔNG có bộ chạy test frontend, và KHÔNG được thêm

NFR15, Ice chốt ở Story 1.5 và giữ qua mười story. Ghi tại chỗ ở `src/commands/registry.ts:10-13`, `src/commands/README.md:20`, `src/i18n/README.md:101`:

> *"Dự án không có bộ chạy test frontend, và thêm một (`vitest`) là thêm một phụ thuộc phải rà tương thích GPLv3 bằng cách mở tệp giấy phép trong nguồn đã tải, rồi vào bảng Stack **trước khi** thêm. Đó là quyết định của Ice."*

🔴 **Story này là story THUẦN FRONTEND đầu tiên của Epic 2.** Nghĩa là mọi thứ nó dựng rơi đúng vào vùng không có lưới tự động — cùng lớp nợ đã ghi cho 1.16 · 1.17 · 1.18 · 1.18b · 1.19 (`deferred-work.md:875-882`). Ba đường nghiệm thu còn lại, dùng cả ba, đừng dùng một:
1. **Cổng tĩnh** `scripts/check-*.mjs` — đọc CSS/template, cưỡng chế được vế **khai báo**;
2. **Bàn đo chạy tay** trong trình duyệt — vế **thị giác** và vế **đo số** (chiều cao vạch, NFR2);
3. **e2e WebdriverIO** — vế **hành vi trong webview thật**. ⚠️ ESLint **cấm** `.click()` trong `e2e/**` từ Story 1.22; dùng `realClick()`. Bộ e2e **chập chờn** (8 lượt gần nhất 6 xanh / 2 đỏ): gặp đỏ không tái lập được thì **bắt nguyên văn lỗi TRƯỚC**, đừng chạy lại ngay.

### 4. BA CỔNG SẼ ĐỎ nếu chép thẳng mockup — đọc trước khi mở tệp mockup

Mockup là bản phác. `EXPERIENCE.md:312` phân xử sẵn: *"Bản dựng là minh hoạ của spine, không phải nguồn sự thật — khi mâu thuẫn, `DESIGN.md` và `EXPERIENCE.md` thắng."* Ba chỗ chép thẳng làm đỏ cổng, mỗi chỗ một cổng khác nhau:

| Chép cái gì | Cổng bắt | Đường ra |
| --- | --- | --- |
| `.bd { color: var(--orn) }` — `⏐` màu `ornament` | **Kiểm C** `check-tokens.mjs:1243-1257` — `ornament` sau `color:` ở **bất kỳ đâu** trong `src/**` là FAIL, và **KHÔNG có miễn trừ nào tồn tại** | Story này **dựng** miễn trừ có tên — Quyết định #5 |
| `.sent:hover .bd { opacity: .55 }` | **Kiểm D** `check-tokens.mjs:1286-1308` — `0` và `1` đi lọt, **mọi giá trị trung gian FAIL** | `/* aura-allow-opacity: <lý do> */` ngay trên khai báo — đường thoát **đã có sẵn** |
| `.caretsent`/`.tmsent { box-shadow: 0 0 0 3px … }` | **Kiểm F** `check-tokens.mjs:1369-1394` — `box-shadow` cấm **tuyệt đối**, `z-index` có miễn trừ còn bóng đổ thì **không**, và sự bất đối xứng đó là có chủ ý | **Đừng dựng chúng** — xem AC12: chúng còn vi phạm *"vạch lề là cách duy nhất"* |

Cộng hai cổng nền: **Kiểm B** cấm màu viết thẳng trong component, **Kiểm B2** cấm cỡ chữ viết thẳng. Mockup ghi `font-size:15px;line-height:1.95` — trong sản phẩm phải là `var(--font-editor)` / `var(--leading-editor)`.

Hai chỗ mockup lệch số so với tài liệu, **tài liệu thắng**: `.gmark{border-radius:1px}` so với token `segment-gutter-rule.radius = sm` (**2px**); và `workspace-dark.html` hover ở `opacity:.75` so với `0.55` mà `DESIGN.md:382` + AC5 chốt cho **cả hai theme**.

### 5. Bốn thứ ĐÃ CÓ SẴN trong `EditorPanel.vue` — đừng dựng lại

Tệp `src/panels/EditorPanel.vue` hôm nay **39 dòng, rỗng có chủ ý**, nhưng bốn mối nối đã cắm xong:

1. **Vỏ `PanelFrame`** — `owner="panel.editor"`, `status-key="panel.editor.status"`. Cả hai khoá i18n đã có trong `vi.json:69,91`.
2. **Hợp đồng tiêu điểm** — `PanelFrame.vue:129-132` gọi `declareFocus(owner, () => root.value)` lúc `mounted` và `releaseFocus` lúc unmount. **Đây chính là "điểm vào đã khai" mà AC7 nói tới**, và nó đã chạy. `FOCUS_OWNERS` (`src/commands/index.ts:64-72`) đã chứa `'panel.editor'`.
3. **Vạch tiêu điểm panel** — `.panel.focused::before` (`PanelFrame.vue:205`), CSS thuần, **không** `box-shadow`. Đừng dựng vạch thứ hai: đây là vạch của **panel** (UX-DR8/DR17), khác hẳn vạch của **segment** (UX-DR19) mà story này dựng.
4. **Hợp đồng vùng chọn** — `useSelectionSurface(surface, 'source')` (Story 1.18). Khi Editor có chữ thật, Auto-Lookup và bốn command mở rộng vùng chọn bằng phím **tự hoạt động, không cần cài lại** — đúng lời hứa `epics.md:1762`. ⚠️ Giữ nguyên lời gọi này; `SELECTION_SURFACE_FLOOR = 7` đang canh nó.

Khung bốn panel cũng đã xong: `PANEL_COMPONENTS['panel.editor'] = 'editor'` (`workspaceLayout.ts:66`), đăng ký component ở `WorkspaceDock.vue:52,104`, và `panel.editor` nằm trong `NEVER_SACRIFICED` (`:155`). **Không đụng `src/layout/**`.**

### 6. Nhãn `Covers: UX-DR13` trỏ SAI nguồn — đây là DR thật

`epics.md:2036` ghi `**Covers:** UX-DR13 · AD-1`. Nhưng UX-DR13 (`epics.md:523`) là *"Workspace là lưới 2×2 mặc định"* — quyết định **bố cục lưới**, và Story 1.14 đã dựng xong nó. Không một chữ nào của UX-DR13 nói về nội thất Panel Editor.

Đặc tả thật của bảy AC trong story này nằm ở:

| AC | DR thật | Nguyên văn ở |
| --- | --- | --- |
| AC1 · AC2 · AC3 | **UX-DR19** | `epics.md:537` · `DESIGN.md:380` |
| AC4 · AC5 | **UX-DR20** | `epics.md:539` · `DESIGN.md:382` |
| AC6 | **UX-DR2** *(token `editor`)* + **UX-DR12** *(họ `read` cho nội dung)* | `epics.md:495`, `:519` |
| AC7 | **UX-DR7** + **AD-34 §2** | `epics.md:507` · `ARCHITECTURE-SPINE.md:406-417` |

Kèm bốn DR ràng buộc ngang: **UX-DR3** (`gutter-width 22px`) · **UX-DR5** (`ornament`/`tm-rule` là màu của **nét**, không bao giờ màu của chữ — *"ngoại lệ duy nhất đã đặc tả: ký tự ranh giới câu `⏐`"*) · **UX-DR6** (`opacity` nghỉ chỉ cho nét và nền) · **UX-DR16** (không elevation).

⚠️ **Dev KHÔNG sửa `epics.md`.** Tiền lệ quyết định #3 của Ice ở Story 1.3, giữ qua toàn Epic 1: chỉnh tài liệu quy hoạch là **một lượt riêng của Ice**. Ghi vào `deferred-work.md` với chủ là Ice.

### 7. Mọi bằng chứng chỉ xanh trên macOS — nhưng CI nay tự chạy lại

Ice chốt 2026-08-12 (`deferred-work.md:1861-1918`): trọn phần Windows dời về **cuối dự án**. Mọi thứ Epic 2 → Epic 9 thêm vào chạy **chỉ trên macOS** cho tới lượt đó, và *"khoảng mù không đứng yên — nó dày lên theo từng epic"*.

✅ **Đổi so với Story 2.1:** commit `f950332` đã **mở lại `push` + `pull_request`** trong `ci.yml` sau khi repo thành công khai. Không còn phải tự bấm `workflow_dispatch`. Nhưng bài học §8.1 của retro vẫn đứng: *12 lượt CI đỏ trôi qua 6 ngày vì không ai đọc* — **push xong thì đọc kết quả**.

---

## Story

As a người dịch,
I want gõ trên một trang văn bản liền chứ không phải một cái bảng, mà vẫn đọc được trạng thái từng câu,
So that tôi viết tự do trong khi sổ sách segment vẫn sạch.

---

## Acceptance Criteria

Nguyên văn từ `epics.md:2042-2071`, đánh số để tham chiếu:

**AC1** — **Given** Panel Editor · **When** hiển thị · **Then** văn bản là một trang liền mạch — **không ô, không bảng, không khối**

**AC2** — **Given** trạng thái của từng segment · **When** hiển thị · **Then** đọc ở vạch lề dọc 2px trong máng rộng 22px bên trái, cao đúng bằng câu tương ứng · **And** đây là **cách duy nhất** trạng thái segment được hiển thị

**AC3** — **Given** năm giá trị trạng thái · **When** hiển thị · **Then** `confirmed` đã xác nhận · `primary` đang sửa · `tm-rule` điền sẵn từ TM chưa xác nhận · **không vạch** chưa dịch · `ornament` mờ đã về hưu

**AC4** — **Given** ranh giới câu ở trạng thái nghỉ · **When** hiển thị · **Then** ký tự `⏐` màu `ornament` ở `opacity: 0`

**AC5** — **Given** con trỏ chuột rê qua hoặc tiêu điểm bàn phím chạm tới một câu · **When** xảy ra · **Then** ranh giới câu hiện ở `opacity: 0.55`

**AC6** — **Given** văn bản trong Editor · **When** hiển thị · **Then** dùng token `editor` họ `read`, giãn dòng 1.95

**AC7** — **Given** Panel Editor · **When** nhận tiêu điểm · **Then** dời focus DOM tường minh tới điểm vào đã khai

### AC bổ sung — dẫn xuất từ kiến trúc, từ UX, và từ đo đạc mã nguồn

Bảy AC trên không nói hết thứ phải đúng để tính năng chạy được trong hệ thống đang có. Mười một AC dưới đây **cùng hạng ràng buộc**, mỗi cái neo vào một nguồn kiểm chứng được:

**AC8 — mọi số đo đi qua token, không một giá trị nào viết thẳng.** `--space-gutter-width` (22px, `tokens.json:481`) · `--font-editor` · `--leading-editor` · `--face-editor` (`tokens.json:397-403`) · `--color-confirmed` · `--color-primary` · `--color-tm-rule` · `--color-ornament`. **Không token mới nào cần dựng** — đo ngày 2026-08-12: cả tám biến đã tồn tại. Nghiệm thu: `npm run check:tokens` xanh (Kiểm B cấm màu viết thẳng, Kiểm B2 cấm cỡ chữ viết thẳng).

**AC9 —** `opacity: 0.55` **mang miễn trừ có tên, và miễn trừ đó nêu đúng lý do UX-DR6.** Kiểm D (`check-tokens.mjs:1286-1308`) cho `0` và `1` đi lọt và đỏ với **mọi** giá trị trung gian; đường thoát `/* aura-allow-opacity: <lý do> */` đã có sẵn và đã nghiệm thu ở story trước. Lý do phải viết ra là *"đây là **nét**, không phải chữ"* (UX-DR6 · UX-DR5). Nghiệm thu **đỏ-rồi-xanh**: gỡ dòng miễn trừ ⇒ `check:tokens` đỏ đúng dòng đó.

**AC10 — ngoại lệ** `ornament` **làm màu ký tự `⏐` được dựng thành một MIỄN TRỪ CÓ TÊN, không phải một lượt nới luật.** Kiểm C (`check-tokens.mjs:1243-1257`) hôm nay đỏ với **mọi** `color:`/`-webkit-text-fill-color:` chứa `--color-ornament`, và **không có đường thoát nào**. Chính `tokens.json:99` đã hẹn trước ca này: *"Ngoại lệ duy nhất đã đặc tả: ký tự ranh giới câu `⏐` — **thuộc Story 2.x**, không phải story này."* Story này **là** Story 2.x đó. 🔴 **Cấm** hai đường tắt: gỡ `ornament` khỏi `EXPECTED_NEVER_TEXT` *(mất luôn vế `tm-rule` dùng chung tập đó)*, và khai một biến CSS cục bộ chép giá trị hex *(đúng thứ AD-34 tồn tại để chặn)*. Đường đúng là khuôn `aura-allow-…` mà Kiểm D và Kiểm F đã dựng tiền lệ. Nghiệm thu **đỏ-rồi-xanh** ba ca: có miễn trừ ⇒ xanh · gỡ miễn trừ ⇒ đỏ · miễn trừ đặt ở một khai báo `color:` **khác** ⇒ vẫn đỏ.

**AC11 — không** `box-shadow` **nào, kể cả kiểu "đệm giả".** Kiểm F cấm tuyệt đối, **không miễn trừ**. Mẫu `box-shadow: 0 0 0 3px <cùng màu nền>` của mockup là một kỹ thuật đệm, không phải bóng đổ — nhưng cổng đọc **thuộc tính**, không đọc ý định, và sự bất đối xứng đó là có chủ ý (`check-tokens.mjs:1364-1368`). Nghiệm thu: `npm run check:tokens` xanh.

**AC12 — không kênh thị giác thứ HAI nào cho trạng thái segment.** AC2 nói *"đây là cách duy nhất"*, `DESIGN.md:380` lặp lại, và `EXPERIENCE.md:99` giải thích cái giá: *"vạch lề **đã dùng hết năm giá trị**"* — nên UX-DR22 buộc Proofreader phải đi gạch chân lượn sóng. ⇒ **Không** tô nền câu (`.caretsent` · `.tmsent` của mockup), **không** badge, **không** icon, **không** số thứ tự segment chen giữa dòng (`key-screen-workspace.html:82` cấm đích danh), **không** giá trị vạch thứ sáu. Nghiệm thu: một cổng tĩnh đếm số giá trị vạch mà mã khai và khẳng định **đúng năm**, đỏ khi thêm giá trị thứ sáu.

**AC13 — đường nạp segment là một lệnh IPC MỚI đọc theo `(chapter_id, ord)`, và không một ranh giới nào tính lại ở TypeScript.** Hôm nay **không lệnh nào** trả nội dung segment về webview — `split_chapter_into_segments` chỉ trả `SplitOutcome { chapter_id, segment_count }` (`commands/segment.rs:204-218`), và `read_open_chapter` vẫn trả `chapter.source_text` **nguyên khối** (`commands/chapter.rs:26-37`). Index `idx_segment_chapter_ord` đã được dựng sẵn **đúng cho lượt đọc này** (`schema.rs:309-313`). Lỗi mới theo AD-21 `{ code, message_key, params, retryable }`, khoá chuỗi vào `vi.json`, **không chữ tiếng Việt có dấu ở vị trí mã `.rs`**. AC12 của Story 2.1 vẫn áp nguyên: bộ tách sống ở Rust, `segment_boundary.rs` canh bằng máy. Nghiệm thu: `npm run check:i18n` xanh · lệnh mới có test hành vi ở `segment_contract.rs`.

**AC14 — chiều cao vạch lề ĐO từ hình học thật của câu, và có một trần đo được.** *"Cao đúng bằng câu tương ứng"* không cài được bằng CSS thuần: câu chảy **inline** trong một dòng văn liên tục, nên một câu bắt đầu giữa dòng và kết thúc giữa dòng khác, chiếm nhiều hình chữ nhật. Phải đo (`Range.getClientRects()` hoặc tương đương) và **tính lại** khi: đổi kích thước panel · font web nạp xong · nội dung đổi · đổi theme. Nghiệm thu: bàn đo ghi số thật cho một Chương **lớn nhất có thật** và đối chiếu NFR2 — **không frame nào vượt 50 ms**. Nếu vượt, đó là **số của Story 2.4**, ghi lại và báo, đừng tối ưu mù.

**AC15 — 0 phụ thuộc npm mới.** `package.json` hôm nay có **đúng ba** dependency runtime: `@tauri-apps/api` · `dockview-vue` · `vue`. Bảng Stack ghim chính xác, và NFR15 bắt mọi phụ thuộc mới phải rà giấy phép rồi vào bảng Stack **trước khi** thêm. 🔴 Nếu dev kết luận story này **cần** một thư viện editor, **dừng lại và báo** — đó là hàng Deferred *"thư viện editor cho panel Editor"* (`ARCHITECTURE-SPINE.md:886`) mà **Story 2.4** mang AC ghi lại lựa chọn (`epics.md:2142-2145`), không phải một quyết định gõ vào giữa story này. Xem Quyết định #1.

**AC16 — mọi sàn `*_FLOOR` bị vượt được nâng theo SỐ THẬT, đo chứ không ước.** Quần thể đo 2026-08-12: `src/**` = **32** `.ts` + **15** `.vue`; `src-tauri/src/**` = **42** `.rs`. Sàn phải rà nếu story thêm tệp: `VUE_FLOOR = 13` và `TS_FLOOR = 27` (`check-commands.mjs:211,216`) · `VUE_FLOOR = 13` và `RS_FLOOR = 36` (`check-i18n.mjs:279,289`) · `FILE_FLOOR = 37` và `COMPONENT_FILE_FLOOR = 35` (`check-tokens.mjs:86-87`) · `FILE_FLOOR = 35` (`check-layout.mjs:95`) · `COMMAND_FLOOR = 29` nếu thêm command (`check-commands.mjs:223`) · `SELECTION_SURFACE_FLOOR = 7` (`:1835`). Số thật đo được ghi vào §Completion Notes.

**AC17 — món nợ `insert_segments` có chủ là story này được ĐO, rồi mới quyết.** `deferred-work.md:2012-2024` giao đích danh: `commands/segment.rs:74-88` gọi `tx.execute` với SQL literal **trong vòng lặp**, nên `rusqlite` parse lại câu lệnh mỗi hàng — quy mô thật đã đo ở 2.1 là **~9.850 lượt parse trong một giao dịch** cho Chương lớn nhất, tất cả trên writer **duy nhất, nối tiếp** của AD-11. Nợ ghi rõ *"hoãn vì **chưa ai đo**, không phải vì nó nhỏ"*. ⇒ Việc của story này là **một phép đo**, rồi vá hoặc đóng kèm số. Đừng vá mù, và đừng bỏ qua.

**AC18 — bề mặt Editor KHÔNG gõ được ở lượt này, và một cổng cưỡng chế điều đó.** Hệ quả trực tiếp của phán quyết Ice cho Quyết định #1 *(§Task 0)*: gõ hạ cánh ở **Story 2.3**, cùng lượt với hợp đồng flush AD-35, nên **không tồn tại** lúc nào trên nhánh chính mà người dùng gõ được vào một bề mặt chưa có đường lưu. Nghiệm thu: một cổng tĩnh khẳng định `EditorPanel.vue` không mang `contenteditable`, không chứa `<textarea>`/`<input>`, và không có handler sửa văn bản nào; **đỏ-rồi-xanh** bằng cách tiêm `contenteditable="true"` vào template. ⚠️ Đây **không** phải một lời khuyên về phạm vi — nó là một mệnh đề nghiệm thu, cùng hạng với mười bảy AC trên.

---

## Task 0 — NĂM QUYẾT ĐỊNH, chốt TRƯỚC dòng mã đầu tiên

Khuôn cố định của mọi story lớn trong dự án (1.17 · 1.18 · 1.19 · 1.20 · 1.21 · 2.1). Mỗi quyết định có **đề xuất mặc định kèm lý lẽ đo được**. Dev đọc, xác nhận hoặc phản biện **bằng số** — không im lặng thi hành, và không tự đổi sau khi đã gõ mã.

🔵 **Quyết định #1 đã được Ice CHỐT lúc dựng story (2026-08-12) — dev KHÔNG mở lại nó.** Bốn quyết định còn lại (#2 → #5) vẫn mở và vẫn theo đúng khuôn trên: xác nhận hoặc phản biện bằng số.

### Quyết định #1 — Editor hiển thị CÁI GÌ hôm nay, và có gõ được không? · ĐÃ CHỐT

🔴 **Đây là quyết định chặn cả story.** Editor là panel **Bản dịch** (`vi.json:69`), token `editor` khai vai *"Bản dịch trong Editor"* — nhưng **không cột nào chứa bản dịch tồn tại** (§Điều kiện khởi hành mục 2). Ba đường, và mỗi đường có một cái giá phải nói ra:

**(a) Chỉ-đọc trên `source_text`, không thêm cột.** Rẻ nhất, không di trú. **Loại** — nó dựng một lời nói dối nhìn thấy được: panel *"Bản dịch"* hiện nguyên văn tiếng Trung. Và AC6 *"văn bản trong Editor"* mất nghĩa.

**(b) Thêm `target_text` (bước di trú 6), bề mặt CHỈ-ĐỌC, không gõ.** Trung thực về vai, và `schema.rs:293` cho phép đích danh *(`target_text` → "Story **2.2**/2.3")*. Cái giá: mọi `target_text` hôm nay là chuỗi rỗng ⇒ **trang trắng** ⇒ AC2 *(vạch cao bằng câu)*, AC4 và AC5 *(ranh giới `⏐`)* **không nghiệm thu bằng mắt được** trên dữ liệu thật, chỉ nghiệm thu được trên một fixture.

**(c) Thêm `target_text`, bề mặt GÕ ĐƯỢC vào state cục bộ, KHÔNG flush.** AD-1 cho phép đúng điều này bằng chữ: *"ngoại lệ duy nhất, tường minh: **văn bản đang gõ trong Editor là state cục bộ frontend**, chỉ qua IPC khi auto-save, xác nhận segment, hoặc rời segment."* Story demo được trọn vẹn. Cái giá **nghiêm trọng và phải nói thẳng**: giữa 2.2 và 2.3 tồn tại một cửa sổ mà người dùng gõ và **mất trắng khi đóng app**, không một dấu hiệu nào — mà AC của 2.3 lại cấm *"dấu chấm chưa lưu"*.

**Đề xuất mặc định: (b), CỘNG một điều kiện.** Lý lẽ: cửa sổ mất dữ liệu im lặng của (c) là đúng lớp khuyết tật mà cả Epic 2 tồn tại để chống (NFR18 — *"mất tối đa 5 giây"*), và ship nó lên nhánh chính dù chỉ một story là tự mâu thuẫn. Đường (b) giữ nhánh chính luôn trung thực. **Điều kiện kèm theo:** vì (b) làm AC2/AC4/AC5 không nghiệm thu được trên dữ liệu thật, story phải dựng **một fixture có bản dịch thật** *(một Chương ngắn, `target_text` bơm thẳng bằng SQL trong bàn đo — **không** qua app)* và nghiệm thu thị giác trên đó, ghi ảnh chụp vào §Debug Log References.

🔵 **PHÁN QUYẾT của Ice 2026-08-12: đường (b), kèm nguyên điều kiện fixture.** Đường (c) và đường gộp-2.2-với-2.3 đều bị loại. Bốn hệ quả cưỡng chế được, mỗi cái là một mệnh đề nghiệm thu chứ không phải một lời khuyên:

1. **Story này thêm `target_text` và bước di trú 6** — Task 1 không còn là một nhánh điều kiện.
2. 🔴 **Bề mặt Editor KHÔNG gõ được ở lượt này.** Không `contenteditable`, không `<textarea>`, không handler `@input`/`@keydown` mang thao tác sửa văn bản. Gõ là **Story 2.3**, cùng lượt với hợp đồng flush AD-35 — nên **không tồn tại** lúc nào trên nhánh chính mà người dùng gõ được vào một bề mặt chưa có đường lưu. Nghiệm thu: một cổng tĩnh khẳng định `EditorPanel.vue` không mang thuộc tính `contenteditable` và không có handler sửa văn bản nào; cổng đó **đỏ** khi tiêm một `contenteditable="true"` vào template.
3. **Fixture là bắt buộc, không tuỳ chọn.** AC2 · AC4 · AC5 chỉ nghiệm thu được trên nó. Một Chương ngắn, `target_text` bơm bằng SQL **trong bàn đo**, và bàn đo **không mở `.atproj` nào của Ice bằng app** — cùng kỷ luật Story 2.1 đã giữ ở Task 8 *(lược đồ nay có target 6, nên một lượt mở là một lượt di trú thật trên dữ liệu thật)*.
4. **Món nợ `isTypingZone`** (`deferred-work.md:180`, ghi *"nhặt lại ở Epic 2 — Editor là vùng gõ tự do đầu tiên"*) **đi qua story này mà KHÔNG đóng**, và nay lý do là dứt khoát: vùng gõ tự do đầu tiên sinh ra ở **2.3**, không ở đây. Chuyển chủ sang **Story 2.3** trong `deferred-work.md`.

⚠️ **Hệ quả Ice đã được báo trước và vẫn chọn:** kết thúc story này, Panel Editor trên dữ liệu thật của Ice là **một trang trắng có máng lề trống** — vì mọi `target_text` đều rỗng và chưa có đường nào điền chúng. Đó là trạng thái **đúng** *(mọi câu "chưa dịch" ⇒ "không vạch", theo đúng AC3)*, nhưng nó **không nhìn ra được là đã chạy hay đã hỏng** nếu chỉ mở app. Bằng chứng story này chạy đúng nằm ở fixture của Task 7, không ở màn hình Workspace.

### Quyết định #2 — Cơ chế đo chiều cao vạch lề

Câu chảy **inline**: `.doc` là một dòng văn liên tục, mỗi câu là một `<span>` có thể bắt đầu giữa dòng, xuống dòng, kết thúc giữa dòng khác. *"Cao đúng bằng câu"* vì thế là **từ mép trên hình chữ nhật đầu tới mép dưới hình chữ nhật cuối** của câu đó.

Ba đường: (a) `Range.getClientRects()` trên mỗi câu, vạch `position: absolute` trong máng — đúng mockup, đo được; (b) mỗi câu một `display: block` riêng — **loại**, nó đúng là *"chia thành khối"* mà AC1 cấm; (c) CSS `::before` gắn vào chính câu — **loại**, một pseudo-element inline không trải theo chiều cao nhiều dòng của câu.

**Đề xuất mặc định: (a).** Kèm ba điều kiện đo được: tính lại qua `ResizeObserver` trên `.doc`; tính lại sau `document.fonts.ready` *(ba font nhúng — UX-DR4 — nạp xong mới có hình học đúng)*; gộp mọi lượt tính vào **một** `requestAnimationFrame`, không tính từng câu một. Nghiệm thu là số của AC14.

⚠️ **Ảo hoá là hàng Deferred Giai đoạn 3** (`ARCHITECTURE-SPINE.md:888`). Chương lớn nhất có thật đo ở 2.1 là **9.850 segment** trong một Chương. Nếu bàn đo cho thấy (a) vượt NFR2 ở quy mô đó, **đó là một phép đo phải báo**, không phải giấy phép để dev tự dựng ảo hoá trong story này.

### Quyết định #3 — `⏐` là span thật hay pseudo-element?

Mockup dùng **span thật**: `<span class="bd">⏐</span>` nằm trong `.sent`.

🔴 **Có một vết sẹo trực tiếp trong kho về đúng lớp lỗi này.** Story 1.18b chèn `WORD_JOINER` (`U+2060`) vào DOM Panel Source, và hệ quả là **rò ký tự lúc copy** — `deferred-work.md:839-848` ghi rằng trên WKWebView, bôi đen bằng phím rồi `⌘C` dán ra chuỗi lẫn ký tự chèn, và `onCopy` phải dựng lại chuỗi từ đường đọc DOM. Một `⏐` là **span thật** trong Editor lặp lại đúng lỗi đó, lần này trên một bề mặt sẽ **gõ được ở Story 2.3** — nên nó còn bị gõ đè, bị con trỏ đi xuyên qua, và bị đếm vào độ dài văn bản.

**Đề xuất mặc định: pseudo-element** — `.sent::after { content: '⏐' }`. Nó không nằm trong cây văn bản, nên không copy được, không chọn được, không gõ đè được. Cái giá phải biết: pseudo-element không nhận `:hover` riêng *(dùng `.sent:hover::after`, đúng khuôn mockup)*, và nó **không** hiện trong bàn đo nếu bàn đo chép DOM thay vì chép CSS — ghi rõ điều đó trong bàn đo.

⚠️ Vế **tiêu điểm bàn phím** của AC5 (*"con trỏ chạm tới"*) không dùng `:hover` được. Đường mặc định: `.sent:hover::after, .sent[data-caret]::after` — cờ `data-caret` do tầng TS đặt theo segment đang có tiêu điểm, cùng nguồn với vạch `primary` của AC3.

### Quyết định #4 — Ba trong năm giá trị vạch chưa có nguồn dữ liệu. Cài thế nào?

Hôm nay chỉ *không vạch* và `primary` đến được (§Điều kiện khởi hành mục 2). Hai đường: (a) cài **đúng hai** giá trị, ba giá trị kia thêm khi story chủ của chúng tới; (b) cài **cả năm** ngay, ba giá trị chưa có nguồn thì đọc từ một hàm phân giải duy nhất mà hôm nay **không bao giờ trả về chúng**.

**Đề xuất mặc định: (b).** Lý lẽ: `EXPERIENCE.md:99` nói năm giá trị là **tài nguyên hữu hạn đã tiêu hết** — bảng ánh xạ *trạng thái → token vạch* là một **hợp đồng**, và một hợp đồng cài nửa vời là chỗ để story sau chép sai. Cài cả năm ở **một** hàm phân giải, cộng cổng đếm của AC12, làm 2.5/2.8/Epic 7 chỉ phải nối nguồn dữ liệu, không phải sửa tầng hiển thị. Kèm điều kiện: mỗi nhánh chưa có nguồn mang một comment nêu **đích danh story chủ** (`confirmed` → 2.5 · `ornament` → 2.8 · `tm-rule` → Epic 7).

⚠️ **Không** dựng dữ liệu giả để ba nhánh kia "chạy" trong sản phẩm. Chúng chạy trong **bàn đo**, không trong `.atproj` của Ice.

### Quyết định #5 — Hình dạng miễn trừ cho `ornament` làm màu ký tự

Kiểm C hôm nay không có đường thoát nào (AC10). Hai khuôn đã có tiền lệ trong chính tệp cổng: `/* aura-allow-z-index: <lý do> */` (Kiểm F) và `/* aura-allow-opacity: <lý do> */` (Kiểm D), cả hai dùng chung hàm `exemptAt(p, index, tên)`.

**Đề xuất mặc định:** thêm `/* aura-allow-never-text: <lý do> */`, **có tham số là tên token** để một miễn trừ cấp cho `ornament` không lặng lẽ cấp luôn cho `tm-rule`. Nghiệm thu ba ca của AC10.

⚠️ **Cổng mới phải vào CI, không chỉ chạy tay** — bài học §4 của retro Epic 1 (`check:lint` từng sống một ngày ngoài CI). Ở đây nó là một lượt sửa **trong** `check-tokens.mjs`, vốn đã nằm trong CI, nên điều kiện thoả sẵn — nhưng `npm run check:gates` vẫn phải xanh.

---

## Tasks / Subtasks

- [x] **Task 0 — Chốt BỐN quyết định còn mở (#2 → #5).** Ghi phán quyết vào §Dev Agent Record **trước** dòng mã đầu tiên. Xác nhận hoặc phản biện **bằng số**. ⚠️ Quyết định **#1 đã chốt bởi Ice** — không mở lại, chỉ thi hành.
- [x] **Task 1 — Cột `target_text` và bước di trú 6** (AC13)
  - [x] 1.1 `SEGMENT_TARGET_DDL` hoặc `ALTER TABLE` trong `schema.rs`, `to_version: 6`. Doc-comment nêu **vì sao 6** và cập nhật dòng *"bốn bước"*.
  - [x] 1.2 Cập nhật `pinned_contract.rs` theo lược đồ mới (khuôn đã có từ 2.1).
  - [x] 1.3 Test: `project.db` mới đạt `user_version = 6`; một db ở 5 di trú lên 6 không mất hàng `segment` nào.
  - [x] 1.4 Cột mặc định chuỗi rỗng, **không** `NULL` — *"chưa dịch"* là một chuỗi rỗng, không phải một giá trị vắng mặt; nó quyết nhánh *"không vạch"* của AC3 và một `Option<String>` ở đó là hai cách nói cùng một điều.
- [x] **Task 2 — Lệnh IPC nạp segment của một Chương** (AC13)
  - [x] 2.1 Hàm thuần trước, `#[tauri::command]` là vỏ — khuôn `commands/chapter.rs:63-92`.
  - [x] 2.2 Đọc `ORDER BY ord` để index `idx_segment_chapter_ord` thành covering. Trả `id`, `ord`, `source_text`, `target_text`, `is_paragraph_end`, `retired_at`.
  - [x] 2.3 Lỗi theo AD-21; khoá chuỗi mới vào `vi.json`; `check:i18n` xanh.
  - [x] 2.4 Adapter TS ở `src/config/segment.ts` — khuôn `{ data, error }`, `hasIpcBridge()`, **không** ném.
  - [x] 2.5 Test hành vi ở `segment_contract.rs`.
- [x] **Task 3 — Trang liền mạch** (AC1, AC6, AC8)
  - [x] 3.1 `.doc` là **một** dòng văn liên tục; mỗi câu một `<span class="sent">`. Không `display: block`, không grid, không bảng.
  - [x] 3.2 Typography qua token: `var(--face-editor)` · `var(--font-editor)` · `var(--leading-editor)`. **Đóng nửa Editor** của nợ `deferred-work.md:130-133` — bề mặt đọc phải tự khai token, không kế thừa `ui-md` của `body`.
  - [x] 3.3 Cờ kết đoạn (`is_paragraph_end`) quyết chỗ ngắt đoạn — **đọc từ dữ liệu đã lưu**, AD-37 cấm suy ra lúc render.
- [x] **Task 4 — Máng lề và năm giá trị vạch** (AC2, AC3, AC12, AC14; phụ thuộc Quyết định #2, #4)
  - [x] 4.1 Máng `var(--space-gutter-width)`, vạch 2px, thụt trái 8px, bo `sm` — theo `DESIGN.md`, **không** theo `border-radius:1px` của mockup.
  - [x] 4.2 Một hàm phân giải duy nhất *trạng thái → token vạch*, cả năm nhánh, mỗi nhánh chưa có nguồn ghi đích danh story chủ.
  - [x] 4.3 Đo chiều cao bằng `Range.getClientRects()`; tính lại theo `ResizeObserver` + `document.fonts.ready`; gộp vào một `rAF`.
  - [x] 4.4 Cổng tĩnh đếm **đúng năm** giá trị vạch, đỏ khi có giá trị thứ sáu.
- [x] **Task 5 — Ranh giới câu `⏐`** (AC4, AC5, AC9, AC10; phụ thuộc Quyết định #3, #5)
  - [x] 5.1 Pseudo-element `content: '⏐'`, `color: var(--color-ornament)`, `opacity: 0`.
  - [x] 5.2 `.sent:hover::after` và `.sent[data-caret]::after` ⇒ `opacity: 0.55`, **cả hai theme** *(bỏ `.75` của mockup tối)*.
  - [x] 5.3 Miễn trừ `aura-allow-opacity` kèm lý do UX-DR6.
  - [x] 5.4 Dựng miễn trừ `aura-allow-never-text` trong `check-tokens.mjs` Kiểm C; nghiệm thu **đỏ-rồi-xanh** ba ca của AC10.
- [x] **Task 6 — Tiêu điểm** (AC7)
  - [x] 6.1 Xác nhận đường `PanelFrame` → `declareFocus` đã thoả AC7 cho **panel**; nếu điểm vào cần chi tiết hơn gốc `<section tabindex="-1">` *(ví dụ câu đang dở — `EXPERIENCE.md` KF-2)*, khai nó tường minh, **không** dựa vào focus mặc định của trình duyệt.
  - [x] 6.2 Không `outline: none` nào ngoài gốc `tabindex="-1"` — Kiểm H.
  - [x] 6.3 **Không** dựng vạch tiêu điểm panel thứ hai; `.panel.focused::before` đã có.
- [x] **Task 6b — Cổng "chưa gõ được"** (AC18; hệ quả 2 của phán quyết Quyết định #1)
  - [x] 6b.1 Cổng tĩnh: `EditorPanel.vue` không mang `contenteditable`, không `<textarea>`/`<input>`, không handler sửa văn bản.
  - [x] 6b.2 Nghiệm thu **đỏ-rồi-xanh**: tiêm `contenteditable="true"` ⇒ cổng đỏ; gỡ ⇒ xanh.
  - [x] 6b.3 Comment tại chỗ trong `EditorPanel.vue` nêu **đích danh Story 2.3** là chủ của vùng gõ, cùng khuôn comment mà 1.14 đã để lại cho Epic 2.
- [x] **Task 7 — Bàn đo chạy tay + fixture** (AC14; fixture là **bắt buộc**, hệ quả 3 của phán quyết Quyết định #1)
  - [x] 7.1 Fixture một Chương ngắn có `target_text` thật, bơm bằng SQL **chỉ trong bàn đo** — không mở `.atproj` của Ice bằng app.
  - [x] 7.2 Nghiệm thu thị giác cả hai theme: năm giá trị vạch, chiều cao khớp câu nhiều dòng, `⏐` ẩn/hiện.
  - [x] 7.3 Đo NFR2 trên Chương lớn nhất có thật; ghi số vào §Completion Notes. Vượt ngưỡng ⇒ **báo**, không tự tối ưu.
  - [x] 7.4 Ghi thẳng giới hạn của bàn đo *(chép CSS chứ không mount component thật — cùng lớp nợ `deferred-work.md:826`)*.
- [x] **Task 8 — Đo món nợ `insert_segments`** (AC17)
  - [x] 8.1 Đo chi phí parse-mỗi-hàng trên Chương lớn nhất có thật (~9.850 hàng).
  - [x] 8.2 Vá bằng `prepare_cached` **hoặc** đóng nợ kèm số. Quyết định đi theo số, không theo linh cảm.
- [x] **Task 9 — Cổng và sàn** (AC15, AC16)
  - [x] 9.1 `npm run check:gates` xanh; mọi cổng mới có mặt ở cả `package.json` lẫn `ci.yml`.
  - [x] 9.2 Nâng mọi `*_FLOOR` bị vượt theo **số thật đo được**, ghi số vào §Completion Notes.
  - [x] 9.3 Xác nhận **0** phụ thuộc npm mới. Nếu cần một cái ⇒ dừng và báo (AC15).
- [x] **Task 10 — Ghi nợ có chủ.** Ba giá trị vạch chưa có nguồn *(chủ: 2.5 · 2.8 · Epic 7)* · nhãn `Covers` sai của `epics.md:2036` *(chủ: Ice)* · giới hạn bàn đo · kết quả Task 8. Mỗi món ghi `deferred-work.md` **kèm chủ**.
  - [x] 10.1 Chuyển chủ của nợ `isTypingZone` (`deferred-work.md:180`) từ *"Epic 2"* sang **Story 2.3** đích danh — hệ quả 4 của phán quyết Quyết định #1: vùng gõ tự do đầu tiên sinh ra ở 2.3, không ở đây.

---

## Dev Notes

### Cái đã có, cái chưa có — đo ngày 2026-08-12

| Thứ | Trạng thái | Nguồn |
| --- | --- | --- |
| `EditorPanel.vue` | **39 dòng**, thân rỗng có chủ ý. Đã có `PanelFrame` + `useSelectionSurface(surface, 'source')` | đọc tệp |
| Hợp đồng tiêu điểm panel | **ĐÃ CÓ** — `PanelFrame.vue:129-132` `declareFocus(owner, () => root.value)` | đọc tệp |
| `'panel.editor'` trong `FOCUS_OWNERS` | **ĐÃ CÓ** | `commands/index.ts:64-72` |
| Khung bốn panel, `NEVER_SACRIFICED` | **ĐÃ CÓ**, đừng đụng | `layout/workspaceLayout.ts:66,103-108,155` |
| Token `editor` (`read`/15px/1.95/`wraps:true`) | **ĐÃ CÓ** | `tokens.json:397-403` |
| `gutter-width: 22px` | **ĐÃ CÓ** | `tokens.json:481` |
| Bốn màu vạch (`confirmed`·`primary`·`tm-rule`·`ornament`) | **ĐÃ CÓ**, cả hai theme | `tokens.json:16-52` |
| Khoá i18n `panel.editor.title` / `.status` | **ĐÃ CÓ** | `vi.json:69,91` |
| Cột `target_text` | **CHƯA CÓ** — bước di trú 6, chủ là 2.2/2.3 | `schema.rs:293-296` |
| Cột `status` | **CHƯA CÓ** — chủ là **Story 2.5**. Không enum trạng thái nào tồn tại ở Rust lẫn TS | `schema.rs:295` |
| Lệnh IPC trả nội dung segment | **CHƯA CÓ** — chỉ có `split_chapter_into_segments` trả `{ chapter_id, segment_count }` | `commands/segment.rs:204-218` |
| Miễn trừ cho `ornament` làm màu chữ | **CHƯA CÓ**, và `tokens.json:99` hẹn nó *"thuộc Story 2.x"* | `check-tokens.mjs:1243-1257` |
| Thư viện editor · ảo hoá · vitest | **CHƯA CÓ**, và cả ba đều **không** thuộc story này | `package.json` · SPINE:886,888 · NFR15 |

### Hợp đồng UX — số đo chính xác, đã đối chiếu ba nguồn

`DESIGN.md` component token `segment-gutter-rule`: `{ width: 2px, radius: sm, inset-left: 8px }`; máng `gutter-width: 22px`; `.doc` thụt trái **8px**, máng `padding-top: 4px`.

Bảng năm giá trị (`DESIGN.md:380` · `EXPERIENCE.md:105-113`), kèm story chủ của nguồn dữ liệu:

| Vạch | Nghĩa | Nguồn dữ liệu | Có hôm nay? |
| --- | --- | --- | --- |
| `confirmed` | đã xác nhận | `segment.status` (FR24) | ❌ **Story 2.5** |
| `primary` | đang sửa, con trỏ ở đây | tiêu điểm, tầng TS | ✅ |
| `tm-rule` | điền sẵn từ TM 100%, chưa xác nhận | FR58 | ❌ **Epic 7** |
| *(không vạch)* | chưa dịch | `target_text` rỗng | ✅ *(sau Task 1)* |
| `ornament` mờ | đã về hưu do gộp/tách | `segment.retired_at` — **cột đã có** | ❌ **Story 2.8** *(chưa đường nào về hưu)* |

Màu: `confirmed` `#5a6b3f`/`#9cb37a` · `primary` `#2f5d63`/`#7fb3ba` · `ornament` `#a9a196`/`#6a6459` · `tm-rule` `#b99a5e` **cùng một giá trị ở cả hai theme** *(nó là vạch, không chịu ràng buộc tương phản chữ — `DESIGN.md:184`)*. ⚠️ **Đọc qua token, không chép hex** — Kiểm B.

⚠️ **Tên biến trong mockup sáng LỆCH tên token thật:** `key-screen-workspace.html` dùng `--accent` và `--confirm`; token thật là `primary` và `confirmed`. Bản tối dùng đúng tên. Đừng chép tên biến từ mockup sáng.

### Ranh giới AD — cái Editor được phép và không được phép

**AD-1** (`SPINE:75-79`) — *"frontend chỉ render và giữ state UI... Ngoại lệ duy nhất, tường minh: **văn bản đang gõ trong Editor là state cục bộ frontend**, chỉ qua IPC khi auto-save, xác nhận segment, hoặc rời segment."* Ngoại lệ này là của **2.3**, không phải giấy phép để 2.2 dựng quy tắc nghiệp vụ ở TS.

**AD-31** (`SPINE:368-392`) — máy trạng thái sống ở `core/segment/`. Editor **không tự suy** trạng thái. Auto-save **không đổi trạng thái và không tạo `SegmentVersion`**. Xuất xứ so **văn bản đích hiện tại với bản lúc nạp segment**, **không cờ dirty**.

**AD-37** — cờ kết đoạn tính lúc nhập, **lưu xuống**, *"không suy ra lúc render"*. Task 3.3 đọc cờ, không đoán đoạn từ nội dung.

**AD-34 §2** (`SPINE:406-417`) — *"Mỗi chế độ và mỗi panel khai báo điểm vào focus. Chuyển panel trong dockview phải **dời focus DOM tường minh**; không chế độ nào được để focus rơi về `body`."* Đây là AC7 ở dạng kiến trúc.

**Cấm kỵ dễ vi phạm nhất trong story này:** tính lại ranh giới câu ở TypeScript. `Intl.Segmenter` **đã có mặt** trong kho (`wordBoundary.ts`, cấp **TỪ**, Story 1.18b) — đừng nhìn thấy nó rồi dùng nó cho câu. `segment_boundary.rs` canh vế Rust bằng máy; vế TS do người giữ.

### Chuẩn kiểm thử của kho

- **Hai loại tệp test Rust**, phân theo hậu tố: `*_contract.rs` = hành vi lúc chạy · `*_boundary.rs` = kiểm tĩnh trên cây nguồn.
- **Tên hàm test** là một câu mô tả hành vi, `snake_case`, **không** tiền tố `test_`. Ví dụ có thật: `a_retired_chapter_id_is_never_handed_out_again` · `the_language_branch_comes_from_source_lang_not_from_the_content`.
- **Không** script `test` trong `package.json` — Rust test chạy `cargo test --locked --manifest-path src-tauri/Cargo.toml`.
- **Thứ tự CI bắt buộc:** 11 cổng `check:*` → `npm run build` **trước** `cargo test` *(vì `tauri::generate_context!` nhúng `dist/` lúc biên dịch)* → `cargo test` → build ứng dụng thật → `check:scope`/`check:scope:bundled`.
- Cổng cưỡng chế hành vi ở TS chạy bằng cách **`import()` thẳng hàm thật** từ `scripts/check-*.mjs` *(Node ≥ 22.18 bóc kiểu TypeScript)* — đó là đường thay cho vitest, và nó chỉ áp cho **module thuần**, không cho `.vue`.

### Bài học Epic 1 và Story 2.1 áp thẳng vào story này

1. **Đo trước khi tin** (retro §7.1) — Quyết định #1 của Story 2.1 được quyết bằng một cây thăm dò chạy thật, không bằng lý lẽ; và nó **tái lập 5/5 hàng** của bảng. Task 7 và Task 8 của story này cùng khuôn đó.
2. **Cổng mới phải vào CI, không chỉ chạy tay** (retro §4) — Quyết định #5.
3. **Nợ nghiệm thu thị giác có hệ số nhân** (retro §5) — Story 1.21 đi từ 12 lên 19 hàng bàn đo treo. Story này là story **thị giác nhất** của Epic 2. Ghi từng hàng treo **kèm chủ**, đừng gom thành một câu.
4. **Một luật ngoài đơn hàng phải được ghi ra và lật được** — Story 2.1 thêm luật *"một câu phải có ít nhất một chữ"* do phép đo dựng ra, ghi `deferred-work.md` với chủ là Ice và *"chỗ lật là một dòng"*. Cùng chuẩn nếu story này sinh ra một luật hiển thị ngoài AC.
5. **Dev không sửa tài liệu quy hoạch** — tiền lệ quyết định #3 của Ice ở Story 1.3. Nhãn `Covers` sai (§mục 6) và mọi lệch `DESIGN.md` đi vào `deferred-work.md`, không vào `epics.md`.
6. **`in-progress` không phải chỗ đậu** (retro §8.2) — phải để dở thì ghi **nguyên nhân cụ thể** trong story file.
7. **Ký hiệu cấm** — emoji "biển cấm" `U+26D4` đã gỡ khỏi toàn kho (8.298 ca, 0 còn lại). Viết `không`/`KHÔNG` thẳng.

### Git intelligence — 5 commit gần nhất

`c86c2fb` Story 2.1 hạ cánh *(bảng `segment`, bước 5, bộ tách ở `core/segment/split.rs`, 34 ca test mới)* · `f950332` mở lại `push` + `pull_request` trong CI · `8ae61cd` thoát chuỗi PowerShell trong step đo `.msi` · `788a4ae` story 2.1 sẵn sàng cho dev · `5ec8e3d` gỡ 3.098 tệp công cụ AI khỏi index.

Đọc được từ đó: lượt gần nhất chạm **rất sâu** vào `src-tauri/src/core/**` và `schema.rs`, nhưng **không một dòng nào** vào `src/panels/**` — Story 2.1 giữ đúng lời hứa *"không đụng `src/panels/**` — Editor là 2.2"*. Story này vào một vùng frontend **đang yên** kể từ Story 1.21 (2026-08-11). Khuôn thông điệp commit của kho: `<type>(<scope>): <câu tiếng Việt mô tả điều đã thay đổi>`.

### Phụ thuộc mới — không có, và đó là chủ ý

Ba dependency runtime, ghim chính xác. Bảng Stack trở thành thứ `Cargo.lock`/`package-lock.json` xác nhận, không phải một danh sách mà mỗi story diễn giải lại. Story này thêm **0** gói. Xem AC15 nếu dev kết luận khác.

### Thông tin kỹ thuật mới nhất — vì sao phần này ngắn

Story này **không thêm phụ thuộc nào** (AC15) và không chạm API bên ngoài nào: toàn bộ bề mặt là CSS/DOM tiêu chuẩn (`Range.getClientRects`, `ResizeObserver`, `document.fonts.ready`, pseudo-element `content`) cộng SQLite qua `rusqlite` đã ghim. Không có phiên bản thư viện nào phải tra, không có breaking change nào áp vào. Ràng buộc phiên bản duy nhất đáng nhắc đã ghim sẵn trong bảng Stack: Vue **3.5.40** · TypeScript **5.9.3** · Vite **8.2.0** · `dockview-vue` **7.0.4** · `@tauri-apps/api` **2.11.1** · toolchain Rust **1.97.1** *(ghim đúng số máy Ice đang chạy — `@stable` sẽ trôi và làm số đo hết so sánh được)*.

⚠️ Một ràng buộc nền tảng **không tra được từ tài liệu, chỉ đo được**: mọi bằng chứng của story này chạy trên **WKWebView của macOS**, không phải Blink. Hai thứ trong story có tiền sử lệch giữa hai engine — hình học `getClientRects()` với chữ có dấu tiếng Việt, và hành vi copy/selection quanh ký tự chèn *(vết sẹo `WORD_JOINER` của 1.18b, `deferred-work.md:839-848`)*. Nghiệm thu trên Chrome rồi viết *"tương đương"* là đúng lỗi mà `deferred-work.md:145` đã ghi tên. Đo trong `npm run tauri dev` hoặc qua bộ e2e.

---

### Project Structure Notes

Tệp **mới** story này dự kiến tạo:

```
src/panels/editorSegments.ts        # state + phân giải trạng thái → token vạch (Quyết định #4)
src/panels/editorGutter.ts          # đo hình học câu, tính lại theo rAF (Quyết định #2)
```

Tệp **sửa**:

```
src/panels/EditorPanel.vue               # thân panel: trang liền mạch, máng, ranh giới câu
src/config/segment.ts                    # + adapter lệnh nạp segment
src-tauri/src/core/store/schema.rs       # + cột target_text, + Migration to_version: 6
src-tauri/src/commands/segment.rs        # + lệnh nạp segment (hàm thuần + vỏ), Task 8
src-tauri/src/core/i18n/mod.rs           # + MessageKey mới
src-tauri/src/lib.rs                     # + đăng ký lệnh mới
src-tauri/tests/segment_contract.rs      # ca mới
src-tauri/tests/pinned_contract.rs       # theo lược đồ 6
src/i18n/vi.json                         # khoá lỗi mới
scripts/check-tokens.mjs                 # miễn trừ `aura-allow-never-text` (Quyết định #5)
scripts/check-commands.mjs               # cổng đếm năm giá trị vạch (AC12), sàn
```

**Không** đụng: `src/layout/**` *(khung panel là 1.14, đã xong)* · `PanelFrame.vue` *(hợp đồng focus đã thoả AC7)* · `src/panels/SourcePanel.vue` · `core/segment/split.rs` *(AD-4 đóng băng ranh giới; sửa bộ tách ở đây là tái tách toàn kho)* · `epics.md` và `DESIGN.md` *(lượt riêng của Ice)*.

Quy ước đặt tên đã đo: Rust `snake_case` · Vue `PascalCase.vue` · state của panel là `<tênPanel>State.ts` cùng thư mục · khoá i18n phẳng theo dấu chấm · command trên dây `snake_case`, tham số `camelCase` *(do `invoke()` tự chuyển)* · struct qua biên IPC **không** đặt `#[serde(rename_all)]`.

---

### References

- AC nguyên văn — `_bmad-output/planning-artifacts/epics.md:2042-2071`
- UX-DR2 · DR3 · DR5 · DR6 · DR7 · DR8 · DR12 · DR16 · DR19 · DR20 — `epics.md:495`, `:497`, `:503`, `:505`, `:507`, `:509`, `:519`, `:529`, `:537`, `:539`
- Vạch lề segment và ranh giới câu — `ux-designs/.../DESIGN.md:380`, `:382`, `:400`; `EXPERIENCE.md:99`, `:105-115`, `:312`
- Mockup Editor — `mockups/key-screen-workspace.html:57-77` (CSS), `:113-133` (HTML), `:188-191` (chú giải); `mockups/workspace-dark.html:59-71`, `:126-141`
- AD-1 — `ARCHITECTURE-SPINE.md:75-79` · AD-21 — `:302-306` · AD-31 — `:368-392` · AD-34 — `:406-417` · AD-35 — `:419-425` · AD-37 — `:437-453`
- Hàng Deferred *thư viện editor* — `ARCHITECTURE-SPINE.md:886` · *ảo hoá danh sách dài* — `:888` · *ngưỡng WAL + nhịp flush* — `:883`
- `SEGMENT_DDL` và ba cột cố ý vắng — `src-tauri/src/core/store/schema.rs:257-296`, `:322-333`
- Lý do index `(chapter_id, ord)` — `schema.rs:298-321`
- `OpenChapter` trả nguyên khối — `src-tauri/src/commands/chapter.rs:26-37`
- `split_chapter_into_segments` — `src-tauri/src/commands/segment.rs:204-218` · adapter TS — `src/config/segment.ts`
- Hợp đồng focus của panel — `src/panels/PanelFrame.vue:129-132`, `:205` · `FOCUS_OWNERS` — `src/commands/index.ts:64-72`
- Token `editor` — `src/tokens/tokens.json:397-403` · `gutter-width` — `:481` · ngoại lệ `⏐` hẹn trước — `:99`
- Kiểm C `neverTextTokens` — `scripts/check-tokens.mjs:1242-1257` · Kiểm D `opacity` — `:1260-1311` · Kiểm F elevation — `:1358-1398` · Kiểm H focus ring — `:1401`
- Sàn cổng — `scripts/check-commands.mjs:211,216,223,1835` · `scripts/check-i18n.mjs:279,289` · `scripts/check-tokens.mjs:86-87` · `scripts/check-layout.mjs:95`
- NFR15 *(không bộ chạy test frontend)* — `src/commands/registry.ts:10-13` · `src/commands/README.md:20` · `src/i18n/README.md:101`
- Nợ `insert_segments` chủ là 2.2 — `_bmad-output/implementation-artifacts/deferred-work.md:2012-2024`
- Vết sẹo rò ký tự chèn lúc copy — `deferred-work.md:839-848` · nợ *"nghiệm thu chạy trên Blink, không phải WKWebView"* — `:145`
- Nợ bề mặt đọc phải tự khai token — `deferred-work.md:130-133`
- Bài học Epic 1 — `_bmad-output/implementation-artifacts/epic-1-retro-2026-08-11.md` §4, §5, §7.1, §8.1, §8.2
- Story trước — `_bmad-output/implementation-artifacts/2-1-tach-segment-cap-cau-va-co-ket-doan.md`

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, dev-story workflow) — 2026-08-12.

### Phán quyết Task 0 — ghi TRƯỚC dòng mã đầu tiên

**Quyết định #1 — ĐÃ CHỐT bởi Ice, không mở lại.** Thi hành đường **(b)** nguyên vẹn: bước di
trú 6 (`target_text`), bề mặt chỉ-đọc, cổng AC18, fixture bắt buộc, chuyển chủ nợ `isTypingZone`
sang 2.3. Cả bốn hệ quả đã cài và nghiệm thu — xem §Completion Notes.

**Quyết định #2 — cơ chế đo chiều cao vạch: XÁC NHẬN (a), `Range.getClientRects()`.**
Hai đường kia bị loại bởi *mệnh đề*, không bởi khẩu vị: (b) một `display: block` mỗi câu **là**
*"chia thành khối"* mà AC1 cấm bằng chữ; (c) một `::before` gắn vào chính câu là pseudo-element
**inline** và không trải theo chiều cao nhiều dòng. Ba điều kiện kèm theo đã cài: `ResizeObserver`
trên `.doc`, `document.fonts.ready`, và gộp mọi lượt tính vào **một** `requestAnimationFrame`
(`createRuleScheduler`).
**Số phản biện lại chính đề xuất — cơ chế RẺ, chỗ đắt nằm chỗ khác** *(bàn đo, 9.850 câu)*:
đo + vẽ **1** vạch *(ca thật hôm nay)* = **8,5 ms** Blink · **5,0 ms** WebKit; đo + vẽ **cả
9.850** vạch = **63,1 / 64,0 ms**. Trong khi **dựng DOM + bố cục** = **300,1 ms** Blink ·
**1.308,0 ms** WebKit. ⇒ Quyết định #2 không phải thứ vượt NFR2; **ảo hoá danh sách dài** mới là,
và nó là hàng Deferred Giai đoạn 3 → ghi và báo, **không** dựng trong story này (AC14 nói thẳng).
🔴 **Một điều kiện ĐƯỢC THÊM, có số đỡ:** chỉ đo những câu **thật sự có vạch để vẽ**
(`measureGutterRules(gutter, doc, wanted)`). Không phải một lượt tối ưu mù — `getClientRects()` là
một lượt đọc hình học **đồng bộ**, và hôm nay **nhiều nhất một** trong 9.850 câu có vạch. Chênh đo
được: 8,5 ms so với 63,1 ms.
**Vế thứ tư của AC14 — "đổi theme" — KHÔNG có listener riêng, và đó là một phép đo:** `applyTheme()`
ghi typography từ `tokens.typography`, một bảng **không** phân theo theme *(chỉ `tokens.colors`
phân theo theme)*, nên đổi theme không đổi được cỡ chữ/họ chữ/giãn dòng ⇒ không đổi hình học. Và
nếu một ngày nó có đổi, chiều cao `.doc` đổi theo ⇒ `ResizeObserver` bắt được. Kho cũng chưa có
công tắc đổi theme lúc chạy — `applyTheme` chỉ được gọi một lần ở `main.ts`.

**Quyết định #3 — `⏐` là PSEUDO-ELEMENT: XÁC NHẬN, và nay có số trên đúng engine đã sinh ra vết sẹo.**
Vết sẹo `WORD_JOINER` của Story 1.18b (`deferred-work.md:839-848`) là rò ký tự chèn lúc copy **trên
WKWebView**. Bàn đo chạy trên **WebKit** *(605.1.15 / Safari 26)* — không phải Blink — và
`doc.innerText` **không** chứa `⏐`, `getComputedStyle(el, '::after').content === '"⏐"'`. Tức nội
dung pseudo-element nằm ngoài cây văn bản: không copy được, không chọn được, và ở Story 2.3 sẽ
không gõ đè được. Cái giá đã ghi tại chỗ: pseudo-element không nhận `:hover` riêng
(`.sent:hover::after`), và nó **không** hiện trong một bàn đo chép DOM thay vì chép CSS.
Vế **tiêu điểm bàn phím** của AC5 đi qua `.sent[data-caret]::after`, cờ do tầng TS đặt — xem
Quyết định #6 mới ngay dưới.

**Quyết định #4 — cài CẢ NĂM giá trị ở một hàm phân giải: XÁC NHẬN (b).**
`SEGMENT_RULE_VALUES` + `resolveSegmentRule` sống ở `src/panels/editorSegments.ts`, một **module
thuần** (không `import` giá trị nào, không Vue, không DOM) — điều kiện để `check-commands.mjs`
Kiểm I `import()` và **chạy** hàm thật thay vì đọc nó bằng regex. Ba nhánh chưa có nguồn đọc từ hai
trường `false` cứng, mỗi trường ghi đích danh story chủ (`isConfirmed` → 2.5 · `isTmFilled` → Epic 7
· `retiredAt` → 2.8). Không dữ liệu giả nào trong sản phẩm; chúng chạy ở bàn đo và ở một ca hợp
đồng bơm bằng SQL.
🔴 **Thứ tự năm nhánh là một quyết định, và nó được ghi ra:** `ornament` ▸ `primary` ▸ `confirmed`
▸ `tm-rule` ▸ *không vạch*. Kiểm I chạy **năm ca** để khoá đúng thứ tự đó lại.
⚠️ **Một khe hở CÓ THẬT trong bảng năm giá trị, tìm ra lúc cài, ghi thành nợ có chủ:** một câu *đã
dịch bằng tay, chưa xác nhận, con trỏ ở chỗ khác* không ứng với giá trị nào trong năm. Hôm nay
không chạm tới được (`target_text` chỉ nhận giá trị qua đường gõ của 2.3). **Chủ: Story 2.5.**

**Quyết định #5 — `aura-allow-never-text` CÓ THAM SỐ LÀ TÊN TOKEN: XÁC NHẬN.**
Dựng trong `check-tokens.mjs` Kiểm C, dùng lại đúng luật khoảng cách một dòng của `exemptAt`.
Nghiệm thu **bốn** ca đỏ-rồi-xanh *(ba của AC10 cộng một ca chứng minh chính tham số)*:
① có miễn trừ ⇒ xanh · ② gỡ miễn trừ ⇒ đỏ đúng dòng đó · ③ đặt miễn trừ ở một khai báo `color:`
**khác** ⇒ vẫn đỏ · ④ đổi `--color-ornament` thành `--color-tm-rule` **giữ nguyên** dấu miễn trừ
`ornament` ⇒ **đỏ** — tức một miễn trừ cấp cho `ornament` **không** lặng lẽ cấp cho `tm-rule`,
đúng thứ tham số tồn tại để bảo đảm. Cổng nằm **trong** `check-tokens.mjs`, vốn đã ở CI và ở
`.githooks/pre-push`, nên bài học §4 của retro Epic 1 thoả sẵn; `npm run check:gates` xanh.
🔴 **Một số đo làm đổi hình dạng miễn trừ:** `exemptAt` so **dòng bắt đầu** của comment với dòng
khai báo và đòi khoảng cách ≤ 1, nên một khối chú thích bốn dòng đặt **ngay trên** khai báo vẫn
**ĐỎ** *(đo lúc chạy cổng 2026-08-12)*. ⇒ dấu miễn trừ viết **một dòng**, lý lẽ dài sống ở khối
doc-comment phía trên. Ghi tại chỗ trong `EditorPanel.vue` để người sau không phải chẩn đoán lại.

**Quyết định #6 — MỚI, không có trong khuôn story, và nó cần một lượt ký của Ice.**
AC5 đòi *"tiêu điểm bàn phím chạm tới một câu"* và AC3 đòi `primary` = *"đang sửa, con trỏ ở đây"*.
Nhưng phán quyết Quyết định #1 làm bề mặt **chỉ-đọc**, và một bề mặt không `contenteditable`
**không có caret** — nên cả hai mệnh đề mất nguồn dữ liệu, dù §Điều kiện khởi hành mục 2 xếp
`primary` vào nhóm *"có nguồn thật hôm nay"*.
**Đường đã chọn:** `data-caret` đọc từ **neo vùng chọn DOM** (`Selection.anchorNode`), cộng
`tabindex="0"` trên `.doc`. Đây **không** phải một cách nói tránh — nó là **đúng cơ chế** mà Story
1.18 đã dựng cho Panel Source để đóng `deferred-work.md:608` *(`Shift+Mũi tên` trên một bề mặt
không sửa được)*, và một cú bấm chuột cũng đặt một vùng chọn thu gọn. Nó cũng làm hợp đồng vùng
chọn đã đăng ký từ 1.18 **thật sự chạy được** trên bề mặt này — đúng lời hứa `epics.md:1762`.
🔴 **Cái giá phải nói thẳng, và nó chạm hợp đồng tiêu điểm:** một `tabindex="0"` **bên trong** một
`PanelFrame` mang `tabindex="-1"` làm phím `Tab` nay **dừng ở thân Panel Editor**. Ice đã ký đúng
đánh đổi này cho Panel Source ngày 2026-08-07; story này áp nó cho panel **thứ hai** mà **chưa có
một lượt ký riêng**. Ghi `deferred-work.md` với **chủ: Ice**, chỗ lật là một thuộc tính.

### Debug Log References

**Bàn đo thị giác** — `_bmad-output/implementation-artifacts/2-2-ban-do-editor.html`, chạy
headless qua `playwright-core` **cài ngoài kho** *(thư mục tạm của phiên; `package.json` và
`package-lock.json` KHÔNG bị chạm — AC15 giữ nguyên **0** phụ thuộc mới)*. Cùng khuôn phép đo
Playwright mà Story 1.16 đã dùng cho trần render kiểu song song.

Ảnh chụp: `2-2-ban-do/ban-do-webkit-light.png` · `ban-do-webkit-dark.png` ·
`ban-do-blink-light.png`.

| Nghiệm thu | Blink (HeadlessChrome 151) | WebKit (605.1.15 / Safari 26) |
| --- | --- | --- |
| `⏐` ở trạng thái nghỉ (AC4) | `opacity: 0`, `content: "⏐"` | `opacity: 0`, `content: "⏐"` |
| `⏐` khi rê chuột (AC5) | `0.55` | `0.55` |
| `⏐` khi `[data-caret]` (AC5, vế bàn phím) | `0.55` | `0.55` |
| `innerText` có rò `⏐` không? | **false** | **false** ← đóng vết sẹo 1.18b trên đúng engine |
| Vạch vẽ ra / 5 câu fixture | **4** *(câu chưa dịch KHÔNG có vạch)* | **4** |
| Vạch trên Chương chưa dịch (thật hôm nay) | **0** / 5 câu | **0** / 5 câu |
| Câu 2 chiếm 2 dòng ⇒ vạch cao | 46,25 px *(2 hình chữ nhật)* | 46,00 px *(2 hình chữ nhật)* |
| Câu 3 sau `<br>` kết đoạn | `top` = 64,50 px | `top` = 64,00 px |
| Cả hai theme | sáng ✅ ảnh chụp · tối ⚠️ **chưa chụp** | ✅ ảnh chụp cả hai |

🔴 **Hai đính chính của code review 2026-08-12 — bảng trên là số của lượt đo GỐC, và lượt đó
chạy trên fixture **năm** câu:**

1. **Ô *"Cả hai theme"* của cột Blink nói quá.** Thư mục `2-2-ban-do/` có **ba** tệp, không bốn:
   `ban-do-webkit-light.png` · `ban-do-webkit-dark.png` · `ban-do-blink-light.png`. §File List ghi
   đúng ba từ đầu. Task 7.2 *(nghiệm thu thị giác cả hai theme)* **vẫn được thoả** — trên WebKit,
   engine gần WKWebView của Tauri hơn hẳn Blink.
2. **Fixture nay có SÁU câu, không năm** *(`2-2-ban-do-editor.html`, câu id=6 thêm ở lượt review)*.
   Câu mới là ca *đã dịch bằng tay, chưa xác nhận, con trỏ ở chỗ khác* — nó **phơi** khe hở mà
   `editorSegments.ts:95-106` và §Quyết định #4 ghi ra, và mà `CARET_ID = 2` của bản gốc vô tình
   che mất *(caret rơi đúng câu duy nhất phơi được nó, nên nhánh `primary` thắng)*.
   ⚠️ **Mọi con số trong bảng trên vẫn đọc được**, và đó là lý do câu mới được thêm ở **cuối**:
   nó không dời hình học của năm câu trên, và nó rơi vào nhánh *không vạch* nên số vạch vẫn là
   **4**. Chỉ mẫu số đổi — `4 / 6 câu`.
   🔴 **Cái CHƯA làm được, ghi thẳng:** ba ảnh chụp **có trước** câu thứ sáu, nên chúng còn hiện
   năm câu. Lượt review không đo lại được — `playwright-core` sống ở thư mục tạm của phiên dev và
   đã mất, còn `--dump-dom` của Chrome headless chụp **trước** frame đầu tiên nên không bắt được
   `type="module"` + `requestAnimationFrame` của bàn đo *(thử ba lượt, cả `--headless=new` lẫn
   `--headless=old`, cả `file://` lẫn HTTP)*. **Chủ: một lượt chụp lại khi có bộ chạy —
   Story 2.4** *(story đó đã nhận số NFR2 của bàn đo này)*.

🔴 **Hình học khớp giữa hai engine, lệch dưới một pixel** — tức mối lo *"`getClientRects()` với
chữ dày dấu tiếng Việt lệch giữa Blink và WKWebView"* (`deferred-work.md:145`) **không** hiện ra
ở lượt đo này. Nhưng WebKit-của-Playwright **không phải** WKWebView-của-Tauri; xem §Completion
Notes mục *"cái CHƯA đo"*.

**Đo NFR2 (AC14)** — Chương lớn nhất có thật, **9.850** câu:

| | dựng DOM + bố cục | đo + vẽ **1** vạch *(ca THẬT)* | đo + vẽ **9.850** vạch *(ca trần)* |
| --- | --- | --- | --- |
| Blink | **300,1 ms** | 8,5 ms | 63,1 ms |
| WebKit | **1.308,0 ms** | 5,0 ms | 64,0 ms |

**Đo món nợ `insert_segments` (AC17 · Task 8)** — `cargo test --release`, macOS, 9.850 hàng,
ba lượt. Bàn đo là một tệp test tạm, **đã xoá** sau khi ghi số:

| lượt | `tx.execute` literal mỗi hàng | `prepare_cached` một lần | chênh |
| --- | --- | --- | --- |
| 1 | 105,51 ms | 44,76 ms | **60,75 ms** (57,6 %) |
| 2 | 106,90 ms | 49,75 ms | **57,15 ms** (53,5 %) |
| 3 | 112,47 ms | 48,28 ms | **64,19 ms** (57,1 %) |

*(Bản debug cho 223–283 ms so với 74–83 ms — cùng chiều, lớn hơn hai lần. Số chốt là số release,
vì đó là thứ người dùng chạy.)*

**Nghiệm thu đỏ-rồi-xanh của ba cổng mới:**

| Cổng | Ca | Kết quả |
| --- | --- | --- |
| Kiểm C `aura-allow-never-text` | có miễn trừ | `check:tokens` exit **0** |
| | gỡ dòng miễn trừ | exit **1**, đỏ đúng dòng đó |
| | miễn trừ đặt ở một `color:` **khác** | exit **1** |
| | miễn trừ `ornament` áp cho `--color-tm-rule` | exit **1** ← tham số làm việc của nó |
| Kiểm I *(năm giá trị vạch)* | thêm giá trị thứ sáu `proofread` | exit **1**, hai FAIL *(lệch bảng + thiếu khối CSS)* |
| Kiểm J *(chưa gõ được)* | tiêm `contenteditable="true"` | exit **1** |
| | gỡ ra | exit **0** |

### Completion Notes List

**Trạng thái:** 18 AC giao đủ. 11 cổng xanh · `npm run build` xanh · `cargo test` **310 đạt /
0 trượt / 5 bỏ qua** *(trước story: 304/0/5 — thêm đúng **6** ca mới; hai ca cũ đổi tên và đổi số
kỳ vọng theo lược đồ 6)*.

**Cái đã dựng:**

1. **Bước di trú 6 và cột `target_text`** — `ALTER TABLE`, **không** sửa `SEGMENT_DDL`: bước 5 đã
   chạy trên `project.db` thật, nên sửa hằng tại chỗ cho hai lược đồ khác nhau dưới cùng một số
   phiên bản, đúng vết sẹo số 4. `NOT NULL DEFAULT ''` — *"chưa dịch"* là **chuỗi rỗng**, không
   một giá trị vắng mặt (Task 1.4). Ba ca mới canh: tệp mới dừng ở 6 kèm `PRAGMA table_info` thật;
   một tệp **đang ở 5 có dữ liệu** di trú lên 6 **không mất hàng nào**; tệp mắc kẹt ở số 4 nay đi
   thẳng lên 6.
2. **Lệnh IPC `read_open_chapter_segments`** — hàm thuần trước, `wire` là vỏ. Đọc
   `ORDER BY ord` nên `idx_segment_chapter_ord` thành covering. Trả `chapter_id` kèm danh sách.
3. **Trang liền mạch** — một dòng văn liên tục, mỗi câu một `<span>` inline; chỗ ngắt đoạn là
   `<br>` đọc từ cờ `is_paragraph_end` **đã lưu** (AD-37).
4. **Máng lề + năm giá trị vạch** — một hàm phân giải duy nhất ở module thuần; đo hình học bằng
   `getClientRects()`, gộp vào một `rAF`.
5. **Ranh giới câu `⏐`** — pseudo-element, hai miễn trừ có tên.
6. **Ba cổng mới**: `aura-allow-never-text` *(có tham số)* trong `check-tokens.mjs`; Kiểm I và
   Kiểm J trong `check-commands.mjs`. Cả ba nằm trong tệp đã ở CI và ở `pre-push`.
7. **Vá `insert_segments`** theo số đo (AC17) — đóng `deferred-work.md:2012-2024`.

**Ba quyết định KHÔNG có trong khuôn story, ghi ra vì chúng đổi mã:**

- **Ba module frontend, không hai.** Story dự kiến `editorSegments.ts` + `editorGutter.ts`. Cài
  ra **ba**: `editorSegments.ts` *(module thuần — bảng năm giá trị + phép phân giải)*,
  `editorGutter.ts` *(module thuần DOM — hình học)*, `editorPanelState.ts` *(state Vue)*. Lý do
  không phải khẩu vị: quy ước của kho là state panel sống ở `<tênPanel>State.ts`, **và** trộn Vue
  vào `editorSegments.ts` sẽ giết Kiểm I — cổng đó `import()` **hàm thật** và chạy nó, đường duy
  nhất thay cho một bộ chạy test frontend mà NFR15 cấm thêm.
- **`resetEditorPanel()` nối vào `libraryImport.ts::finishSubmit`.** Không task nào đòi, nhưng
  Panel Editor mang **cùng** lớp cache module-level đã sinh ra lỗi *"đọc nội dung Tác phẩm A dưới
  nhãn Tác phẩm B"* mà code review 2026-08-06 bắt cho Panel Source. Bỏ nó ra là ship lại đúng lỗi
  đó ở panel thứ ba. Kèm `ensureSegmentsLoaded()` ở cùng chỗ, cùng lý do `<KeepAlive>` đã ghi.
- **Không thêm khoá `err.*` mới, dù Task 2.3 nói *"khoá chuỗi mới"*.** Lệnh nạp tái dùng
  `project.no_work_open` *(cùng câu, cùng nghĩa — và luật của kho là một khoá thứ hai cho cùng câu
  là hai chuỗi phải giữ khớp bằng kỷ luật)*. **Không** tái dùng `segment.chapter_not_found`: chuỗi
  của nó kết bằng *"chưa tách được câu nào"*, sai cho một lượt **nạp** — nhưng ca đó cũng không
  tồn tại, vì lệnh tự phân giải Chương đang mở thay vì nhận `chapter_id`. Hai khoá **mới** vẫn được
  thêm, chỉ là khoá **giao diện**: `panel.editor.no_segments` · `panel.editor.nothing_translated`.
  `npm run check:i18n` xanh.

**Số thật đo được 2026-08-12 (AC16), và bốn sàn đã nâng:**

| Sàn | Cũ | Số thật | Mới | Tỷ lệ |
| --- | --- | --- | --- | --- |
| `check-commands.mjs::TS_FLOOR` | 27 | **35** `.ts` | **28** | 80,0 % |
| `check-tokens.mjs::FILE_FLOOR` | 37 | **53** tệp | **43** | 81,1 % |
| `check-tokens.mjs::COMPONENT_FILE_FLOOR` | 35 | **50** component | **40** | 80,0 % |
| `check-layout.mjs::FILE_FLOOR` | 35 | **50** tệp `src/**` | **40** | 80,0 % |

Ba sàn cuối **không** tụt vì story này — chúng đã ở 69,8 % / 70,0 % / 70,0 % **trước** lượt này
(1.20 · 1.21 · 2.1 thêm tệp mà không ai nâng), tức đã ở đúng trạng thái *"canh không được gì"* mà
chính doc-comment của chúng cảnh báo. Sáu sàn còn lại **trong dải** và giữ nguyên: `VUE_FLOOR`
13/15 = 86,7 % *(cả hai cổng)* · `COMMAND_FLOOR` 29/34 = 85,3 % · `CLICK_FLOOR` 17/21 = 81,0 % ·
`DISPATCH_FLOOR` 23/28 = 82,1 % · `SELECTION_SURFACE_FLOOR` 7/7 · `RS_FLOOR` 36/43 = 83,7 %.

**AC15 — 0 phụ thuộc npm mới.** `package.json` vẫn đúng ba dependency runtime
(`@tauri-apps/api` · `dockview-vue` · `vue`) và `package-lock.json` không đổi một dòng. Bàn đo
chạy bằng `playwright-core` cài trong **thư mục tạm của phiên**, ngoài kho hoàn toàn — cùng khuôn
Story 1.16. **Không** kết luận rằng story này cần một thư viện editor: bề mặt là CSS/DOM tiêu
chuẩn, và hàng Deferred *"thư viện editor"* vẫn thuộc **Story 2.4**.

**Cái CHƯA đo, ghi thẳng:**

- 🔴 **WKWebView THẬT trong cửa sổ Tauri.** Bàn đo chạy WebKit **của Playwright** — khác phiên
  bản, khác lượt nhúng font, khác tầng phân phối sự kiện của OS. Đây vẫn là **lượt đầu tiên** của
  dự án có bằng chứng WebKit cho một bề mặt DOM *(mọi story trước đo trên Blink)*, và nó trả lời
  được hai câu hỏi nóng nhất *(hình học khớp; `⏐` không rò lúc copy)* — nhưng đừng viết
  *"tương đương"*. Nợ ghi kèm chủ.
- ⚠️ **Ba font nhúng của UX-DR4 vắng mặt trong bàn đo** *(chúng do `src/tokens/fonts.ts` nạp từ
  bundle)*, nên bàn đo rơi về `serif` hệ thống. Con số chiều cao vạch là số của **cơ chế**, không
  phải của **sản phẩm**.
- ⚠️ **Bàn đo CHÉP CSS/DOM chứ không mount component thật** — một lượt sửa template sau này có
  thể làm hai bên lệch nhau mà không cổng nào đỏ. Cùng lớp nợ `deferred-work.md:826`.
- ⚠️ **Ca Windows chưa đo** — Ice đã dời trọn phần Windows về cuối dự án (2026-08-12).

**Một phát hiện của bàn đo mà story KHÔNG vá, và lý do:** hai câu ngắn nằm **cùng một dòng** cho
hai vạch lề **cùng `top`, cùng `left`** ⇒ vạch vẽ sau che vạch vẽ trước *(fixture 5 câu vẽ 4 vạch,
chỉ nhìn thấy 2 vị trí — tái lập trên **cả hai** engine)*. Hôm nay **không chạm tới được** trong
sản phẩm: chỉ `primary` có nguồn dữ liệu và caret chỉ có một, nên nhiều nhất một vạch tồn tại cùng
lúc. `DESIGN.md:380` và `EXPERIENCE.md:105-113` **không** phân xử ca này, và máng 22px còn 12px
trống nên xếp cạnh nhau là khả thi — nhưng đó là một **quyết định thiết kế**, không phải một bản
vá kỹ thuật, và story này không có thẩm quyền ký nó. **Chủ: Story 2.5 + một lượt ký của Ice.**

### File List

**Mới (7):**

```
src/panels/editorSegments.ts
src/panels/editorGutter.ts
src/panels/editorPanelState.ts
_bmad-output/implementation-artifacts/2-2-ban-do-editor.html
_bmad-output/implementation-artifacts/2-2-ban-do/ban-do-webkit-light.png
_bmad-output/implementation-artifacts/2-2-ban-do/ban-do-webkit-dark.png
_bmad-output/implementation-artifacts/2-2-ban-do/ban-do-blink-light.png
```

**Sửa (16), kể cả chính story file này:**

```
src/panels/EditorPanel.vue
src/modes/libraryImport.ts
src/config/segment.ts
src/i18n/vi.json
src-tauri/src/core/store/schema.rs
src-tauri/src/core/store/mod.rs
src-tauri/src/commands/segment.rs
src-tauri/src/lib.rs
src-tauri/tests/segment_contract.rs
src-tauri/tests/pinned_contract.rs
scripts/check-tokens.mjs
scripts/check-commands.mjs
scripts/check-layout.mjs
_bmad-output/implementation-artifacts/deferred-work.md
_bmad-output/implementation-artifacts/sprint-status.yaml
_bmad-output/implementation-artifacts/2-2-panel-editor-lien-mach.md
```

⚠️ **Không** đụng: `src/layout/**` · `PanelFrame.vue` · `SourcePanel.vue` · `core/segment/split.rs`
· `epics.md` · `DESIGN.md` · `package.json` · `package-lock.json`.

### Change Log

| Ngày | Mốc gốc | Ghi chú |
| --- | --- | --- |
| 2026-08-12 | `c86c2fb` | Story dựng, cây làm việc sạch 0 dòng |
| 2026-08-12 | `c86c2fb` | **Ice chốt Quyết định #1 ngay lúc dựng story: đường (b)** — thêm `target_text` + bước di trú 6, bề mặt **chỉ-đọc**, gõ để lại cho 2.3. Sinh ra **AC18** *(cổng "chưa gõ được")*, **Task 1.4** *(mặc định chuỗi rỗng, không `NULL`)*, **Task 6b** *(cổng + nghiệm thu đỏ-rồi-xanh)*, **Task 10.1** *(chuyển chủ nợ `isTypingZone` sang 2.3)*, và làm fixture của Task 7 thành **bắt buộc**. Bốn quyết định #2 → #5 vẫn mở cho dev |
| 2026-08-12 | `c86c2fb` | **Task 1–2 — tầng dữ liệu.** Bước di trú **6** (`ALTER TABLE segment ADD COLUMN target_text TEXT NOT NULL DEFAULT ''`) · lệnh IPC `read_open_chapter_segments` *(hàm thuần + vỏ `wire`, đọc `ORDER BY ord`)* · adapter TS. `PROJECT_MIGRATIONS` nay **năm** bước, đích **6**; `pinned_contract.rs` theo lược đồ mới. **6** ca test mới |
| 2026-08-12 | `c86c2fb` | **Task 3–6b — tầng hiển thị.** Ba module frontend mới *(hai module **thuần** để cổng `import()` chạy được hàm thật)* · trang liền mạch + máng lề + `⏐` pseudo-element · `data-caret` từ neo vùng chọn DOM. Bốn quyết định #2 → #5 chốt **XÁC NHẬN**, cộng một **Quyết định #6 mới** cần Ice ký *(`tabindex="0"` trên thân panel thứ hai)* |
| 2026-08-12 | `c86c2fb` | **Ba cổng mới, cả ba nghiệm thu đỏ-rồi-xanh.** `aura-allow-never-text` **có tham số là tên token** *(Quyết định #5 — bốn ca, kể cả ca chứng minh miễn trừ `ornament` KHÔNG cấp cho `tm-rule`)* · Kiểm I *(đúng năm giá trị vạch, đối chiếu hai chiều với CSS)* · Kiểm J *(bề mặt chưa gõ được — AC18)* |
| 2026-08-12 | `c86c2fb` | **Task 8 — nợ `insert_segments` ĐO rồi mới vá, và ĐÓNG.** 9.850 hàng, `--release`, ba lượt: 105–112 ms → 45–50 ms, chênh **57–64 ms** (53,5–57,6 %). Vá bằng `prepare_cached` vì khoản tiết kiệm **một mình nó** đã trên trần 50 ms/frame của NFR2, trên writer duy nhất nối tiếp của AD-11 |
| 2026-08-12 | `c86c2fb` | **Task 7 — bàn đo + fixture, chạy trên CẢ HAI engine.** Lần đầu dự án có bằng chứng **WebKit** cho một bề mặt DOM: hình học khớp Blink dưới một pixel, và `innerText` **không rò** `⏐` — tức Quyết định #3 đóng vết sẹo `WORD_JOINER` của 1.18b trên đúng engine đã sinh ra nó. 🔴 **NFR2 VƯỢT TRẦN ở khâu dựng DOM**: 300 ms (Blink) · **1.308 ms** (WebKit) cho 9.850 câu — cơ chế đo chỉ tốn 5–9 ms. Đó là số của **Story 2.4**, báo chứ không tự dựng ảo hoá |
| 2026-08-12 | `c86c2fb` | **Task 9–10.** Bốn `*_FLOOR` nâng theo số thật *(ba trong bốn đã tụt xuống ~70 % **trước** story này)* · **0** phụ thuộc npm mới · 11 cổng xanh · `cargo test` 310/0/5. Mười một món nợ ghi `deferred-work.md` **kèm chủ**, hai món **đóng** *(`insert_segments`; nửa Editor của nợ token)*, và nợ `isTypingZone` chuyển chủ từ *"Epic 2"* sang **Story 2.3** đích danh |
| 2026-08-12 | `c86c2fb` | ⚠️ **BA thứ cần Ice phán, ghi ra thay vì tự quyết:** ① Quyết định #6 — `tabindex="0"` trong thân Panel Editor làm `Tab` dừng ở đó *(tiền lệ 1.18 có, lượt ký riêng thì chưa)* · ② **hai vạch chồng nhau** khi hai câu cùng một dòng — bàn đo tái lập trên cả hai engine, hôm nay chưa chạm tới được, thành thật ở Story 2.5 · ③ nhãn `Covers: UX-DR13` sai ở `epics.md:2036`. Story chuyển **`review`** |

### Review Findings

Code review 2026-08-12 — ba lớp song song *(Blind Hunter · Edge Case Hunter · Acceptance Auditor)*,
cộng một lượt nghiệm thu cơ học chạy lại từ đầu trên cây làm việc: **9 cổng `check:*` xanh** ·
`npm run build` xanh · `cargo test --locked` **310 đạt / 0 trượt / 5 bỏ qua** — khớp đúng con số
§Completion Notes khai. Bốn sàn `*_FLOOR` đối chiếu quần thể thật: khớp. Vùng cấm chạm: **không tệp
nào** trong `src/layout/**` · `PanelFrame.vue` · `SourcePanel.vue` · `core/segment/split.rs` ·
`epics.md` · `DESIGN.md` · `package.json` · `package-lock.json` xuất hiện trong diff. §File List
khớp diff thật (7 mới + 16 sửa).

- [x] [Review][Decision] ✅ **ICE KÝ 2026-08-12 — giữ nguyên `tabindex="0"`.** Tiền lệ Panel Source (2026-08-07) mở rộng sang panel **thứ hai**; cái giá *(`Tab` dừng ở thân Panel Editor)* được chấp nhận có chủ ý, vì gỡ nó làm vế **bàn phím** của AC5 mất nguồn và AC5 không giao đủ. Chữ ký phủ **cơ chế hôm nay**, **không** phủ Story 2.3 — story đó vẫn phải xét lại đường `Selection.anchorNode` khi caret thật xuất hiện. Nợ ở `deferred-work.md` đã chuyển từ *chờ ký* sang *đã ký*. Nguyên văn món đã trình:
      **Quyết định #6 — `tabindex="0"` trong thân Panel Editor** — Chính §Dev Agent Record ghi *"nó cần một lượt ký của Ice"* và *"story này áp nó cho panel **thứ hai** mà **chưa có một lượt ký riêng**"*, nhưng mã đã cài và chạy: `EditorPanel.vue:279`. Hệ quả có thật và đo được: phím `Tab` nay **dừng ở thân Panel Editor**, tức vòng tiêu điểm của toàn Workspace dài thêm một chặng. Đây **không** phải một lỗi cài đặt — cơ chế đúng, tiền lệ Story 1.18 có thật, và nó là thứ làm vế **bàn phím** của AC5 tồn tại. Nó là một **quyết định về hợp đồng tiêu điểm** mà chỉ Ice ký được. Story đang ở `review` và chưa commit ⇒ đây đúng là lúc ký.

- [x] [Review][Patch] **Panel khẳng định *"Chưa có Chương nào để dịch"* trong lúc lượt nạp đang bay** [`src/panels/EditorPanel.vue:96-98` · `src/panels/editorPanelState.ts:63`] — `showFrameStatus` không xét `editorPending`, nên trong khoảng chờ IPC nó đọc ra `true` và `PanelFrame` hiện `panel.editor.status`. Đó là **đúng lớp lỗi** mà doc-comment của `editorHasLoaded()` (`editorPanelState.ts:67-75`) tự đặt ra để chặn — *"màn hình khẳng định dứt khoát một điều nó chưa biết"* — chỉ là nó lọt qua bằng một chuỗi khác. `editorPending` được export **đúng cho vai này** và **không một chỗ nào trong kho tiêu thụ nó**.

- [x] [Review][Patch] **Doc-comment AC18 hứa cổng canh `@keydown`, cổng không canh; và cổng canh `@cut` mà chú thích không nhắc** [`scripts/check-commands.mjs:2084` · `src/panels/EditorPanel.vue:16-19`] — `TYPING_BANS` là `contenteditable` · `<textarea>` · `<input>` · `v-model` · `@(input|beforeinput|paste|cut)`. Chú thích đầu `EditorPanel.vue` khai *"`@input`, `@beforeinput`, `@paste`, `@keydown`"* và khẳng định *"một cổng tĩnh… cưỡng chế cả bốn mệnh đề đó"*. Hai bên lệch **cả hai chiều**. Rủi ro chạy thật hôm nay ≈ 0 *(không `contenteditable`/`<textarea>`/`<input>` thì một `@keydown` không gõ được vào đâu)*, nhưng một cấm-lệnh **cấm rộng hơn thứ nó cưỡng chế được** là thứ story sau đọc thành đã-được-canh. ⚠️ Cấm thẳng `@keydown` là **sai đường** — Story 2.10 *(điều hướng segment)* cần đúng handler đó; đường đúng là sửa chú thích cho khớp cổng.

- [x] [Review][Patch] **Hàm thoát regex của Kiểm I hỏng — phép thoát là một lượt no-op** [`scripts/check-commands.mjs:2024`] — `/[.*+?^${}()|[\\]\\\\]/g`: lớp ký tự **đóng sớm** ở `]` sau `\\`, nên regex thật là *"một ký tự trong lớp, rồi hai dấu `\`, rồi `]`"*; đo được: `'a.b*c'.replace(re, '\\$&')` trả `a.b*c` **nguyên vẹn**. Chuỗi thay thế `'\\\\$&'` cũng chèn **hai** dấu `\` chứ không một. Hôm nay vô hại *(năm giá trị vạch không chứa ký tự đặc biệt nào, và AC12 khoá con số năm)*, nhưng khuôn đúng đã có sẵn ngay trong cùng tệp: `scripts/check-commands.mjs:101`.

- [x] [Review][Patch] **Kiểm I quét văn bản THÔ, trong khi Kiểm J cùng tệp cùng lượt cố ý quét bản đã che** [`scripts/check-commands.mjs:2006,2026`] — Kiểm J tự ghi lý do bằng một phép đo: bản đầu quét `p.text` và **đỏ ngay trên chính tệp nó canh**, vì doc-comment gọi tên đủ thứ bị cấm. Kiểm I ăn đúng rủi ro đó mà không hưởng bài học: một chú thích sau này viết ví dụ `.rule-x { … }` sẽ làm chiều **ngược** của phép đối chiếu đỏ oan. Hôm nay xanh *(bốn khối `.rule-*` đều là CSS thật, không chú thích nào chứa chuỗi đó)*. `maskStyle` chỉ xoá `/* */` nên CSS sống nguyên ⇒ đổi sang `.masked` là an toàn và chặt hơn.

- [x] [Review][Patch] **Bảng §Debug Log khai `✅ ảnh chụp` cho Blink ở cả hai theme, nhưng ảnh Blink tối không tồn tại** [`2-2-panel-editor-lien-mach.md:522`] — thư mục `2-2-ban-do/` có **ba** tệp: `ban-do-webkit-light.png` · `ban-do-webkit-dark.png` · `ban-do-blink-light.png`. §File List ghi đúng ba; bảng ghi bốn. Task 7.2 *(nghiệm thu thị giác cả hai theme)* **được thoả** trên WebKit — engine gần WKWebView của Tauri hơn — nên đây là một ô bảng nói quá, không phải một lượt nghiệm thu thiếu.

- [x] [Review][Patch] **`SEGMENT_ID_ATTR` không được dùng ở nơi ĐẶT thuộc tính** [`src/panels/editorGutter.ts:41-44` · `EditorPanel.vue:285`] — doc-comment của hằng khai *"một hằng, không một chuỗi viết thẳng ở ba chỗ: `EditorPanel.vue` đặt nó, hàm dưới đây đọc nó, và bàn đo chép nó"*. Thật ra hằng chỉ dùng ở hai chỗ **đọc**; chỗ **ghi** là `:data-segment-id="s.id"` viết thẳng, và `check-commands.mjs:2102` là một bản chép thứ tư cũng viết thẳng. ⚠️ Đổi template sang `:[SEGMENT_ID_ATTR]` sẽ **làm đỏ sàn nội dung của Kiểm J** *(nó tìm chuỗi `data-segment-id` trong bản đã che)* — chuỗi viết thẳng ở cổng là **có lý**, vì cổng không được phụ thuộc tệp nó đang kiểm. Đường rẻ và đúng: sửa doc-comment cho khớp thực tế.

- [x] [Review][Patch] **Fixture bàn đo đặt caret đúng vào câu duy nhất phơi được khe hở *"đã dịch, chưa xác nhận"*** [`2-2-ban-do-editor.html:218`] — `CARET_ID = 2`, và câu id=2 là câu duy nhất trong fixture có `target_text` thật với `isConfirmed=false`/`isTmFilled=false`/`retired_at=null`. Nhánh `primary` thắng ⇒ nó vẽ ra một vạch, nên khe hở mà §Quyết định #4 và `editorSegments.ts:95-106` ghi ra *(câu ấy rơi về "không vạch", nhìn y hệt câu chưa dịch)* **không nhìn thấy được** trên chính bàn đo dựng để nghiệm thu năm giá trị. ⚠️ Dời caret sang câu 4 **không** phải lời giải: câu 4 rỗng, một `<span>` rỗng không có hình học nên vạch `primary` mất chỗ đo. Đường đúng là thêm một câu fixture **thứ sáu** *(có bản dịch, không cờ nào, không caret)*. Khe hở tự nó đã có chủ là Story 2.5 và **không chạm tới được trong sản phẩm hôm nay** — đây là món nợ về **bằng chứng**, không về hành vi.

- [x] [Review][Defer] **`:data-caret` buộc dựng lại toàn bộ danh sách mỗi lượt `selectionchange`** [`src/panels/EditorPanel.vue:286`] — deferred, đã có chủ. `editorCaretSegmentId` là một ref phản ứng đọc trong hàm render, nên mỗi lượt đổi caret *(bắn liên tục lúc kéo chọn)* chạy lại `v-for` trên tới **9.850** `<span>`. ⚠️ Chuyển sang `Map` **không** vá được — Vue vẫn duyệt cả danh sách; lời giải thật là **ảo hoá danh sách dài**, hàng Deferred Giai đoạn 3 (`ARCHITECTURE-SPINE.md:888`). Story này **đã đo và đã báo** đúng trần đó *(dựng DOM 300 ms Blink · 1.308 ms WebKit cho 9.850 câu)* và giao số cho **Story 2.4**. Cùng chủ, cùng số.

#### Lượt vá 2026-08-12 — cả bảy món, và nghiệm thu sau khi vá

Bảy bản vá đã áp. **Không món nào chạm hành vi sản phẩm ngoài món ①**; sáu món còn lại làm mã nói
đúng thứ mã làm.

① `showFrameStatus` nay xét `editorPending` — khoảng chờ IPC hiện **trống** thay vì khẳng định
*"Chưa có Chương nào để dịch."* ② khối AC18 đầu `EditorPanel.vue` khai **đúng năm** mệnh đề cổng
canh, và nói thẳng vì sao `@keydown` **cố ý không** bị cấm *(Story 2.10 cần nó; luật "không sửa văn
bản trong `@keydown`" do người giữ)*. ③ hàm thoát regex của Kiểm I chép lại khuôn đã chạy đúng ở
`:101`. ④ Kiểm I đọc bản **đã che**, cùng bài học Kiểm J. ⑤ ô *"Cả hai theme"* của cột Blink đính
chính. ⑥ doc-comment `SEGMENT_ID_ATTR` khai đủ **bốn** chỗ và vì sao hai trong bốn cố ý viết thẳng.
⑦ fixture bàn đo thêm **câu thứ sáu** phơi khe hở *"đã dịch, chưa xác nhận"*.

**Nghiệm thu sau khi vá — chạy lại từ đầu, không suy từ lượt trước:**

| Phép đo | Kết quả |
| --- | --- |
| 9 cổng `check:*` | **9/9 xanh** |
| `npm run build` | xanh |
| `cargo test --locked` | **310 đạt / 0 trượt / 5 bỏ qua** *(không đổi — bảy món không chạm Rust)* |
| Kiểm I · thêm giá trị vạch **thứ sáu** | exit **1**, hai FAIL |
| Kiểm I · đổi tên khối `.rule-tm-rule` *(chiều đối chiếu CSS)* | exit **1** |
| Kiểm J · tiêm `contenteditable="true"` | exit **1** |
| Phục hồi cả ba ca | **xanh lại** |

🔴 Ba ca đỏ-rồi-xanh trên là **bắt buộc**, không trang trí: lượt vá này sửa **chính hai cổng**
Kiểm I và Kiểm J, và một cổng đã sửa mà chỉ chứng minh *"còn xanh"* thì chưa chứng minh gì.

**Bác bỏ bằng phép đo (1):** *"`.gutter { padding-top: 4px }` làm mọi vạch lệch xuống 4px vì containing block là padding-box"*. Đo trên Blink headless với đúng bộ CSS của `EditorPanel.vue`: `markTop − gutterBorderBoxTop = **0**`. Mép trên *padding box* trùng mép trên *border box* khi không có viền — `padding-top` nằm **trong** padding box, không dời gốc của nó. `origin = gutter.getBoundingClientRect().top` là đúng.
