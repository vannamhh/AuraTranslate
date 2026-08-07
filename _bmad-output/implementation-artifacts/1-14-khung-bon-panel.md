---
baseline_commit: 7e38de8625c76dfb218fc6b613314123c69e455e
---

# Story 1.14: Khung bốn panel

Status: done

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-06 | **Code review xong — Status → `done`.** Ba lớp song song (Blind Hunter · Edge Case Hunter · Acceptance Auditor) trên diff đã lọc (loại rác chưa commit của Story 1.13). 1 decision *(AC10 — Ice chấp nhận độ lệch `deviations`-map thay vì literal)*, 7 patch *(đã vá và verify: cổng `check:layout` gắn vào CI · `dockController` gỡ ở `onDeactivated` chặn lỗ ghi đè bố cục ngoài Workspace · token `ui-md-strong` cho tab chế độ · `try/catch` quanh `applyPreset()` · ba export chết gỡ · comment cho sentinel tự tham chiếu · huỷ `requestAnimationFrame` chồng lấp)*, 3 defer *(ghi vào `deferred-work.md`)*, 4 dismiss. Xem §Review Findings. |
| 2026-08-06 | **Triển khai xong — Status → `review`.** 13 task, 12 AC. 11 tệp mới · 17 tệp sửa. Cổng thứ năm `npm run check:layout`. **Sáu cổng xanh**, `cargo test --locked` **165 passed**. Nghiệm thu: **35/35** ca thị giác/hành vi trên Blink + **4 bước** vòng lưu–khôi phục trong **app Tauri thật (WKWebView + IPC)** + **44 ca** đỏ-rồi-xanh cho ba phép kiểm mới. **Bốn lỗi thật** do lượt nghiệm thu tay bắt được đã sửa *(dock cao 0px · dockview tự dán theme `abyss` · `rememberSpot` chọn sai neo · focus rơi về `body` sau ẩn/hiện)*. Mười hai mục `deferred-work.md` đóng kèm bằng chứng; **12 mục mới** mở cho những gì cố ý không làm. |
| 2026-08-06 | Tạo story. Baseline `7e38de8`, ⚠️ **cây làm việc KHÔNG sạch** — code + story file của Story 1.13 chưa commit (xem §Bối cảnh git). Phân tích toàn bộ: `epics.md` §Story 1.14 · `ARCHITECTURE-SPINE.md` AD-1/AD-24/AD-34 · `DESIGN.md` · `EXPERIENCE.md` · UX-DR13/14/15/17 · `mockups/key-screen-workspace.html` · `mockups/narrow-layout.html` · **15 mục `deferred-work.md` giao đích danh cho story này** · toàn bộ `src/**` hiện có · API thật của `dockview-vue@7.0.4` đọc từ `node_modules`. |

**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story key:** `1-14-khung-bon-panel`
**Covers:** FR16 · FR17 · FR18 · *(nửa cơ chế của)* UX-DR15
**Governed by:** **AD-24** *(chủ — một cửa sổ OS, ba chế độ)* · **AD-34** *(chủ — CommandRegistry + điểm vào focus + màu từ token)* · **AD-1** *(bố cục panel là state UI của frontend)* · AD-18 *(`LayoutPreset` = `GlobalOnly`)* · AD-11 *(mọi ghi qua `store::Writer`)* · AD-21/NFR16 *(không chuỗi hiển thị trong `.vue`)* · NFR14 · NFR15 · NFR17
**Ngày tạo:** 2026-08-06

---

## 🔴 ĐỌC TRƯỚC TIÊN — HAI VIỆC STORY NÀY KHÔNG LÀM, VÀ NÓI THẲNG THAY VÌ ĐÁNH DẤU ĐẠT

### ① Ngưỡng màn hình hẹp của UX-DR15 KHÔNG đóng ở đây

`epics.md:1617` viết nguyên văn cho story này:

> **And** **ngưỡng kích thước cụ thể** đóng ở Story 4.12, không đóng ở đây
> **And** không được cài cơ chế ẩn theo cách khiến Story 4.12 phải mổ lại bố cục để nhét thứ tự này vào

⇒ Story này giao **CƠ CHẾ** *(một API ẩn/hiện panel theo thứ tự đã khai, kiểm được bằng máy)*, **không** giao bốn ngưỡng `1100×820 / <820 cao / <1100 rộng hoặc <700 cao / <860 rộng`. **Đừng viết một `matchMedia` nào trong story này.** Thứ tự hy sinh thì **có** — nó là *quyết định*, không phải *số hiệu chỉnh được* (`epics.md:1614`).

⚠️ Ghi kèm một sự thật đã có mà 4.12 sẽ đụng: `tauri.conf.json:19-20` khai `minWidth: 960` · `minHeight: 600`. Ngưỡng *"< 860 rộng ⇒ báo không hỗ trợ"* của UX-DR15 vì vậy **không đến được bằng cách kéo cửa sổ** trên cấu hình hôm nay. **Story này không sửa `tauri.conf.json`** — ghi ra để 4.12 quyết một lần.

### ② Vế DOM của AD-34 vẫn KHÔNG có test tự động — và story này không dựng bộ chạy test frontend

Dự án **không có** test runner frontend, và thêm một cái (`vitest`) là thêm một phụ thuộc phải rà GPLv3 rồi vào bảng Stack **trước** (NFR15) — quyết định của Ice, không phải hệ quả phụ của story này. Cùng tiền lệ Story 1.4 · 1.5 · 1.6.

⇒ Mọi mệnh đề *"nhìn thấy trên màn hình"* của story này *(khe 2px theme tối · vạch tiêu điểm · bốn panel lấp đầy chỗ trống)* nghiệm thu bằng **một lượt chạy tay có bảng và có ảnh chụp**, ghi vào §Debug Log References. **Đừng đánh dấu đạt bằng suy luận** — dùng đúng tiền lệ `unmeasured` của Story 1.3.

---

## Story

As a **người dịch**,
I want **bốn panel trong một cửa sổ duy nhất và sắp xếp được theo cách tôi làm việc**,
So that **tôi không phải mở bốn năm cửa sổ rời như trước**.

---

## Ranh giới phạm vi — ĐỌC TRƯỚC KHI GÕ DÒNG ĐẦU TIÊN

Story này là **khung bố cục**. Nó **không** phải một lượt dựng nội dung panel, **không** phải một lượt chạm đường tra cứu, **không** phải một lượt dựng màn hình gán phím.

| Thứ | Trong phạm vi? | Chủ sở hữu thật |
|---|---|---|
| `src/layout/` — dockview: dock · undock · gộp tab · đổi kích thước · preset | ✅ **CÓ** | story này |
| Bốn component panel **rỗng có tiêu đề + chuỗi trạng thái** (`Source` · `Lookup` · `AiTranslation` · `Editor`) | ✅ **CÓ** | story này |
| Ẩn/hiện panel hoàn toàn + **thứ tự hy sinh** khai được bằng máy | ✅ **CÓ** | story này |
| Lưu và khôi phục bố cục qua các phiên; preset đặt tên | ✅ **CÓ** | story này |
| Phím cho preset bố cục **và** cho `focus.next_panel` | ✅ **CÓ** | story này |
| Theme dockview map `--dv-*` → token của dự án | ✅ **CÓ** | story này |
| Đóng bảy món nợ `deferred-work.md` giao đích danh 1.14 *(§Bàn giao)* | ✅ **CÓ** | story này |
| Nội dung Panel Source · tab Hán Việt | ❌ | **Story 1.16** |
| Nội dung Panel Lookup — bản ghi có cấu trúc | ❌ | **Story 1.17** |
| Nội dung Panel AI · Editor thật | ❌ | **Epic 4 · Epic 2** |
| **Bốn ngưỡng** màn hình hẹp + ngăn kéo + rút Tra cứu về thanh trạng thái | ❌ | **Story 4.12** |
| Preset **Review Mode** *(`Bản dịch của tôi` cạnh `Bản Reviewer đã sửa`)* | ❌ | **Story 8.11** |
| Màn hình gán phím / giải quyết xung đột hợp âm | ❌ | **Story 1.21** |
| Sync scrolling · Auto-Lookup | ❌ | **Story 2.12 · 1.18** |
| Sửa `tauri.conf.json` · `Cargo.toml` · `[profile.release]` | ❌ | **KHÔNG ĐỤNG** *(`deferred-work.md` [D4], Ice chốt lần thứ tư)* |
| Sửa `epics.md` · `prd.md` · `DESIGN.md` · `EXPERIENCE.md` · `mockups/**` | ❌ | **Ice**, một lượt riêng *(tiền lệ quyết định #3 Story 1.3)* |

---

## 🔴 SÁU QUYẾT ĐỊNH PHẢI CHỐT — MỖI CÁI CÓ MẶC ĐỊNH, KHÔNG CÁI NÀO CHẶN

Chốt **cả sáu trước dòng mã đầu tiên** và ghi phán quyết vào §Completion Notes, đúng khuôn Task 0 của Story 1.13.

### Quyết định #1 — Phím nào cho **preset bố cục**? *(nợ `deferred-work.md:136`)*

`mockups/key-screen-workspace.html:89` vẽ `⌘1` `⌘2` cho preset bố cục. Xung đột đó **đã phân xử ở Story 1.6: chế độ thắng** (UX-DR34 · `EXPERIENCE.md:49` · AC3 Story 1.6). Mockup **chưa sửa** và dev **không sửa nó**. Việc còn lại là của story này: **chọn phím khác**.

| | Hợp âm | Đánh đổi |
|---|---|---|
| **A ✅ mặc định** | `Mod+Alt+1` *(2×2)* · `Mod+Alt+2` *(4 cột)* | Giữ nguyên "số thứ tự preset" mà mockup dạy, chỉ thêm một phím bổ trợ. `Mod+Alt+3` để trống cho **Review Mode** ở 8.11 — đúng thứ tự mockup. Khớp bằng `event.code` (`Digit1`) nên `⌥1` sinh ký tự lạ trên macOS không thành vấn đề. |
| B | `Mod+Shift+1` · `Mod+Shift+2` | `⌘⇧…` là không gian của UX-DR35 (`⌘⇧↵`); nhét preset vào đó là trộn hai họ thao tác. |
| C | `Mod+BracketLeft` · `Mod+BracketRight` *(xoay vòng)* | Xoay vòng không gọi thẳng được một preset cụ thể, và `[` `]` đã có chủ ở màn xem trước nhập (`EXPERIENCE.md:145`, dù không kèm `Mod`). |

⚠️ `Alt` đã được `keys.ts::parseChord` hỗ trợ; `Digit1`/`Digit2` phân giải qua `keyToCode`. Không thêm tên phím mới vào `NAMED_CODES`.

### Quyết định #2 — Phím nào cho `focus.next_panel`? *(nợ `deferred-work.md:134` + `:161`)*

Hôm nay **không có đường bàn phím nào vào panel** — §Quyết định #5 của Story 1.6 cố ý để trống vì *"bốn panel chưa tồn tại, nên vòng xoay chưa biết gồm những gì"*. Nay chúng tồn tại.

| | Hợp âm | Đánh đổi |
|---|---|---|
| **A ✅ mặc định** | `focus.next_panel` = `Mod+Alt+ArrowRight` · thêm `focus.prev_panel` = `Mod+Alt+ArrowLeft` | Cùng họ phím bổ trợ với preset (#1) nên người dùng học một lần. Không đụng `Tab` (thứ tự tiêu điểm của trình duyệt), không đụng `⌥←` `⌥→` trần *(Chương trước/sau — `EXPERIENCE.md:148`, Story 2.11)*. |
| B | `F6` / `Shift+F6` | Quy ước Windows cho xoay pane, nhưng `F6` chưa có trong `NAMED_CODES` và trên macOS phím F thường bị hệ thống chiếm. |
| C | Chỉ `next`, không `prev` | Rẻ hơn một command, nhưng vòng bốn panel mà chỉ đi một chiều là ba lần bấm để lùi một bước. |

🔴 **Ràng buộc kèm theo, đừng làm hỏng:** AC6 của Story 1.6 nghiệm thu bằng việc `unbound()` trả về **ít nhất một** phần tử thật. Gán phím cho `focus.next_panel` **lấy mất** phần tử duy nhất đang có ⇒ story này phải để **bốn command `layout.toggle_*` không gán phím** *(xem #3)*, và ghi rõ điều đó trong §Completion Notes. Không đăng ký một command rỗng cho đủ số.

### Quyết định #3 — Ẩn panel: **gỡ khỏi dockview** hay một cờ hiển thị?

🔴 **Sự thật kỹ thuật, không phải khẩu vị:** `dockview-core@7.0.4` khai
`interface DockviewPanelApi extends Omit<GridviewPanelApi, 'setVisible' | 'onDidConstraintsChange'>`
*(`node_modules/dockview-core/dist/cjs/api/dockviewPanelApi.d.ts:21`)* — tức **`setVisible` bị gỡ khỏi API của panel**. Không có đường "ẩn tại chỗ".

| | Cách | Đánh đổi |
|---|---|---|
| **A ✅ mặc định** | `api.removePanel(panel)` để ẩn; `api.addPanel({...})` để hiện lại, kèm một **sổ vị trí đã nhớ** trong `src/layout/` | Đúng thứ FR17 đòi (*"ẩn hoàn toàn, các panel còn lại lấp đầy chỗ trống"*) và dockview tự lấp chỗ — không phải tự viết. Giá: phải nhớ vị trí để hiện lại đúng chỗ. |
| B | Giữ panel, đặt `width: 0` | không Panel vẫn trong DOM, vẫn trong vòng focus, vẫn nhận `Tab` — *"ẩn hoàn toàn"* thành lời hứa. |
| C | `api.setEdgeGroupVisible(...)` | Chỉ áp cho **edge group**, không áp cho panel thường. Sai công cụ. |

Bốn command: `layout.toggle_source` · `layout.toggle_lookup` · `layout.toggle_ai_translation` · `layout.toggle_editor` — **đăng ký, có handler thật, KHÔNG gán phím** *(xem ràng buộc ở #2)*. ⚠️ **Lỗ NFR17 mở ra ở đây phải được ghi ra**: hôm nay ẩn/hiện panel chỉ tới được bằng chuột. Cùng hình dạng với lỗ mà Story 1.6 đã ghi và đóng ở story này; cái này đóng ở **Story 1.21**.

### Quyết định #4 — Thanh tiêu đề panel: header **của ta** hay tab bar **của dockview**?

`UX-DR17` + `mockups/key-screen-workspace.html:31-34` vẽ mỗi panel có một thanh 34px, tiêu đề trái, tab phải. dockview cũng vẽ tab bar riêng cho mỗi group. **Hai thanh chồng nhau là hỏng thị giác ngay lượt dựng đầu.**

| | Cách | Đánh đổi |
|---|---|---|
| **A ✅ mặc định** | **Tab bar của dockview LÀ thanh tiêu đề panel** — dựng một `tabComponents` riêng (`PanelTab.vue`), cao `var(--space-head-height)`, tiêu đề `ui-md`/`on-surface-variant`, group có tiêu điểm ⇒ `primary` + nét đậm. `PanelFrame.vue` **bỏ `<header>` của nó**. | Một thanh, đúng mockup; và *"gộp thành tab"* (FR17) hiện ra ngay trên chính thanh đó — không phải một cơ chế thứ hai. Giá: sửa `PanelFrame.vue` *(đã lường trước — doc-comment của nó nói thẳng vỏ này là của 1.6 và 1.14 sẽ mổ)*. |
| B | Ẩn tab bar của dockview, giữ header của `PanelFrame` | không Ẩn tab bar là ẩn luôn affordance kéo-thả và gộp-tab của FR17. Tự dựng lại = tự viết lại dockview, đúng thứ `EXPERIENCE.md:21` cấm. |

⚠️ Vạch tiêu điểm 2px `primary` mép trái *(AC5 Story 1.6, UX-DR8)* **ở lại `PanelFrame`**, không chuyển sang tab.

### Quyết định #5 — **Bố cục hiện tại** lưu ở đâu? *(FR18 vế "khôi phục giữa các phiên")*

`kinds.rs:206-213` đã phân xử một nửa: *"**bố cục đang hiển thị** là của frontend; **preset đã đặt tên và lưu lại** là dữ liệu Rust"*, và cảnh báo thẳng *"cách đọc kia dẫn thẳng tới `localStorage`"*.

| | Cách | Đánh đổi |
|---|---|---|
| **A ✅ mặc định** | **Preset đặt tên** → `ScopeKind::LayoutPreset` *(đã có, `GlobalOnly`)*. **Bố cục hiện tại** → `ScopeKind::AppConfig` khoá `workspace_layout`, cùng cửa với `theme` và `mode`. Thêm `KEY_LAYOUT` + accessor ở `core/scope/store.rs` và trường `workspace_layout: String` vào `commands::config::BootstrapConfig`. | Nhất quán với `theme`/`mode` *(cùng là "trạng thái cuối cùng của ứng dụng")*, đi đúng `store::Writer` nối tiếp (AD-11), và không đụng `localStorage`. Giá: ~8 dòng Rust + 1 trường TS. ⚠️ `tests/ipc_contract.rs:159` dựng `BootstrapConfig` bằng struct literal ⇒ **không biên dịch được cho tới khi dev thêm trường** — đó là hành vi đúng, đừng "sửa" bằng `..Default::default()`. |
| B | Nhét bố cục hiện tại vào `layout_presets` dưới một khoá dành riêng (`__current`) | Không đụng Rust, nhưng bẻ nghĩa của *"preset đã đặt tên"* và Story 1.21 sẽ hiện `__current` ra màn hình như một preset. |
| C | `localStorage` | không Bị `kinds.rs:212` gọi tên là đường sai. |

🔴 **Nhịp ghi:** `onDidLayoutChange` bắn **liên tục** trong lúc kéo sash. **Đừng `putConfig` mỗi lần bắn** — đó là đúng thứ AD-11/AD-12 tồn tại để chặn. Mượn **hình dạng** hợp đồng AD-35: idle ~500 ms **cộng** một trần cứng ~5 s **không reset bởi sự kiện kế tiếp**, cộng một lượt ghi ở `beforeunload`/khi rời chế độ. ⚠️ Đây là *mượn hình dạng*, **không** phải "áp AD-35 cho bố cục" — AD-35 là hợp đồng của **Editor**; bố cục không có `SegmentVersion` và mất một lượt kéo sash không phải mất công việc.

### Quyết định #6 — Đóng nợ `deferred-work.md:36` *(cổng i18n đo **DẤU**, không đo **CHUỖI HIỂN THỊ**)* ngay ở story này?

Ice chốt 2026-08-04: *"giữ nguyên cổng, không mở rộng phạm vi trong Story 1.5 […] **Mở lại ở Story 1.14**, khi bốn panel thật có nhãn thật để định nghĩa 'đúng' nghĩa là gì."* Nay chúng có nhãn thật.

| | Cách | Đánh đổi |
|---|---|---|
| **A ✅ mặc định** | Thêm **Kiểm A2** vào `scripts/check-i18n.mjs`: mọi **text node** trong `.vue` dưới `src/**` phải là `{{ t('…') }}` · `{{ tError(…) }}` · rỗng/khoảng trắng. Miễn trừ **có tên**, in ra mỗi lượt chạy. | Đóng đúng lỗ Ice mô tả (`<button>Dong</button>` hôm nay **xanh**). Bộ phân tích template đã có sẵn từ Kiểm A — không phải viết parser mới. |
| B | Hoãn tiếp sang 1.16/1.17 | Hợp lệ **chỉ khi** ghi số: bao nhiêu text node thật, bao nhiêu ca dương tính giả. Không hoãn bằng một câu. |

🔴 **Đây là task ƯU TIÊN THẤP NHẤT** (Task 13). Nếu story chạy dài, **ghi số rồi dừng** và bàn giao — đúng doctrine §Ngân sách CI của Story 1.3. Đừng để nó nuốt phần khung bố cục.

---

## Acceptance Criteria

> Tám AC đầu là **tám khối `Given/When/Then` của `epics.md:1589-1622`**, giữ nguyên nghĩa. Bốn AC sau là nợ kỹ thuật mà `deferred-work.md` giao **đích danh** story này — chúng không phải phạm vi mới, chúng là điều kiện để story trước không bị đánh dấu đạt sai.

### AC1 — Bốn slot panel tồn tại trong **MỘT** cửa sổ hệ điều hành duy nhất *(FR16, AD-24)*

**Given** Workspace · **When** mở · **Then** bốn slot `Source` · `Lookup` · `AiTranslation` · `Editor` tồn tại trong **một** cửa sổ OS.

- Bốn component `.vue` dưới `src/panels/`, mỗi cái khai **điểm vào focus riêng**: `panel.source` · `panel.lookup` · `panel.ai_translation` · `panel.editor`. Cả bốn có mặt trong `FOCUS_OWNERS` *(`src/commands/index.ts`)*, và Kiểm E của `npm run check:commands` đối chiếu **hai chiều**.
- 🔴 **KHÔNG gọi `api.addPopoutGroup()` ở bất kỳ đâu.** Đo thật trên bundle: `addPopoutGroup` là **đường duy nhất** trong `dockview-core` gọi `window.open` *(2 lần)* và **đường duy nhất** tạo `<style>` để chép stylesheet sang cửa sổ mới. Nó ⇒ **cửa sổ OS thứ hai**, tức vi phạm thẳng AD-24 — thứ chính `epics.md` gọi là *"trả bằng chính thứ sản phẩm bán"*. Một cổng test cưỡng chế mệnh đề này *(AC12)*.
- Đo được: mở Workspace ⇒ đúng **4** `api.panels.length`; `api.getPopouts().length === 0`.

### AC2 — Kéo thả: dock · undock · gộp thành tab · đổi kích thước *(FR17)*

**Given** một panel · **When** kéo thả · **Then** dock, undock, gộp thành tab và đổi kích thước được.

- **Undock = `api.addFloatingGroup()`** *(nhóm nổi **trong cùng cửa sổ**)*, **không** phải `addPopoutGroup` — xem AC1.
- Bốn năng lực này là **của dockview**, **không tự viết lại** *(`EXPERIENCE.md:21`: "dock, undock, gộp tab, đổi kích thước và preset đều là năng lực sẵn có của nó, không tự viết lại")*. Nghiệm thu bằng lượt chạy tay có bảng, không bằng test tự động.

### AC3 — Ẩn một panel là **ẩn hoàn toàn**; các panel còn lại lấp đầy chỗ trống *(FR17)*

**Given** một panel bất kỳ · **When** người dùng chọn ẩn · **Then** nó **ẩn hoàn toàn** **And** các panel còn lại lấp đầy chỗ trống.

- Cài theo §Quyết định #3A: `removePanel` + sổ vị trí đã nhớ. Panel đã ẩn **không** còn trong DOM, không trong vòng `focus.next_panel`, không nhận `Tab`.
- Hiện lại đưa panel về **đúng vị trí tương đối đã nhớ**, không dồn về cuối.
- Bốn command `layout.toggle_*` đăng ký ở `CommandRegistry`, handler thật, **không** gán phím *(§Quyết định #2 — ràng buộc AC6 Story 1.6)*.

### AC4 — Bố cục khôi phục **nguyên trạng** sau khi đóng rồi mở lại ứng dụng *(FR18)*

**Given** bố cục hiện tại · **When** đóng rồi mở lại ứng dụng · **Then** bố cục khôi phục nguyên trạng.

- Đường đi theo §Quyết định #5A: `api.toJSON()` → `putConfig('app_config', 'workspace_layout', json)` → `store::Writer` → `global.db`; lúc mở: `bootstrap_config` → `api.fromJSON(...)`.
- 🔴 **Nhịp ghi có trần cứng không reset** — xem §Quyết định #5. Một cổng đọc được số lần `putConfig` trong một lượt kéo sash dài là bằng chứng; không đủ thì ghi lượt đếm tay vào §Debug Log References.
- 🔴 **JSON hỏng ⇒ rơi về preset mặc định, KHÔNG cửa sổ trắng.** `fromJSON` **ném** với dữ liệu sai hình dạng, và `WorkspaceMode` dựng sau `mount()` nên một lần ném ở đó giết cả chế độ. Cùng lớp lỗi mà `bindingsAreUsable()` *(`commands/index.ts` §Bẫy 5)* và khối `try` quanh `installCommands` *(`main.ts`)* đã chặn — **dùng lại đúng khuôn đó**: `try` → `console.error` nêu đích danh → `api.clear()` → dựng preset mặc định.
- Nghiệm thu vòng thật: đổi bố cục → thoát ứng dụng → mở lại → bố cục đúng. **Chạy tay, có bảng.** *(`scope_contract.rs::the_last_mode_survives_a_write_and_a_reopen` là khuôn có sẵn cho nửa Rust của vòng này.)*

### AC5 — Nhiều preset bố cục, chuyển được **bằng phím qua `CommandRegistry`** *(FR18, AD-34)*

**Given** nhiều preset bố cục đã lưu · **When** người dùng chuyển · **Then** chuyển được bằng phím qua `CommandRegistry`.

- Hai preset ở story này: **2×2** *(mặc định)* và **4 cột** `Nguyên văn | Tra cứu | Đề xuất AI | Bản dịch` *(UX-DR13)*. Review Mode là **Story 8.11** — không dựng.
- Hợp âm theo §Quyết định #1. **Không** `⌘1` `⌘2` — chúng là **ba chế độ** *(đã phân xử ở Story 1.6)*.
- Mọi thao tác đi qua `dispatch()`; **không** một `@click` nào tự cài đặt thao tác tại chỗ *(Kiểm A của `check-commands.mjs` cưỡng chế bằng cú pháp)*.
- Preset đọc/ghi qua `ScopeKind::LayoutPreset` — **`GlobalOnly`**. 🔴 **KHÔNG dựng thanh chuyển phạm vi Toàn cục/Tác phẩm cho preset** — `kinds.rs:36` gọi tên đích danh cái bẫy này: *"Story 1.14/1.21 sẽ dựng thanh chuyển phạm vi cho một thứ không nên có."*

### AC6 — Preset mặc định là **lưới 2×2**, đúng thứ tự đã chốt *(UX-DR13)*

**Given** preset mặc định · **When** mở lần đầu · **Then** là lưới 2×2 — `Nguyên văn | Bản dịch` hàng trên, `Tra cứu | Đề xuất AI` hàng dưới.

- ⚠️ **`Nguyên văn` và `Bản dịch` cạnh nhau theo chiều NGANG** — UX-DR13 nêu lý do và nó không phải khẩu vị: *"đối chiếu ngang là thao tác lặp hàng trăm lần mỗi Chương"*, và đó cũng là lý do Sync Scrolling (FR20) tồn tại. Đừng xếp dọc "cho cân".
- Kho rỗng *(lần chạy đầu)* ⇒ 2×2, không lỗi. Cùng luật `DEFAULT_THEME`/`DEFAULT_MODE`.

### AC7 — Cơ chế ẩn cho phép **thứ tự hy sinh** của UX-DR15, và KHÔNG bắt Story 4.12 mổ lại bố cục

**Given** UX-DR15 khai thứ tự hy sinh là **quyết định**, không phải số hiệu chỉnh được · **When** dựng cơ chế ẩn/hiện panel · **Then** cơ chế phải cho phép ẩn theo đúng thứ tự đó.

- Thứ tự, **chép nguyên văn `epics.md:1616`**: **Đề xuất AI nhường trước · Tra cứu nhường sau nhưng rút về thanh trạng thái, không bao giờ mất hẳn · cặp `Nguyên văn | Bản dịch` không bao giờ nhường.**
- Cài thành **một hằng số đã sắp thứ tự, export được, kiểm được bằng máy** — ví dụ `SACRIFICE_ORDER = ['panel.ai_translation', 'panel.lookup']` cộng `NEVER_SACRIFICED = ['panel.source', 'panel.editor']`, cộng một hàm `nextToSacrifice(visible)` thuần *(không đọc kích thước cửa sổ)*.
- 🔴 Cổng test cưỡng chế **ba mệnh đề**: (1) hai tập rời nhau và hợp lại đúng bốn panel; (2) `panel.source`/`panel.editor` **không bao giờ** là đầu ra của `nextToSacrifice`; (3) `panel.ai_translation` đứng **trước** `panel.lookup`.
- **KHÔNG** viết `matchMedia`, **không** viết bốn ngưỡng, **không** dựng ngăn kéo, **không** rút Tra cứu về thanh trạng thái — cả bốn là **Story 4.12**. Story này chỉ bảo đảm 4.12 **chỉ phải điền ngưỡng**, không phải mổ lại bố cục.

### AC8 — Panel `AiTranslation` và `Editor` nêu rõ trạng thái bằng chuỗi trong `vi.json` *(NFR16, AD-21, UX-DR27)*

**Given** hai panel chưa có nội dung ở epic này · **When** hiển thị · **Then** chúng nêu rõ trạng thái bằng chuỗi trong `vi.json`, không phải một khung trống không giải thích.

- **Không chuỗi tiếng Việt nào trong `.vue`** — mọi text node đi qua `t('…')`.
- ⚠️ **`AiTranslation` chưa cấu hình KHÔNG phải trạng thái lỗi** *(UX-DR27, FR77)*: panel **mời cấu hình**, không cảnh báo, không màu `error`.
- Giọng văn theo UX-DR47 / `EXPERIENCE.md §Voice and Tone`: nói việc, không xưng "chúng tôi", không gọi người dùng là "bạn", không dấu chấm than. Kiểm D của `check-i18n.mjs` chấm phần máy chấm được.
- Bốn panel đều có chuỗi *(Source/Lookup sẽ bị 1.16/1.17 thay)* — đừng để hai panel có, hai panel là khung trống.

### AC9 — 🔴 Vế **panel** của AC4 Story 1.6 đóng TRỌN: có đường dời focus tường minh chạy được *(nợ `deferred-work.md:161`)*

`deferred-work.md:161` ghi nguyên văn: *"**Không đánh dấu AC4 đạt trọn cho tới lúc đó**"*, và *"lúc đó"* là story này.

- `focus.next_panel` **có phím** *(§Quyết định #2)* ⇒ handler hết là **mã sống nhưng bất khả đạt**.
- Vòng xoay đi theo **thứ tự bố cục** *(trái→phải, trên→dưới của lưới hiện tại)*, không theo thứ tự khai báo. `focus.ts::next()` hôm nay xoay theo thứ tự `declare()` — story này phải cấp cho nó thứ tự thật. ⚠️ `indexOfLiveFocus()` đọc `document.activeElement` **thật** *(không đọc con trỏ ứng dụng tự giữ)* — giữ nguyên kỷ luật đó.
- Panel đã ẩn *(AC3)* **không** nằm trong vòng.
- 🔴 **Chuyển panel trong dockview phải dời focus DOM tường minh** *(AD-34 §2, UX-DR7)* — bấm một tab dockview ⇒ `enterFocus(owner)` chạy, không phải chỉ dựa vào hành vi focus mặc định của trình duyệt.
- ⚠️ `<KeepAlive>` *(§Quyết định #6 Story 1.6)* làm ca *"phần tử đã tháo khỏi DOM"* thành thường trực — `focus.ts:165` đã có phép kiểm `isConnected`; dockview **cũng** đỗ/dựng lại DOM của panel khi đổi group. **Đối chiếu lại**: rời Workspace khi một panel có tiêu điểm rồi quay lại ⇒ vạch tiêu điểm **không** được nói dối *(`PanelFrame.vue:60` `onDeactivated`)*.

### AC10 — Nợ **token trọng lượng nhãn đậm** đóng bằng một quyết định có chữ ký *(nợ `deferred-work.md:138`)*

Hôm nay `PanelFrame.vue:178` và `App.vue:288` **mượn** `var(--weight-read-title)` cho nhãn giao diện đậm, vì bộ token không có biến nào cho việc đó *(`ui-md` = 400, `ui-label` = 700, `DESIGN.md §Components` đòi 600)*.

| | Cách | Việc phải làm |
|---|---|---|
| **A ✅ mặc định** | Thêm token typography thứ **15**: `ui-md-strong` *(family `ui` · 12px · 600 · 1.5 · `wraps: false`)* | Sửa **ba** chỗ trong `scripts/check-tokens.mjs`: `EXPECTED_TYPOGRAPHY` *(bảng đóng băng, `:140`)*, `EXPECTED_COUNTS.typography` **14 → 15** *(`:198`)*, và thêm cặp tương phản mới vào `contrast.pairs` nếu có. Cộng một mục `deviations` trong `tokens.json` nêu rằng `DESIGN.md` còn ghi 14 và việc sửa tài liệu là **một lượt riêng của Ice**. |
| B | Chốt rằng **mượn là đúng**, ghi vào `DESIGN.md` | không Dev không sửa `DESIGN.md` *(tiền lệ Ice)*, nên đường này để món nợ mở tiếp **và** để nguyên rủi ro đã ghi: *"`--weight-read-title` đổi giá trị thì hai chỗ này đổi theo mà không ai biết."* |

⚠️ Bảng đóng băng trong cổng là **chỗ một con người phải ký** — đó là thiết kế, không phải một trở ngại cần lách. **Đừng** khai một biến CSS cục bộ `--weight-…: 600` để lách Kiểm B2.

### AC11 — Ba món nợ **cổng** đóng, và một món **nghiệm thu bằng mắt** đóng

1. **Sàn cổng đếm tệp** *(`deferred-work.md:48` · `:146`)* — nâng `VUE_FLOOR`/`TS_FLOOR` của `check-commands.mjs` và `VUE_FLOOR` của `check-i18n.mjs` cho khớp quần thể **thật sau story này**. ⚠️ Nâng sàn **không** phải "sửa cho vừa": ghi con số thật vào comment cạnh hằng số, đúng khuôn `RS_FLOOR` đang có.
2. **Không cổng nào canh focus ring** *(`deferred-work.md:140`)* — thêm một phép kiểm: `outline: none` **chỉ** được xuất hiện trên gốc `tabindex="-1"` của chế độ/panel, và **phải** kèm miễn trừ có tên `/* aura-allow-outline-none: <lý do> */`. Cùng khuôn `aura-allow-z-index` đã có ở Kiểm F. 🔴 Một `*:focus { outline: none }` hôm nay đi qua **cả** `check-commands` **lẫn** `check-tokens`.
3. **Bộ lọc phần mở rộng bỏ qua `.tsx` · `.mts` · `.cts`** *(`deferred-work.md:163`)* — `endsWith('.ts')` sai với cả ba. Sửa hoặc **ghi lý do giữ nguyên**; không im lặng.
4. **AC6 của Story 1.4 — khe 2px theme tối** *(`deferred-work.md:104`)* — nay có panel thật. Chụp **hai ảnh** *(sáng: nét 1px `outline`; tối: khe 2px lộ `background`, panel bo 3px)* và dán bảng vào §Debug Log References. ⚠️ Cổng chỉ chứng minh **hai cơ chế đã khai và không bị thống nhất**; nó không chứng minh khe hiện ra đúng.

⚠️ Kèm theo, không phải AC nhưng phải **soát và ghi số**: *(a)* `check-tokens.mjs` in `Tầm quét: N tệp · M khai báo CSS` — con số `M` **tụt bất thường** là dấu hiệu parser CSS "đủ dùng" bỏ sót cả vùng *(`deferred-work.md:110`)*; *(b)* đối chiếu lại cờ `wraps` của từng token với chuỗi **thật** chạy qua nó *(`:106`)*; *(c)* mọi bề mặt chữ mới phải **tự khai token**, không kế thừa `ui-md` 1.5 của `body` *(`:119`)*.

### AC12 — Ranh giới KHÔNG CHẠM, và **mọi cổng xanh**

- Không `localStorage` · `sessionStorage` · `window.open` · `document.write` ở `src/**`.
- Không thêm **một** phụ thuộc npm hay crate nào. `dockview-vue@7.0.4` **đã có trong `package.json` và đã cài** *(vào từ Story 1.2, đã rà NFR15 — MIT, dấu **⚠️** trong bảng Stack: nhãn đúng, bằng chứng yếu hơn)*. Thêm bất cứ gì khác phải rà GPLv3 và vào bảng Stack **trước** (NFR15) — tức không phải việc của story này.
- Không đụng `src-tauri/tauri.conf.json` · `Cargo.toml` · `[profile.release]` · `dict-manifest.toml` · `tools/**` · `core/dict/**` · `core/matching/**`.
- ⚠️ **CSP KHÔNG được nới.** `style-src 'self'` không có `'unsafe-inline'`. Đo thật: `dockview.css` *(3.436 dòng)* **không** có `@import`, không `url(...)`, không `@font-face` — nó bundle được qua Vite thành một tệp CSS `'self'`, hợp lệ. Chỗ **duy nhất** dockview tạo `<style>` lúc chạy là đường **popout**, và AC1 cấm đường đó. **Đừng thêm `'unsafe-inline'`** — tiền lệ Ice 2026-08-03: *"giữ nguyên CSP, không nới chỉ để một phép kiểm đo được."*
- Sáu cổng phải **xanh**: `npm run check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `npm run build` · `cargo test --locked`.

---

## Tasks / Subtasks

- [x] **Task 0 — Chốt sáu quyết định TRƯỚC dòng mã đầu tiên** (§Quyết định #1–#6)
  - [x] Ghi phán quyết + lý do vào §Completion Notes. Không bắt đầu Task 1 khi còn một cái để ngỏ.
  - [x] Chạy đường cơ sở: `npm run build` · `cargo test --locked` · bốn cổng `.mjs`. Ghi số **trước** khi sửa gì — không có nó thì không phân biệt được "story làm đỏ" với "vốn đã đỏ". ⚠️ Cây làm việc **không sạch** (xem §Bối cảnh git).

- [x] **Task 1 — Theme dockview: `--dv-*` → token của dự án** (AC1, AC12)
  - [x] Import `dockview-vue/dist/styles/dockview.css` **một lần**, ở `src/main.ts` cạnh `./tokens/reset.css`.
  - [x] Dựng `src/layout/dockview-theme.css`: một lớp `.dockview-theme-aura` gán **mọi** `--dv-*` mà dockview đọc, và **mọi giá trị là `var(--color-*)` / `var(--space-*)` / `var(--radius-*)`** *(113 biến `--dv-*` tồn tại; phần lớn thuộc 12 theme dựng sẵn — chỉ map những biến lớp `.dv-*` thật sự đọc)*.
  - [x] **Không dùng một theme dựng sẵn nào của dockview** (`dockview-theme-light`, `-dark`, …): chúng viết màu thẳng, không qua bộ token đã kiểm tương phản (AD-34 §3).
  - [x] Đặt `border-radius` của group theo `--panel-radius` để khớp cơ chế đảo ngược hai theme *(UX-DR14)*.
  - [x] Nghiệm thu: `npm run check:tokens` xanh và **không** báo một giá trị màu viết thẳng nào trong `src/layout/**`.

- [x] **Task 2 — Bốn component panel + `PanelTab.vue`** (AC1, AC4-#4, AC8)
  - [x] `src/panels/SourcePanel.vue` · `LookupPanel.vue` · `AiTranslationPanel.vue` · `EditorPanel.vue` — mỗi cái bọc `PanelFrame`, truyền `owner` **literal** *(Kiểm E đọc literal, không đọc biểu thức)* và `titleKey`.
  - [x] Sửa `PanelFrame.vue`: **bỏ `<header>`** *(§Quyết định #4A)*, giữ vạch tiêu điểm 2px `primary`, giữ `focusin`/`focusout`/`onDeactivated`, giữ `declareFocus`/`releaseFocus` và chốt `owner` ở `setup` *(doc-comment `:65-72` nói thẳng vì sao — `v-for`/re-key của dockview là đúng ca đó)*.
  - [x] `src/panels/PanelTab.vue` — tab component của dockview: cao `var(--space-head-height)`, `ui-md` / `on-surface-variant`; group có tiêu điểm ⇒ `primary` + nét đậm *(dùng token của AC10)*.
  - [x] Bốn khoá `vi.json` tiêu đề + bốn khoá trạng thái. Giọng văn UX-DR47. ⚠️ `AiTranslation` **mời cấu hình**, không cảnh báo.
  - [x] Cập nhật `FOCUS_OWNERS` lên **7** mục *(3 chế độ + 4 panel)*.

- [x] **Task 3 — `src/layout/` — vỏ dockview và hai preset** (AC1, AC5, AC6)
  - [x] `src/layout/workspaceLayout.ts` *(thuần, không import `vue`)*: định nghĩa `PANEL_IDS`, hai preset dựng bằng `SerializedDockview`, `SACRIFICE_ORDER`, `NEVER_SACRIFICED`, `nextToSacrifice()`.
  - [x] `src/layout/WorkspaceDock.vue`: `<DockviewVue>` + `components` map + `tabComponents` + `@ready`.
  - [x] `WorkspaceMode.vue` thay hai `PanelFrame` bằng `WorkspaceDock`; **giữ nguyên** `declareFocus('mode.workspace')` và `onActivated → enterFocus`.
  - [x] Preset mặc định **2×2**: `Nguyên văn | Bản dịch` trên, `Tra cứu | Đề xuất AI` dưới *(UX-DR13 — không đúng thứ tự này)*.
  - [x] Preset **4 cột**: `Nguyên văn | Tra cứu | Đề xuất AI | Bản dịch`.

- [x] **Task 4 — Ẩn/hiện panel + thứ tự hy sinh** (AC3, AC7)
  - [x] `hidePanel(id)` / `showPanel(id)` theo §Quyết định #3A, kèm sổ vị trí đã nhớ.
  - [x] Bốn command `layout.toggle_*` — handler thật, **không** `keys`.
  - [x] `nextToSacrifice()` là **hàm thuần** *(không đọc kích thước cửa sổ)* — điều kiện để 4.12 chỉ phải nối ngưỡng vào.
  - [x] Cổng ba mệnh đề của AC7. ⚠️ Nếu cổng viết bằng `.mjs`: nó nạp `workspaceLayout.ts` bằng **Node thuần** ⇒ tệp đó phải theo luật *"erasable-only"* *(không `enum`, không `namespace`, không parameter property)* và không `import` giá trị từ `vue` — đúng luật `src/commands/**` đang giữ.

- [x] **Task 5 — Hai command preset + hai command focus** (AC5, AC9)
  - [x] `layout.preset_grid` · `layout.preset_columns` với hợp âm của §Quyết định #1.
  - [x] `focus.next_panel` nhận hợp âm của §Quyết định #2; thêm `focus.prev_panel` nếu chọn A.
  - [x] Khoá `command.*` tương ứng trong `vi.json` *(quy ước `'command.' + id`, §Quyết định #4 của Story 1.6)*.
  - [x] ⚠️ **Đăng ký ở `src/commands/index.ts`**, và handler phụ thuộc trạng thái đi vào qua **tiêm** *(`CommandDeps`)* — **không** `import` `vue`/`dockview` ở tệp đó, nếu không Kiểm C/D/E chết cùng lúc.
  - [x] Xác nhận `unbound()` vẫn trả về **≥ 1** phần tử *(bốn `layout.toggle_*`)* — AC6 Story 1.6 không được mất bằng chứng.

- [x] **Task 6 — Vòng xoay focus theo thứ tự bố cục** (AC9)
  - [x] Cấp cho `focus.next()` thứ tự **bố cục hiện tại**, không phải thứ tự `declare()`. Panel đã ẩn không trong vòng.
  - [x] Nối sự kiện đổi panel/group hoạt động của dockview → `enterFocus(owner)` **tường minh** (AD-34 §2).
  - [x] Nghiệm thu tay: `focus.next_panel` bốn lần ⇒ đi hết bốn panel rồi quay lại; ẩn một panel ⇒ vòng còn ba; rời Workspace rồi quay lại ⇒ không vạch tiêu điểm nào nói dối.

- [x] **Task 7 — Lưu và khôi phục bố cục** (AC4)
  - [x] **Rust** *(§Quyết định #5A)*: `KEY_LAYOUT = "workspace_layout"` + accessor ở `core/scope/store.rs`; trường `workspace_layout: String` ở `commands::config::BootstrapConfig`. Không `#[serde(rename_all)]` — khoá trên dây là `snake_case` *(`config.rs:49-53`)*.
  - [x] Sửa `tests/ipc_contract.rs` *(struct literal `:159`)* và `tests/scope_contract.rs` cho khớp; thêm ca *"kho rỗng ⇒ chuỗi rỗng ⇒ preset mặc định"*.
  - [x] **TS**: trường tương ứng ở `BootstrapConfig`; ghi qua `putConfig('app_config', 'workspace_layout', …)`.
  - [x] Nhịp ghi: idle ~500 ms + trần cứng ~5 s không reset + một lượt lúc rời chế độ/đóng cửa sổ.
  - [x] `fromJSON` bọc `try` → `console.error` → `clear()` → preset mặc định. Không cửa sổ trắng.

- [x] **Task 8 — Token nhãn đậm** (AC10)
  - [x] Theo §AC10 A: thêm `ui-md-strong` vào `tokens.json`, sửa **ba** chỗ trong `check-tokens.mjs`, thêm mục `deviations` có `question` + `reason` không rỗng.
  - [x] Thay `var(--weight-read-title)` ở `PanelFrame`/`PanelTab` và `App.vue:288` bằng token mới; gỡ hai comment "mượn" và ghi lại lý do mới.

- [x] **Task 9 — Ba món nợ cổng** (AC11 mục 1–3)
  - [x] Nâng sàn `check-commands.mjs` *(`VUE_FLOOR` · `TS_FLOOR`)* và `check-i18n.mjs` *(`VUE_FLOOR`)* theo quần thể thật; ghi con số vào comment.
  - [x] Phép kiểm focus ring + miễn trừ có tên `aura-allow-outline-none`.
  - [x] `.tsx`/`.mts`/`.cts`: sửa bộ lọc, hoặc ghi lý do giữ nguyên.
  - [x] 🔴 Nghiệm thu **ĐỎ trước, XANH sau** cho từng phép kiểm mới — không một cổng chưa từng đỏ là một cổng chưa từng canh. Ghi bảng ca vào §Debug Log References *(khuôn: Task 10 của Story 1.6, Task 3 của Story 1.4)*.

- [x] **Task 10 — Cổng cấm popout + cấm `localStorage`** (AC1, AC12)
  - [x] Một phép kiểm quét `src/**`: không `addPopoutGroup` · `window.open` · `localStorage` · `sessionStorage`.
  - [x] ⚠️ Viết dạng **danh sách CHO PHÉP hoặc mệnh đề hẹp**, không phải một danh sách cấm dài — `config_invariants.rs:92-94` lập luận thẳng rằng *"một danh sách cấm chỉ chặn được những hình dạng ai đó đã nghĩ ra"*, và `deferred-work.md:81` đã ghi một ca dự án tự mâu thuẫn về đúng điểm này.

- [x] **Task 11 — Nghiệm thu THỊ GIÁC, có bảng và ảnh chụp** (AC2, AC3, AC6, AC11 mục 4)
  - [x] Bảng ≥ 10 hàng: 2×2 mặc định · 4 cột · kéo sash · gộp hai panel thành tab · undock thành floating group · ẩn từng panel *(bốn hàng)* · hiện lại đúng chỗ · đóng/mở lại giữ nguyên bố cục · vạch tiêu điểm theo đúng panel · khe 2px theme tối / nét 1px theme sáng.
  - [x] ⚠️ Ghi rõ **engine và nền tảng** của lượt đo. Tiền lệ `deferred-work.md:130`: lượt đo DOM của Story 1.6 chạy trên **Blink/Chrome**, không phải WKWebView, vì cổng 1420 bị một dự án khác chiếm. **Đừng viết "tương đương" bằng suy luận.** Ca **Windows** chưa đo được — bàn giao **Story 1.3 / 10.9**.

- [x] **Task 12 — Cập nhật `deferred-work.md`**
  - [x] Đánh dấu **đóng** từng mục story này thật sự đóng, kèm bằng chứng *(tên phép kiểm, số hàng bảng)*. Không đánh dấu đóng cái chưa đóng.
  - [x] Mở mục mới cho những gì story này **cố ý không làm**: bốn ngưỡng 4.12 · ca Windows · `.ts` mang chuỗi *(`:35`)* · lỗ NFR17 của bốn `layout.toggle_*` · preset Review Mode.

- [x] **Task 13 — (ưu tiên thấp nhất) Kiểm A2 của cổng i18n** (§Quyết định #6)
  - [x] Chỉ làm sau khi Task 1–12 xanh. Chạy dài thì **ghi số rồi dừng** và bàn giao.

---

### Review Findings

*(Code review 2026-08-06 — Blind Hunter · Edge Case Hunter · Acceptance Auditor, diff đã lọc bỏ rác chưa commit của Story 1.13.)*

- [x] [Review][Decision] AC10 triển khai khác chữ với chỉ dẫn task — `EXPECTED_TYPOGRAPHY` giữ nguyên 14 hàng, cơ chế `deviations` được mở rộng thay vì thêm hàng 15 như phương án A yêu cầu literal. Dev đã tự khai ở §Completion Notes ("🔴 CHỖ TÔI LÀM KHÁC STORY") với lý lẽ: hai "bản chép độc lập" chỉ bắt lỗi khi cả hai còn chép cùng một thứ, nên thêm hàng thứ 15 vào bảng đóng băng sẽ làm chúng trôi khỏi nhau trong im lặng. Kết quả vẫn là một chữ ký cưỡng chế được (gỡ mục `deviations` ⇒ đỏ). **Ice chốt 2026-08-06: CHẤP NHẬN độ lệch — lý lẽ của dev vững hơn văn bản literal của spec.** Không sửa gì thêm.

- [x] [Review][Patch] Cổng thứ năm `check:layout` KHÔNG được gắn vào CI, trái với chính tuyên bố "chạy MỖI LƯỢT CI" ở §Debug Log References mục 2 và AC12 ("mọi cổng xanh") [package.json:17, .github/workflows/ci.yml] — **ĐÃ SỬA**: thêm bước `check workspace layout` vào `.github/workflows/ci.yml`, kề `check:commands`, trước `npm run build`.
- [x] [Review][Patch] `layout.preset_*`/`layout.toggle_*` là hợp âm TOÀN CỤC (như `mode.*`), nhưng `dockController` KHÔNG bị gỡ ở `onDeactivated` (chỉ gỡ ở `onBeforeUnmount`, mà `<KeepAlive>` không gọi khi đổi chế độ) — bấm `Mod+Alt+1`/`Mod+Alt+2` lúc đang ở Library/Reading sẽ âm thầm `api.clear()` + dựng lại dockview ẩn, rồi TỰ GHI xuống đĩa qua `onDidLayoutChange` → `flush()`, đè mất bố cục người dùng đã sắp mà không có dấu hiệu gì trên màn hình. Chính thông điệp lỗi của `dockController.ts::absent()` giả định "chế độ đang hiện không phải Workspace" là lý do duy nhất command không chạy — giả định đó không còn đúng sau khi mode bị `<KeepAlive>` đỗ [src/layout/WorkspaceDock.vue:598-600 (onDeactivated), src/layout/dockController.ts:49-61] — **ĐÃ SỬA**: `setDockController(null)` thêm vào `onDeactivated`, `onActivated` đã đăng ký lại lúc quay về nên hành vi trong Workspace không đổi.
- [x] [Review][Patch] `App.vue` — tab chế độ đang chọn (`.mode-tab.on`) vẫn "mượn" `var(--weight-read-title)`, CHƯA migrate sang `var(--weight-ui-md-strong)` dù Task 8 đánh dấu `[x]` hoàn tất và `deferred-work.md` (hunk mới) tuyên bố "hai chỗ mượn cũ ... KHÔNG còn mượn" cho **ba** chỗ — chỉ `PanelTab.vue` được migrate thật [src/App.vue:295-301] — **ĐÃ SỬA**: `.mode-tab.on` nay dùng `var(--weight-ui-md-strong)`, comment "mượn" gỡ bỏ.
- [x] [Review][Patch] `applyPreset()` không có `try/catch` quanh vòng lặp `addPanel()` — nếu một lời gọi ném giữa chừng, `api.clear()` đã chạy và `hidden` đã bị xoá, để lại bố cục dở dang và exception văng thẳng lên `registry.dispatch` không bọc; cùng lớp lỗi mà chính tệp này đã bọc `try/catch` ở `restore()` và `flush()` nhưng bỏ sót ở đây [src/layout/WorkspaceDock.vue:219-240] — **ĐÃ SỬA**: vòng lặp bọc `try/catch`, lỗi ⇒ `console.error` nêu đích danh + `api.clear()` + `hidden.clear()` + trả `false`.
- [x] [Review][Patch] Ba export chết trong `workspaceLayout.ts`: `mayBeSacrificed()`, `toggleCommandId()`, `TOGGLE_COMMAND_IDS` — không có nơi tiêu thụ nào trong `src/**` lẫn `scripts/check-layout.mjs` [src/layout/workspaceLayout.ts:192-202] — **ĐÃ SỬA**: cả ba đã gỡ.
- [x] [Review][Patch] `restore()` gán sổ vị trí `{ reference: id, direction: 'right' }` (panel tự tham chiếu chính nó) cho panel vắng mặt trong bố cục đã lưu — mẹo này chỉ đúng vì nó cưỡng ép `showPanel()` rơi vào nhánh dự phòng "neo không còn", nhưng không có comment nào tại điểm gọi giải thích cơ chế đó, khác hẳn kỷ luật ghi chú dày đặc của phần còn lại trong tệp [src/layout/WorkspaceDock.vue:518-520] — **ĐÃ SỬA**: thêm comment giải thích cơ chế tự tham chiếu tại điểm gọi.
- [x] [Review][Patch] `restoreFocusIfLost()` xếp một `requestAnimationFrame` MỚI cho mỗi lần gọi mà không huỷ lời gọi trước đó đang chờ — bấm liên tiếp nhiều `layout.toggle_*` (khả dĩ khi Story 1.21 gán phím cho hơn một cái) xếp chồng nhiều lượt phục hồi focus đua nhau trên `document.activeElement`. Rủi ro thấp: `enterFocus()` đã có canh `isConnected` nên trường hợp xấu nhất chỉ là focus nhấp nháy, không sập, không focus sai vĩnh viễn [src/layout/WorkspaceDock.vue:402-415] — **ĐÃ SỬA**: `pendingFocusRaf` theo dõi và huỷ lượt hẹn trước đó; cũng huỷ ở `onBeforeUnmount`.

*Verify sau khi vá: `npm run check:layout` · `check:tokens` · `check:i18n` · `check:commands` · `npm run build` — cả năm xanh.*
- [x] [Review][Defer] Khoá tiêu đề panel (`PANEL_TITLE_KEYS` ở `workspaceLayout.ts`, một tệp `.ts`) chảy qua một lời gọi `t()` KHÔNG literal ở `PanelTab.vue` (`t(props.params.params.titleKey ?? '')`) — nằm ngoài tầm quét `.vue`-only của `check-i18n.mjs`; một lỗi gõ tương lai trong bốn khoá đó sẽ không bị cổng nào bắt (giá trị hôm nay đều khớp `vi.json`, xác minh trực tiếp) [src/panels/PanelTab.vue:80, src/layout/workspaceLayout.ts:54-59] — deferred, pre-existing gate-coverage gap, cùng lớp rủi ro với `PANEL_COMPONENTS`/`components` map đã được ghi nhận trong `deferred-work.md`
- [x] [Review][Defer] `PANEL_SUFFIXES` ở `commands/index.ts` là bản chép tay của `PANEL_IDS`, chỉ có một dòng comment "chép từ", không cổng nào đối chiếu hai bảng — cùng hình dạng rủi ro với cặp `PANEL_COMPONENTS`/`components` đã ghi trong `deferred-work.md`, nhưng cặp này chưa được ghi [src/commands/index.ts:172-173] — deferred, pre-existing gate-coverage gap
- [x] [Review][Defer] `applyPreset()` luôn `api.clear()` rồi dựng lại TOÀN BỘ bốn panel kể cả khi preset yêu cầu đã là preset đang áp — vô hại hôm nay vì bốn panel là khung rỗng, nhưng sẽ mất trạng thái thật (cuộn, nội dung đang gõ, state AI) một khi Story 1.16/1.17/Epic 2 cho chúng nội dung thật; chưa có mục nào trong `deferred-work.md` ghi nhận rủi ro này [src/layout/WorkspaceDock.vue:219-240] — deferred to Story 1.16/1.17/Epic 2

---

## Dev Notes

### Trạng thái repo hôm nay — SỐ, không phải mô tả

| Thứ | Số |
|---|---|
| `.vue` dưới `src/` | **5** — `App.vue` · `modes/LibraryMode.vue` · `modes/ReadingMode.vue` · `modes/WorkspaceMode.vue` · `panels/PanelFrame.vue` |
| `.ts` dưới `src/` | **14** |
| `.rs` dưới `src-tauri/src/` | **31** *(126 kể cả `tests/` và `tools/`)* |
| Khoá `vi.json` | **16** |
| `FOCUS_OWNERS` | **5** ⇒ sau story này **7** |
| Command đã đăng ký | **4** *(`mode.library` · `mode.workspace` · `mode.reading` · `focus.next_panel`)* |
| `unbound()` | **1** *(`focus.next_panel`)* — xem ràng buộc §Quyết định #2 |
| Sàn cổng | `check-commands`: `VUE_FLOOR = 4` · `TS_FLOOR = 10` · `COMMAND_FLOOR = 4` · `CLICK_FLOOR = 3` · `DISPATCH_FLOOR = 3` — `check-i18n`: `RS_FLOOR = 21` · `VUE_FLOOR = 1` — `check-tokens`: `FILE_FLOOR = 5` · `COMPONENT_FILE_FLOOR = 4` |
| Đếm token đóng băng | `EXPECTED_COUNTS = { colorsPerTheme: 16, typography: 14, families: 4 }` |

`src/layout/` hôm nay chỉ có `.gitkeep` + `README.md`. `src/panels/` có `PanelFrame.vue` + `.gitkeep` + `README.md`.

### API thật của `dockview-vue@7.0.4` — đọc từ `node_modules`, không phải từ trí nhớ

| Sự thật | Nguồn | Hệ quả cho story này |
|---|---|---|
| `<DockviewVue>` nhận `IDockviewVueProps = DockviewOptions & VueProps`; `VueProps` có `components` · `tabComponents` · `watermarkComponent` · `defaultTabComponent` · … | `dist/types/dockview/types.d.ts` | Panel và tab đều là **component Vue**, đăng ký qua map. §Quyết định #4A khả thi. |
| Ba emit: `ready(DockviewReadyEvent)` · `didDrop` · `willDrop` | `dist/types/dockview/dockview.vue.d.ts` | `@ready` là chỗ **duy nhất** lấy được `api` để dựng preset / `fromJSON`. |
| `DockviewApi`: `toJSON()` · `fromJSON(data, { reuseExistingPanels })` · `clear()` · `addPanel` · `removePanel` · `addGroup` · `removeGroup` · `addFloatingGroup` · `moveToNext` · `moveToPrevious` · `onDidLayoutChange` · `onDidLayoutFromJSON` · `panels` · `groups` · `activePanel` · `activeGroup` | `dockview-core/dist/cjs/api/component.api.d.ts` | AC2 · AC4 đi thẳng qua các hàm này. |
| 🔴 `DockviewPanelApi extends Omit<GridviewPanelApi, 'setVisible' \| 'onDidConstraintsChange'>` | cùng nguồn, `dockviewPanelApi.d.ts:21` | **Không có `panel.api.setVisible`.** Ẩn panel = `removePanel`. Đây là lý do §Quyết định #3 tồn tại. |
| `addPopoutGroup(...)` là đường **duy nhất** gọi `window.open` và **duy nhất** tạo `<style>` lúc chạy | đo trên `dockview-core/dist/package/main.esm.mjs` | **Cấm** (AD-24 + CSP `style-src 'self'`). `addFloatingGroup` là undock đúng nghĩa. |
| `dockview.css` — 3.436 dòng, **không** `@import`, không `url(...)`, không `@font-face`; 113 biến `--dv-*`; 12 theme dựng sẵn viết màu thẳng | đo trên `dist/styles/dockview.css` | Bundle được qua Vite → CSP hợp lệ. Nhưng **đừng dùng theme dựng sẵn** — map `--dv-*` sang token của dự án (Task 1). |
| `dockview-vue` ⇒ `dockview` ⇒ `dockview-core`, cả ba **7.0.4**, peer `vue ^3.4.0` | `package.json` các gói | Khớp `vue@3.5.40` đang ghim. Không nâng, không thêm gói. |

### Bàn giao — mười lăm mục `deferred-work.md` gọi tên Story 1.14

| Dòng | Nợ | Story này |
|---|---|---|
| `:161` | 🔴 AC4 Story 1.6 **đạt một phần** — panel chưa có đường dời focus tường minh chạy được. *"Không đánh dấu AC4 đạt trọn cho tới lúc đó."* | **ĐÓNG** — AC9 |
| `:134` | `focus.next_panel` chưa có phím ⇒ không đường bàn phím nào vào panel | **ĐÓNG** — §QĐ #2 |
| `:136` | `⌘1` `⌘2` mockup vs UX-DR34 — *"1.14 phải chọn phím KHÁC cho preset bố cục"* | **ĐÓNG** — §QĐ #1 |
| `:138` | Bộ token thiếu biến trọng lượng nhãn đậm; hai đường ra cho 1.14 | **ĐÓNG** — AC10 |
| `:140` | Không cổng nào canh focus ring | **ĐÓNG** — AC11.2 |
| `:104` | AC6 Story 1.4 *(khe 2px theme tối)* nghiệm thu ở **tầng token**, không trên màn hình | **ĐÓNG** — AC11.4 |
| `:48` · `:146` | Sàn cổng **đếm tệp**, không đếm nội dung | **ĐÓNG** — AC11.1 |
| `:163` | Bộ lọc cổng bỏ qua `.tsx` · `.mts` · `.cts` | **ĐÓNG** — AC11.3 |
| `:36` | Cổng i18n đo **DẤU**, không đo **CHUỖI HIỂN THỊ**. *"Mở lại ở Story 1.14."* | **§QĐ #6** — mặc định làm, ưu tiên thấp nhất |
| `:106` | Cờ `wraps` chưa đối chiếu với chuỗi thật | **SOÁT + GHI SỐ** — AC11 ⚠️(b) |
| `:110` | Parser CSS của cổng là *"đủ dùng"*; soát lại **số khai báo đã quét** khi 1.14 dựng CSS thật | **SOÁT + GHI SỐ** — AC11 ⚠️(a) |
| `:119` | `body` chạy giãn dòng 1.5; bề mặt đọc phải tự khai token `read-*` | **SOÁT + GHI SỐ** — AC11 ⚠️(c); vế đầy đủ ở 1.16/1.17 |
| `:128` | Vế DOM của AC4 không có phép kiểm tự động; *"lưới thật là lượt rà soát khi 1.14 dựng bốn panel trong dockview"* | **RÀ SOÁT TAY** — Task 11; không dựng test runner |
| `:35` | Tệp `.ts` mang chuỗi tiếng Việt và không cổng nào canh. *"không Hệ quả: dời một chuỗi từ `.vue` sang `.ts` là cách hợp lệ về mặt cổng để cho xanh — đừng dùng."* | **KHÔNG dùng đường đó**; nợ giữ nguyên cho 10.9 |
| `kinds.rs:36` | *"Khai `LayoutPreset` là `Override` là sai im lặng […] Story 1.14/1.21 sẽ dựng thanh chuyển phạm vi cho một thứ không nên có."* | **KHÔNG dựng thanh chuyển phạm vi** — AC5 |

### 🧠 Trí tuệ từ story trước — thứ đắt tiền, đừng học lại bằng tiền

1. **Ném TRƯỚC `mount()` = cửa sổ trắng.** `main.ts:107-157` đã dựng một khối `try` + một hộp chẩn đoán vì đúng chuyện đó. `fromJSON` của dockview **ném** với dữ liệu sai hình dạng ⇒ AC4 phải bọc. Đừng để một `global.db` sửa tay giết ứng dụng.
2. **Registry nháp trước khi áp dữ liệu từ đĩa** — `bindingsAreUsable()` *(§Bẫy 5, Story 1.8)*. Cùng hình dạng áp cho bố cục đọc từ đĩa: thử, hỏng thì rơi về mặc định **kèm chẩn đoán nêu đích danh**, không chết.
3. **`src/commands/**` phải nạp được bằng NODE THUẦN.** Ba phép kiểm hành vi *(Kiểm C/D/E)* `import()` thẳng các tệp đó. Một dòng `import` giá trị của `vue`/`dockview`/`@tauri-apps/api` ở đó giết cả ba **cùng lúc**. Đường đúng: **tiêm** qua `CommandDeps`, và `main.ts` nối hai đầu.
4. **Cờ ứng dụng tự giữ sẽ nói dối.** `PanelFrame.vue:26-31` và `focus.ts:186-199` cùng bác bỏ khuôn mẫu đó: đọc `document.activeElement` **thật**. dockview có con trỏ `activePanel` riêng — **đừng** dùng nó làm nguồn sự thật cho vạch tiêu điểm.
5. **`<KeepAlive>` giữ subtree thay vì tháo nó** ⇒ `focusout` không chắc bắn ⇒ `onDeactivated` phải tắt cờ *(`PanelFrame.vue:60`)*. dockview **cũng** đỗ/dựng lại DOM khi đổi group — đối chiếu lại cả hai đường.
6. **Sàn cổng đếm tệp thì một tệp rỗng vẫn qua.** Nâng sàn **có comment ghi số thật**, không nâng cho vừa.
7. **Nghiệm thu ĐỎ trước, XANH sau.** Mọi phép kiểm mới phải chứng minh nó **đỏ được**. Khuôn: Story 1.4 Task 3 *(28 ca)*, Story 1.6 Task 10 *(28 ca)*, Story 1.5 *(16 + 23 ca)*.
8. **Dev không sửa tài liệu quy hoạch.** `epics.md` · `prd.md` · `DESIGN.md` · `EXPERIENCE.md` · `mockups/**` là **một lượt riêng của Ice** *(tiền lệ quyết định #3, Story 1.3)*. Lệch thì **ghi ra**, không sửa.

### ⚠️ Năm cái bẫy — bốn trong năm cho ra một lượt CI XANH với kết quả vô nghĩa

1. **Dùng theme dựng sẵn của dockview.** Giao diện *"trông chạy được"*, `check:tokens` **vẫn xanh** *(nó chỉ quét `src/**`)*, và sản phẩm âm thầm có một bảng màu thứ hai chưa ai kiểm tương phản — đúng thứ AD-34 §3 tồn tại để chặn.
2. **Ẩn panel bằng `width: 0`.** Mọi cổng xanh, mắt thấy panel biến mất, nhưng nó vẫn nhận `Tab` và vẫn trong vòng focus. *"Ẩn hoàn toàn"* của FR17 thành lời hứa.
3. **Ghi bố cục ở mỗi `onDidLayoutChange`.** Không cổng nào đỏ. Nhưng một lượt kéo sash 3 giây là hàng trăm lượt `putConfig` ⇒ hàng trăm job qua `store::Writer` nối tiếp ⇒ đúng thứ AD-11/AD-12 tồn tại để chặn, và nó sẽ hiện ra ở Epic 2 dưới dạng *"gõ bị khựng"* mà không ai lần ra được.
4. **Nhét ngưỡng màn hình hẹp vào cho gọn.** `epics.md:1617` cấm tường minh. Cài sớm = 4.12 phải **mổ lại**, đúng thứ AC7 tồn tại để tránh.
5. **Gán phím cho cả bốn `layout.toggle_*` cho "đủ NFR17".** `unbound()` trả mảng rỗng ⇒ **AC6 của Story 1.6 mất bằng chứng**, và không cổng nào đỏ. Lỗ NFR17 hôm nay là **có tên và có chủ** *(Story 1.21)*; một lỗ có tên tốt hơn một bằng chứng bị xoá.

### Testing standards

- **Rust**: `cargo test --locked`. Ca mới của story này chỉ chạm `tests/ipc_contract.rs` và `tests/scope_contract.rs` *(trường `workspace_layout`)*. Không thêm test crate mới.
- **Frontend**: **không có bộ chạy test**, và **không thêm** *(NFR15 — mọi phụ thuộc mới phải rà GPLv3 và vào bảng Stack trước; đó là quyết định của Ice)*. Thay thế, đúng ba đường mà bốn story trước đã dùng:
  1. **Cổng `.mjs`** — `import()` tệp `.ts` thuần bằng Node *(type-stripping, Node ≥ 22.18)* rồi gọi hàm thật. Đây là chỗ AC7 và AC11 sống.
  2. **Chốt tự kêu lúc chạy** — `console.error` nêu đích danh owner *(khuôn `focus.ts::armBodyGuard`)*. Nó **kêu**, không **vá**.
  3. **Nghiệm thu tay có bảng** trong §Debug Log References, ghi rõ engine + nền tảng + cái gì chưa đo.
- **Đỏ-rồi-xanh là bắt buộc** cho mọi phép kiểm mới.
- Ranh giới kiến trúc cưỡng chế **bằng test, không bằng kỷ luật** *(`epics.md:381`)* — AC7 và AC1 *(cấm popout)* là hai chỗ áp luật đó trong story này.

### Project Structure Notes

Cây nguồn của Structural Seed *(`ARCHITECTURE-SPINE.md:806-812`)* khai đúng những thư mục story này chạm:

```text
src/
  modes/     # Library · Workspace · ReadingMode · ReviewMode (AD-24)
  panels/    # Source · Lookup · AiTranslation · Editor      ← Task 2
  layout/    # dockview: dock/undock/tab/preset (FR17, FR18) ← Task 1, 3, 4
  commands/  # CommandRegistry — MỌI thao tác đăng ký ở đây  ← Task 5
  tokens/    # token màu đã kiểm tương phản WCAG AA hai theme ← Task 8
  i18n/vi.json
```

⇒ Story này **không** tạo thư mục mới nào ngoài khai báo. *(`src/config/` là thư mục duy nhất ngoài khai báo, đã có lý do viết ra ở đầu `bootstrap.ts` — đừng tạo cái thứ hai mà không viết lý do tương đương.)*

Quy ước đặt tên *(`ARCHITECTURE-SPINE.md:639-641`)*: Vue component `PascalCase.vue`; khoá `vi.json` **phẳng, chấm, có tiền tố miền**; `Panel Lookup → LookupPanel`. Cấm `Project`/`Book`/`Novel`/`Document` cho `Work`.

### 🌐 Phiên bản đang ghim — KHÔNG đổi một dòng nào

`vue 3.5.40` · `dockview-vue 7.0.4` *(⇒ `dockview` 7.0.4 ⇒ `dockview-core` 7.0.4, peer `vue ^3.4.0`)* · `typescript 5.9.3` · `vite 8.2.0` · `@vitejs/plugin-vue 6.0.8` · `vue-tsc 3.3.9` · `@tauri-apps/api 2.11.1` · `@tauri-apps/cli 2.11.4` · `tauri 2.11.5`.

⚠️ `dockview-vue` mang dấu **⚠️** trong bảng Stack: gói npm **không kèm tệp giấy phép**; `package.json` khai `MIT`, và `dockview-core` mang banner `@license MIT` **nhúng trong bundle đã phát hành**. Nhãn đúng, bằng chứng yếu hơn 16 hàng ✓ khác. **Đừng nâng phiên bản trong story này** — nâng là một lượt rà NFR15 mới.

⚠️ Cổng phụ thuộc *(`npm run check:deps`)* có `NPM_TREE_FLOOR` và một danh sách cấm. `dockview-vue` **đã** trong cây từ Story 1.2 ⇒ không lượt cài mới nào. Nếu con số cây npm đổi, **ghi số** và kiểm lại `check:deps` trước khi kết luận.

### 📌 Bối cảnh git — ⚠️ CÂY LÀM VIỆC KHÔNG SẠCH

Baseline `7e38de8` *(Add behavioral tests for core::matching functionality)*. Năm commit gần nhất đều thuộc tầng dữ liệu/từ điển — **không** commit nào chạm `src/**` kể từ `d9bc252` *(Story 1.8)*.

🔴 **`git status` lúc tạo story:** 6 tệp **M** *(`deferred-work.md` · `sprint-status.yaml` · `core/dict/mod.rs` · `lib.rs` · `ports/mod.rs` · `tests/dict_boundary.rs`)* và 5 tệp **??** *(story file 1.13 · `core/dict/layer.rs` · `core/dict/senses.rs` · `ports/dict_source.rs` · `tests/dict_sources.rs`)* — **toàn bộ là công việc của Story 1.13 chưa commit**.

⇒ **Việc đầu tiên của Task 0**: xác nhận với Ice rằng lượt của Story 1.13 được commit *(hoặc chấp nhận có ý thức là chưa)*, rồi **chạy đường cơ sở và ghi số**. Không có nó thì không phân biệt được *"story này làm đỏ"* với *"vốn đã đỏ trước khi gõ dòng nào"* — bài học đã ghi ở §Debug Log References Task 1 của Story 1.6.

### References

- `epics.md:1579-1622` — tám khối AC của Story 1.14 · `:1614-1618` ranh giới với 4.12 · `:94-100` FR16/FR17/FR18 · `:523-527` UX-DR13/14/15 · `:533` UX-DR17 · `:507-509` UX-DR7/UX-DR8 · `:820` ghi chú *"ba chế độ đăng ký ngay từ epic này"*
- `ARCHITECTURE-SPINE.md` — AD-24 `:322-326` *(một cửa sổ, ba chế độ)* · AD-34 `:406-417` *(CommandRegistry · điểm vào focus · màu từ token)* · AD-1 `:75-79` *(frontend giữ state UI, gồm **bố cục panel**)* · AD-18 bảng `:242-255` + ghi chú `:256-264` *(`Preset bố cục` = **chỉ toàn cục**)* · AD-11 `:153-157` · AD-21 `:302-306` · Consistency `:637-658` · Structural Seed `:806-815` · Stack `:660-712`
- `DESIGN.md` — §Layout & Spacing `:307-313` · §Elevation `:315-330` *(phân tách panel đảo ngược hai theme)* · §Components `:356-366` · bảng token màu `:169-186` · bảng token chữ `:260-275` · §Giãn dòng `:287-303`
- `EXPERIENCE.md` — §Foundation `:17-25` *(**dockview-vue**, không tự viết lại)* · §Information Architecture `:27-49` · §Component Patterns `:63-73` · §Accessibility Floor `:156-167`
- `mockups/key-screen-workspace.html` — `:23` tab chế độ · `:31-34` thanh panel 34px + vạch tiêu điểm · `:89` ⚠️ **dòng `⌘1`/`⌘2` ĐÃ BỊ PHÂN XỬ, đừng theo** · `:181-184` thanh trạng thái
- `mockups/narrow-layout.html` — `:195-232` bảng bốn ngưỡng + **thứ tự hy sinh** *(cơ chế ở story này, ngưỡng ở 4.12)* · `:248-249` *"bố cục tự đổi nhưng người dùng ghi đè được và ứng dụng nhớ"*
- `deferred-work.md` — `:35` `:36` `:48` `:104` `:106` `:110` `:119` `:128` `:134` `:136` `:138` `:140` `:146` `:161` `:163`
- Mã hiện có — `src/App.vue` · `src/main.ts` · `src/modes/WorkspaceMode.vue` · `src/modes/modeState.ts` · `src/panels/PanelFrame.vue` · `src/commands/{index,registry,focus,keys}.ts` · `src/config/bootstrap.ts` · `src/tokens/tokens.json` · `src-tauri/src/commands/config.rs` · `src-tauri/src/core/scope/{kinds,store}.rs` · `scripts/check-{tokens,i18n,commands}.mjs`
- `node_modules` *(đọc thật, không suy đoán)* — `dockview-vue/dist/types/**` · `dockview-core/dist/cjs/api/{component,dockviewPanelApi}.api.d.ts` · `dockview-vue/dist/styles/dockview.css`

### Câu hỏi cho Ice — đã có mặc định, KHÔNG chặn

1. **Sáu quyết định của Task 0** — mặc định lần lượt **A · A · A · A · A · A**. Cái đắt nhất là **#5** *(thêm một trường vào `BootstrapConfig`, tức chạm Rust và hai tệp test)*; cái dễ đổi ý nhất là **#1** *(hợp âm — Story 1.21 cho phép gán lại, chi phí đổi ý gần bằng 0)*.
2. **Lỗ NFR17 mở ra có ý thức**: bốn `layout.toggle_*` không có phím ⇒ ẩn/hiện panel hôm nay chỉ tới được bằng chuột. Đổi lại: `unbound()` giữ được bằng chứng cho AC6 của Story 1.6, và Story 1.21 là màn hình để gán. **Chấp nhận?**
3. **AC10** — thêm token thứ 15 *(A)* hay chốt "mượn là đúng" *(B)*? B đòi một lượt sửa `DESIGN.md` của Ice; A đòi sửa bảng đóng băng trong cổng, tức **một chữ ký**.
4. **§Quyết định #6** — mở rộng cổng i18n sang *chuỗi hiển thị* ngay ở story này hay hoãn tiếp? Mặc định **làm, ưu tiên thấp nhất, được phép ghi số rồi dừng**.

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, dev agent Amelia) — 2026-08-06.

### Debug Log References

#### 1 · Đường cơ sở Task 0 — ĐO TRƯỚC KHI SỬA GÌ

⚠️ Cây làm việc **KHÔNG sạch** đúng như §Bối cảnh git cảnh báo: 6 tệp `M` + 5 tệp `??` của Story 1.13 chưa commit. **Ice chưa xác nhận** — lượt này chạy trên cây đó, có ý thức. Baseline đo ngay trên nó, nên mọi so sánh dưới đây là *"story này làm đỏ"* chứ không lẫn với *"vốn đã đỏ"*.

| Cổng | Trước story | Sau story |
|---|---|---|
| `check:deps` | ✅ 326 crate · 104 gói npm | ✅ **không đổi** — 0 phụ thuộc mới |
| `check:tokens` | ✅ 21 tệp (18 component) · 116 khai báo CSS | ✅ 32 tệp (29 component) · **195** khai báo |
| `check:i18n` | ✅ 32 `.rs` + 5 `.vue` · 16 khoá | ✅ 32 `.rs` + **11** `.vue` · **28** khoá |
| `check:commands` | ✅ 5 `.vue` + 14 `.ts` · **4** command · **5** điểm vào focus | ✅ 11 `.vue` + 18 `.ts` · **11** command · **7** điểm vào focus |
| `check:layout` | *(chưa tồn tại)* | ✅ 29 tệp · 10 thành viên global |
| `npm run build` | ✅ | ✅ |
| `cargo test --locked` | ✅ | ✅ **165 passed** |

#### 2 · Nghiệm thu ĐỎ-RỒI-XANH — mọi phép kiểm MỚI phải chứng minh nó đỏ được

⚠️ *"Một cổng chưa từng đỏ là một cổng chưa từng canh."*

**Kiểm H — focus ring** (`check-tokens.mjs`, AC11.2) · **13 ca**

| Ca | Mong | Nhận |
|---|---|---|
| `*:focus { outline: none }` | ĐỎ | ĐỎ |
| `:focus` trần | ĐỎ | ĐỎ |
| `button:focus` | ĐỎ | ĐỎ |
| hậu duệ `.panel *:focus` | ĐỎ | ĐỎ |
| con trực tiếp `.mode > a:focus` | ĐỎ | ĐỎ |
| `outline: 0` | ĐỎ | ĐỎ |
| `outline-style: none` | ĐỎ | ĐỎ |
| `outline: none !important` | ĐỎ | ĐỎ |
| nhóm `.panel:focus, a:focus` *(một vế hợp lệ, một vế không)* | ĐỎ | ĐỎ |
| `.panel:focus` | xanh | xanh |
| `.mode:focus` | xanh | xanh |
| miễn trừ có tên `aura-allow-outline-none` | xanh | xanh |
| `outline: 2px solid var(--color-primary)` *(vẽ thật)* | xanh | xanh |

**Kiểm A2 — text node phải qua `t()`** (`check-i18n.mjs`, §Quyết định #6) · **14 ca**

| Ca | Mong | Nhận |
|---|---|---|
| `<button>Dong</button>` *(tiếng Việt KHÔNG dấu — Kiểm A mù với nó)* | ĐỎ | ĐỎ |
| `<span>Save</span>` | ĐỎ | ĐỎ |
| `{{ "Đã lưu" }}` | ĐỎ | ĐỎ |
| `{{ label }}` *(biến không qua `t()`)* | ĐỎ | ĐỎ |
| `{{ t() }} results {{ t() }}` *(chữ lọt giữa)* | ĐỎ | ĐỎ |
| `<i>3 muc</i>` | ĐỎ | ĐỎ |
| miễn trừ ĐẶT SAI CHỖ *(cách hai thẻ)* | ĐỎ | ĐỎ |
| `{{ t('panel.source.title') }}` | xanh | xanh |
| `{{ tError(e) }}` | xanh | xanh |
| `{{ t() }} · {{ t() }}` *(dấu phân cách)* | xanh | xanh |
| chỉ khoảng trắng | xanh | xanh |
| comment HTML tiếng Việt | xanh | xanh |
| miễn trừ có tên đứng trước THẺ | xanh | xanh |
| miễn trừ có tên trải NHIỀU DÒNG | xanh | xanh |

🔴 Hai ca cuối là **lỗi thật của lượt dựng**, sửa tại chỗ: bản đầu chỉ nhìn lại **hai dòng** và không lùi qua thẻ mở, nên miễn trừ **không dùng được ở đúng chỗ người ta viết comment**. Một đường thoát không dùng được chỉ là trang trí.

**Kiểm D — tự kiểm của `check-layout.mjs`** · **17 ca, chạy MỖI LƯỢT CI** *(không phải một bảng chạy tay rồi thôi đúng trong im lặng)*: 11 ca đỏ *(`addPopoutGroup` · `window.open` · `globalThis.open` · `self.open` · `localStorage` · `sessionStorage` · `window.localStorage` · `document.write` · `document.cookie` · khoảng trắng chen giữa `window . open` · tên chỉ GIỐNG)* + 6 đối chứng âm *(comment dòng/khối/HTML nhắc tên · dấu nháy lẻ trong văn xuôi · hai thành viên hợp lệ)*.

**Bảng đóng băng token** — gỡ mục `deviations` của `ui-md-strong` ⇒ `FAIL typography: thừa 1 token KHÔNG có chữ ký — ui-md-strong`.

**Bộ lọc phần mở rộng** — `src/layout/__probe.mts` và `__probe.cts` mang `dispatch("khong.ton_tai")` ⇒ **nay bị Kiểm B bắt**; trước lượt sửa chúng vô hình.

**Sàn quần thể** — di dời `src/panels/` ⇒ `check-i18n` *(5 `.vue` < sàn 9)*, `check-commands` *(5 < 9)*, `check-layout` *(23 < 24)* đều `abort()`. Thêm `src/layout/` ⇒ `check-tokens` cũng `abort()`.

#### 3 · Nghiệm thu THỊ GIÁC + HÀNH VI — **35/35 PASS, 0 console error**

🔴 **ENGINE: Blink/Chromium (Playwright headless) · NỀN TẢNG: macOS 24.6 arm64 · cửa sổ 1400×900.**
**KHÔNG phải WKWebView.** Đừng viết "tương đương" bằng suy luận. Playwright cài trong một venv **ngoài repo** — `package.json` không đụng tới (AC12).

| # | AC | Ca | Đo được |
|---|---|---|---|
| 01 | AC1 | bốn slot panel trong MỘT cửa sổ | 4 |
| 02 | AC6 | preset mặc định | 2 hàng × 2 cột |
| 03 | AC6 | thứ tự đọc | `Nguyên văn \| Bản dịch \| Tra cứu \| Đề xuất AI` |
| 04 | AC6 | `Nguyên văn`/`Bản dịch` cạnh nhau NGANG | ✓ |
| 05 | AC1 | panel lấp đầy chiều cao thật | 431px × 2 hàng |
| 06–08 | AC8 | bốn câu trạng thái · panel AI MỜI cấu hình · 0 câu dùng màu `error` | 4 · ✓ · 0 |
| 09–11 | AC5 | `Mod+Alt+2` ⇒ 4 cột · thứ tự UX-DR13 · `Mod+Alt+1` ⇒ về 2×2 | ✓ |
| 12 | AC5 | `⌘1` VẪN là **chế độ**, không phải preset | chuyển sang Library |
| 13 | AC2 | kéo sash | 700px → **900px** |
| 14 | AC2 | kéo tab vào group khác ⇒ **gộp thành tab** | 3 group, tab/group `[2,1,1]` |
| 15 | AC2 | `Shift`+kéo tab ⇒ **nhóm NỔI** | 1 `.dv-resize-container` |
| 16 | AC1 | không VẪN đúng MỘT cửa sổ | 1 |
| 17×4 | AC3 | ẩn từng panel ⇒ khỏi DOM, ba panel còn lại lấp đầy | 3 group · 3 `.panel` · 1.206.800px² |
| 18×4 | AC3 | hiện lại ⇒ **khớp TỪNG PIXEL** vị trí và kích thước cũ | ✓ ×4 |
| 19 | AC9 | `Mod+Alt+→` ×4 ⇒ hết bốn panel theo **thứ tự bố cục** rồi quay lại | ✓ |
| 20 | AC9 | `Mod+Alt+←` lùi đúng một bước | ✓ |
| 21 | AC9 | ẩn một panel ⇒ vòng còn **ba**, panel đã ẩn không có mặt | ✓ |
| 22–23 | AC9 | đúng MỘT vạch tiêu điểm · vạch 2px `rgb(47,93,99)` = `primary` | ✓ |
| 23b | 1.6-AC4 | sau khi ẩn panel, focus **không** rơi về `body` | "trong panel" |
| 24 | AC9 | rời Workspace rồi quay lại ⇒ không vạch nào nói dối | 0 |
| 25 | 1.4-AC6 | **theme sáng**: nét 1px, khe 0px, bo 0, vỏ nền `rgb(244,241,234)` | ✓ |
| 26 | AC12 | không theme dựng sẵn nào của dockview sót lại | chỉ `dockview-theme-aura` |
| 27 | 1.4-AC6 | **theme tối**: không nét, bo **3px**, nửa khe **1px** mỗi bên | ✓ |
| 28 | 1.4-AC6 | **khe THẬT giữa hai panel = 2px** | 2px |

#### 4 · Nghiệm thu AC4 trong **APP TAURI THẬT** — WKWebView + IPC thật + `global.db` thật

⚠️ Cổng 1420 **rảnh** lần này *(tiến bộ so với Story 1.6, `deferred-work.md:130`)*, nên `npm run tauri dev` chạy được. Kho được **sao lưu trước và khôi phục sau** lượt đo.

| Bước | Làm gì | Kết quả |
|---|---|---|
| 1 | kho **rỗng** → mở app | `app_config/workspace_layout` xuất hiện, **1.273 byte**, root `branch → [branch, branch]`, đủ 4 panel ⇒ **đường ghi frontend → IPC → `store::Writer` chạy thật** |
| 2 | nạp một bố cục **hoán vị** *(cấu trúc do chính app sinh, chỉ đổi tên panel giữa lá 1 và lá 4)* → đóng → mở lại | thứ tự lá **GIỮ NGUYÊN hoán vị** ⇒ **KHÔI PHỤC NGUYÊN TRẠNG**, không rơi về mặc định |
| 3 | ghi đè `{"grid": KHONG-PHAI-JSON` → mở lại | **app VẪN CHẠY**, không cửa sổ trắng, rơi về **preset mặc định 2×2** rồi ghi lại một bố cục hợp lệ |
| 4 | nạp một cây bị cắt tay *(sai hình dạng)* → mở lại | cùng nhánh: app chạy, rơi về mặc định ⇒ nhánh `fromJSON` ném **có lưới thật** |

#### 5 · Nhịp ghi — ĐẾM ĐƯỢC, không đếm tay

`check-layout.mjs` Kiểm B nạp `src/layout/writeSchedule.ts` bằng Node thuần:
- **kéo sash 3 s · 188 sự kiện ⇒ ĐÚNG 1 lượt ghi** *(ở 3.492 ms = sự kiện cuối + idle 500)*;
- **kéo liên tục 20 s · 1.251 sự kiện ⇒ 4 lượt ghi**, **không thay đổi nào chờ quá 5.000 ms**;
- sạch ⇒ không ghi · một thay đổi ⇒ bẩn · sau khi ghi ⇒ sạch lại.

🔴 Lượt dựng cổng bắt được **một mệnh đề sai của chính tôi**: bản đầu khẳng định *"khoảng cách giữa hai lượt ghi ≤ trần"* và cổng đỏ với `5008 ms`. Con số đúng, mệnh đề sai — trần nổ ở mốc 5.000 còn chu kỳ kế tiếp bắt đầu ở **sự kiện tiếp theo** (5.008), và giữa hai mốc đó không có gì chưa ghi. Bất biến đúng là **tuổi của thay đổi chưa ghi**. Đã sửa và ghi lý do vào cổng.

### Completion Notes List

#### Task 0 — SÁU PHÁN QUYẾT

| # | Chốt | Lý do ngắn |
|---|---|---|
| **#1** | **A** — `Mod+Alt+1` / `Mod+Alt+2` cho preset | Giữ "số thứ tự preset" mà mockup dạy, chỉ thêm một phím bổ trợ; `Mod+Alt+3` để trống cho Review Mode (8.11). Khớp bằng `event.code` nên `⌥1` sinh `¡` trên macOS không thành vấn đề. Đo được: `⌘1` **vẫn** là chế độ. |
| **#2** | **A** — `focus.next_panel` = `Mod+Alt+→`, thêm `focus.prev_panel` = `Mod+Alt+←` | Cùng họ phím với #1 nên học một lần. Không đụng `Tab`, `⌥←/→` trần, `⌘⇧…`. |
| **#3** | **A** — `removePanel` + sổ vị trí đã nhớ | `dockview-core@7.0.4` **gỡ `setVisible` khỏi `DockviewPanelApi`** — không có đường "ẩn tại chỗ". `width: 0` thì panel vẫn nhận `Tab` (§Bẫy 2). |
| **#4** | **A** — tab bar của dockview LÀ thanh tiêu đề panel | Một thanh, đúng mockup; gộp-tab của FR17 hiện ngay trên chính thanh đó. `PanelFrame` bỏ `<header>`; vạch tiêu điểm **ở lại** `PanelFrame`. |
| **#5** | **A** — bố cục hiện tại → `ScopeKind::AppConfig` khoá `workspace_layout` | Cùng cửa với `theme`/`mode`, đi qua `store::Writer` (AD-11), không `localStorage`. `__current` trong `layout_presets` bẻ nghĩa *"đã ĐẶT TÊN"*. |
| **#6** | **A** — làm Kiểm A2 ngay ở story này | Bốn panel thật có nhãn thật, nên "đúng" định nghĩa được. Bộ phân tích template đã có sẵn từ Kiểm A. |

#### 🔴 CHỖ TÔI LÀM KHÁC STORY, và vì sao

**AC10 — chữ ký cho token thứ 15 đặt ở `deviations`, KHÔNG ở bảng đóng băng.**
Story bảo thêm hàng `ui-md-strong` vào `EXPECTED_TYPOGRAPHY` của `check-tokens.mjs`. Tôi không làm vậy, vì bảng đó tự khai mình là *"bản chép **ĐỘC LẬP** thứ hai của `DESIGN.md`"* — mà `DESIGN.md` có **14** hàng. Thêm hàng thứ 15 vào đó là để hai bản chép trôi khỏi nhau trong im lặng, và **hai bản chép chỉ bắt được lỗi khi cả hai còn chép cùng một thứ**. Thay vào đó: bảng ở lại đúng 14 hàng, và `compare()` được mở rộng để coi **token THỪA** là một chỗ lệch phải có `deviations` với `question` + `reason` không rỗng. Kết quả vẫn là *"một chữ ký"* như AC10 đòi, chỉ đặt ở chỗ cổng **cưỡng chế được** *(nghiệm thu: gỡ mục deviation ⇒ đỏ)*. Ba chỗ trong cổng vẫn bị sửa như story dặn — chỉ khác chỗ thứ nhất.

#### 🔴 BỐN LỖI THẬT mà lượt nghiệm thu tay bắt được — không cổng nào thấy

1. **Dock cao 0px.** `<style scoped>` biên dịch `.dock` thành `.dock[data-v-xxx]`, nhưng phần tử gốc của `<DockviewVue>` **không nhận thuộc tính scope**. Luật `height: 100%` không bao giờ khớp ⇒ dockview đo container rỗng, tự chọn 100px, và bốn panel hiện ra cao **100px** trong cửa sổ 900px. **Mọi cổng đều xanh.** Sửa: một `<div class="dock-host">` **của tệp này** giữ chiều cao, con nhận qua `:deep()`.
2. **dockview tự dán `dockview-theme-abyss`.** Khi không ai truyền prop `theme`, dockview đặt lớp đó lên `.dv-shell` — **nằm TRONG** phần tử mang `.dockview-theme-aura`. Custom property kế thừa theo phần tử gần nhất ⇒ **bảng màu abyss thắng**. Sản phẩm âm thầm chạy một bảng màu thứ hai chưa ai kiểm tương phản, và `check:tokens` **vẫn xanh** vì nó chỉ quét `src/**`. Đúng §Bẫy 1. Sửa: truyền `:theme="auraTheme"`; ca 26 của bảng nghiệm thu canh mệnh đề đó.
3. **`rememberSpot` chọn sai neo.** Bản đầu dùng `adjacentGroupInDirection()` — hàng xóm **trên MÀN HÌNH**, không phải anh em **trong CÂY lưới**. Ở lưới 2×2 thật, `Tra cứu` có hai hàng xóm hình học và chỉ một là anh em thật. Hệ quả đo được: hiện lại `Tra cứu` cắt đôi ô của `Đề xuất AI` ở góc **dưới-phải** thay vì trả nó về góc **dưới-trái**. Sửa: đọc cây từ `api.toJSON()`, leo ngược tìm nhánh còn anh em, rồi lấy hướng từ **hình học thật** (không suy ra từ `Orientation`, thứ đảo ở mỗi tầng lồng).
4. **Focus rơi về `body` sau mỗi lượt ẩn/hiện panel.** `removePanel` gỡ đúng phần tử đang giữ focus; `addPanel` tái cấu trúc group nên dockview đỗ rồi dựng lại DOM. Đo được chữ `BODY` ở cả hai đường. Vi phạm **AC4 của Story 1.6**, và **không chốt nào kêu**: `armBodyGuard` chỉ chạy sau một `enter()`, mà ở đây không ai gọi `enter()`. Một lỗ **im lặng giữa hai cơ chế đều đúng**. Sửa: `restoreFocusIfLost()` — chỉ can thiệp khi focus THẬT SỰ đã mất.

Cộng một lỗi thứ năm về **chẩn đoán ồn**: `onDidActivePanelChange` bắn cả khi chính ta `addPanel`, lúc component Vue chưa `declareFocus()` ⇒ **hàng chục** dòng *"chưa khai điểm vào"* mỗi lượt dựng bố cục. Nửa nặng hơn: nó **cướp focus** khỏi chỗ người dùng đang đứng ở mỗi lượt `fromJSON`. Sửa: chỉ hành động khi `origin === 'user'` — *"CHUYỂN panel"* của AD-34 §2 là thao tác của **người**. Sau lượt sửa: **0 console error** trong toàn bộ 35 ca.

#### NHỮNG GÌ STORY NÀY CỐ Ý KHÔNG LÀM

- **Bốn ngưỡng màn hình hẹp** — `epics.md:1617` cấm tường minh. **0 `matchMedia` trong toàn `src/**`**; `nextToSacrifice()` là hàm thuần không đọc kích thước cửa sổ. Story 4.12 **chỉ phải nối ngưỡng vào**.
- **Vế "Tra cứu rút về thanh trạng thái"** và **ngăn kéo** — Story 4.12.
- **Preset Review Mode** — Story 8.11; `Mod+Alt+3` để trống.
- **Bốn `layout.toggle_*` không gán phím** — lỗ NFR17 mở ra **có ý thức** để `unbound()` giữ được bằng chứng cho AC6 của Story 1.6. Chủ: Story 1.21.
- **Thanh chuyển phạm vi cho preset** — `kinds.rs:36` gọi tên đích danh cái bẫy đó.
- **Nội dung panel** — 1.16 / 1.17 / Epic 2 / Epic 4. Thân panel để trống, nhưng **không câm**: mỗi panel nêu trạng thái bằng chuỗi `vi.json`.
- **0 phụ thuộc npm/crate mới** — `check:deps` xác nhận cây không đổi. Playwright của lượt đo sống trong một venv **ngoài repo**.
- **Không đụng** `tauri.conf.json` · `Cargo.toml` · `[profile.release]` · `dict-manifest.toml` · `tools/**` · `core/dict/**` · `core/matching/**`. **CSP không nới** — `dockview.css` bundle được qua Vite thành một tệp `'self'`, và đường popout *(chỗ duy nhất tạo `<style>` lúc chạy)* bị `check:layout` cấm.
- **Không sửa** `epics.md` · `prd.md` · `DESIGN.md` · `EXPERIENCE.md` · `mockups/**`. **Hai chỗ lệch còn nguyên và đã ghi ra**: mockup vẫn vẽ `⌘1`/`⌘2` cho preset, và `DESIGN.md` vẫn ghi **14** token.

#### ⚠️ NHỮNG GÌ CHƯA ĐO ĐƯỢC — bàn giao đích danh

- **Vế THỊ GIÁC trên WKWebView** và **ca Windows** — bảng 35 ca chạy trên **Blink**. Lượt `tauri dev` nghiệm thu **AC4** trong WKWebView thật, nhưng không lái được cửa sổ native nên bố cục/khe/kéo-thả/vòng focus chưa đo ở đó. → **Story 1.3 / 10.9**.
- **`ui-md` giãn dòng 1.5 nhưng `panel.ai_translation.status` XUỐNG DÒNG THẬT** (96 ký tự, hai dòng ở panel 700px) — dưới sàn 1.66. Ba đường ra, **chưa chốt**, chạm `DESIGN.md` ⇒ **quyết định của Ice**. → **Story 1.16/1.17**.
- **Kiểm B đo nhịp ghi nhưng không đo chỗ NỐI** vào `WorkspaceDock.vue`.
- **`PANEL_COMPONENTS` ↔ map `components`** không cổng nào đối chiếu.
- **Ba biến `--dv-*`** mang tên không khớp thuộc tính mà Kiểm D/F đọc — đặt đúng luật **bằng tay**, chưa cưỡng chế.
- **Cây làm việc lúc bắt đầu KHÔNG sạch** — công việc Story 1.13 chưa commit, Ice chưa xác nhận.

### File List

**Mới — 11**

| Tệp | Vai |
|---|---|
| `src/layout/workspaceLayout.ts` | tầng THUẦN: `PANEL_IDS` · hai preset · `SACRIFICE_ORDER` · `nextToSacrifice()` |
| `src/layout/writeSchedule.ts` | tầng THUẦN: nhịp ghi idle + trần cứng, `simulateWrites()` |
| `src/layout/dockController.ts` | cổng nối `CommandRegistry` ↔ dockview đang sống |
| `src/layout/panelProps.ts` | hình dạng prop mà dockview-vue mount |
| `src/layout/dockview-theme.css` | `.dockview-theme-aura` — ~50 biến `--dv-*` từ token |
| `src/layout/WorkspaceDock.vue` | vỏ dockview: preset · ẩn/hiện · focus · lưu/khôi phục |
| `src/panels/PanelTab.vue` | tab của dockview = thanh tiêu đề panel |
| `src/panels/SourcePanel.vue` · `LookupPanel.vue` · `AiTranslationPanel.vue` · `EditorPanel.vue` | bốn panel |
| `scripts/check-layout.mjs` | cổng thứ **năm** — AC1 · AC4 · AC7 · AC12 |

**Sửa — 15**

`src/main.ts` · `src/App.vue` · `src/modes/WorkspaceMode.vue` · `src/panels/PanelFrame.vue` · `src/commands/index.ts` · `src/commands/focus.ts` · `src/config/bootstrap.ts` · `src/i18n/vi.json` · `src/tokens/tokens.json` · `scripts/check-tokens.mjs` · `scripts/check-i18n.mjs` · `scripts/check-commands.mjs` · `package.json` · `src-tauri/src/core/scope/store.rs` · `src-tauri/src/commands/config.rs` · `src-tauri/tests/ipc_contract.rs` · `src-tauri/tests/scope_contract.rs`

**Tài liệu** — `_bmad-output/implementation-artifacts/deferred-work.md` · `sprint-status.yaml` · story file này.

### Review Findings

*(điền sau code review)*
