Bốn chế độ màn hình: `Library` · `Workspace` · `ReadingMode` · `ReviewMode` — **một cửa sổ OS, nhiều chế độ** (AD-24).

Ba chế độ đầu là **ngang hàng**, không phân cấp, và chuyển bằng `⌘1` `⌘2` `⌘3` hoặc ba tab ở thanh tiêu đề (UX-DR34). Review Mode là một **bố cục** bên trong Workspace, không phải một cửa sổ và không phải chế độ thứ tư ở tầng này (AD-24, Epic 8).

---

## Ranh giới sở hữu — đọc trước khi thêm bất cứ thứ gì vào đây

**Story sở hữu nội dung: 1.14** (khung bốn panel). Chế độ đọc thuộc Epic 5, Review Mode thuộc Epic 8. Story 1.6 chỉ dựng **vỏ chuyển chế độ** và ba khung rỗng — bảng dưới vạch rõ chỗ nào đã có chủ.

| | Story | Trạng thái |
|---|---|---|
| Vỏ chuyển chế độ, `modeState.ts`, ba khung rỗng có một câu trạng thái | **1.6** | ✅ đã dựng |
| `dockview`, lưới 2×2, dock/undock/tab, preset bố cục, khung bốn panel | **1.14** | ⬜ |
| Bốn trạng thái rỗng của UX-DR31 *(cần nội dung thật mới viết đúng)* | **1.14 / 1.15 / 5.x** | ⬜ |
| Nội dung Library: lưới Tác phẩm, bộ lọc, sắp xếp, vòng đời | **Epic 5** | ⬜ |
| Nội dung Chế độ đọc: typography đọc dài, song ngữ, ba mức chữ | **Epic 5** | ⬜ |
| Review Mode | **Epic 8** | ⬜ |

Ba tệp `.vue` hôm nay là **khung rỗng có chủ ý**: một câu trạng thái lấy từ `vi.json`, gốc mang `tabindex="-1"`, và một điểm vào focus đã khai. ⛔ Đừng đổ nội dung vào chúng trước story sở hữu.

---

## Ba điều đã chốt, đừng sửa lại mà không đọc lý do

**1. `<KeepAlive>`, không phải `v-if` trần** *(`App.vue`)*. UX-DR34 và FR12 hứa chuyển chế độ **giữ ngữ cảnh** — *"rời Workspace sang Chế độ đọc rồi quay lại thì vẫn đúng Chương, đúng câu, đúng vị trí cuộn"*. Hôm nay chưa có ngữ cảnh nào để mất nên hai cách cho kết quả quan sát được y hệt; khác biệt hiện ra ở Epic 2, khi Editor mang văn bản đang gõ. Hệ quả trực tiếp: mỗi chế độ vào focus trong **`onActivated`**, không phải `onMounted` — lần hiện thứ hai trở đi không có `mounted`.

**2. Hướng phụ thuộc là `modes/` → `commands/`, một chiều.** ⛔ Đừng đảo lại: `src/commands/**` phải nạp được bằng Node thuần để `npm run check:commands` khẳng định hành vi, và một cạnh trỏ về đây là kéo `vue` vào cổng. Đó là lý do `setMode` được **tiêm vào** `installCommands()` từ `src/main.ts` thay vì `commands/index.ts` tự import.

**3. Không `#[tauri::command]` nào cho việc đổi chế độ.** Chuyển chế độ, tiêu điểm và bố cục panel là **state UI**, và AD-1 nói thẳng đó là phần frontend được phép sở hữu. Một vòng IPC cho một thao tác phải mượt là quy tắc nghiệp vụ giả đặt sai chỗ.

---

Component đặt tên `PascalCase.vue` (Consistency Conventions). Ánh xạ thuật ngữ: `ReadingMode` là *Chế độ đọc*, `ReviewMode` là *Review Mode* — PRD §5.2 chốt **không dịch** `Library` và `Workspace`.
