---
baseline_commit: 564be15cfe9142ae8c514ce8f64ced5ea2df4a2b
---

# Story 1.16: Panel Source và tab Hán Việt

Status: done

> 🔴🔴 **STORY NÀY BỊ CHẶN — ⛔ KHÔNG BẮT ĐẦU.** Ice chốt 2026-08-06:
> **`1-10c-am-han-viet-dung-nguon-va-dung-nhan` phải `done` trước.**
> Lý do: Task 0 của chính story này đo `dict-core.db` thật và phát hiện cột
> `dict_entry.han_viet` của **lớp NỀN** ⛔ không mang âm Hán Việt — nó mang **âm Nôm**
> (`Unihan kVietnamese`). Kiểm chứng độc lập trên en.wiktionary: **92,4 %** âm Unihan trùng
> một âm đã gắn nhãn `nom-reading`, chỉ **22,7 %** trùng một âm `han-viet-reading`. Dựng tab
> Hán Việt trên dữ liệu đó là dựng một màn hình **nói sai**, rồi phải mổ lại khi tầng dưới sửa.
>
> ✅ **Ba quyết định đã chốt 2026-08-06** — **#1** *(nguồn: Thiều Chửu + en.wiktionary(vi) +
> Trần Văn Chánh; TVC là **lớp gỡ rời**)* · **#4 (a)** *(song song = **cặp theo từng ký tự**)*
> · **#6 (a)** *(**token thứ 16** cho nguyên văn Latin, qua sổ `deviations`)*.
> ⬜ Còn **#2 · #3 · #5 · #7** — chốt ở Task 0; cả bốn có mặc định đề xuất kèm lý do.

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-06 | **Triển khai xong, Status → review.** Cả 11 Task hoàn tất, cả chín cổng DoD xanh ở lượt chạy cuối. Tầng dữ liệu: method thứ ba `han_viet()` trên `DictionarySource` (đọc theo lô, phủ headword+headword_simp) + tầng gom `lookup_han_viet` (ưu tiên lớp gỡ rời trước lớp nền, tách nhiều âm một luật, ba trạng thái AC4) — 40+ test hành vi mới trên fixture ba lớp của Story 1.13, tái dùng nguyên vẹn. Tầng IPC: `read_open_chapter` (AC8) + vá `replace_open_work` thả `Store` ngoài khoá (AC10, kiểm bằng một `ReentrantProbe` tự khoá lại trong `Drop`) + một command MỚI `read_han_viet` (quyết định kỹ thuật không có trong Project Structure Notes của story — xem Completion Notes #1). Tầng giao diện: token thứ 16 `source-latin`; `PanelFrame` prop `show-status`; `SourcePanel.vue`/`SourceHanViet.vue` mới, state module-level `sourcePanelState.ts` (AC9); ba command CommandRegistry có phím (`source.select_tab_original/han_viet`, `source.toggle_han_viet_view`). 🔴 **Một lỗi thật bắt được bằng đo Playwright THẬT** (⛔ không lời hứa): CSS `inline-flex column` ban đầu cho `.hv-unit` làm Chromium chèn `\n` vào `window.getSelection().toString()` ở AC6 — vá bằng `position: absolute` cho `.hv-reading`, đo lại XANH. Trần render kiểu song song chốt **50.000 ký tự Hán** từ bảng số đo thật (5k/50k/500k × 2 kiểu xem). Ba mục ⛔ chưa đo được trong phiên: vế thị giác hai nền tảng thật (WKWebView/WebView2 — món nợ cũ), AC9 chưa xác nhận bằng webview thật đang chạy, và phép đo Task 8 chạy trên Chromium chứ không phải engine mục tiêu — cả ba ghi rõ ở Completion Notes, ⛔ không đánh dấu đạt rồi im. |
| 2026-08-06 | **Task 0 đóng, story hết chặn.** Đối chiếu `1-10c` (Status: done, chưa commit): `SUPPORTED_SCHEMA_VERSION = 2` (`core/dict/layer.rs:53`), `dict-manifest.toml` có `[[detachable]] name = "tran-van-chanh"`, `cargo test` xanh cả hai workspace (`src-tauri` + `tools/dict-build`), 8/9 cổng DoD xanh (`check:dict-manifest` · `check:i18n` · `check:deps` · `check:tokens` · `check:layout` · `check:commands` · `check:dict` · `npm run build`); `check:scope` KHÔNG chạy được vì cổng 1420 bị chiếm bởi một `tauri dev` khác đang chạy — chạy lại ở Task 10. **Chốt bốn quyết định còn lại theo đúng mặc định đề xuất, không có bất đồng nào phát sinh:** **#2** — (a) `han_viet(&self, chars: &[&str])` là method thứ ba trên `DictionarySource`, đọc theo LÔ. **#3** — (a) tách nhiều âm bằng một luật duy nhất (cắt trên `\|` và khoảng trắng) áp cho mọi tệp; hiện âm ĐẦU TIÊN, danh sách đầy đủ vẫn đi qua IPC; ⛔ không đánh dấu UI cho ca nhiều âm (theo mặc định của §Câu hỏi #2 — chưa có chữ ký UX). **#5** — (a) dải tab `Trung`/`Hán Việt` ở đầu THÂN panel Source; `PanelFrame.vue` nhận prop boolean `show-status` (mặc định `true`), `status-key` ở lại literal ở cả bốn panel. **#7** — trần render là một hằng CÓ TÊN, hiện bằng chuỗi `vi.json`, con số chốt ở Task 8 từ phép đo thật (KHÔNG đoán trước). Ký tự không có âm ở bất kỳ lớp nào: dùng chuỗi `vi.json` riêng (khác với ca "0 lớp gắn"), ⛔ không ô trống câm, ⛔ không `ornament`/`opacity` (theo đề xuất §Câu hỏi #3). |
| 2026-08-06 | 🔴 **Ice chốt ba quyết định, và story BỊ CHẶN.** **#1** — nguồn Hán Việt = Thiều Chửu + en.wiktionary(vi) + Trần Văn Chánh, TVC làm **lớp gỡ rời**; `Unihan kVietnamese` **đổi vai** thành âm Nôm chứ ⛔ không xoá. **#4 (a)** — *song song* = **cặp theo từng ký tự**. **#6 (a)** — **token thứ 16** qua sổ `deviations`. **Thứ tự:** tầng dữ liệu **trước**, giao diện **sau** ⇒ story mới **`1-10c-am-han-viet-dung-nguon-va-dung-nhan`** nhận toàn bộ việc tầng dữ liệu, và 1.16 chờ nó `done`. Cơ sở: bốn phép đo mới trong ngày — 92,4 % âm Unihan trùng một âm `nom-reading` đã gắn nhãn; 79,7 % ký tự "chỉ Unihan có" nằm ở **CJK Ext B** (chữ Nôm, ⛔ không tồn tại âm Hán Việt); Thiều Chửu đơn độc phủ **97,9–100 %** văn xuôi thật; Thiều Chửu ∩ Trần Văn Chánh trùng âm đầu **94,9 %**. |
| 2026-08-06 | Tạo story. Baseline `564be15`, cây làm việc **sạch**. Phân tích: `epics.md` §Story 1.16 + Epic 1/3 (ràng buộc xuôi dòng FR50/FR113) · `ARCHITECTURE-SPINE.md` 856 dòng · `prd.md` FR19/FR33 · `DESIGN.md` §Typography + bảng 14 token · `EXPERIENCE.md` · `mockups/key-screen-workspace.html` · story `1-14`, `1-15`, `1-13`, `1-4` · **toàn bộ `deferred-work.md` 544 dòng** (5 mục gọi đích danh story này) · mã thật `src-tauri/src/**` + `src/**` + `scripts/*.mjs`. 🔴 **Và đo thật trên ba tệp `.db` đã dựng** (`tools/dict-build/out/`) — bốn phép đo ở §Quyết định #1 là thứ ⛔ không tài liệu quy hoạch nào có. Phát hiện: **1 lỗ hổng dữ liệu tầng PRD** *(Unihan `kVietnamese` ⛔ không phải Hán Việt thuần)*, **1 lỗ hổng bảng token** *(⛔ không có token cho nguyên văn TIẾNG ANH, trong khi FR19 phủ cả tiếng Anh)*, **1 đường IPC chưa tồn tại** *(⛔ không có cách nào đọc `chapter.source_text` từ webview)*, **7 quyết định phải chốt ở Task 0**. |

**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story key:** `1-16-panel-source-va-tab-han-viet`
**Covers:** **FR19** *(Panel Source hiển thị văn bản gốc Anh **hoặc** Trung, kèm tab Hán Việt cho tài liệu tiếng Trung — chuyển đổi hoặc song song)* · **FR33** *(tab Hán Việt hiển thị âm cho **từng ký tự**)*
**Governed by:** **AD-1** *(quy tắc nghiệp vụ ở Rust; webview chỉ render + giữ state UI)* · **AD-2** *(đúng ba cổng — `DictionarySource` là cổng, ⛔ không thêm cổng thứ tư)* · **AD-10** *(mỗi lớp gỡ rời một tệp `.db`; **runtime ⛔ không có mã riêng cho từng nguồn**)* · **AD-16** *(nội dung ngoài ⛔ không bao giờ render thành HTML — ⛔ không `v-html`)* · **AD-19** *(⛔ không tồn tại bước hợp nhất nguồn từ điển)* · **AD-21 / NFR16** *(Rust ⛔ không bao giờ trả văn bản hiển thị)* · **AD-34** *(mọi thao tác qua `CommandRegistry`; mỗi panel khai điểm vào focus; ⛔ cấm màu viết thẳng)* · AD-25 *(dữ liệu từ điển là artifact có checksum)* · AD-30 *(lược đồ có phiên bản)*
**UX-DR phải tôn trọng:** **UX-DR12** *(phân vai hai họ chữ tuyệt đối — nguyên văn và Hán Việt đều là **nội dung**, họ `read`/`read-cjk`, ⛔ không `ui`)* · **UX-DR10** *(giãn dòng 1.66 là sàn cứng cho chữ nội dung)* · **UX-DR5** *(`ornament` là màu của **nét**, ⛔ không bao giờ là màu của chữ)* · **UX-DR6** *(`opacity` ⛔ không dùng để làm mờ chữ)* · UX-DR8/UX-DR17 *(hợp đồng thị giác tiêu điểm — **⛔ đã dựng ở 1.6/1.14, ⛔ không đụng**)* · UX-DR16 *(⛔ không elevation)*
**Ràng buộc xuôi dòng phải để lại chỗ đứng:** **FR50 / Story 3.4** *(đánh dấu thuật ngữ Glossary **trong Panel Source** — story này ⛔ không cài, nhưng ⛔ không được dựng cây DOM khiến 3.4 phải mổ lại)* · **FR113 / Story 3.7** *(đề xuất bản dịch bằng âm Hán Việt — **đọc qua cổng `DictionarySource`, ⛔ không cài lại**, `ARCHITECTURE-SPINE.md:435`)* · **FR21 / Story 1.18** *(Auto-Lookup gắn vào **hợp đồng vùng chọn dùng chung cho mọi panel văn bản**)* · **FR20 / Epic 2** *(Sync Scrolling)* · **FR42 / Epic 6** *(ảnh nhúng đúng vị trí trong Panel Source)* · **AD-4 / Story 2.1** *(⛔ không tách segment)*
**NFR:** NFR14 *(hai nền tảng)* · NFR15 *(⛔ 0 phụ thuộc mới)* · NFR16 *(chuỗi ở `vi.json`)* · NFR17 *(mọi thao tác làm được bằng bàn phím)* · NFR13 *(ngoại tuyến)* · *(chuẩn bị chỗ đứng cho)* NFR1
**Ngày tạo:** 2026-08-06

---

## 🔴 ĐỌC TRƯỚC TIÊN — SÁU VIỆC STORY NÀY ⛔ KHÔNG LÀM

### ① ⛔ KHÔNG dựng Auto-Lookup, ⛔ không dựng hợp đồng vùng chọn

`epics.md` Story 1.18 nói rõ Auto-Lookup **gắn vào một hợp đồng vùng chọn dùng chung cho mọi
panel văn bản**, và Panel AI Translation + Editor phải nhận **cùng hành vi** ở các epic sau
*"⛔ không cần cài lại"*. Đó là một trừu tượng phải thiết kế **một lần cho bốn panel** —
dựng nó ở đây, khi mới có **một** panel văn bản, là thiết kế nó trên một mẫu duy nhất.

⇒ Story này chỉ phải bảo đảm **một điều**, và nó là điều kiện tiên quyết của 1.18: văn bản
nguyên văn phải **bôi đen được bằng chuột và bằng bàn phím**, và **ranh giới ký tự trong
DOM phải trùng ranh giới ký tự trong dữ liệu** *(xem Bẫy 3 — đây là chỗ chế độ **song song**
có thể phá 1.18 trong im lặng)*.

### ② ⛔ KHÔNG đánh dấu thuật ngữ Glossary — FR50 là Story 3.4

⛔ Không một lớp highlight, một `<mark>`, hay một cấu trúc "khoảng có kiểu" nào. Mockup
`key-screen-workspace.html:100` **có** vẽ `<mark>打</mark>` — đó là hình ảnh của **sản phẩm
hoàn chỉnh**, ⛔ không phải phạm vi story này.

🔴 **Nhưng ràng buộc phải để lại:** Story 3.4 sẽ cần bọc **những khoảng ký tự tuỳ ý** trong
nguyên văn. ⇒ ⛔ **Đừng** dựng nguyên văn thành một cây DOM mà mỗi ký tự là một node độc lập
**không có** đường gom lại thành khoảng — xem **Quyết định #4** và **Bẫy 3**.

### ③ ⛔ KHÔNG tách segment, ⛔ không tách câu, ⛔ không tách Chương

AD-4: ranh giới segment tính **một lần lúc nhập** — Story 2.1. `deferred-work.md:508` ghi
thẳng: mọi Chương nhập ở Epic 1 có `segment_count = 0`, `chapter.source_text` mang **nguyên
khối** văn bản. Story này hiển thị **nguyên khối đó**, ⛔ không chia câu, ⛔ không chia đoạn
bằng một luật của riêng nó.

⚠️ **Ngoại lệ duy nhất và nó ⛔ không phải một luật tách:** ngắt dòng (`\n`) trong văn bản
nguồn phải hiện ra **thành ngắt dòng trên màn hình**. Đó là trình bày một ký tự đã có trong
dữ liệu, ⛔ không phải suy ra một ranh giới mới. ⚠️ Và `deferred-work.md:527` cảnh báo: văn
bản nhập từ tệp Windows mang `\r\n` — `\r` **⛔ không được** hiện thành một ô trống.

### ④ ⛔ KHÔNG cài lại việc đọc từ điển — mọi đường đi qua cổng `DictionarySource`

`ARCHITECTURE-SPINE.md:435` nói cho FR113: *"Âm Hán Việt … đọc **qua cổng `DictionarySource`**,
⛔ không cài lại."* Mệnh đề đó áp cho story này **trước tiên**, vì đây là story **đầu tiên**
đọc `dict_entry.han_viet`. ⛔ Không `Connection::open`, ⛔ không gõ tên `rusqlite` ngoài
`core/store/**` *(`store_boundary.rs::only_core_store_may_name_rusqlite` cưỡng chế)*.

⛔ **Và ⛔ không thêm cổng thứ tư** (AD-2). Thứ story này thêm là **một method trên cổng đã
có** — xem Quyết định #2.

### ⑤ ⛔ KHÔNG mở màn hình "mở lại một `.atproj` đã có"

`deferred-work.md:533` ghi: `WorkMeta::read` hôm nay ⛔ **không có một chỗ gọi sản phẩm nào**,
và `err.project.meta_too_new` + `MessageKey::ProjectMetaTooNew` **đã bị gỡ** vì lý do đó.
Story này đọc Tác phẩm **đang mở trong `OpenWorkState`** — thứ `create_work_from_*` vừa đặt
vào — ⛔ **không** dựng đường mở lại từ đĩa. ⛔ **Đừng** thêm lại ba thứ đã gỡ.

### ⑥ ⛔ KHÔNG đụng ngưỡng màn hình hẹp, ⛔ không đụng `applyPreset` ngoài đúng một chỗ

Bốn ngưỡng UX-DR15 thuộc **Story 4.12** (`deferred-work.md:472`). Story này ⛔ không đọc
`window.innerWidth`, ⛔ không `matchMedia` — cây `src/**` hôm nay có **0** lời gọi và con số
đó phải giữ nguyên.

⚠️ **Đúng một chỗ được đụng:** `deferred-work.md:504` — `applyPreset()` `api.clear()` rồi dựng
lại **toàn bộ** bốn panel, và mục đó giao đích danh *"Story 1.16 / 1.17 / Epic 2, nơi panel
lần đầu có trạng thái đáng giữ"*. **Đây chính là story đó.** Xem **AC9**.

---

## Story

As a người dịch tiếng Trung,
I want thấy âm Hán Việt của từng ký tự ngay cạnh nguyên văn,
So that tôi đọc được văn bản mà ⛔ không phải tra từng chữ một.

---

## Ranh giới phạm vi — ĐỌC TRƯỚC KHI GÕ DÒNG ĐẦU TIÊN

| Trong phạm vi | ⛔ Ngoài phạm vi (và ai sở hữu) |
|---|---|
| Đường IPC đọc **Chương đang mở** từ `OpenWorkState` | Chọn Chương, chuyển Chương *(Epic 2)* · mở lại `.atproj` từ đĩa *(Epic 5)* |
| Method **thứ ba** trên cổng `DictionarySource` — đọc `dict_entry.han_viet` theo **lô ký tự** | Cổng thứ tư *(AD-2 — ⛔ cấm)* · đường đọc `dict_sense` *(đã có ở 1.13)* |
| Tầng gom âm Hán Việt qua **nhiều lớp**, thứ tự đọc là **quyết định của tầng gom** | Bật/tắt từng nguồn *(Story 1.19)* · màn hình Attribution *(Story 10.4)* |
| Nội dung thân Panel Source: nguyên văn `source-cjk` · tab Hán Việt · hai kiểu xem | Nội dung Panel Lookup *(1.17)* · Editor *(Epic 2)* · AI *(Epic 4)* |
| Token cho nguyên văn **tiếng Anh** *(lỗ hổng bảng token — Quyết định #6)* | Đường tra cứu tiếng Anh *(đã có ở 1.11b)* |
| `font-synthesis: var(--synthesis-source-hanviet)` — người tiêu thụ **đầu tiên** của lời giải chữ Hán nghiêng giả *(`deferred-work.md:133`)* | Đổi bảng token màu · đổi cờ `wraps` của `ui-md` *(⛔ quyết định của Ice, `deferred-work.md:115`)* |
| Giữ trạng thái panel qua một lượt đổi preset *(`deferred-work.md:504`)* | Ngưỡng màn hình hẹp *(Story 4.12)* · preset người dùng đặt tên *(1.21)* |
| Vá `replace_open_work` thả `Store` **ngoài** vùng khoá *(`deferred-work.md:525` — story này **là** ca mà lượt review dự báo)* | Sáu số `Tuning` *(Story 2.4)* |

**⛔ KHÔNG ĐỤNG** *(ranh giới đã chốt năm story liên tiếp)*: `tools/**` · `dict-manifest.toml`
· `src-tauri/capabilities/main.json` *(thêm permission ⇒ **phải là một AD mới trước đã**)* ·
`package.json` *(⛔ **0 phụ thuộc npm mới** — NFR15; xem Bẫy 6 về vì sao cám dỗ ở đây rất mạnh)*
· `src-tauri/Cargo.toml` *(⛔ chốt lần thứ **năm** — `deferred-work.md` [D4])* · `[profile.release]`
· `_bmad-output/planning-artifacts/**` *(`epics.md` · `prd.md` · `DESIGN.md` · `EXPERIENCE.md` ·
`mockups/**` — **lệch thì GHI RA, ⛔ không sửa**; tiền lệ quyết định #3 Story 1.3)*.

**✅ ĐỤNG ĐÃ ĐƯỢC PHÉP** *(và ⛔ chỉ khi Ice chốt Quyết định tương ứng)*:
`src/tokens/tokens.json` + `scripts/check-tokens.mjs` §`deviations` *(token thứ 16 — Quyết định
#6, theo **đúng** tiền lệ `ui-md-strong` của Story 1.14)* · `src/panels/PanelFrame.vue` *(một
prop mới — Quyết định #5)* · các hằng `*_FLOOR` của bốn script cổng *(AC11)*.

---

## 🔴 BẢY QUYẾT ĐỊNH — PHẢI CHỐT Ở TASK 0, TRƯỚC DÒNG MÃ ĐẦU TIÊN

> Khuôn Task 0 của Story 1.13 / 1.14 / 1.15. Mỗi mục có **mặc định đề xuất** kèm lý do; ⛔
> không mục nào được cài theo cảm tính. **#1, #4, #6 chặn thật** — chúng đổi hình dạng dữ
> liệu hoặc hình dạng màn hình.

---

### ✅ Quyết định #1 — ĐÃ CHỐT 2026-08-06. **Nguồn của âm Hán Việt**

> 🔴 **PHÁN QUYẾT CỦA ICE:** nguồn Hán Việt = **Thiều Chửu + en.wiktionary(vi) + Trần Văn
> Chánh**; **Trần Văn Chánh đóng gói làm LỚP GỠ RỜI riêng** *(FR36: gỡ = xoá một tệp)*.
> Và `Unihan kVietnamese` ⛔ **không bị xoá** — nó **đổi vai** sang đúng bản chất là **âm Nôm**.
>
> 🔴 **Việc đó thuộc `1-10c`, ⛔ KHÔNG thuộc story này.** Story 1.16 nhận dữ liệu **đã sạch**:
> cột `han_viet` mang **đúng một** ngữ nghĩa ở **mọi** tệp, phủ **12.463** ký tự
> *(Thiều Chửu 9.897 ∪ TVC 12.081 ∪ en.wikt 1.136)*. ⇒ Tầng gom của Task 3 ⛔ **không** còn
> phải phòng thủ trước một lớp nền nói dối; nó chỉ còn làm đúng một việc: **thứ tự ưu tiên
> theo lớp**, cộng **đánh dấu nguồn cho mỗi ký tự**.
>
> Phần dưới đây giữ nguyên để dev đọc được **vì sao**, ⛔ không phải để cân nhắc lại.

#### *(bản gốc — lý lẽ dẫn tới phán quyết)*

#### Số đo — chạy trên ba tệp `.db` THẬT ở `tools/dict-build/out/`, 2026-08-06

| Phép đo | Số |
|---|---|
| `dict-core.db` — đầu mục có `han_viet` | **8.306** — và **100 % đến từ nguồn `unihan`** *(năm nguồn nền còn lại: `NULL` toàn bộ)* |
| `dict-core.db` — tổng đầu mục `unihan` | 49.870 ⇒ chỉ **16,7 %** có âm |
| `dict-core.db` — dạng **giản thể** tra được qua `headword_simp` | +**1.277** ⇒ hợp **9.412** ký tự |
| `dict-thieu-chuu.db` — đầu mục một ký tự có `han_viet` | **9.897** *(100 % đầu mục của tệp, 100 % nằm trong khối CJK cơ bản)* |
| Giao của hai nguồn | **3.239** ký tự |
| 🔴 **Trong phần giao — hai nguồn cho âm ĐẦU KHÁC NHAU** | **1.243 = 38,4 %** |
| Hợp của hai nguồn | **15.233** ký tự *(Thiều Chửu thêm **5.821** ký tự mà lớp nền ⛔ không có; lớp nền thêm **5.067** mà Thiều Chửu ⛔ không có)* |

#### 🔴 Phát hiện — `Unihan kVietnamese` ⛔ KHÔNG PHẢI âm Hán Việt thuần

Mẫu đối chiếu, cột trái là lớp **NỀN** (`unihan`), cột phải là **Thiều Chửu — Hán Việt Tự Điển**:

| Chữ | `unihan` | `thieu-chuu` | Nhận xét |
|---|---|---|---|
| 繭 | `kén` | `kiển` | `kén` là **âm Nôm/thuần Việt**, ⛔ không phải Hán Việt |
| 抉 | `khoét` | `quyết` | cùng lớp lỗi |
| 蓉 | `rong` | `dong` | cùng lớp lỗi |
| 女 | `nữa` | `nữ\|nứ\|nhữ` | lớp nền **sai** ở một chữ sơ đẳng |
| 死 | `tợ tử` | `tử` | lớp nền trả **hai âm** trong một chuỗi, âm đầu là Nôm |
| 🔴 **北** | **⛔ KHÔNG CÓ** | `bắc` | — |

🔴 **Hệ quả trực tiếp, và nó vượt khỏi story này:** `EXPERIENCE.md:410` giải thích FR113
bằng đúng ví dụ **`北涼` → *Bắc Lương***. Với **chỉ** lớp nền, `北` trả **rỗng** ⇒ ví dụ trụ
cột của FR113 ⛔ **không chạy được**. Story 3.7 sẽ dựng trên đúng dữ liệu này.

#### Phủ trên câu thật — đo trên hai câu của `mockups/key-screen-workspace.html`

| Văn bản | Lớp nền, tra thẳng | Lớp nền + `headword_simp` | Thiều Chửu |
|---|---|---|---|
| **Phồn thể** (45 ký tự riêng) | **44/45** *(thiếu 鏽)* | 44/45 | — |
| **Giản thể** (32 ký tự riêng) | **19/32 = 59 %** | **31/32** *(thiếu 锈)* | **32/32** |

⚠️ **Đọc đúng con số 59 %:** người dùng mục tiêu dịch **cả** truyện mạng **giản thể** lẫn cổ
văn phồn thể *(`DESIGN.md` §Typography, lý do chọn biến thể TC)*. Một tab Hán Việt trống
**41 %** trên văn bản giản thể ⛔ không nghiệm thu được mệnh đề *"âm Hán Việt cho **từng** ký
tự"* của FR33.

#### Ba đường, và vì sao hai đường bị loại

| | Đường | Phán quyết |
|---|---|---|
| **(A)** | **Thiều Chửu trước → lớp nền (Unihan) sau**, thứ tự do **tầng gom** quyết | ✅ **MẶC ĐỊNH ĐỀ XUẤT** |
| (B) | Chỉ lớp nền | ⛔ **Loại** — 北 rỗng · 38,4 % lệch · 59 % phủ trên giản thể. Nó cho ra một tab **sai một cách im lặng**, đúng lớp lỗi đắt nhất |
| (C) | Hiện **cả hai** âm cạnh nhau, theo tinh thần AD-19 | ⛔ **Loại** — xem lý lẽ ngay dưới |

**Vì sao (C) bị loại, và vì sao ⛔ nó không vi phạm AD-19:** AD-19 cấm *"bước **hợp nhất
nghĩa** giữa các nguồn"* — nó nói về `dict_sense`, về **định nghĩa**. Âm Hán Việt là một
**âm đọc** nằm ở `dict_entry.han_viet`, và `schema.rs:41` đã tách bạch chuyện đó bằng chữ:
*"`han_viet` là **ÂM ĐỌC**, ⛔ không phải NGHĨA — trộn hai thứ vào `dict_sense` làm Panel
Lookup hiện âm đọc như một định nghĩa."* **Chọn một nguồn theo thứ tự ưu tiên là một phép
CHỌN, ⛔ không phải một phép HỢP NHẤT.** Còn hiện hai âm cho **mỗi** ký tự thì phá chính
thứ tab này tồn tại để làm: đọc trôi một dòng.

🔴 **Ba mệnh đề bắt buộc đi kèm (A) — ⛔ không được bỏ mệnh đề nào:**

1. **Thứ tự đọc sống ở TẦNG GOM, ⛔ không trong adapter.** Cùng doctrine với `route`/`branch`
   của Story 1.11/1.13: *"adapter ⛔ không tự phân xử lại một câu hỏi thuộc về cả lượt tra"*
   (`dict_source.rs`). ⛔ **Và ⛔ không được viết cứng chuỗi `"thieu-chuu"` trong tầng gom**
   — AD-44 ① vá A2 cấm *"một sổ tệp `.db` nào chứa gì"*. Hình dạng đúng: một **thứ tự ưu
   tiên theo `layer()`** đọc từ chính tập lớp đang gắn, có một quy tắc phát biểu được
   *(mặc định đề xuất: **lớp gỡ rời trước lớp nền**, và trong nhóm lớp gỡ rời thì theo thứ
   tự `DictLayers::layers()` đã ổn định)*. Lý do quy tắc đó đúng ⛔ không phải "Thiều Chửu
   giỏi hơn": lớp nền là **nguồn tổng hợp máy** (Unihan), lớp gỡ rời là **từ điển Hán Việt
   do người biên soạn** — quy tắc phát biểu được là *"từ điển chuyên đề đứng trước dữ liệu
   tổng hợp"*.
2. **FR36 phải nghiệm thu được, và nó nghiệm thu ở mức DEGRADATION.** Xoá `dict-thieu-chuu.db`
   ⇒ tab Hán Việt **vẫn chạy**, vẫn trả âm từ lớp nền, ⛔ không một đường nào hỏng. Phủ giảm
   và chất lượng giảm — điều đó **phải hiện ra bằng số trong Completion Notes**, ⛔ không
   được đánh dấu đạt rồi im.
3. **Mỗi ký tự mang theo NGUỒN của âm nó đang hiện.** Bắt buộc vì hai lý do: (a) FR31 nói
   nhãn nguồn là bắt buộc trên mọi bản ghi; (b) **Story 3.7 (FR113)** sẽ đề xuất bản dịch từ
   chính dữ liệu này và nó phải ghi được nguồn. Hiển thị: **⛔ không** một nhãn cho **mỗi**
   ký tự *(tiếng ồn thị giác, và ⛔ không mockup nào vẽ)* — **một** dòng `ui-label` màu
   `primary` ở đầu tab liệt kê các nguồn đã dùng cho **lượt hiện tại**.

---

### Quyết định #2 — **Hình dạng của method mới trên cổng `DictionarySource`**

Cổng hôm nay có ba method: `layer()` · `sources()` · `lookup()` · `senses()`. `EntryHit`
(`core/dict/mod.rs:214`) mang `entry_id · source_code · lang · headword · headword_simp` —
🔴 **⛔ không mang `han_viet`**. ⇒ ⛔ **không** đường nào hôm nay đọc được cột đó.

**Hai đường:**

| | Đường | Phán quyết |
|---|---|---|
| **(a)** | **Thêm method thứ ba: `han_viet(&self, chars: &[&str]) -> Result<Vec<HanVietHit>, StoreError>`** — nhận **lô ký tự**, trả các cặp `(ký tự, âm, source_code)` | ✅ **MẶC ĐỊNH ĐỀ XUẤT** |
| (b) | Thêm trường `han_viet` vào `EntryHit` rồi gọi `lookup()` với `LookupMode::Exact` cho từng ký tự | ⛔ **Loại** |

**Vì sao (b) bị loại — hai lý do, cả hai đo được:**
1. **N lời gọi cho N ký tự.** Đúng cái bẫy mà `senses()` đã được thiết kế để tránh:
   *"đọc theo **LÔ**, ⛔ **không** một truy vấn cho mỗi đầu mục … một cài đặt N+1 *"chạy
   đúng"* trên một fixture 20 hàng"* (`dict_source.rs`). Một Chương 3.000 ký tự ⇒ ~1.500 ký
   tự riêng ⇒ 1.500 × 3 tệp = **4.500** lượt tra.
2. **`lookup()` trả `Vec<EntryHit>` cho MỌI nguồn khớp.** Chữ 山 ở `dict-core.db` khớp nhiều
   đầu mục từ nhiều nguồn, và **chỉ `unihan`** có `han_viet` — tức 90 % dữ liệu đọc lên rồi
   bị vứt.

**Số đo cho (a)** *(1.500 ký tự riêng, một câu `WHERE headword IN (…)`, SQLite qua Python
trên chính hai tệp thật, trung vị của 7 lượt)*: `dict-core.db` **6,16 ms** · `dict-thieu-chuu.db`
**3,83 ms**. ⚠️ Đây là số của **Python**, ⛔ không phải của `rusqlite` bản release — nó là
**cận trên thô**, và nó nói đúng một điều: **hình dạng theo lô là đúng, hình dạng N+1 thì không**.

⚠️ **Đây ⛔ KHÔNG phải đường nóng NFR1.** NFR1 (100 ms đầu-cuối, backend ≤ 10 ms) là ngân
sách của **Auto-Lookup** (Story 1.18) — chạy hàng trăm lần mỗi Chương. Lượt đọc âm Hán Việt
chạy **một lần cho mỗi lượt nạp Chương**. ⛔ Đừng bắt nó vào ngân sách 10 ms, và ⛔ đừng
dùng nó làm cớ để bỏ qua phép đo.

🔴 **Ràng buộc AD-10 phải giữ:** *"Runtime **⛔ không có mã riêng cho từng nguồn**"*. ⇒ câu
SQL trong adapter ⛔ **không** được lọc theo `source.code`. Hình dạng đúng:
`SELECT headword, han_viet, <source code> FROM dict_entry … WHERE han_viet IS NOT NULL AND headword IN (…)`
— **mọi** tệp trả lời bằng **cùng một** câu; tệp nào ⛔ không có dữ liệu thì trả rỗng. *(Đã
đo: năm nguồn nền còn lại đều `NULL` toàn bộ, nên bộ lọc `IS NOT NULL` **là** bộ lọc đúng —
⛔ không cần biết tệp nào chứa gì.)*

⚠️ **Ca giản thể (Quyết định #1 đo: 19/32 → 31/32).** Câu SQL phải phủ **cả** `headword`
**lẫn** `headword_simp` — cùng bài học **Bẫy 8** của Story 1.9: *"phủ mỗi phồn thể làm `国`
trả rỗng trong 0,01 ms mà ⛔ không lỗi nào được ném"*. Chỉ mục `idx_entry_headword_simp` đã
có sẵn.

---

### Quyết định #3 — **Một ký tự nhiều âm** *(đo: 138 hàng ở lớp nền, và Thiều Chửu thì phổ biến)*

**Hai quy ước phân tách khác nhau trong hai tệp:**
- Thiều Chửu dùng `|` — `丁 → "đinh|chênh"`, `中 → "trung|trúng"` *(`thieu_chuu.rs:70` giữ
  nguyên chuỗi có chủ ý)*
- Unihan dùng **khoảng trắng** — `死 → "tợ tử"`

🔴 **Đây là một mâu thuẫn thật với AD-10** *(*"runtime ⛔ không có mã riêng cho từng nguồn"*)*:
tách đúng đòi biết tệp nào dùng quy ước nào.

**Ba đường:**

| | Đường | Phán quyết |
|---|---|---|
| **(a)** | Tách bằng **một luật duy nhất áp cho mọi tệp**: cắt trên `\|` **và** khoảng trắng | ✅ **MẶC ĐỊNH ĐỀ XUẤT** |
| (b) | Chuẩn hoá ở `tools/dict-build` để cả ba tệp dùng một dấu | ⛔ **Loại** — `tools/**` nằm trong ⛔ KHÔNG ĐỤNG; và dựng lại `.db` đổi SHA-256 trong `dict-manifest.toml` *(AD-25)*, tức kéo theo một lượt phát hành |
| (c) | Một bảng "tệp nào dùng dấu nào" | ⛔ **Loại** — đúng thứ AD-44 ① vá A2 cấm |

**(a) an toàn vì một dữ kiện kiểm được:** một âm Hán Việt là **một âm tiết tiếng Việt** —
nó ⛔ không bao giờ chứa `|` và ⛔ không bao giờ chứa khoảng trắng. ⇒ một luật, ⛔ không mã
riêng cho nguồn nào. **Nghiệm thu bằng test trên cả hai hình dạng thật** (`"đinh|chênh"` và
`"tợ tử"`).

**Hiển thị:** tab hiện **âm ĐẦU TIÊN**. Danh sách đầy đủ **vẫn đi qua IPC** và nằm trong mô
hình dữ liệu — Story 1.17 (Panel Lookup) và 3.7 (FR113) cần nó.

⚠️ **Lỗ hổng đã biết, ⛔ ghi ra thay vì lấp bằng phát minh:** ⛔ **không** mockup nào và ⛔
không UX-DR nào nói cách báo cho người dùng biết *"chữ này còn âm khác"*. Cám dỗ tự nhiên
— một chấm, một gạch chân, một màu — đều là **phát minh UX ⛔ không có chữ ký**, và UX-DR5
đã đóng đúng đường rẻ nhất *(`ornament` ⛔ không bao giờ là màu của chữ)*. ⇒ **⛔ Không
đánh dấu gì ở story này.** Xem §Câu hỏi cho Ice #2.

---

### ✅ Quyết định #4 — ĐÃ CHỐT 2026-08-06: **(a) cặp theo TỪNG KÝ TỰ**

> 🔴 Ice chốt **(a)**. ⇒ **AC6** *(vùng chọn ⛔ không bị ô nhiễm)* và **Task 8** *(đo trần
> render)* là **bắt buộc**, ⛔ không phải tuỳ chọn — chúng là hai cái giá của (a).
> §Câu hỏi cho Ice #1 **đóng**.

AC của epic: *"xem được ở chế độ **chuyển đổi** hoặc **song song**"*. **Chuyển đổi** ⛔ không
mơ hồ *(tab Hán Việt thay chỗ nguyên văn — đúng `mockups/key-screen-workspace.html:96`)*.
**Song song** thì có ba cách đọc, và chúng ⛔ không tương đương về hệ quả:

| | Hình dạng | Phán quyết |
|---|---|---|
| **(a)** | **Cặp theo TỪNG KÝ TỰ** — mỗi ký tự là một khối dọc `chữ Hán trên / âm dưới`, xuống dòng tự nhiên | ✅ **MẶC ĐỊNH ĐỀ XUẤT** |
| (b) | Xen kẽ theo **DÒNG** *(một dòng nguyên văn, một dòng âm ngay dưới — đúng khối `.hv` của mockup)* | ⚠️ **Loại, có điều kiện** |
| (c) | `<ruby>` / `<rt>` | ⛔ **Loại** |

**Vì sao (a):** FR33 nói *"âm Hán Việt cho **từng ký tự**"*. Chỉ (a) và (c) giữ được **tương
ứng một-một nhìn thấy được**. (b) mất tương ứng ngay khi dòng xuống hàng — và nó đòi một
**bộ đoán chỗ ngắt dòng**, thứ ⛔ không tồn tại trước khi trình duyệt đã layout xong.

**Vì sao (c) bị loại — ba lý do, ⛔ không lý do nào là sở thích:**
1. `source-hanviet` khai `fontStyle: italic` + `fontSynthesis: none`. Hành vi kế thừa `font-*`
   vào `<rt>` **khác nhau giữa WebView2 và WKWebView** — đúng lớp lỗi **NFR14**, và dự án
   ⛔ **không có** runner đo được vế thị giác hai nền tảng *(`deferred-work.md:478`)*.
2. `<ruby>` đặt cỡ chữ `<rt>` mặc định theo tỷ lệ của engine, trong khi bảng token chốt
   **12,5 px** — tức token bị engine ghi đè, đúng thứ AD-34 ③ tồn tại để chặn.
3. Vùng chọn trong `<ruby>` **kéo theo `<rt>` vào chuỗi được chọn** ở một số engine — nó
   sẽ nhét âm Hán Việt vào truy vấn Auto-Lookup của **Story 1.18**, và hỏng ở **story sau**.

**(b) vẫn giữ mở** nếu Ice muốn khớp mockup từng nét: nó đơn giản hơn hẳn và rẻ hơn hẳn,
đổi lại mất tương ứng một-một. ⇒ §Câu hỏi cho Ice #1.

🔴 **Ràng buộc bắt buộc của (a) — ⛔ ĐỌC KỸ, đây là chỗ nó phá Story 1.18 và 3.4 trong im lặng:**
mỗi ký tự thành một node riêng nghĩa là **bôi đen một cụm từ cho ra một chuỗi có thể lẫn
âm Hán Việt, hoặc lẫn khoảng trắng giữa các ký tự**. Cả Story 1.18 (Auto-Lookup) lẫn 3.4
(đánh dấu Glossary) đứng trên mệnh đề *"chuỗi bôi đen = đúng chuỗi trong dữ liệu"*.
⇒ **AC6 cưỡng chế mệnh đề này bằng một phép kiểm thật**, ⛔ không bằng lời hứa.

---

### Quyết định #5 — **Chỗ đặt dải tab `Trung` / `Hán Việt`**

Mockup vẽ nó ở **bên phải thanh tiêu đề panel** (`.phead > .tabs`). Nhưng Story 1.14
§Quyết định #4A đã **gỡ `<header>`** khỏi `PanelFrame.vue`: *"tab bar của dockview **LÀ**
thanh tiêu đề panel"*.

| | Đường | Phán quyết |
|---|---|---|
| **(a)** | Một dải tab ở **đầu THÂN** panel Source | ✅ **MẶC ĐỊNH ĐỀ XUẤT** |
| (b) | `rightHeaderActionsComponent` của dockview | ⛔ **Loại** |

**Vì sao (b) bị loại — một dữ kiện cấu trúc, ⛔ không phải một sở thích:** vùng đó là của
**GROUP**, ⛔ không phải của **PANEL**. FR17 cho phép gộp panel thành tab trong một group —
ngày người dùng gộp `Nguyên văn` với `Tra cứu`, dải `Trung / Hán Việt` sẽ hiện ra **trong
khi tab đang chọn là Tra cứu**. Hỏng im lặng, và chỉ hỏng với người dùng đã tuỳ biến bố cục.

**Hệ quả lên `PanelFrame.vue`:** vỏ hôm nay **luôn** render `<p class="status">`. Panel
Source có nội dung thật thì câu trạng thái phải **biến mất**.
🔴 ⛔ **KHÔNG** giải bằng cách bind `:status-key` có điều kiện: Kiểm E của
`npm run check:commands` đọc **tĩnh** thuộc tính đó và đối chiếu **hai chiều** với `vi.json`
— một biểu thức ở đó **bị đếm rồi bỏ qua, tức mất lưới** *(doc-comment `SourcePanel.vue:9-12`
nói đúng câu này)*. Đường đúng: thêm prop boolean *(đề xuất `:show-status`, mặc định `true`)*;
`status-key` **ở lại literal**.

---

### ✅ Quyết định #6 — ĐÃ CHỐT 2026-08-06: **(a) thêm token thứ 16 `source-latin`**

> 🔴 Ice chốt **(a)** — đi qua sổ `deviations`, **đúng** tiền lệ `ui-md-strong` của Story 1.14.
> ⛔ **Không** thêm hàng vào bảng đóng băng của `check-tokens.mjs` *(nó ở lại 14 hàng của
> `DESIGN.md`)*, và mục `deviations` phải có `question` + `reason` **không rỗng**.

FR19 nói nguyên văn là *"tiếng Anh **hoặc** tiếng Trung"*. Bảng token có `source-cjk`
*(họ `read-cjk` — **chỉ** `Noto Serif CJK TC`)* và `source-hanviet`. ⛔ **Không có hàng nào
cho văn bản nguồn Latin.** Đây là một lỗ hổng **của tài liệu quy hoạch**, ⛔ không phải một
thiếu sót của story trước.

| | Đường | Phán quyết |
|---|---|---|
| **(a)** | Thêm token **thứ 16** `source-latin` *(đề xuất: họ `read` · 16px · line-height **1.66** · `wraps: true`)*, đi qua sổ `deviations` | ✅ **MẶC ĐỊNH ĐỀ XUẤT** |
| (b) | Mượn `editor` (15px/1.95, họ `read`) | ⛔ **Loại** — `editor` khai vai *"Bản dịch trong Editor"*; mượn nó là làm nhoè đúng ranh giới UX-DR12 dựng để giữ |
| (c) | Mượn `read-sm` | ⛔ **Loại** — vai của nó là *"Chế độ đọc — mức Đặc"*, và ba token `read-*` đi kèm `read-measure-*`, thứ ⛔ không áp cho Panel Source |
| (d) | Để `source-cjk` gánh cả hai | ⛔ **Loại** — họ `read-cjk` là **chỉ** `Noto Serif CJK TC`; chữ Latin dựng bằng font CJK cho ra chữ Latin **của** một font CJK, và NFR14 vỡ ở đúng chỗ dễ thấy nhất |

**Tiền lệ có sẵn cho (a):** Story 1.14 · AC10 đã thêm token thứ **15** (`ui-md-strong`) theo
đúng thủ tục này — bảng đóng băng trong `check-tokens.mjs` **ở lại 14 hàng của `DESIGN.md`**,
hàng mới sống trong `tokens.deviations` **kèm `question` + `reason` không rỗng**
*(`check-tokens.mjs:700-708`)*. ⛔ **Đừng thêm hàng vào bảng đóng băng.**

⚠️ `1.66` ⛔ không phải một con số tuỳ chọn: `wraps: true` + họ `read` ⇒ Kiểm của
`check-tokens.mjs:1316` (`LINE_HEIGHT_FLOOR = 1.66`) làm mọi giá trị nhỏ hơn **đỏ**.

---

### Quyết định #7 — **Trần render, và nó đến từ một quyết định của Story 1.15**

🔴 **Dữ kiện phải biết:** Story 1.15 tạo **đúng MỘT Chương** cho **toàn bộ** văn bản nhập, và
trần nhập là `MAX_IMPORT_BYTES` = **100 MB** *(`deferred-work.md:529` — một con số **TẠM,
chưa ai đo**)*. ⇒ một Chương hợp lệ hôm nay có thể mang **hàng chục triệu ký tự**, và
Quyết định #4(a) dựng **một node cho mỗi ký tự**.

⛔ **Không có đường ảo hoá:** mọi thư viện virtual-scroll là một phụ thuộc npm mới, và
`package.json` nằm trong ⛔ KHÔNG ĐỤNG *(NFR15)*.

**Mặc định đề xuất:** một **trần render có tên, hiện ra bằng một chuỗi `vi.json`**, ⛔
không phải một lần cắt im lặng. Con số trần **phải do phép đo của Task 8 quyết**, ⛔ không
đoán trước trong story này. Ràng buộc lên phép đo: dựng thật, đếm **thời gian tới frame đầu**
và **đỉnh bộ nhớ**, ở ít nhất ba mức *(5.000 · 50.000 · 500.000 ký tự)*, ở **cả hai** kiểu
xem, và ghi số vào Completion Notes.

⚠️ Chế độ **chuyển đổi** *(nguyên văn thuần)* rẻ hơn **chuyển đổi** *(song song)* nhiều bậc
độ lớn — một khối văn bản với `white-space: pre-wrap` ⛔ không sinh node cho mỗi ký tự. ⇒
trần có thể **khác nhau theo kiểu xem**, và đó là kết quả hợp lệ của phép đo.

---

## Acceptance Criteria

### AC1 — Nguyên văn hiện bằng token `source-cjk`, họ `read-cjk` *(FR19; UX-DR12)*

**Given** một Chương có văn bản nguồn tiếng Trung
**When** mở Workspace
**Then** Panel Source hiển thị **nguyên văn** đọc từ `chapter.source_text`
**And** bề mặt đó khai **token `source-cjk` của chính nó** — `--face/--font/--leading-source-cjk`
— ⛔ **không** kế thừa `ui-md` của `body` *(đóng nửa Source của `deferred-work.md:129-131`)*
**And** ⛔ **không** một chuỗi nào đi qua `v-html` hay tương đương *(AD-16)*
**And** ngắt dòng của văn bản nguồn hiện ra thành ngắt dòng, và `\r` của tệp Windows ⛔
**không** hiện thành một ô trống *(`deferred-work.md:527`)*

### AC2 — Nguyên văn **tiếng Anh** có token của chính nó *(FR19 — nửa bị bỏ quên)*

**Given** ngôn ngữ nguồn của Tác phẩm là **tiếng Anh**
**When** Panel Source hiển thị
**Then** nguyên văn dựng bằng token do **Quyết định #6** chốt, họ `read`, giãn dòng **≥ 1.66**
**And** nếu là token mới thì nó có một mục `deviations` **kèm `question` và `reason` không rỗng**,
và bảng đóng băng của `check-tokens.mjs` **⛔ vẫn đúng 14 hàng**

### AC3 — Tab Hán Việt tồn tại **khi và chỉ khi** nguồn là tiếng Trung *(FR19)*

**Given** `work.source_lang` là tiếng Trung
**When** Panel Source hiển thị
**Then** có **tab Hán Việt**
**And Given** `work.source_lang` là tiếng Anh
**Then** ⛔ **không** có tab Hán Việt — ⛔ không tab bị vô hiệu hoá, ⛔ không tab ẩn bằng CSS;
phần tử **⛔ không tồn tại trong DOM**
**And** phép phân biệt đọc từ `work.source_lang` — trường **bất biến** ghi lúc tạo (AD-18) —
⛔ **không** đoán từ nội dung văn bản

### AC4 — Tab Hán Việt hiển thị âm cho **từng ký tự**, đọc qua cổng, chạy ngoại tuyến *(FR33)*

**Given** tab Hán Việt bật
**When** hiển thị
**Then** **mỗi ký tự Hán** trong nguyên văn có âm Hán Việt của nó
**And** âm đọc **qua cổng `DictionarySource`** — ⛔ không một `Connection::open`, ⛔ không một
lời gọi `rusqlite` nào ngoài `core/store/**` *(`store_boundary.rs` cưỡng chế)*
**And** đường đọc phủ **cả `headword` lẫn `headword_simp`** *(⛔ không lặp lại Bẫy 8 của Story 1.9)*
**And** hoạt động khi **ngắt kết nối mạng** — nghiệm thu bằng chính lưới đã có: `check:scope`
báo **0** lời gọi ra ngoài
**And** một ký tự ⛔ không có âm ở bất kỳ lớp nào vẫn hiện ra **có tên** *(chuỗi `vi.json`
hoặc ký tự giữ chỗ đã chốt)*, ⛔ **không** là một ô trống câm
**And** 🔴 ca **⛔ KHÔNG LỚP TỪ ĐIỂN NÀO ĐANG GẮN** — trạng thái **bình thường có tên** hôm
nay, vì `src-tauri/resources/dict/` ⛔ **rỗng trong git** *(AD-25; `lib.rs::open_dict_layers`
nói đúng câu này)* — hiện ra bằng một chuỗi **KHÁC** với ca *"đã tra mà ký tự này ⛔ không
có âm"*. **Ba trạng thái, ⛔ không phải một** *(doctrine `QueryBranch::NoBranchQueryTooShort`
của Story 1.13: một trạng thái **không hỗ trợ** phải phân biệt được với một lượt đã chạy mà
⛔ không tìm thấy gì)*

### AC5 — Thứ tự đọc giữa các lớp là **quyết định của tầng gom**, và FR36 nghiệm thu ở mức degradation

**Given** nhiều lớp cùng có âm cho một ký tự
**When** tầng gom chọn
**Then** thứ tự ưu tiên tính **một lần cho cả lượt**, ở tầng gom — ⛔ **không** trong adapter,
⛔ **không** bằng một chuỗi `layer()` viết cứng *(AD-44 ① vá A2)*
**And** mỗi ký tự mang theo `source_code` của âm đang hiện
**And Given** `dict-thieu-chuu.db` **và** `dict-tran-van-chanh.db` bị **xoá khỏi đĩa**
**When** chạy lại toàn bộ bộ test tra cứu **và** đường Hán Việt
**Then** ⛔ **không** một đường nào hỏng, tab **vẫn** trả âm từ nguồn nền `en-wiktionary-vi`
*(FR36, AD-10)*
**And** mức phủ tụt xuống ghi thành **số thật** trong Completion Notes *(dự kiến **12.463 → 1.136**)*
— ⛔ không đánh dấu đạt rồi im
**And** ⚠️ sau `1-10c`, lớp **nền** ⛔ **không** còn mang âm Hán Việt nào từ `Unihan` — ⛔ đừng
viết một nhánh dự phòng đọc cột Nôm khi ⛔ không tìm thấy âm Hán Việt; đó là dựng lại đúng lỗi
mà `1-10c` vừa gỡ

### AC6 — Hai kiểu xem, **và vùng chọn ⛔ không bị ô nhiễm**

**Given** tab Hán Việt
**When** người dùng chọn kiểu xem
**Then** xem được ở **chuyển đổi** *(âm thay chỗ nguyên văn)* và **song song** *(hình dạng do
Quyết định #4 chốt)*
**And** cả hai lệnh đổi tab lẫn đổi kiểu xem đi qua **`CommandRegistry`** và **⛔ có phím** —
⛔ không thao tác nào chỉ tới được bằng chuột *(AD-34 ①, NFR17)*
**And** 🔴 **bôi đen một cụm từ ở chế độ song song cho ra ĐÚNG chuỗi ký tự nguồn** — ⛔ không
lẫn âm Hán Việt, ⛔ không lẫn khoảng trắng chèn thêm. **Nghiệm thu bằng một phép kiểm thật
trên `window.getSelection().toString()`**, ⛔ không bằng lời hứa *(điều kiện tiên quyết của
Story 1.18 và 3.4)*

### AC7 — `font-synthesis` được **tiêu thụ**, ⛔ không chết im lặng

**Given** token `source-hanviet` khai `fontSynthesis: "none"`
**When** bề mặt Hán Việt dựng
**Then** nó áp `font-synthesis: var(--synthesis-source-hanviet)` **tại chỗ**
**And** đây là **người tiêu thụ đầu tiên** của lời giải chữ Hán nghiêng giả — `deferred-work.md:133`
ghi nguyên văn: *"**Bỏ sót dòng đó là cách lời giải này chết im lặng.**"*

### AC8 — Đường IPC đọc Chương, và **⛔ không một chuỗi hiển thị nào từ Rust**

**Given** một Tác phẩm đang mở trong `OpenWorkState`
**When** webview yêu cầu nội dung Chương
**Then** một `#[tauri::command]` trả về `source_text` + `source_lang` + định danh Chương
**And** lỗi có hình dạng `{ code, message_key, params, retryable }` *(AD-21)* — ⛔ không một
ký tự tiếng Việt nào trong mã Rust *(Kiểm A của `check:i18n`)*
**And** ⛔ **không** thêm một permission nào vào `capabilities/main.json` *(command của chính
ứng dụng ⛔ không cần ACL — `lib.rs:72-78`)*
**And** ⛔ **không** dựng đường mở lại `.atproj` từ đĩa, ⛔ không thêm lại `MessageKey::ProjectMetaTooNew`
*(`deferred-work.md:533`)*

### AC9 — Trạng thái panel sống sót qua một lượt đổi preset *(`deferred-work.md:504`)*

**Given** người dùng đang ở tab Hán Việt, kiểu xem song song, đã cuộn xuống
**When** đổi preset bố cục *(`Mod+Alt+1` ↔ `Mod+Alt+2`)*
**Then** tab đang chọn và kiểu xem **giữ nguyên**
**And** ⛔ **không** gọi lại đường IPC đọc Chương, ⛔ không tra lại âm Hán Việt
**And** mục *"`applyPreset()` luôn `api.clear()` rồi dựng lại"* của `deferred-work.md` được
đánh dấu **tại chỗ** — đóng, đóng một nửa, hay chuyển tiếp, **kèm lý do**

### AC10 — `replace_open_work` thả `Store` cũ **ngoài** vùng khoá *(`deferred-work.md:525`)*

**Given** story này thêm command **đầu tiên** đọc `OpenWorkState`
**When** một lượt "mở Tác phẩm khác" chạy
**Then** `Store` cũ được thả **ngoài** vùng khoá mutex — khuôn đã biết:
`let old = { let mut g = state.lock()…; g.replace(new_work) }; drop(old);`
**And** lượt review 2026-08-06 đã dự báo đúng ca này: *"nó trở thành rủi ro thật khi một
story sau thêm **bất kỳ command nào đọc `OpenWorkState`**"* — ⛔ **không** để nó sang story sau

### AC11 — Mọi cổng xanh, sàn nâng theo số thật, ranh giới ⛔ KHÔNG CHẠM giữ nguyên

**Given** cây nguồn sau story
**When** chạy đủ bộ DoD *(§Testing standards)*
**Then** **cả chín** lệnh PASS và `cargo test` xanh
**And** mọi hằng `*_FLOOR` bị story này vượt qua được **nâng theo số THẬT**, con số thật ghi
vào comment cạnh hằng *(tiền lệ Story 1.14 · AC11.1)*
**And** `src/**` vẫn có **0** lời gọi `matchMedia` và **0** lần đọc `window.innerWidth`
**And** ⛔ **0** phụ thuộc npm mới, ⛔ **0** crate mới, ⛔ không đụng `Cargo.toml`,
`dict-manifest.toml`, `tools/**`, `capabilities/main.json`, `_bmad-output/planning-artifacts/**`

---

## Tasks / Subtasks

- [x] **Task 0 — ⛔ KHÔNG gõ một dòng mã nào cho tới khi `1-10c` `done`.**
  - [x] ✅ #1 *(nguồn Hán Việt)* · #4 *(song song = cặp từng ký tự)* · #6 *(token thứ 16)* — **Ice chốt 2026-08-06**
  - [x] Đối chiếu `1-10c` đã `done`: `dict_entry.han_viet` mang **đúng một** ngữ nghĩa, `SUPPORTED_SCHEMA_VERSION = 2`, lớp `tran-van-chanh` có trong `dict-manifest.toml`
  - [x] Chốt **#2 · #3 · #5 · #7** *(bốn mục còn lại, đều có mặc định đề xuất)*
  - [x] Ghi phán quyết vào §Change Log **trước** khi bắt đầu Task 1

- [x] **Task 1 — Đường IPC đọc Chương** (AC8, AC10)
  - [x] Hàm **thuần** trước, `#[tauri::command]` là **vỏ mỏng** — khuôn `commands::config` và `commands::project`
  - [x] Đọc qua `Store::read`; ⛔ không `Connection::open`
  - [x] Vá `replace_open_work` — thả `Store` cũ ngoài vùng khoá (AC10)
  - [x] Đăng ký vào `generate_handler![…]`; ⛔ **không** đụng `capabilities/main.json`
  - [x] Adapter phía webview theo khuôn `src/config/project.ts` *(⚠️ `invoke` gửi tham số **camelCase**)*
  - [x] Test: Chương có văn bản · ⛔ chưa Tác phẩm nào mở · hình dạng lỗi `IpcError`

- [x] **Task 2 — Method thứ ba trên cổng `DictionarySource`** (AC4, Quyết định #2)
  - [x] Khai **hình dạng** ở `ports/dict_source.rs` — ⛔ tệp đó ⛔ không gõ tên crate SQLite
  - [x] Kiểu bản ghi sống ở `core::dict`, ⛔ **không** ở `ports/` *(doctrine đã có)*
  - [x] Cài ở `core/dict/layer.rs` (`impl DictionarySource for DictLayer`) + câu SQL ở `core/dict/han_viet.rs` *(tệp riêng, ⛔ không `query.rs` — cùng khuôn tách biệt `senses.rs`)*
  - [x] 🔴 Câu SQL phủ **cả `headword` lẫn `headword_simp`**, ⛔ **không** lọc theo `source.code` (AD-10)
  - [x] Đọc theo **LÔ**, ⛔ không N+1 — nghiệm thu bằng test batching tương đương `senses.rs` (200 ký tự ⇒ 4 lô, so khớp với truy vấn thẳng)

- [x] **Task 3 — Tầng gom âm Hán Việt** (AC5, Quyết định #1 & #3)
  - [x] Thứ tự ưu tiên **tính một lần cho cả lượt**, ở tầng gom; ⛔ không chuỗi `layer()` viết cứng
  - [x] Tách nhiều âm bằng **một luật** áp cho mọi tệp: cắt trên `|` **và** khoảng trắng (Quyết định #3a)
  - [x] Mỗi ký tự mang `source_code`; dedupe ký tự **trước** khi tra
  - [x] Test trên **cả hai** hình dạng thật: `"đinh|chênh"` và `"tợ tử"`
  - [x] 🔴 Test **FR36**: xoá cả hai lớp gỡ rời (fixture) ⇒ đường vẫn chạy, rơi về lớp nền — con số thật (12.463→1.136) ghi ở Completion Notes vì đo trên `.db` thật, ⛔ không đo được trên fixture

- [x] **Task 4 — Token cho nguyên văn tiếng Anh** (AC2, Quyết định #6)
  - [x] Thêm token vào `tokens.json` **+ một mục `deviations` có `question` và `reason`**
  - [x] ⛔ **Không** thêm hàng vào bảng đóng băng của `check-tokens.mjs` *(bảng vẫn 14 hàng của DESIGN.md; chỉ nâng hằng `EXPECTED_COUNTS.typography` 15→16, đúng tiền lệ `ui-md-strong` 14→15 của Story 1.14)*
  - [x] Đối chiếu: `wraps: true` ⇒ `lineHeight ≥ 1.66` *(16px/1.66, qua `LINE_HEIGHT_FLOOR`)*

- [x] **Task 5 — `PanelFrame.vue`: prop `show-status`** (Quyết định #5)
  - [x] `status-key` **ở lại literal** ở cả bốn panel — ⛔ không bind biểu thức
  - [x] Ba panel còn lại ⛔ không đổi hành vi *(hồi quy: câu trạng thái của chúng vẫn hiện — mặc định `showStatus: true`)*

- [x] **Task 6 — Thân Panel Source: nguyên văn + dải tab** (AC1, AC2, AC3, AC6)
  - [x] Dải tab ở **đầu thân panel**; ⛔ không `rightHeaderActionsComponent`
  - [x] Tab Hán Việt **⛔ không tồn tại trong DOM** khi nguồn là tiếng Anh (AC3) *(`v-if="showHanVietTab"` trên cả dải tab lẫn nút đổi kiểu xem)*
  - [x] Bề mặt nguyên văn khai token của **chính nó** (`source-cjk` / `source-latin` của Task 4, chọn theo `source_lang`)
  - [x] Chuỗi mới vào `vi.json`; ⛔ 0 chuỗi tiếng Việt trong `.vue`/`.rs` *(Kiểm A + A2 — nội dung Chương/âm Hán Việt đi qua miễn trừ `aura-allow-text` có tên, vì đó là DỮ LIỆU chứ không phải chuỗi giao diện)*

- [x] **Task 7 — Bề mặt Hán Việt + hai kiểu xem** (AC4, AC6, AC7)
  - [x] Áp `font-synthesis: var(--synthesis-source-hanviet)` — **⛔ đừng bỏ sót dòng này** (AC7, cộng bốn biến còn lại của token)
  - [x] Ba lệnh *(chọn tab Trung · chọn tab Hán Việt · đổi kiểu xem)* đăng ký ở `CommandRegistry` **và có phím** (`Mod+Alt+O`/`Mod+Alt+H`/`Mod+Alt+V`, NFR17) — hai lệnh CHỌN tab thay vì một lệnh TOGGLE, cùng khuôn `layout.preset_grid`/`layout.preset_columns` (tránh bẫy bấm-vào-tab-đang-mở làm lật nhầm)
  - [x] Ký tự ⛔ không có âm: hai chuỗi `vi.json` riêng theo `layers_loaded` (đã tra mà không có / chưa có lớp nào), ⛔ không ô trống câm
  - [x] 🔴 **Phép kiểm vùng chọn** (AC6) — `window.getSelection().toString()` bằng đúng chuỗi nguồn, **đo THẬT bằng Playwright headless** *(không phải lời hứa)*: bản đầu (`display: inline-flex; flex-direction: column`) **ĐỎ** — Chromium chèn `\n` ở mỗi ranh giới hộp dòng bất kể `user-select: none`; bản vá (`.hv-reading` định vị `position: absolute`, ra khỏi luồng bố cục) **XANH** — xem doc-comment `.hv-reading` trong `SourceHanViet.vue`

- [x] **Task 8 — ĐO trần render** (Quyết định #7)
  - [x] Ba mức *(5.000 · 50.000 · 500.000 ký tự)* × hai kiểu xem; đo thời gian tới frame đầu + đỉnh bộ nhớ — **Playwright headless Chromium**, DOM y hệt cấu trúc thật của `SourceHanViet.vue` (xem bảng số ở §Completion Notes)
  - [x] Chốt trần **từ số đo**: **50.000 ký tự Hán** cho kiểu song song (1.408,5 ms ở 50k — còn chấp nhận được cho một thao tác chạy một lần; 13.621,5 ms ở 500k — không); kiểu chuyển đổi ⛔ không có trần (222,4 ms ở 500k). Trần hiện ra bằng chuỗi `vi.json` (`panel.source.parallel_view_unavailable`) + khoá nút đổi kiểu xem ở `SourcePanel.vue` + hàng rào phòng thủ thứ hai ở chính `SourceHanViet.vue` (`effectiveViewMode`)
  - [x] Ghi bảng số vào §Completion Notes

- [x] **Task 9 — Trạng thái panel sống sót qua đổi preset** (AC9)
  - [x] Nhấc state *(tab đang chọn · kiểu xem · nội dung đã nạp · âm Hán Việt đã tra)* ra **ngoài** component — `src/panels/sourcePanelState.ts`, module-level singleton
  - [x] Nghiệm thu: `ensureChapterLoaded`/`ensureHanVietLoaded` idempotent qua cờ module-level ⇒ đổi preset (tháo/dựng lại instance) ⛔ không gọi lại `read_open_chapter`/`read_han_viet` — đúng theo cấu trúc mã, cùng khuôn `bootstrap.ts`/`dockController.ts` đã có tiền lệ; ⛔ chưa đo được bằng webview thật trong phiên này (không có instance Tauri dev rảnh để chạy — xem Task 11)
  - [x] Đánh dấu mục `deferred-work.md:504` **tại chỗ**: ĐÓNG cho Panel Source, vẫn MỞ cho Lookup/Editor/AI

- [x] **Task 10 — Cổng, sàn, hồi quy** (AC11)
  - [x] Chạy đủ chín lệnh DoD; nâng mọi `*_FLOOR` bị vượt, ghi **số thật** vào comment *(chỉ `EXPECTED_COUNTS.typography` của `check-tokens.mjs` cần nâng, 15→16 — các `*_FLOOR` khác là SÀN, số thật vẫn vượt qua nên ⛔ không cần đụng)*
  - [x] Đối chiếu: **0** `matchMedia`, **0** `window.innerWidth`, **0** phụ thuộc mới *(xác nhận qua `check:layout`/`check:deps` xanh)*
  - [x] Cập nhật `src/panels/README.md` — hàng *"Nội dung panel Source + tab Hán Việt | 1.16"* → ✅

- [x] **Task 11 — Bàn giao trung thực**
  - [x] Ghi mọi mục còn mở vào `deferred-work.md` §`1-16-…` — ⛔ **không** đánh dấu đạt thứ chưa đo
  - [x] 🔴 **Nói thẳng vế ⛔ chưa đo được:** vế thị giác trên **WKWebView** và trên **Windows**
        *(`deferred-work.md:478` — dự án ⛔ không có runner đo được vế đó)* — cộng AC9 và trần
        render Task 8, cả hai đo bằng cấu trúc mã/Chromium chứ chưa bằng webview thật

---

## Dev Notes

### Trạng thái repo hôm nay — SỐ, ⛔ không phải mô tả

| | Số thật (2026-08-06, `564be15`) |
|---|---|
| `#[tauri::command]` đã đăng ký | **4** — `bootstrap_config` · `put_config` · `create_work_from_text` · `create_work_from_file` |
| Đường đọc `chapter.source_text` từ webview | 🔴 **⛔ KHÔNG CÓ** — story này dựng nó |
| Method trên `DictionarySource` | 4 — `layer()` · `sources()` · `lookup()` · `senses()`. 🔴 ⛔ **Không** method nào đọc `han_viet` |
| `EntryHit` mang `han_viet` | 🔴 **⛔ KHÔNG** |
| Thân bốn panel | **trống** — `PanelFrame.vue` render đúng một `<p class="status">` |
| Chỗ tiêu thụ `--synthesis-*` | 🔴 **0** *(`deferred-work.md:133`)* |
| Token typography | **15** *(14 của `DESIGN.md` + `ui-md-strong` trong `deviations`)*. ⛔ Không token nào cho nguyên văn Latin |
| `matchMedia` / `window.innerWidth` trong `src/**` | **0** / **0** — phải giữ nguyên |
| Chỗ chạm `OpenWorkState` | **2** — `lib.rs::open_work_slot` · `lib.rs::close_open_work`. Story này là chỗ **thứ ba**, và là chỗ **đầu tiên đọc** |

### API thật — chép từ MÃ, ⛔ không từ trí nhớ

```rust
// core/store/mod.rs
pub type ReadHandle<'a> = &'a rusqlite::Connection;
impl Store { pub fn read<T, F>(&self, job: F) -> Result<T, StoreError>
             where F: FnOnce(ReadHandle<'_>) -> SqlResult<T>; }

// core/store/schema.rs — CHAPTER_DDL
// chapter(id INTEGER PRIMARY KEY AUTOINCREMENT, ord, title, source_text TEXT NOT NULL, status, …)
// work(id INTEGER PRIMARY KEY CHECK(id=1), work_id, name, source_lang TEXT NOT NULL, genre, …)

// commands/project.rs
pub struct OpenWork { pub folder: PathBuf, pub store: Store, pub scope: …, pub meta: WorkMeta }
pub type OpenWorkState = Mutex<Option<OpenWork>>;   // lấy qua `try_state`, KHÔNG panic khi vắng

// tools/dict-build/src/schema.rs — DICT_ENTRY_DDL (tệp `.db`, CHỈ ĐỌC)
// dict_entry(id, source_id, lang, headword, headword_simp, reading, han_viet)
// CREATE INDEX idx_entry_headword ON dict_entry(headword);
// CREATE INDEX idx_entry_headword_simp ON dict_entry(headword_simp);
```

### Doctrine đã chốt ở Story 1.11/1.13 mà story này **thừa kế nguyên**

- **Vị từ điều phối chạy ĐÚNG MỘT LẦN cho cả lượt, ở tầng gom** — `route`, `branch` **nhận
  từ chỗ gọi**, adapter ⛔ không tự tính lại. Thứ tự ưu tiên lớp ở Task 3 đi **đúng khuôn đó**.
- **Khoá theo `source_code` (chuỗi), ⛔ không `source_id` (số)** — mỗi tệp `.db` có bảng
  `dict_source` **riêng**, nên `id = 1` tồn tại ở **cả ba** tệp và trỏ ba nguồn khác nhau.
- **⛔ Không tồn tại sổ đăng ký "tệp nào chứa gì"** *(AD-44 ① vá A2)* — **mọi** tệp đang gắn
  đều được hỏi; bộ lọc nằm **trong SQL**.
- **`DictLayers` mở ở `setup()`, có thể RỖNG, và rỗng ⛔ không phải lỗi** — `src-tauri/resources/dict/`
  hôm nay ⛔ không có tệp `.db` nào trong git *(AD-25)*. ⇒ 🔴 **tab Hán Việt phải xử lý được
  ca "0 lớp"** và nói ra bằng một chuỗi `vi.json`, ⛔ không sập, ⛔ không im.

### ⚠️ CHÍN CÁI BẪY — bảy trong chín cho ra một lượt CI **XANH** với kết quả **VÔ NGHĨA**

1. **Đọc `han_viet` bằng `lookup()` từng ký tự** ⇒ N+1, *"chạy đúng"* trên fixture 20 hàng,
   sập trên Chương thật. Đã có tiền lệ: doc-comment `senses()` viết ra đúng bẫy này.
2. **Chỉ phủ `headword`, quên `headword_simp`** ⇒ đo được: **19/32** trên câu giản thể thay
   vì 31/32. ⛔ Không lỗi nào được ném. **Bẫy 8 của Story 1.9, tái sinh.**
3. 🔴 **Chế độ song song làm ô nhiễm vùng chọn** ⇒ Story 1.18 tra một chuỗi lẫn âm Hán Việt.
   Hỏng ở **story sau**, tức đắt gấp đôi. **AC6 tồn tại chỉ để chặn cái này.**
4. **Lấy `layer()` viết cứng `"thieu-chuu"`** ⇒ đúng thứ AD-44 ① vá A2 cấm; và nó **sai im
   lặng** vào đúng ngày một lớp được thêm hay gỡ *(FR112)*.
5. **Bind `:status-key` có điều kiện** ⇒ Kiểm E của `check:commands` đếm rồi **bỏ qua** —
   mất lưới, cổng vẫn xanh *(Quyết định #5)*.
6. **Bề mặt chữ quên khai token của chính nó** ⇒ chạy ở `ui-md` giãn dòng **1.5**, dưới sàn
   1.66, dấu `ườ` chạm dấu `ộ` — và Kiểm E của `check-tokens.mjs` **chỉ đọc `tokens.json`**
   nên hoàn toàn mù *(`deferred-work.md:129`)*.
7. **Quên `font-synthesis`** ⇒ lời giải chữ Hán nghiêng giả **chết im lặng** *(AC7)*.
8. **Tách nhiều âm chỉ trên `|`** ⇒ `死` hiện *"tợ tử"* thành một "âm". Chỉ tách trên khoảng
   trắng ⇒ `丁` hiện *"đinh|chênh"*. **Cả hai hình dạng tồn tại thật.**
9. **Thả `Store` cũ trong vùng khoá** ⇒ hôm nay vô hại, ngày mai là một lượt "mở Tác phẩm
   khác" **chặn mọi lượt đọc state** *(`deferred-work.md:525`)*.

### 🔴 Ba mâu thuẫn tài liệu đã phát hiện — ⛔ dev KHÔNG sửa tài liệu, chỉ NÓI RA

1. **`Unihan kVietnamese` ⛔ không phải Hán Việt thuần** — `epics.md:1693` và `prd.md` FR33
   nói *"đọc từ dữ liệu đã nhúng"* như thể đó là một dữ kiện đồng nhất. Số đo nói khác.
   `EXPERIENCE.md:410` còn xây ví dụ trụ cột của FR113 trên một chữ (`北`) mà lớp nền ⛔
   không có. **Ghi ra, chuyển cho Ice** — tiền lệ quyết định #3 Story 1.3.
2. **Bảng 14 token ⛔ không có hàng cho nguyên văn Latin**, trong khi FR19 phủ tiếng Anh
   tường minh *(Quyết định #6)*.
3. **Mockup vẽ tab ở thanh tiêu đề panel**, nhưng Story 1.14 §Quyết định #4A đã gỡ `<header>`
   *(Quyết định #5)*. Cùng hạng với `deferred-work.md:521` — *đường vào ⛔ không có mockup nào*.

### Bàn giao — năm mục `deferred-work.md` gọi đích danh Story 1.16

| Dòng | Mục | Story này làm gì |
|---|---|---|
| `:115` | Cờ `wraps` của `ui-md` — *"Nhặt lại ở Story 1.16/1.17"* | ⚠️ **⛔ KHÔNG chốt** *(quyết định của Ice, chạm `DESIGN.md`)*. Story này chỉ phải bảo đảm **nhãn tab của chính nó ⛔ không xuống dòng** |
| `:129`, `:131` | Bề mặt ĐỌC phải khai `read-*` / `source-*` của chính nó | ✅ **Đóng nửa Source** (AC1, AC2). Nửa Lookup ở 1.17, nửa Editor ở Epic 2 |
| `:133` | `--synthesis-*` chưa có người tiêu thụ | ✅ **Đóng** (AC7) |
| `:504` | `applyPreset()` `api.clear()` mất trạng thái panel | ✅ (AC9) |
| `:525` | `replace_open_work` thả `Store` trong vùng khoá | ✅ (AC10) |

### 🧠 Trí tuệ từ story trước — thứ đắt tiền, ⛔ đừng học lại bằng tiền

- **Story 1.15:** một `invoke()` trượt bằng thứ ⛔ không phải `IpcError` **khi có cầu IPC** là
  một **lỗi THẬT**, ⛔ không phải *"chạy ngoài Tauri"*. Nuốt nó thành `{null, null}` cho ra
  thất bại **im lặng** ở đúng thao tác đầu tiên. Đường mới của Task 1 dùng **lại** khuôn
  `callCreateWork`, ⛔ không viết khuôn thứ hai.
- **Story 1.14:** `dockview-vue` mount **mọi** component với **đúng một** prop tên `params`
  (`src/layout/panelProps.ts`). ⛔ Đừng khai prop khác cho `SourcePanel`.
- **Story 1.14:** trạng thái tiêu điểm đọc từ **DOM thật**, ⛔ không từ `activePanel` của
  dockview — hai thứ tách nhau ngay lần đầu bấm chuột vào thân panel khác. **⛔ Đừng đụng
  `PanelFrame` ở vế này.**
- **Story 1.13:** *"⛔ không một câu SQL nào được chuẩn bị"* cho trạng thái không hỗ trợ —
  trạng thái phải **phân biệt được** với *"đã chạy mà ⛔ không tìm thấy"*. Áp thẳng cho ca
  **0 lớp từ điển** và ca **ký tự ⛔ không có âm**: ba trạng thái, ⛔ không phải một.
- **Story 1.6/1.5:** ⛔ **không** dựng một `#[tauri::command]` giả chỉ để đóng một mục. Nhưng
  story này có nhu cầu IPC **thật** — Task 1 ⛔ không phải một command giả.

### Testing standards

Bộ DoD **chín lệnh** *(khuôn Story 1.14/1.15)* — mã thoát là phán quyết, ⛔ không phải đầu ra:

```
cargo test --manifest-path src-tauri/Cargo.toml
npm run build            npm run check:tokens     npm run check:i18n
npm run check:commands   npm run check:layout     npm run check:deps
npm run check:dict-manifest                       npm run check:scope
```

- **⛔ Không có bộ chạy test frontend, và ⛔ không được thêm** *(NFR15, quyết định của Ice
  đã chốt ở 1.5 và giữ qua bốn story)*. ⇒ vế DOM nghiệm thu bằng **bảng chạy tay có số**,
  ghi vào §Debug Log References — **⛔ không** bằng văn xuôi.
- **Test Rust là test HÀNH VI qua biên**, đặt ở `src-tauri/tests/**`. Đường Hán Việt thuộc
  họ `dict_*` — **dùng lại fixture của `dict_sources.rs`/`dict_lookup.rs`**, ⛔ đừng dựng
  bộ fixture thứ hai.
- 🔴 **Test FR36 phải XOÁ TỆP THẬT rồi chạy lại**, ⛔ không mock — `epics.md:816` nói nguyên
  văn: *"Nghiệm thu FR36 bằng test thật: **xoá file `.db` rồi chạy lại toàn bộ bộ test tra
  cứu — phải vẫn xanh**"*.
- **Đỏ-rồi-xanh cho mọi cổng bị đụng**: mỗi mệnh đề mới phải có **ít nhất một ca làm cổng
  ĐỎ** cộng **một đối chứng âm**. Con số ghi vào Completion Notes.

### Project Structure Notes

```
src-tauri/src/
  commands/<đường đọc Chương>.rs   NEW   hàm thuần + mod wire — khuôn commands/project.rs
  commands/mod.rs                  UPDATE
  commands/project.rs              UPDATE  vá replace_open_work (AC10)
  lib.rs                           UPDATE  generate_handler![…] — ⛔ KHÔNG đụng capabilities/
  ports/dict_source.rs             UPDATE  method thứ ba — ⛔ KHÔNG gõ tên crate SQLite
  core/dict/mod.rs                 UPDATE  kiểu bản ghi + tầng gom
  core/dict/query.rs               UPDATE  câu SQL theo lô (headword + headword_simp)
  core/dict/layer.rs               UPDATE  impl method mới
  tests/dict_*.rs, tests/project_contract.rs, tests/ipc_contract.rs   UPDATE

src/
  panels/SourcePanel.vue           UPDATE  ⛔ thay doc-comment "khung, không phải nội dung"
  panels/PanelFrame.vue            UPDATE  prop show-status — status-key Ở LẠI LITERAL
  panels/README.md                 UPDATE  hàng 1.16 → ✅
  <bề mặt Hán Việt>                NEW     ⚠️ khai token của CHÍNH NÓ
  <state panel Source>             NEW     ⚠️ NGOÀI component (AC9)
  config/<adapter IPC Chương>.ts   NEW     khuôn src/config/project.ts
  commands/index.ts                UPDATE  hai command mới + phím
  i18n/vi.json                     UPDATE
  tokens/tokens.json               UPDATE  token thứ 16 + deviations (nếu #6 = (a))
scripts/check-*.mjs                UPDATE  chỉ các hằng *_FLOOR + bảng deviations
```

⚠️ **`src/commands/**` ⛔ không được `import` Vue** — `scripts/check-commands.mjs` nạp thư mục
đó bằng **Node thuần**. Hướng phụ thuộc một chiều: `panels/` → `commands/`.

### 📌 Bối cảnh git

`564be15` *(1.15 — Library import form)* · `c3efb20` *(1.14 — bốn panel + `dockview`; cùng lượt
mang `core/dict/{layer,mod,senses}.rs` và `ports/dict_source.rs` của 1.13)* · `7e38de8`
*(test hành vi `core::matching`)* · `dd7af61` *(`VIWIKTIONARY_EN` — 1.10b)*.

**Đọc gì trước khi gõ:** `src/panels/PanelFrame.vue` *(trọn — nó là thứ Task 5 sửa)* ·
`src/panels/README.md` · `src/layout/panelProps.ts` · `src-tauri/src/ports/dict_source.rs`
*(trọn — doctrine cổng)* · `src-tauri/src/core/dict/mod.rs:200-330` · `src-tauri/src/commands/project.rs`
*(khuôn hàm-thuần-vỏ-mỏng)* · `src/config/project.ts` *(khuôn adapter IPC)*.

### 🌐 Phiên bản đang ghim — ⛔ KHÔNG đổi một dòng nào

`tauri 2.11.5` · `tauri-runtime 2.11.3` · `rusqlite` bundled · `dockview-vue` · Rust toolchain
`1.97.1`. **⛔ 0 phụ thuộc mới, cả npm lẫn crate** — mọi phụ thuộc mới phải qua rà NFR15 và
vào bảng Stack **trước**, và `check-deps.mjs` có danh sách cấm cùng ngưỡng sàn.

### References

- `epics.md:1672-1704` — Story 1.16, năm mệnh đề AC · `:1706+` — Story 1.17 *(ranh giới ngay dưới)*
- `epics.md:102` FR19 · `:142` FR33 · `:170` FR50 · `:184` FR113 · `:816` *(nghiệm thu FR36 bằng test thật)*
- `prd.md:399` — FR19 nguyên văn
- `ARCHITECTURE-SPINE.md:75` AD-1 · `:81` AD-2 · `:147` AD-10 · `:218` AD-16 · `:290` AD-19 ·
  `:302` AD-21 · `:406` AD-34 · `:435` *(âm Hán Việt đọc **qua cổng**, ⛔ không cài lại)* ·
  `:639` *(ánh xạ tên: Hán Việt → `HanViet`)*
- `DESIGN.md:266-267` — `source-cjk` · `source-hanviet` · `:225` UX-DR12 · §Giãn dòng 1.66
- `EXPERIENCE.md:410` — FR113, ví dụ `北涼 → Bắc Lương`
- `mockups/key-screen-workspace.html:96-105` — dải tab `Trung`/`Hán Việt` + khối `.hv`
- `deferred-work.md:115` · `:129-133` · `:504` · `:525` · `:527`
- `tools/dict-build/src/schema.rs:41-51` *(`han_viet` là **ÂM ĐỌC**, ⛔ không phải NGHĨA)* ·
  `sources/unihan.rs:116` *(`kVietnamese`)* · `sources/thieu_chuu.rs:70` *(nhiều âm tách `|`)*

---

## Câu hỏi cho Ice

1. ~~**Quyết định #4** — (a) hay (b)?~~ → ✅ **ĐÓNG 2026-08-06: (a) cặp theo từng ký tự.**
2. **Chữ có nhiều âm — báo hay ⛔ không báo?** Mặc định story: **⛔ không báo** *(⛔ không
   mockup nào vẽ, và đường rẻ nhất — màu `ornament` — bị UX-DR5 cấm)*. Nếu Ice muốn báo thì
   đó là một lượt thiết kế thị giác, ⛔ không phải một dòng CSS.
3. **Ký tự ⛔ không có âm ở bất kỳ lớp nào hiện ra thế nào?** Đề xuất: một ký tự giữ chỗ màu
   `on-surface-variant` **giữ được cột** ở chế độ song song. ⛔ **Không** dùng `ornament`
   *(UX-DR5: màu của nét, ⛔ không bao giờ của chữ)*, ⛔ **không** dùng `opacity` *(UX-DR6)*.
4. ~~Phát hiện của Quyết định #1 vượt khỏi story này.~~ → ✅ **ĐÓNG 2026-08-06** — nhận về
   story **`1-10c`**. ⚠️ **Một dữ kiện Story 3.7 (FR113) thừa hưởng và phải biết:** sau `1-10c`,
   chất lượng đề xuất bản dịch **phụ thuộc vào hai lớp GỠ RỜI** *(Thiều Chửu + Trần Văn Chánh)*.
   FR36 vẫn đúng — gỡ chúng ⛔ không làm hỏng đường nào — nhưng phủ tụt **12.463 → 1.136**.
   Đó là **degradation có tên**, và 3.7 phải phát biểu được nó chứ ⛔ không giả định 12.463.
5. **Cờ `wraps` của `ui-md`** *(`deferred-work.md:115`)* — story này ⛔ **không** chốt, đúng
   như mục đó ghi. Nhưng nó là story thứ hai bị nhắc tên. Ice có muốn đóng nó trong cùng
   lượt review không?

---

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (`claude-sonnet-5`), qua `/bmad-dev-story`.

### Debug Log References

- `cargo test --manifest-path src-tauri/Cargo.toml` — **205 test** xanh (12 tệp test tích hợp + unit test trong `src/`), 0 fail, 3 ignored *(⚠️ bản đầu ghi "255 test / 15 tệp" — đo lại ở lượt code review 2026-08-06 cho **205 / 12**, lệch +24 %)* *(có chủ ý: `bench_the_grouped_path_on_the_real_dictionaries` cần thư mục `.db` thật; hai `nom_guard`/`ac4_fr36…` của `tools/dict-build` cần `raw/` thật — cả ba đã `ignored` từ trước Story 1.16)*.
- Chín lệnh DoD — **cả chín xanh** ở lượt chạy cuối cùng: `cargo test` · `npm run build` · `check:tokens` · `check:i18n` · `check:commands` · `check:layout` · `check:deps` · `check:dict-manifest` · `check:scope` *(cổng cuối bị chặn suốt giữa phiên vì cổng 1420 bị một `tauri dev` khác của Ice chiếm — chạy lại được sau khi phiên đó tự đóng, KHÔNG bị dev tắt).*
- **Playwright headless Chromium** (`npx playwright`, cài tạm trong scratchpad, không vào cây phụ thuộc dự án) dùng cho hai phép đo THẬT mà câu chữ story đòi "⛔ không bằng lời hứa":
  - **AC6 — vùng chọn.** Bản đầu (`.hv-unit { display: inline-flex; flex-direction: column }`) cho `window.getSelection().toString()` trên đoạn *"他打開了那扇門，走進了黑暗之中。"* ra `"他\n打\n開了那扇門，\n走\n進了黑暗之中。"` — **ĐỎ**, dù `.hv-reading` đã `user-select: none`. Nguyên nhân: Chromium chèn `\n` ở MỌI ranh giới **hộp dòng** (line box), và cả flex container lẫn một `display: block` bên trong một inline-block đều tạo hộp dòng riêng — bất kể phần tử đó có được chọn hay không. Bản vá (`.hv-reading { position: absolute; top: 100%; … }`, ra khỏi luồng bố cục) cho ra đúng `"他打開了那扇門，走進了黑暗之中。"` — **XANH**. Kịch bản đo: `/private/tmp/.../scratchpad/selection_test3.mjs` + `hv-selection-test3.html`.
  - **Task 8 — trần render.** DOM dựng bằng `document.createElement` theo đúng cấu trúc `.hv-unit`/`.hv-char`/`.hv-reading` của `SourceHanViet.vue` đã biên dịch (đối chiếu bằng `grep` trên `dist/assets/*.css`, xem bảng số ở Completion Notes). Kịch bản đo: `perf_test.mjs` cùng thư mục.

### Completion Notes List

**Tổng quan.** Cả 11 Task hoàn tất. Tầng dữ liệu (Task 1–3) đọc âm Hán Việt qua đúng một
method mới trên cổng `DictionarySource`, gom theo thứ tự ưu tiên lớp gỡ rời → lớp nền, tách
nhiều âm bằng một luật cho cả hai hình dạng thật. Tầng giao diện (Task 4–9) dựng dải tab
Trung/Hán Việt, hai kiểu xem, và một trần render có tên — với MỘT lỗi thật bị bắt và vá nhờ
đo bằng Playwright thay vì tin CSS trên giấy (xem Debug Log References).

**Task 0 — bốn quyết định chốt bằng mặc định đề xuất** *(#2 method theo lô · #3 tách `|`+khoảng trắng · #5 dải tab ở thân + prop `show-status` · #7 trần có tên, số chốt ở Task 8)*, không có bất đồng nào phát sinh so với đề xuất trong story. Xem Change Log.

**AC5 / FR36 — con số phủ kế thừa từ Story 1-10c, KHÔNG đo lại ở story này.** `12.463 → 1.136`
là số đo của chính `1-10c` (dữ liệu THẬT, `tools/dict-build/out/*.db`) và không đổi bởi mã của
story này — 1.16 chỉ TIÊU THỤ cột `han_viet` đã sạch, ⛔ không tính lại độ phủ. Test FR36 của
1.16 (`removing_every_detachable_layer_still_serves_readings_from_the_base_layer`,
`tests/dict_sources.rs`) xác nhận đúng CƠ CHẾ degradation (xoá lớp gỡ rời ⇒ rơi về lớp nền,
⛔ không đường nào hỏng) trên fixture ba lớp — ⛔ không xoá tệp `.db` thật vì các tệp đó
⛔ không tồn tại trong git (AD-25) và cũng ⛔ không được build trong phiên làm việc này.

**Task 8 — bảng số đo THẬT (Playwright headless Chromium, `--enable-precise-memory-info`):**

| Số ký tự Hán | Kiểu **song song** — tới frame đầu | Kiểu **chuyển đổi** — tới frame đầu | Δ heap (song song) |
|---|---|---|---|
| 5.000   | 163,7 ms   | 2,7 ms   | 0,26 MiB |
| 50.000  | 1.408,5 ms | 24,2 ms  | 0,43 MiB |
| 500.000 | 13.621,5 ms | 222,4 ms | 0,92 MiB |

Trần chốt: **50.000 ký tự Hán** cho kiểu song song *(`PARALLEL_VIEW_RENDER_CEILING` ở
`sourcePanelState.ts`)* — 1,4 s là ranh giới còn chấp nhận được cho một thao tác chạy MỘT LẦN
mỗi lượt nạp Chương (⛔ không phải đường nóng NFR1 — Quyết định #7), 13,6 s ở 500k thì ⛔
không. Kiểu chuyển đổi ⛔ không có trần — 222 ms ở 500k vẫn rẻ. Vượt trần ⇒ nút đổi kiểu xem
bị ẩn (`SourcePanel.vue`) VÀ bề mặt tự rơi về chuyển đổi nếu `viewMode` vẫn mang giá trị
`'parallel'` từ trước (`effectiveViewMode` ở `SourceHanViet.vue`) — hai hàng rào, ⛔ không một.

⚠️ **Giới hạn của phép đo:** DOM dựng thẳng bằng `document.createElement` mô phỏng ĐÚNG cấu
trúc đã biên dịch của component, chạy trên **headless Chromium**, ⛔ không phải WKWebView/
WebView2 thật và ⛔ không đi qua bộ máy reactivity của Vue (tạo `.hv-unit` bằng tay rẻ hơn
Vue một chút vì bỏ qua VDOM diff — số thật trong ứng dụng có thể cao hơn nhẹ). Đây là **cận
dưới hợp lý**, ⛔ không phải con số cuối cùng đã đóng dấu trên hai nền tảng thật.

**Task 9 / AC9 — đúng cấu trúc mã, CHƯA đo bằng webview thật.** `ensureChapterLoaded`/
`ensureHanVietLoaded` dùng cờ module-level (`chapterRequested`/`hanVietRequested`) nên về mặt
cấu trúc KHÔNG THỂ gọi lại IPC ở lượt mount thứ hai — nhưng phiên làm việc này không có một
instance `tauri dev` rảnh để bấm `Mod+Alt+1`/`Mod+Alt+2` và đọc DevTools Network thật (chỉ có
`check:scope`, tự đóng cửa sổ ngay sau self-check). Ghi thẳng thay vì đánh dấu đạt rồi im.

**Quyết định kỹ thuật KHÔNG có trong bảng Project Structure Notes của story:**
1. **Command IPC thứ hai** (`commands::dict::wire::read_han_viet`), tách khỏi
   `read_open_chapter` — lý do: gộp chung buộc MỌI lượt mở Chương (kể cả nguồn tiếng Anh,
   AC3 — ⛔ không tab Hán Việt) tính qua ba tệp `.db` từ điển. `read_open_chapter` giữ đúng
   phạm vi AC8 (`source_text` + `source_lang`); `read_han_viet` chỉ gọi khi `source_lang ===
   'zh'`. ⛔ Không phải cổng thứ tư (AD-2) — vẫn là `DictionarySource`, chỉ thêm một vỏ IPC.
2. **`sources_used` mang `dict_source.code`, ⛔ không `display_name`.** FR31 (nhãn nguồn bắt
   buộc) thoả bằng `code`; ánh xạ sang tên hiển thị đẹp là việc của màn hình Attribution
   (**Story 10.4**, đã ghi rõ trong Ranh giới phạm vi của chính story này) — Panel Source
   hôm nay hiện thẳng mã (`fx-hv`, `thieu-chuu`, …), ⛔ không phải "Thiều Chửu".
3. **§Câu hỏi cho Ice #2 (nhiều âm)** — dùng mặc định story: ⛔ không đánh dấu gì cho ca một
   ký tự nhiều âm; danh sách đầy đủ (`HanVietReading.all`) đã đi qua IPC, sẵn cho Story 1.17.
   **§Câu hỏi #3 (ký tự không âm)** — dùng mặc định: hai chuỗi `vi.json` riêng theo
   `layersLoaded`, ⛔ không ô trống câm. Cả hai ⛔ chưa được Ice xác nhận lại trong phiên này.

**Ba mâu thuẫn tài liệu đã phát hiện ở Task 0 của story** *(Unihan/Hán Việt, bảng token
thiếu Latin, mockup vẽ tab ở thanh tiêu đề)* — cả ba đã **đóng bằng mã** trong story này
(#1 đã đóng ở 1-10c; #2 đóng bằng token thứ 16; #3 đóng bằng Quyết định #5), ⛔ không sửa
tài liệu quy hoạch — đúng tiền lệ đã ghi.

**Vế thị giác CHƯA đo được, nói thẳng (Task 11):** hành vi hai nền tảng (WKWebView macOS,
WebView2 Windows) của dải tab, bề mặt song song, và `font-synthesis` chữ Hán nghiêng giả —
dự án ⛔ không có runner đo được vế đó (`deferred-work.md:478`, món nợ cũ, ⛔ không phải món
nợ mới của story này). Phép đo Playwright ở trên là **Chromium**, một engine thứ ba, không
phải một trong hai engine mục tiêu — nó xác nhận CƠ CHẾ (position:absolute thoát hộp dòng)
đúng theo đặc tả CSS, nhưng ⛔ không thay được một lượt nghiệm thu mắt trên máy thật.

### File List

**Mới:**
- `src-tauri/src/commands/chapter.rs`
- `src-tauri/src/commands/dict.rs`
- `src-tauri/src/core/dict/han_viet.rs`
- `src/config/chapter.ts`
- `src/config/dict.ts`
- `src/panels/SourceHanViet.vue`
- `src/panels/sourcePanelState.ts`

**Sửa:**
- `_bmad-output/implementation-artifacts/deferred-work.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `scripts/check-tokens.mjs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/commands/project.rs`
- `src-tauri/src/core/dict/layer.rs`
- `src-tauri/src/core/dict/mod.rs`
- `src-tauri/src/core/i18n/mod.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/ports/dict_source.rs`
- `src-tauri/tests/dict_sources.rs`
- `src-tauri/tests/project_contract.rs`
- `src/commands/index.ts`
- `src/i18n/vi.json`
- `src/main.ts`
- `src/panels/PanelFrame.vue`
- `src/panels/README.md`
- `src/panels/SourcePanel.vue`
- `src/tokens/tokens.json`

---

### Review Findings

*Lượt code review 2026-08-06 — ba tầng song song (Blind Hunter · Edge Case Hunter · Acceptance Auditor), cả ba xanh, ⛔ không tầng nào hỏng. Mọi mức nghiêm trọng dưới đây do lượt triage tự chấm sau khi **đọc code và đo lại trên bốn tệp `.db` thật**, ⛔ không lấy mức của subagent.*

#### Quyết định — ✅ ĐÃ CHỐT 2026-08-06 (Ice), lượt code review

- [x] ✅ **CHỐT: nới ô theo âm + ĐO LẠI AC6.** ⇒ chuyển thành **patch**. [Review][Decision→Patch] **Kiểu song song: âm Hán Việt chồng đè lên nhau** — `.hv-reading` là `position: absolute` + `white-space: nowrap` ⇒ ⛔ không góp một pixel nào vào bề rộng `.hv-unit` (16,5px), trong khi `"chênh"` ≈ 30px. Mọi cụm Hán liền nhau cho ra âm đè lên nhau. Đây là **cái giá của chính lượt vá AC6** (`inline-flex` → `position: absolute`): mọi cách bù bề rộng đều có nguy cơ dựng lại hộp dòng và làm `Selection.toString()` đỏ trở lại. Cần Ice chốt hướng + **đo lại AC6 sau khi vá**. [`src/panels/SourceHanViet.vue:182-217`] — mức: **high**
- [x] ✅ **CHỐT: một ký tự giữ chỗ giữ được cột** *(đúng đề xuất §Câu hỏi cho Ice #3 — màu `on-surface-variant`, ⛔ không `ornament`, ⛔ không `opacity`)*, câu giải thích chuyển lên **một dòng duy nhất** đầu tab. ⇒ chuyển thành **patch**. [Review][Decision→Patch] **Trạng thái 0 lớp từ điển (mặc định của mọi bản build hôm nay) cho ra một bức tường chữ** — `src-tauri/resources/dict/` rỗng trong git (AD-25), nên `layers_loaded === false` là ca thường. Mỗi ký tự Hán bị thay bằng nguyên câu `"chưa có dữ liệu"` ⇒ kiểu chuyển đổi in ra `chưa có dữ liệuchưa có dữ liệu…`. §Câu hỏi cho Ice #3 đề xuất **một ký tự giữ chỗ giữ được cột**, ⛔ không phải một câu đầy đủ nhân theo số ký tự — đề xuất đó chưa từng được Ice xác nhận. [`src/panels/SourceHanViet.vue:87-90`] — mức: **high**
- [x] ✅ **CHỐT: GHI NỢ tường minh cho Story 1.18.** Lý do: cài `tabindex` lên bề mặt văn bản đụng hợp đồng tiêu điểm mà Story 1.14 dặn ⛔ không chạm; 1.18 nhận cả hợp đồng vùng chọn lẫn vế bàn phím **cùng một lượt**. ⇒ chuyển thành **defer**. [Review][Decision→Defer] **Bôi đen bằng BÀN PHÍM chưa cài, chưa đo, chưa ghi nợ** — §⛔KHÔNG-LÀM ① giao cho story này **đúng một** nghĩa vụ tiên quyết của 1.18: nguyên văn phải bôi đen được *bằng chuột **và bằng bàn phím***. Vế chuột đã đo bằng Playwright; vế bàn phím ⛔ không xuất hiện một lần nào trong Tasks/AC/Completion Notes. Cài `tabindex` lên bề mặt văn bản đụng hợp đồng tiêu điểm mà Story 1.14 dặn ⛔ không chạm ⇒ cần Ice chốt: cài ở đây, hay ghi nợ tường minh cho 1.18. — mức: **medium**
- [x] ✅ **CHỐT: đổi sang `Mod+Alt+J`** *(hợp âm trống, ⛔ không mang nghĩa hệ thống trên macOS lẫn Windows; `Mod+Alt+O`/`Mod+Alt+V` sạch, giữ nguyên)*. ⇒ chuyển thành **patch**. [Review][Decision→Patch] **`Mod+Alt+H` trùng phím hệ thống macOS** (`⌘⌥H` = Hide Others). `check:commands` chỉ kiểm trùng **nội bộ** bộ command, ⛔ không biết gì về phím OS. Cần chọn hợp âm khác hoặc chấp nhận. [`src/commands/index.ts:412`] — mức: **medium**

#### Cần vá

- [x] [Review][Patch] **Kiểu chuyển đổi nối các âm bằng chuỗi RỖNG** — `北涼` ra `"bắclương"` thay vì `Bắc Lương`, đúng ví dụ trụ cột mà story trích từ `EXPERIENCE.md:410`; mockup `key-screen-workspace.html:99` ghi rõ dạng đúng có dấu cách. [`src/panels/SourceHanViet.vue:96-104`] — mức: **high**
- [x] [Review][Patch] **`split_readings` bỏ sót quy ước `,`** — Quyết định #3(a) hứa *"một luật tách áp cho MỌI tệp"*, nhưng luật chỉ cắt trên `|` và khoảng trắng. **Đo lại trên `.db` thật:** `dict-core.db` (lớp **NỀN** — chính lớp FR36 rơi về) có **284/1145 = 24,8 %** hàng dùng `,`; `dict-tran-van-chanh.db` (lớp gỡ rời **ưu tiên cao nhất**) có **2.326** hàng. Hai kiểu hỏng thật: `西 → "tây,tê"` (⛔ không khoảng trắng) hiện nguyên chuỗi như một âm; `譫 → "chiêm, thiềm"` cho `primary = "chiêm,"` — **dấu phẩy đuôi lên màn hình** (`.map(str::trim)` chỉ cắt khoảng trắng). Mục bàn giao của `1-10c` trong `deferred-work.md` đã cảnh báo đích danh ba quy ước và bị bỏ qua. [`src-tauri/src/core/dict/mod.rs:691-697`] — mức: **high**
- [x] [Review][Patch] **Tab Hán Việt ⛔ không có vùng cuộn** — `.original` có `overflow: auto`, nhánh `v-else` thì ⛔ không; `.hv-surface` chỉ khai `min-height: 0` và `.panel` cha là `overflow: hidden` ⇒ mọi thứ vượt chiều cao panel bị cắt và ⛔ không với tới được. Chính AC9 mở đầu bằng *"đã cuộn xuống"* — trạng thái đó ⛔ không tồn tại được. [`src/panels/SourceHanViet.vue:137-139`] — mức: **high**
- [x] [Review][Patch] **State module-level ⛔ không bao giờ được đặt lại ⇒ Tác phẩm thứ hai hiện nội dung Tác phẩm thứ nhất** — `chapterRequested`/`hanVietRequested` đặt `true` vĩnh viễn, ⛔ không đường reset nào. Đường chạm là đường sản phẩm bình thường: tạo Tác phẩm A → Workspace → về Library → tạo Tác phẩm B → Workspace vẫn là A, kèm âm Hán Việt của A và `source_lang` của A (⇒ tab Hán Việt hiện/ẩn sai). [`src/panels/sourcePanelState.ts:60,64,163-174`] — mức: **high**
- [x] [Review][Patch] **Mọi lỗi IPC bị nuốt — `err.project.no_work_open` của AC8 ⛔ không bao giờ tới được người dùng** — `sourceChapterError`/`sourceHanVietError` được export nhưng ⛔ không một chỗ nào import. `store.read_failed` (kho hỏng) hiện ra thành câu trạng thái bình thường *"Chưa có Chương nào được mở."* Toàn bộ đường AC8 chứng minh lỗi được **tạo** đúng, ⛔ không gì chứng minh nó được **hiện**. [`src/panels/sourcePanelState.ts:98,100`] — mức: **high**
- [x] [Review][Patch] **Ba trạng thái của AC4 sập thành một ở đúng đường lỗi và đường chờ** — `layersLoaded` mặc định `true` khi `hanViet === null`, nên cả ca *đang chờ IPC* lẫn ca *lượt tra thất bại* đều hiện `"không rõ"` cho **mọi** ký tự — tức khẳng định dứt khoát *"đã tra mà không có âm"* trong khi sự thật là *"chưa tra được"*. Cộng thêm `hanVietRequested = true` đặt **trước** `await` ⇒ ⛔ không có lượt thử lại nào, kể cả khi `error.retryable`. [`src/panels/sourcePanelState.ts:111,142-157`] — mức: **high**
- [x] [Review][Patch] **Văn bản rỗng ⇒ panel trống câm** — `hasChapter` chỉ kiểm `!== null`, ⛔ không kiểm nội dung, và `:show-status="!hasChapter"` tắt luôn câu trạng thái. Đường chạm thật: `create_work_from_file` ⛔ không có sàn dưới, nhận được tệp 0 byte. [`src/panels/SourcePanel.vue:30,45-46`] — mức: **medium**
- [x] [Review][Patch] **Test `han_viet_across_many_batches_never_duplicates_or_drops_a_row` là test giả** — nó chèn `山`/`國` mà cả hai chỉ khớp qua **một** trường, nên điều kiện tiên quyết của lỗi trùng-hàng ⛔ không bao giờ được dựng ra. Thêm `"国"` vào tập `many` là đủ làm nó đỏ. [`src-tauri/tests/dict_sources.rs:2099`] — mức: **medium**
- [x] [Review][Patch] **Test xoá tệp `.db` khi lớp vẫn đang mở — đỏ trên Windows (NFR14)** — vi phạm đúng Luật 2 mà doc-comment đầu chính tệp đó đặt ra (`dict_sources.rs:30-31`), và test anh em `removing_every_detachable_layer…` thì làm đúng. [`src-tauri/tests/dict_sources.rs:2207,2223`] — mức: **medium**
- [x] [Review][Patch] **AC11 chưa thực hiện — ⛔ không một `*_FLOOR` nào được nâng** — Task 10 lập luận *"sàn thì số thật vẫn vượt qua nên ⛔ không cần đụng"*, nhưng tiền lệ 1.14 · AC11.1 mà chính AC viện dẫn nói ngược lại (`VUE_FLOOR = 1` được nâng lên 9 vì *"⛔ không còn canh được gì"*). Số thật hôm nay: `VUE_FLOOR` 9 vs **12** · `TS_FLOOR` 16 vs **23** · `COMMAND_FLOOR` 10 vs **16** · `CLICK/DISPATCH_FLOOR` 3 vs **8**. Ở mức này, xoá 5 trong 8 lời gọi `dispatch()` — kể cả cả ba lệnh mới của story — vẫn cho `check:commands` XANH. [`scripts/check-commands.mjs:200-222`] — mức: **medium**
- [x] [Review][Patch] **Hợp đồng ARIA của dải tab khai một nửa** — có `role="tablist"`/`role="tab"`/`aria-selected` nhưng ⛔ không `role="tabpanel"`, ⛔ không `aria-controls`, ⛔ không roving `tabindex`. Khai một nửa tệ hơn ⛔ không khai: nó hứa một mô hình tương tác không tồn tại. [`src/panels/SourcePanel.vue:47-63`] — mức: **medium**
- [x] [Review][Patch] **`read_han_viet` trả hàng TRÙNG và âm phụ thuộc vị trí ký tự trong Chương** — `query_set` là tập **đầy đủ** chứ ⛔ không theo lô, nên một hàng có `headword`/`headword_simp` rơi vào hai lô khác nhau bị đẩy hai lần; và doc-comment `mod.rs:747-748` khẳng định *"hàng có `id` nhỏ nhất thắng"* — mệnh đề đó chỉ đúng **trong một lô**. ⚠️ **Đã đo: ⛔ KHÔNG CHẠM TỚI ĐƯỢC trên dữ liệu thật hôm nay** — cả ba tệp `.db` có **0** hàng mang `headword_simp` khác `headword` ở cột `han_viet`. Là lỗi **tiềm ẩn** trên một API công khai mà Story 1.17/3.7 sẽ tiêu thụ, cộng một doc-comment sai. [`src-tauri/src/core/dict/han_viet.rs:93,107-123`] — mức: **low**
- [x] [Review][Patch] **`\r` bị XOÁ chứ ⛔ không chuẩn hoá** — đúng cho `\r\n` (Windows), nhưng tệp dùng `\r` đơn làm ký tự kết dòng bị nối thành một dòng duy nhất. Guard đúng: `/\r\n?/g → '\n'`. [`src/panels/SourcePanel.vue:34`, `src/panels/SourceHanViet.vue:67`] — mức: **low**
- [x] [Review][Patch] **`toggleHanVietView` thiếu guard `source_lang`** — `selectSourceTab` có, hàm này ⛔ không. Ở Tác phẩm tiếng Anh, `Mod+Alt+V` lật `viewMode` **vô hình**, và state module-level đó sống sót sang sau. [`src/panels/sourcePanelState.ts:193-196`] — mức: **low**
- [x] [Review][Patch→Defer] **⛔ KHÔNG VÁ — cần Ice.** Câu giao diện xuống dòng ⛔ không có token nào đỡ: **cả 6 token `ui-*` đều khai `wraps: false`**. Vá đúng nghĩa là đổi cờ `wraps` của `ui-md` *(`deferred-work.md:115` — quyết định của Ice, chạm `DESIGN.md`)* hoặc thêm token thứ 17 — cả hai đều ngoài quyền một lượt code review. Lỗ hổng **của bảng token**, cùng hạng với hàng `source-latin` mà Quyết định #6 vừa vá. Áp cho `.parallel-note` (có sẵn) lẫn `.load-error`/`.hv-notice` (mới). Ghi ra thay vì tự chế token. — mức: **low**
- [x] ~~[Review][Patch] `.parallel-note` … token `ui-sm` khai `wraps: false`~~ (giãn dòng 1.5, dưới sàn 1.66). `check-tokens.mjs` chỉ áp `LINE_HEIGHT_FLOOR` cho token khai `wraps: true` nên cổng mù — đúng cơ chế Bẫy 6, chỉ khác chỗ đứng (chọn sai token thay vì quên khai token). [`src/panels/SourcePanel.vue`] — mức: **low**
- [x] [Review][Patch] **Hằng `"base"` có hai bản, ⛔ không lưới canh** — `BASE_LAYER_NAME` (`core/dict/mod.rs:231`) và `BASE_LAYER` (`core/dict/layer.rs:59`). ⛔ Không phải vi phạm AD-10 (đây là phân loại cấu trúc, ⛔ không phải danh tính nguồn), nhưng đổi một bên mà quên bên kia làm `priority_order()` **đảo ngược im lặng** — lớp nền thắng lớp gỡ rời, lật đúng Quyết định #1, ⛔ không cổng nào đỏ. — mức: **low**
- [x] [Review][Patch] **Doc-comment `sourceChapterError` nói ngược nghĩa** — viết *"`null` … khi ⛔ **có** cầu IPC"*, hành vi thật là `null` khi **KHÔNG** có cầu IPC. [`src/panels/sourcePanelState.ts:97`] — mức: **low**
- [x] [Review][Patch] **Test AC10 tạo chu trình `Arc`** — `ReentrantProbe` giữ `Arc` tới chính mutex chứa nó ⇒ refcount ⛔ không bao giờ về 0, giá trị cuối ⛔ không chạy `Drop`. ⛔ Không làm test sai (phép kiểm thật nằm ở `drop(second)`), nhưng assert cuối ⛔ không mạnh như doc-comment ngụ ý. [`src-tauri/src/commands/project.rs`] — mức: **low**
- [x] [Review][Patch] **§Debug Log References khai sai số test: `255` — đo lại là `205`** (0 fail, 3 ignored, 12 tệp `tests/*.rs` chứ ⛔ không phải 15). Lệch **+24 %**. Cả tám cổng `npm run check:*` + `npm run build` thì **xanh thật**, đã chạy lại từng cái. — mức: **low**

#### Ghi nợ (defer)

- [x] [Review][Defer] **Phép đo Task 8 ⛔ không đo đường mã thật** — bảng số dựng DOM bằng `document.createElement`, ⛔ không đi qua `buildSegments` (một object JS cho **mỗi** ký tự Hán) lẫn `switchText` (`.join()` trên 500.000 phần tử), mà cả hai **luôn chạy ở CẢ HAI kiểu xem**. ⇒ mệnh đề *"kiểu chuyển đổi ⛔ không có trần"* và hằng `PARALLEL_VIEW_RENDER_CEILING = 50_000` đứng trên một phép đo sai đối tượng. Cần một lượt đo lại trên component thật. — deferred, cần đo chứ ⛔ không vá được
- [x] [Review][Defer] **Trần render chỉ đếm ký tự Hán, bỏ qua node của mẩu không-Hán** — `buildSegments` `flush()` mỗi lần gặp một ký tự Hán, nên văn bản xen kẽ `漢a漢a…` với 49.999 ký tự Hán lọt qua trần nhưng dựng ~100.000 node. Bảng đo chỉ đo văn bản Hán liền mạch. — deferred, cần đo
- [x] [Review][Defer] **`source_lang` ⛔ không được validate ở tầng ghi** — `create_work_*` chèn nguyên văn, UI so `=== 'zh'` chính xác từng byte ⇒ `"ZH"`/`"zh-Hans"`/`"cmn"` cho một Tác phẩm tiếng Trung ⛔ không có tab Hán Việt, ⛔ không lỗi nào. Guard đúng nằm ở tầng ghi (Story 1.15), ⛔ không ở so sánh chuỗi phía UI. — deferred, có sẵn từ trước story này
- [x] [Review][Defer] **`read_open_chapter` với 0 Chương trả lỗi KHO thay vì lỗi có tên** — `query_row` ném `QueryReturnedNoRows` ⇒ `store.read_failed` ⇒ người dùng đọc *"không mở được kho dữ liệu"* cho một Tác phẩm lành lặn. AC8 dựng riêng `project.no_work_open` để ⛔ không trộn trạng thái sản phẩm vào từ vựng `store.*`, nhưng chỉ phủ nhánh `open == None`. Epic 1 luôn ghi đúng một Chương nên hôm nay chưa chạm; **Epic 2 (chọn/chuyển Chương) mở đúng nhánh này**. [`src-tauri/src/commands/chapter.rs:60-71`] — deferred, ⛔ chưa chạm tới được ở Epic 1

#### Những gì lượt review ĐÃ kiểm và ⛔ KHÔNG tìm thấy vi phạm

AD-2 *(⛔ không cổng thứ tư)* · AD-10 *(SQL một câu cho mọi tệp, ⛔ không lọc `source.code`, ⛔ không sổ đăng ký)* · AD-16 *(0 `v-html`)* · AD-21/NFR16 *(`check:i18n` xanh, 0 ký tự tiếng Việt trong `.rs`)* · Quyết định #2 *(đọc theo LÔ, ⛔ không N+1)* · Bẫy 2 *(phủ cả `headword` lẫn `headword_simp`)* · AC2/Quyết định #6 *(token thứ 16 đúng từng chữ, bảng đóng băng **vẫn 14 hàng**)* · Quyết định #5 *(dải tab ở thân, `status-key` ở lại literal)* · AC3 *(đọc `source_lang`, `v-if` ⇒ ⛔ không tồn tại trong DOM)* · AC5 *(`priority_order()` tính một lần, ⛔ không chuỗi lớp viết cứng, ⛔ không nhánh dự phòng đọc cột Nôm)* · AC7 *(cả **năm** biến `--*-source-hanviet` được tiêu thụ ở cả `.hv-reading` lẫn `.hv-switch`)* · **AC10** *(`swap_locked` + `drop` ngoài khoá, `ReentrantProbe` tự khoá lại trong `Drop` — một phép kiểm **thật**, một trong những chỗ làm tốt nhất của lượt này)* · NFR15 *(0 phụ thuộc mới)* · ⛔KHÔNG-ĐỤNG *(`tools/**`, `dict-manifest.toml`, `Cargo.toml`, `capabilities/main.json`, `planning-artifacts/**` đều sạch — phần `tools/**` trong cây làm việc thuộc `1-10c` chưa commit)* · parity 7 dải CJK giữa `isHanChar` (TS) và `is_han` (Rust) khớp **từng byte** · **trung thực về vế chưa đo** *(WKWebView/WebView2, AC9 bằng webview thật, engine Task 8 — cả ba ghi rõ, ⛔ không đánh dấu đạt rồi im)*.

#### Số đo THẬT của lượt vá — Playwright headless Chromium, 2026-08-06

**① AC6 giữ nguyên XANH sau khi vá chồng đè** *(điều kiện Ice đặt ra khi duyệt bản vá)*.
Câu đo: `他打開了那扇門，走進了黑暗之中。` với âm Thiều Chửu thật, bôi đen toàn bộ `.hv-parallel`.

| | Âm chồng lên nhau | `Selection.toString()` |
|---|---|---|
| Trước khi vá *(`.hv-unit` ⛔ không `min-width`)* | **8/13** cặp liền kề | XANH |
| Sau khi vá *(`min-width` theo độ dài âm)* | **0/13** | **XANH** |

⇒ Lỗi chồng đè là **thật và đo được** *(8/13)*, và bản vá đóng nó **⛔ không** hồi quy AC6 —
đúng như lý lẽ: `min-width` đổi **bề rộng**, ⛔ không thêm một node văn bản nào, mà
`Selection.toString()` nối các node văn bản.

**② Phần JS mà bảng số Task 8 BỎ SÓT** *(mục defer "phép đo sai đối tượng", nay ĐÓNG)*.
`buildSegments()` + `switchText` chạy ở **CẢ HAI** kiểu xem, bảng cũ ⛔ không đo:

| Số ký tự Hán | `buildSegments` | `switchText` | TỔNG phần JS |
|---|---|---|---|
| 5.000 | 2,3 ms | 0,2 ms | **2,5 ms** |
| 50.000 | 12,1 ms | 5,5 ms | **17,5 ms** |
| 500.000 | 169,4 ms | 68,2 ms | **237,5 ms** |

⇒ Kiểu **chuyển đổi** ở 500k: `222,4 + 237,5 ≈ **460 ms**` — tức bảng cũ bỏ sót **quá nửa**
chi phí. Nhưng **kết luận ⛔ không lật**: 460 ms cho một thao tác chạy MỘT LẦN mỗi lượt nạp
Chương vẫn rẻ ⇒ *"kiểu chuyển đổi ⛔ không có trần"* **vẫn đúng**, nay đứng trên một phép đo
**đủ**. Kiểu song song ở 50k: `1.408,5 + 17,5 ≈ 1.426 ms` — trần **50.000 giữ nguyên**.

**③ Bản vá `min-width` ⛔ không làm trần render xấu đi** *(kiểm chứng riêng, vì ô rộng ra thì
chi phí bố cục có thể đổi)*:

| Số ký tự Hán | `.hv-unit` CŨ | `.hv-unit` ĐÃ VÁ |
|---|---|---|
| 5.000 | 169,1 ms | **159,0 ms** |
| 50.000 | 1.567,3 ms | **1.609,2 ms** |

⇒ Chênh **+2,7 %** ở 50k, trong sai số giữa các lượt. `PARALLEL_VIEW_RENDER_CEILING = 50_000`
**⛔ không cần đổi**.

⚠️ **Giới hạn ⛔ không đổi:** cả ba phép đo trên chạy **Chromium**, ⛔ không phải WKWebView
(macOS) hay WebView2 (Windows). Đây là món nợ **có sẵn** của dự án (`deferred-work.md:478`),
⛔ không phải nợ mới của lượt vá này — nhưng nó ⛔ không được coi là đã nghiệm thu bằng mắt
trên hai nền tảng thật.

#### Cổng sau lượt vá — chạy lại đủ chín, 2026-08-06

`cargo test` **206 passed · 0 failed · 3 ignored** *(+1: `multiple_readings_split_on_the_comma_convention_too`)*
· `npm run build` · `check:tokens` · `check:i18n` · `check:commands` · `check:layout`
· `check:deps` · `check:dict-manifest` · `check:scope` — **cả chín XANH**.

🔴 **Đỏ-rồi-xanh, đo thật cho mệnh đề mới** *(§Testing standards đòi một ca làm cổng ĐỎ)*:
tạm hoàn nguyên phép lọc theo lô ở `read_han_viet` ⇒
`han_viet_across_many_batches_never_duplicates_or_drops_a_row` **ĐỎ** với **5 hit thay vì 3**
*(`国` và `國` mỗi ký tự nhân đôi)* — đúng bằng dự đoán. Khôi phục bản vá ⇒ **XANH**.
