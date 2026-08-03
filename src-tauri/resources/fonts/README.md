Bốn tệp font nhúng vào bản phát hành + ba tệp giấy phép gốc đi kèm (FR38, FR109).

Vùng này khai `$RESOURCE/fonts/**` **chỉ đọc** trong `assetProtocol.scope` (AD-23, UX-DR4). Frontend nạp qua `resolveResource('fonts/…')` → `convertFileSrc` → `FontFace.load()`.

| Tệp | Byte | Họ (đọc từ bảng `name`) | Phiên bản | Giấy phép |
|---|---|---|---|---|
| `NotoSerifCJKtc-Regular.otf` | 24.541.904 | `Noto Serif CJK TC` *(ID 1)* | 2.003 | SIL OFL 1.1 |
| `SourceSerif4[opsz,wght].ttf` | 1.209.508 | `Source Serif 4` *(ID 1)* | 4.004 | SIL OFL 1.1 |
| `SourceSerif4-Italic[opsz,wght].ttf` | 855.432 | `Source Serif 4` *(ID 1)* | 4.004 | SIL OFL 1.1 |
| `SourceSans3[wght].ttf` | 646.340 | `Source Sans 3` *(ID 16)* | 3.052 | SIL OFL 1.1 |
| **Tổng** | **27.253.184** *(25,991 MiB)* | | | |

SHA-256 từng tệp: `font-spike-results-2026-08-03.md §Phép đo 5`. **Đối chiếu trước khi thay bất kỳ tệp nào.**

⚠️ **`NotoSerifCJKtc` ≠ `NotoSerifTC`.** Cái sau là bản subset theo ngôn ngữ (45 MB); lấy nhầm **hỏng im lặng** — phần lớn ký tự vẫn hiện, chỉ tofu khi gặp văn bản khác hệ chữ. Tổng byte lệch khỏi 27.253.184 nghĩa là đã lấy sai tệp.

⚠️ **Chỉ `Source Sans 3` khai Reserved Font Name `'Source'`** — subset riêng tệp đó thì **bắt buộc** đổi tên font nội bộ. Hai tệp Serif không khai nên subset thoải mái.

Story 1.2 chỉ đặt tệp vào đây. `@font-face` và token typography là **Story 1.4**.
