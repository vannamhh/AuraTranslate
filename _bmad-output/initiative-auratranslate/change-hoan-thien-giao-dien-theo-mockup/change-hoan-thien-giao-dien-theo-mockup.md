---
type: change
created: 2026-10-13
skill: bmad-correct-course
title: 'Sprint Change Proposal — hoàn thiện giao diện theo mockup'
date: '2026-10-13'
trigger_story: '—'
scope_classification: 'Major'
status: 'approved'
approved_by: 'Ice'
workflow: 'bmad-correct-course'
mode: 'incremental'
---

# Sprint Change Proposal — 2026-10-13

Đầu vào: `handoff-buoc-1-bang-lech.md` (quyết định D1–D13), `bang-lech.md` (tổng hợp), `bang-lech-{a,b,c}.md` (từng dòng), `phan-loai-loai-5-{a,b,c}.md` (76 dòng loại 5 đối chiếu với mã, PRD, spine, sổ nợ). Tệp này chỉ giữ quyết định; chi tiết từng dòng nằm ở các tệp đó.

## 1. Tóm tắt vấn đề

App thật lệch xa mockup ở bố cục. Thanh tiêu đề dồn 3 tab chế độ cùng 10 nút lối vào (`src/App.vue`, `.titlebar-act`) thành một dãy trông như 13 tab, và không mockup nào đặt nút ở thanh tiêu đề. Library xếp mọi khối thành một cột 420 px, bỏ trống hai phần ba bên phải.

Bước 1 đối chiếu 29 mockup với mã và bốn ảnh Ice chụp: 135 dòng khớp, **118 dòng lệch cần sửa**, 30 dòng lệch có chủ đích, 51 dòng app có mà mockup không vẽ, **76 dòng mockup có mà app chưa có**, 17 dòng còn chờ ảnh.

**Loại vấn đề:** hiểu sai yêu cầu ở tầng thực thi. Các story xây đúng FR, nhưng mỗi story tự đặt lối vào của mình lên thanh tiêu đề và tự chọn hình dạng bề mặt, không đối chiếu mockup. Nguyên tắc Ice ký: giống mockup hết mức có thể; chỉ khác khi mockup không có chức năng đó; chỗ nào `DESIGN.md`/`EXPERIENCE.md` viết lại sau mockup thì văn bản thắng.

## 2. Phân tích tác động

### Tác động Epic

- **Epic mới: Epic 12** gom toàn bộ việc sửa bố cục (loại 2) và chức năng loại 5 mà Ice nhận (D7).
- **Epic 10:** 10.6 (ảnh hướng dẫn cài đặt) và 10.9 (nghiệm thu NFR17/WCAG) chờ Epic 12, vì cả hai chụp hoặc đo giao diện cuối. 10.5, 10.7, 10.8 không bị chặn.
- **Epic 1–9, 11:** không mở lại. Các bề mặt của chúng được Epic 12 sửa bố cục; FR của chúng giữ nguyên nghĩa.

### Tác động Story

Các quyết định trong plan story đã `built`/`done` bị thay, không sửa plan cũ (link ở đây là đủ):

| Plan | Quyết định cũ | Thay bằng |
|---|---|---|
| Story 3.6 (`story-trang-thai-cho-chot-va-dai-moc-chot-lan-dau-gap-plan.md`), Story 7.5 (`story-khop-mo-plan.md`) | Dải mọc ở slot chung trên `StatusBar` (Ice ký 2026-08-22) | Track hàng dưới câu đang sửa (D8, đề xuất 5) |
| Story 1.20 (`story-lich-su-tra-cuu-va-muc-da-ghim-plan.md`) | Quyết định #5: bỏ cả ba chip phạm vi | Hai chip `Phiên này · Chương này` (D13a, đề xuất 4) |
| Story 8.x thu hoạch | Chỉ duyệt trong bảng chờ chung | Thêm màn duyệt thu hoạch trên cùng các hàng (D13b) |
| Story 1.19 | Không phím riêng cho từng nguồn (§KHÔNG-LÀM ⑤) | **Giữ nguyên.** D12 viết "`⌥1…6` vẫn bật/tắt trực tiếp"; phím đó chưa từng tồn tại (đề xuất 1) |

### Xung đột artefact

- **PRD:** FR37 không cần sửa (đề xuất 1). Thêm FR136–FR152 (đề xuất 3).
- **EXPERIENCE.md:** §Information Architecture, §Component Patterns (Bản ghi từ điển, Bộ chọn nguồn, Lịch sử tra cứu, Dải mọc), §Bề mặt nhập › Hình dạng, KF-2 (đề xuất 6).
- **DESIGN.md:** §Vạch lề segment (đề xuất 6.6).
- **Mockup:** dòng 🔵 ở `sources-attribution.html` §1 và `lookup-history-pins.html`.
- **Spine:** chưa sửa. Bốn cửa G1–G4 giao Winston (mục 5).

### Tác động kỹ thuật

76 dòng loại 5 theo tầng: 41 chỉ webview · 26 cần lệnh Rust mới hoặc mở rộng · 8 cần migration hoặc dữ liệu mới · 1 chưa rõ. Không dòng nào bắt buộc thêm dependency; "Mở thư mục Tác phẩm" có một phương án dùng `tauri-plugin-opener` (NFR15 phải đọc giấy phép trước) và một phương án `std::process::Command`.

Rủi ro đáng kể nhất: D8 chèn một track hàng vào lưới `subgrid` năm cột (`src/panels/GridPanel.vue:2125`) mỗi lần dải mọc/thu. Chưa đo trên Chương lớn.

## 3. Đường đi được chọn

**Điều chỉnh trực tiếp bằng một epic mới.** Không rollback: các story cũ xây đúng năng lực; chỉ hình dạng bề mặt sai. Không thu hẹp MVP: Ice chọn làm luôn loại 5 (D7), trừ các cụm va với một quyết định đã ký hoặc với dữ liệu không tồn tại (đề xuất 2), những cụm này thành lệch có chủ đích có nguồn.

- **Công sức:** cao (khoảng 20 story, 4 pha mỗi story).
- **Rủi ro:** trung bình. Mỗi story dời nút phải mang theo đường lui tiêu điểm `data-*-open` (UX-DR17); vòng import mới có thể làm app không khởi động mà vitest vẫn xanh, nên chạy một spec e2e trước khi push.
- **Thời gian:** Epic 10 kéo dài thêm đúng phần 10.6 và 10.9.

## 4. Đề xuất sửa chi tiết (Ice duyệt từng mục, 2026-10-13)

### Đề xuất 1 — Popup nguồn (D2, D12) và FR37 · ✅ duyệt: (a) + (i)

- **FR37 giữ nguyên chữ.** "Người dùng bật/tắt từng nguồn từ điển trong panel Lookup" vẫn đúng khi popup mở từ nút ở đầu panel. Hình dạng popup ghi vào EXPERIENCE.md (6.3), không vào PRD.
- **Không phím riêng cho từng nguồn.** Ba lý do đo của Story 1.19 còn nguyên (`src/panels/dictSourcesState.ts:286-299`): sổ lệnh tĩnh của AD-34, `Mod+Alt+1/2` đã thuộc preset, FR22 đòi mọi lệnh gán lại được. Trong popup: `↑↓` chọn dòng, `Space` gọi `lookup.toggle_source`. Cột "phím" trong D12 bỏ.

### Đề xuất 2 — Phân loại 76 dòng loại 5 (D7) · ✅ duyệt

Khoảng 55 dòng vào Epic 12. Các cụm dưới đây xử lý khác "làm luôn"; dòng nào thành loại 3 thì ghi nguồn như mọi lệch có chủ đích khác, không thành nợ.

| # | Cụm (dòng) | Va với | Xử lý |
|---|---|---|---|
| 1 | Lọc bảng chờ theo phân loại đoán + bỏ hàng loạt (C2) | KHÔNG LÀM phiếu #56 (Story 11.4); không có bộ đoán | Loại 3 |
| 2 | Cộng dồn token/USD cả phiên (B24, B31) | KHÔNG LÀM phiếu #100 (2026-09-24) | Loại 3; token từng hàng lô (B21) vẫn làm |
| 3 | `⌘⌫` xoá lịch sử (B9) · "Tra cứu F3" (B3) | AD-49; F3 thuộc `tm.concordance` | ⌘⌫ loại 3; tiêu đề hiện phím thật của lệnh tra cứu |
| 4 | Duyệt rồi "Áp dụng" khi quét lại (A11, A12, A16) | FR99, AD-8: quét tự áp | Giữ tự áp (loại 3); thêm báo cáo sau quét (FR140) |
| 5 | Nhiều thư mục theo dõi (A14) | AD-23 | Loại 3 |
| 6 | Mục Cài đặt "Cập nhật" (A24) | FR111 là Story 10.7; tự kiểm khi khởi động trái AD-15 | Epic 12 dựng khung; nội dung của 10.7; tự kiểm khi khởi động loại 3 |
| 7 | Bốn nhãn xuất xứ trong lịch sử phiên bản (A17) | Migration `segment_version`; AD-47/AD-50 | Làm (FR141), sau cửa G2 |
| 8 | Hoàn tác `⌘Z` sau chốt Glossary (B18) | Trigger `glossary_entry_lifecycle_is_one_way`; AD-49/AD-36 | Làm (FR147), sau cửa G1 |
| 9 | Diff phía câu đích ở dải TM (B12) | Không có dữ liệu; AD-51 chỉ diff nguồn | Loại 3 |
| 10 | Prompt: cặp ngôn ngữ + phạm vi lời thoại/tả cảnh (C12) | PRD:114; không có bộ phân loại segment | Loại 3; Nhân bản (C13) vẫn làm |
| 11 | Lọc từ loại tiếng Trung (B6, B8) | `pos` NULL 100% ở cả 6 nguồn Trung | Loại 3 |
| 12 | Bốn công tắc "Nội dung" khi xuất (C17) | FR121; AD-38 | Đánh số câu + phụ lục thuật ngữ + đánh dấu chưa xác nhận chỉ ở bản hai cột (FR152); cột ghi chú trống loại 3 |
| 13 | Ngân sách token trước khi gửi (B26) | Spec 4.11 cấm tokenizer cục bộ | Token thật sau khi gửi + tỉ lệ chèn động; ước tính trước gửi loại 3 |
| 14 | Ba đề xuất ở dải chốt (B16) | "Bạn vừa viết" cần căn chỉnh cụm, chưa có | Hán Việt + TM (FR146); "bạn vừa viết" loại 3 |
| 15 | TM "chỉ dùng cặp của tôi" + xuất riêng (C9) | Cột mới trên `work` | Làm (FR149), sau cửa G4 |
| 16 | Tỉ lệ báo động giả (C14) | "Đã nhận/Bỏ qua" chưa lưu; AD-53 | Làm (FR150), sau cửa G3 |
| 17 | Minh hoạ gỡ lớp ở Attribution (B10) | Hình chứng minh FR36, không phải bề mặt | Loại 3 |
| 18 | Xuất xứ ở Chế độ đọc (C15) | Mockup ghi tên người, app không lưu tên | Làm (FR151) với ba giá trị xuất xứ có thật |
| 19 | Đưa cả lô sang (B22) · dấu thời gian chỉ mục (A10) · Mở thư mục Tác phẩm (A19) | Mỗi cái hai phương án hợp lệ | Story đo rồi đưa Ice chọn |

### Đề xuất 3 — PRD: FR136–FR152 · ✅ duyệt

**Tiêu chí:** một dòng cần FR khi người dùng làm được việc mới hoặc khi phải có dữ liệu mới từ Rust. Hiện lại dữ liệu đã có là chi tiết UX.

Mỗi FR đặt vào đúng mục năng lực trong `prd-auratranslate.md` §6, kèm dòng `> Thêm 2026-10-13 qua correct-course (change-hoan-thien-giao-dien-theo-mockup).` Thêm vào `covers` và hai chỗ ghi dải "FR1–FR135" của `initiative-auratranslate.md`.

🔵 2026-10-13, lúc áp: `spec-auratranslate/requirements.md` KHÔNG chép FR136–FR152. Tệp đó dừng ở FR131 (đầu tệp ghi "131 FR"); FR132–FR135 chưa từng được chép sang, nên chép riêng phần mới sẽ để một lỗ ở giữa. PRD là nguồn; đồng bộ lại `requirements.md` là việc riêng nếu Ice cần.

| FR | Mục | Văn bản |
|---|---|---|
| FR136 | 6.3 C3 | Bản ghi tra cứu hiện **âm đọc** cạnh đầu mục khi nguồn có dữ liệu âm đọc. Một chữ nhiều âm thì nghĩa nhóm theo âm, mỗi âm kèm số nghĩa. |
| FR137 | 6.1 C1 Nhập tài liệu | Màn xem trước nhập báo **ước lượng số ứng viên Glossary** sẽ vào bảng chờ, trước khi người dùng xác nhận nhập. Ước lượng chỉ đọc, không ghi bảng chờ (AD-20). |
| FR138 | 6.1 C1 Trạng thái & tiến độ | Danh sách Chương ghi **số câu chưa xác nhận** trên Chương đã xong, như FR120 làm ở Chế độ đọc. |
| FR139 | 6.1 C1 Tìm kiếm & duyệt | Tìm kiếm xuyên Library **thu hẹp phạm vi** (nguyên văn · bản dịch · cả hai) và **lọc** theo trạng thái Tác phẩm và hướng dịch. |
| FR140 | 6.9 C9 Nguồn sự thật | Sau khi quét lại thư mục, ứng dụng **báo những gì đã đổi** (mới · di chuyển · mồ côi). Người dùng **tìm lại** được một mục mồ côi bằng cách chỉ đường dẫn mới, và **dựng lại toàn bộ** chỉ mục Library. |
| FR141 | 6.9 C9 Lưu và phiên bản | Lịch sử phiên bản của một câu ghi **xuất xứ của từng phiên bản** và **so được từng phiên bản với bản đang dùng**. |
| FR142 | 6.9 C9 Lưu và phiên bản | Người dùng **mở thư mục của một Tác phẩm** trong trình quản lý tệp của hệ điều hành. |
| FR143 | 6.6 C6 | Người dùng **thử kết nối** tới nhà cung cấp AI đã cấu hình; kết quả là thành công hoặc một lỗi tường minh. |
| FR144 | 6.2 C2 Thao tác xuyên panel | Màn phím tắt cho **tìm theo tên lệnh hoặc tra ngược theo phím**, **xuất và nhập bộ phím**, **trả toàn bộ về mặc định**. |
| FR145 | 6.6 C6 | Từ màn Xem prompt, người dùng **gửi lại đúng prompt đã lắp**. Kết quả chỉ về panel Đề xuất AI (FR72). |
| FR146 | 6.4 C4 | Dải chốt bản dịch Glossary đề xuất thêm **các bản dịch đã có trong TM** cho thuật ngữ, cạnh âm Hán Việt (FR113); chọn bằng phím số. |
| FR147 | 6.4 C4 | Người dùng **hoàn tác được việc chốt bản dịch** một mục Glossary ngay sau khi chốt. |
| FR148 | 6.1 C1 Đọc lại | Người dùng **gỡ dấu "cần sửa"** đã đặt ở Chế độ đọc: gỡ từng dấu, và gỡ hết dấu của một Tác phẩm. |
| FR149 | 6.5 C5 | Theo từng Tác phẩm, người dùng chọn **chỉ dùng cặp TM của mình**; và **xuất TMX riêng các cặp của mình**. |
| FR150 | 6.7 C7 | Ứng dụng hiện **tỉ lệ đã nhận · không phải lỗi · bỏ qua** của các phát hiện Proofreader. |
| FR151 | 6.1 C1 Đọc lại | Chế độ đọc hiện **xuất xứ bản dịch** ở cấp Chương. Đây là thông tin, không phải công cụ biên tập theo nghĩa FR11. |
| FR152 | 6.8 C8 Xuất | Khi xuất, người dùng chọn: **đánh số câu** · **kèm phụ lục thuật ngữ đã chốt** · **đánh dấu câu chưa xác nhận** (chỉ ở bản hai cột; FR121 không đổi). |

### Đề xuất 4 — D13 · ✅ duyệt: (a1) + màn thu hoạch

- **(a1)** Lịch sử tra cứu có hai chip `Phiên này · Chương này`. `LookupHistoryEntry` (`src/panels/lookupHistoryState.ts:81`) thêm `chapterId`, vẫn chỉ trong RAM. FR41 và AC4 của Story 1.20 giữ nguyên. Chip "Cả Tác phẩm" của mockup không làm: nó đòi lịch sử xuyên phiên.
- **Màn duyệt thu hoạch:** góc nhìn webview trên các hàng `candidate_origin = 'review_harvest'` của bảng chờ. Mỗi dòng X→Y kèm "đổi ở n/m lần"; chọn nhiều; "Nhận N mục đã chọn"; "Để lại trong bảng chờ" đóng màn, không ghi gì. AD-20, FR55 giữ. Mở sau khi nhập bản reviewer và từ `Công cụ ▾`. FR54, FR95 đã phủ.

### Đề xuất 5 — Vị trí dải gợi ý (D8) · ✅ duyệt: (1)

Dải TM, chốt Glossary, Proofreader mọc thành một track hàng ngay dưới hàng đang sửa trong lưới. Quy tắc một dải tại một thời điểm (`topmostStrip`) giữ. `GlossaryQuickAdd` ở lại chân cửa sổ. Story đo thời gian frame khi dải mọc/thu trên Chương ≥ 2.000 câu và ghi số.

### Đề xuất 6 — EXPERIENCE.md, DESIGN.md, mockup · ✅ duyệt

Mỗi chỗ sửa mang 🔵 2026-10-13 kèm mã quyết định.

**6.1 EXPERIENCE.md §Information Architecture.** Thêm dưới cây:

```
Thanh tiêu đề
├── trái   Library · Workspace · Chế độ đọc        ⌘1 ⌘2 ⌘3
└── phải   [Proofreader: "Chương 47 · đã quét N câu"] · công tắc Ⓑ-2/Ⓑ-1 · Công cụ ▾
```

`Công cụ ▾` là mục duy nhất trên thanh tiêu đề mà mockup không vẽ (D10); chỉ webview, giống nhau trên macOS và Windows; danh sách ghi kèm phím.

| Lối vào | Chỗ theo ngữ cảnh | `Công cụ ▾` | Phím |
|---|---|---|---|
| Quản lý Glossary | — | có | `Mod+Alt+M` |
| Quản lý TM | — | có | phím mới, không `Mod+M` (gộp segment) |
| Cài đặt | — | có | `Mod+,` |
| Quét chính tả | bộ chọn phạm vi trong dải Proofreader · trạng thái bên phải thanh tiêu đề | có | `⌘P` (`ai.proofread.run`) |
| Chỗ đã bỏ qua | link trong dải Proofreader | có | — |
| Nhập bản reviewer | thẻ Tác phẩm · thanh hành động danh sách Chương | có | — |
| Nối tay | bước liền sau khi nhập · link trong Review Mode khi còn đoạn chưa nối | có | — |
| Duyệt thu hoạch | bước sau khi nhập bản reviewer | có | — |
| Bảng chờ Glossary | Workspace | — | `Mod+Alt+Q` |
| Review Mode | preset Workspace | — | `Mod+Alt+3` |
| Xuất | thanh hành động danh sách Chương | — | — |

**6.2 §Component Patterns, "Bản ghi từ điển".**
OLD: "Khi hai nguồn ghi khác nhau, một dòng dẫn nói rõ điều đó trước khi liệt kê — người dùng biết mình đang nhìn bất đồng chứ không phải trùng lặp."
NEW: "Không có dòng dẫn trước các khối (D6). Mỗi khối mở bằng một dòng đầu gọn: tên nguồn · số nghĩa · `⌄` thu gọn (D5, `lookup-real-density.html`). Bất đồng đọc ra từ chính các khối xếp chồng; FR32 và AD-19 không đổi."

**6.3 §Component Patterns, thêm "Bộ chọn nguồn từ điển".** Nút `Nguồn N/M ▾` ở đầu panel Tra cứu, cạnh link "Nguồn dữ liệu" (link này lên ngang hàng tiêu đề "Tra cứu"). Popup: mỗi nguồn một dòng — công tắc · tên ngắn · tên đầy đủ · số mục của chữ đang tra. `↑↓` chọn dòng, `Space` gọi `lookup.toggle_source`; không phím riêng cho từng nguồn (AD-34, Story 1.19). `Esc` đóng và trả tiêu điểm về nút (UX-DR17).

**6.4 §Component Patterns, thêm "Lịch sử tra cứu".** Hai chip `Phiên này · Chương này`; không có "Cả Tác phẩm" vì lịch sử chỉ sống trong phiên (FR41).

**6.5 §Component Patterns, "Dải mọc dưới câu đang sửa".** Giữ chữ; thêm: "🔵 2026-10-13 (D8): slot chung trên thanh trạng thái của Story 3.6 (2026-08-22) bị thay; dải là một track hàng ngay dưới hàng đang sửa. `GlossaryQuickAdd` ở lại chân cửa sổ. Vị trí đáy panel của `glossary-confirm-inline.html` không còn hiệu lực."

**6.6 §Bề mặt nhập › Hình dạng (D3).**
OLD: "**vạch trạng thái** · **số câu** · **nguyên văn** · **bản dịch** · **nhãn trạng thái**"
NEW: "**vạch trạng thái** · **số câu** · **nguyên văn** · **bản dịch**. 🔵 2026-10-13 (D3): nhãn trạng thái không còn hiện; chữ trạng thái vẫn có trong hàng, ẩn khỏi mắt, cho trình đọc màn hình."
Kèm **DESIGN.md §Vạch lề segment**: ghi chú 🔵 2026-08-15 ("cột nhãn là lý do vạch được phép `aria-hidden`") sửa thành chữ ẩn thay cột nhãn; thêm yêu cầu các trạng thái của vạch khác nhau **ngoài màu** (WCAG 1.4.1). Sally chọn hình (liền · đứt · chấm · độ dày) trong story.

**6.7 KF-2 bước 3.**
OLD: "Panel Lookup hiện ngay — **ba nguồn, và một dòng dẫn báo CVDICT với Thiều Chửu ghi khác nhau**."
NEW: "Panel Lookup hiện ngay — **ba nguồn xếp chồng; CVDICT và Thiều Chửu ghi khác nhau**."

**6.8 Mockup.** Một dòng 🔵 nhìn thấy được ở đầu `sources-attribution.html` §1 và `lookup-history-pins.html`, trỏ về 6.3 và 6.4. Không vẽ lại.

D4/D9 (bỏ dòng "Nguồn: …" Hán Việt; nguồn chỉ ghi công ở Attribution) không chạm tài liệu nào.

### Đề xuất 7 — Epic 12, thứ tự, cửa chặn, sổ nợ · ✅ duyệt

**Epic 12 · `epic-hoan-thien-giao-dien-theo-mockup`** — "Hoàn thiện giao diện theo mockup". `covers` = FR136–FR152. Trong `tickets.toml` đứng trước Epic 10.

Khung story cho `bmad-ticket` (cắt chính thức ở đó):

| # | Story | Nội dung |
|---|---|---|
| 1 | Khung app | Thanh tiêu đề 3 tab + công tắc bố cục + `Công cụ ▾` (D1, D10, D11); phím mới cho Quản lý TM; `data-*-open` theo nút. **Trước mọi story khác.** |
| 2 | Library: bố cục | Dải trên + lưới 4 cột; trạng thái rỗng |
| 3 | Library: dữ liệu và chỉ mục | FR140, FR142 (2 phương án), dấu thời gian chỉ mục (2 phương án) |
| 4 | Danh sách Chương | Nhóm trạng thái, chọn nhiều, thanh hành động, lọc/tìm/sắp, màn gộp, FR138 |
| 5 | Tìm kiếm Library | FR139; mở kết quả ở Chế độ đọc |
| 6 | Màn xem trước nhập hai cột | Tách Chương · URL · song ngữ "Nhập luôn, nối sau"; FR137 |
| 7 | Workspace: lưới và thanh trạng thái | D3, D4; bộ đếm, gợi ý phím; vị trí câu; dòng tra cứu ở màn hẹp |
| 8 | Panel Tra cứu | D2/D12, D5, D6; Ghim ở đầu panel; FR136; chip lịch sử |
| 9 | Dải mọc dưới câu | D8: track hàng; đo frame |
| 10 | Nội dung ba dải | TM, chốt Glossary (FR146), Proofreader (bộ chọn phạm vi, link Chỗ đã bỏ qua) |
| 11 | Panel AI và lô | Nhà cung cấp/mô hình; hàng lô có chữ và token; đưa cả lô sang (2 phương án); Bỏ lô |
| 12 | Xem prompt | Màn đầy đủ tách tầng; token thật; Chép; FR145; lối sửa dòng bị loại; tab Kết quả |
| 13 | Glossary: bảng chờ, quản lý, thu hoạch | Thanh tiến độ, `Space`, bảng có đầu cột, cột Xuất xứ, màn thu hoạch |
| 14 | TM và Thư viện prompt | Bảng, sắp, FR149, phạm vi TMX, Nhân bản · cửa G4 |
| 15 | Proofreader và Chế độ đọc | FR150, FR148, FR151, "Sửa lần lượt" · cửa G3 |
| 16 | Review Mode, nhập reviewer, nối tay | Lối vào theo D11; các dòng bảng C7 |
| 17 | Xuất | FR152; bốn trường khối ghi nguồn; xem trước theo định dạng |
| 18 | Cài đặt | Bố cục, Chế độ đọc, Dữ liệu, khung Cập nhật; FR143, FR144 |
| 19 | Lịch sử phiên bản | FR141 · cửa G2 |
| 20 | Hoàn tác chốt Glossary | FR147 · cửa G1 |

**Sổ nợ** (`deferred-work.md`, append, không xoá), làm trong phiên `bmad-ticket` NGAY SAU khi Epic 12 vào `tickets.toml` — Kiểm C của `check:debt-owner` coi chủ là một epic chưa có trong cây vé là treo (`scripts/check-debt-owner.mjs:519`): thêm `→ Chủ: Epic 12` vào `:3565` (A17), `:671` (A20), `:3579` (A18), `:6027` (B16). `:6056` (B18) giữ chủ Winston tới khi G1 xong.

## 5. Bàn giao

**Phạm vi: Lớn** — PRD thêm 17 FR, bốn cửa kiến trúc, một epic mới.

| Ai | Việc | Đầu vào |
|---|---|---|
| Dev (lượt áp của correct-course này) — ✅ 2026-10-13 | Đã áp đề xuất 3 vào PRD + `initiative-auratranslate.md`; đề xuất 6 vào EXPERIENCE.md, DESIGN.md, hai mockup | Tệp này |
| `bmad-ticket` | Tạo Epic 12 trong `tickets.toml` (trước Epic 10) và cây `epic-hoan-thien-giao-dien-theo-mockup/`; cắt story từ khung ở đề xuất 7; rồi chuyển bốn mục nợ sang Epic 12 (đề xuất 7, Sổ nợ) | Tệp này, `bang-lech-*.md`, `phan-loai-loai-5-*.md` |
| Winston (`bmad-architecture`, một phiên) | G1 FR147: trigger vòng đời Glossary một chiều (AD-49/AD-36) · G2 FR141: xuất xứ trên `segment_version` (AD-47/AD-50) · G3 FR150: lưu đã nhận/bỏ qua (AD-53) · G4 FR149: công tắc theo Tác phẩm ở cột `work` hay phạm vi cấu hình (AD-18). Số AD mới: quét spine và mọi `ad-brief-*` chưa viết | Mục 2 đề xuất 2, `phan-loai-loai-5-{a,b,c}.md` |
| `bmad-build` | Từng story, 4 pha; story dời nút mang đường lui `data-*-open`; chạy một spec e2e trước push; 17 dòng "cần ảnh" story tự chụp (dpr ghi rõ) | Plan story |
| Ice | Một lượt dùng thật trước khi Epic 12 `done`; chọn ở các chỗ hai phương án (B22, A10, A19) khi story đưa số đo | — |

**Tiêu chí thành công:**
- Thanh tiêu đề chỉ còn 3 tab, công tắc bố cục, `Công cụ ▾` và trạng thái Proofreader khi có.
- Mỗi dòng loại 2 của `bang-lech-{a,b,c}.md` đóng bằng một story của Epic 12, hoặc chuyển loại 3 có nguồn.
- Mỗi dòng loại 5 có đúng một đích: story Epic 12 hoặc loại 3 có nguồn (đề xuất 2).
- FR136–FR152 mỗi FR có ít nhất một story phủ; G1–G4 có AD trước khi story của chúng vào build.
- 10.6 và 10.9 chạy sau Epic 12.
