Bốn chế độ màn hình: `Library` · `Workspace` · `ReadingMode` · `ReviewMode` — **một cửa sổ OS, nhiều chế độ** (AD-24).

Ba chế độ đầu là **ngang hàng**, không phân cấp, và chuyển bằng `⌘1` `⌘2` `⌘3` hoặc ba tab ở thanh tiêu đề (UX-DR34). Review Mode là một **bố cục** bên trong Workspace, không phải một cửa sổ và không phải chế độ thứ tư ở tầng này (AD-24, Epic 8).

---

## Ranh giới sở hữu — đọc trước khi thêm bất cứ thứ gì vào đây

**Story sở hữu nội dung: 1.14** (khung bốn panel — 🔵 **thu xuống BA ở Story 2.5b**). Chế độ đọc thuộc Epic 5, Review Mode thuộc Epic 8. Story 1.6 chỉ dựng **vỏ chuyển chế độ** và ba khung rỗng — bảng dưới vạch rõ chỗ nào đã có chủ.

| | Story | Trạng thái |
|---|---|---|
| Vỏ chuyển chế độ, `modeState.ts`, ba khung rỗng có một câu trạng thái | **1.6** | ✅ đã dựng |
| `dockview`, dock/undock/tab, preset bố cục, khung panel | **1.14** *(lưới 2×2 + bốn panel — **superseded** bởi **2.5b**: hai preset Ⓑ-1/Ⓑ-2, **ba** panel)* | ✅ đã dựng |
| Bốn trạng thái rỗng của UX-DR31 *(cần nội dung thật mới viết đúng)* | **1.14 / 1.15 / 5.x** | ⬜ |
| Nội dung Library: lưới Tác phẩm, bộ lọc, sắp xếp, vòng đời | **Epic 5** | ⬜ |
| Nội dung Chế độ đọc: typography đọc dài, song ngữ, ba mức chữ | **5.11** | ✅ đã dựng |
| Đọc liên tục qua Chương đã xong và mốc biên (FR120) | **5.12** | ✅ đã dựng |
| Đánh dấu chỗ cần sửa khi đang đọc (FR119) | **5.13** | ⬜ |
| Review Mode | **Epic 8** | ⬜ |

🔵 **SỬA 2026-08-30 (Story 5.11)** — hàng "Nội dung Chế độ đọc" hết đúng ở `⬜`: typography ba
mức (đúng token `read-*`), song ngữ, mục lục và cắt bỏ (qua `read_reading_chapter` +
`core::segment::reading`) đã dựng. `ReadingMode.vue` KHÔNG còn là khung rỗng — câu dưới đây chỉ
còn đúng cho **hai tệp** (`LibraryMode.vue` phần "đánh dấu đã xong" của UX-DR31, `ReviewMode.vue`).
Đánh dấu *"cần sửa"* (FR119) và đọc liên tục xuyên Chương/mốc *"chỉ đọc phần đã xong"* (FR120)
vẫn **⬜** — có chủ riêng: **Story 5.12 · 5.13**.

🔵 **SỬA TẠI CHỖ 2026-08-30 (Story 5.12)** — vế FR120 của mục ngay trên hết đúng ở `⬜`: bề mặt
đọc đổi từ MỘT Chương (`read_reading_chapter`) thành MỘT LƯỢT ĐỌC (`read_reading_run` →
`ReadingRun`) chỉ gồm Chương `Done`, kèm một mốc biên (`.frontier`) nói vì sao dãy dừng — xem
hàng riêng "Đọc liên tục qua Chương đã xong và mốc biên" ở trên. Đánh dấu chỗ cần sửa (FR119)
**VẪN `⬜`**, chủ riêng **Story 5.13** — không đụng ở lượt này.

Ba tệp `.vue` khai *"khung rỗng có chủ ý"* trong doc-comment của chính chúng khi CHƯA có story sở
hữu chạm vào: một câu trạng thái lấy từ `vi.json`, gốc mang `tabindex="-1"`, và một điểm vào
focus đã khai. Đừng đổ nội dung vào một khung còn ⬜ ở bảng trên trước story sở hữu.

---

## Ba điều đã chốt, đừng sửa lại mà không đọc lý do

**1. `<KeepAlive>`, không phải `v-if` trần** *(`App.vue`)*. UX-DR34 và FR12 hứa chuyển chế độ **giữ ngữ cảnh** — *"rời Workspace sang Chế độ đọc rồi quay lại thì vẫn đúng Chương, đúng câu, đúng vị trí cuộn"*. Hôm nay chưa có ngữ cảnh nào để mất nên hai cách cho kết quả quan sát được y hệt; khác biệt hiện ra ở Epic 2, khi Editor mang văn bản đang gõ. Hệ quả trực tiếp: mỗi chế độ vào focus trong **`onActivated`**, không phải `onMounted` — lần hiện thứ hai trở đi không có `mounted`.

**2. Hướng phụ thuộc là `modes/` → `commands/`, một chiều.** Đừng đảo lại: `src/commands/**` phải nạp được bằng Node thuần để `npm run check:commands` khẳng định hành vi, và một cạnh trỏ về đây là kéo `vue` vào cổng. Đó là lý do `setMode` được **tiêm vào** `installCommands()` từ `src/main.ts` thay vì `commands/index.ts` tự import.

**3. Không `#[tauri::command]` nào cho việc đổi chế độ.** Chuyển chế độ, tiêu điểm và bố cục panel là **state UI**, và AD-1 nói thẳng đó là phần frontend được phép sở hữu. Một vòng IPC cho một thao tác phải mượt là quy tắc nghiệp vụ giả đặt sai chỗ.

---

Component đặt tên `PascalCase.vue` (Consistency Conventions). Ánh xạ thuật ngữ: `ReadingMode` là *Chế độ đọc*, `ReviewMode` là *Review Mode* — PRD §5.2 chốt **không dịch** `Library` và `Workspace`.
