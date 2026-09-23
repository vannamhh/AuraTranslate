---
name: 'Worksheet gọn spine — bàn giao Pha 2 (evidence) và Pha 3 (sửa spine)'
date: '2026-09-23'
base_commit: 'd861aab73dd54992ae702f7892ec52570230f4b6'
base_file: '158.666 byte / 1.173 dòng / 49 AD — xác nhận `git diff d861aab -- ARCHITECTURE-SPINE.md` rỗng lúc viết worksheet này'
status: 'Pha 1 xong — chỉ đọc, KHÔNG sửa spine'
---

# Worksheet gọn ARCHITECTURE-SPINE.md

Đọc trực tiếp kế hoạch `spine-condense-plan-2026-09-23.md` §6 trước khi thi hành worksheet này — quyết định đã chốt: #1→B (cắt đúng ranh (c)/(d)/(e), giữ mọi (a) Rule và (b) Prevents) · #2→A · #3→A · #4→A. Worksheet này KHÔNG lặp lại lý do, chỉ ra lệnh máy làm.

Mọi số dòng dưới đây đo tại `d861aab` (đã xác nhận cây làm việc khớp nguyên văn, không lệch một byte). Mọi "Cắt" là chuỗi nguyên văn — Pha 2 chép sang `spine-evidence.md`, Pha 3 xoá bằng khớp chuỗi (không phải khớp số dòng, vì số dòng sẽ trôi sau mỗi lượt sửa).

**Quy tắc đã dùng để phân loại** (khắt khe hơn mô tả sơ bộ của kế hoạch, để không phạm "giữ mọi Rule/Prevents"): một đoạn chỉ bị liệt (c)/(d)/(e) khi nó mang ít nhất MỘT trong: ngày tháng cụ thể, tên Story/Epic, con số đo lường minh hoạ/lịch sử tách rời được khỏi mệnh đề bắt buộc, 🔵, hoặc trỏ `deferred-work.md`/`epics.md`/tài liệu khác — VÀ việc cắt nó không đổi nghĩa của Rule/Prevents còn lại. Nghi ngờ ⇒ giữ nguyên cả câu. Xem `## Ghi chú cho Pha 2/3` cuối tệp cho các bẫy đã gặp.

## Bảng tóm tắt kích cỡ

| Mục | Byte hiện tại (`d861aab`) | Byte cắt dự kiến | Byte còn lại dự kiến |
|---|---:|---:|---:|
| Frontmatter + tiêu đề (L1–24) | 991 | 0 | 991 |
| Design Paradigm (L25–72) | 2.308 | 0 | 2.308 |
| Invariants & Rules — heading trần (L73–74) | 23 | 0 | 23 |
| AD-1 (L75–80) | 715 | 0 | 715 |
| AD-2 (L81–88) | 524 | 0 | 524 |
| AD-3 (L89–94) | 753 | 0 | 753 |
| AD-4 (L95–102) | 1.404 | 0 | 1.404 |
| AD-5 (L103–112) | 1.248 | 0 | 1.248 |
| AD-6 (L113–118) | 510 | 0 | 510 |
| AD-7 (L119–132) | 702 | 0 | 702 |
| AD-8 (L133–138) | 904 | 0 | 904 |
| AD-9 (L139–146) | 909 | 0 | 909 |
| AD-10 (L147–152) | 1.441 | 58 | 1.383 |
| AD-11 (L153–158) | 536 | 0 | 536 |
| AD-12 (L159–164) | 815 | 134 | 681 |
| AD-13 (L165–199) | 1.088 | 0 | 1.088 |
| AD-14 (L200–205) | 552 | 0 | 552 |
| AD-15 (L206–217) | 1.502 | 0 | 1.502 |
| AD-16 (L218–229) | 1.467 | 0 | 1.467 |
| AD-17 (L230–237) | 1.333 | 0 | 1.333 |
| AD-18 (L238–289) | 4.973 | 1.072 | 3.901 |
| AD-19 (L290–295) | 519 | 0 | 519 |
| AD-20 (L296–301) | 577 | 0 | 577 |
| AD-21 (L302–307) | 534 | 0 | 534 |
| AD-22 (L308–313) | 579 | 0 | 579 |
| AD-23 (L314–321) | 837 | 0 | 837 |
| AD-24 (L322–327) | 509 | 0 | 509 |
| AD-25 (L328–333) | 747 | 33 | 714 |
| AD-26 (L334–343) | 1.715 | 883 | 832 |
| AD-27 (L344–349) | 704 | 0 | 704 |
| AD-28 (L350–355) | 547 | 0 | 547 |
| AD-29 (L356–361) | 489 | 0 | 489 |
| AD-30 (L362–367) | 1.013 | 0 | 1.013 |
| AD-31 (L368–393) | 2.204 | 0 | 2.204 |
| AD-32 (L394–399) | 777 | 0 | 777 |
| AD-33 (L400–405) | 769 | 0 | 769 |
| AD-34 (L406–418) | 1.844 | 0 | 1.844 |
| AD-35 (L419–426) | 1.649 | 0 | 1.649 |
| AD-36 (L427–436) | 1.303 | 0 | 1.303 |
| AD-37 (L437–454) | 2.478 | 0 | 2.478 |
| AD-38 (L455–464) | 1.600 | 0 | 1.600 |
| AD-39 (L465–505) | 5.283 | 24 | 5.259 |
| AD-40 (L506–522) | 2.011 | 0 | 2.011 |
| AD-41 (L523–545) | 2.938 | 0 | 2.938 |
| AD-42 (L546–562) | 1.994 | 0 | 1.994 |
| AD-43 (L563–570) | 1.261 | 0 | 1.261 |
| AD-44 (L571–634) | 9.282 | 1.910 | 7.372 |
| AD-45 (L635–651) | 2.138 | 249 | 1.889 |
| AD-46 (L652–674) | 3.290 | 510 | 2.780 |
| AD-47 (L675–745) | 9.397 | 689 | 8.708 |
| AD-48 (L746–774) | 6.765 | 215 | 6.550 |
| AD-49 (L775–790) | 3.654 | 0 (ngoại lệ đã chốt) | 3.654 |
| **Cộng dòng "→ chi tiết & bằng chứng" mới (11 AD có cắt: 10,12,18,25,26,39,44,45,46,47,48)** | — | −630 (thêm vào, không phải cắt) | +630 |
| Consistency Conventions (L791–818) | 5.091 | 0 (không đụng) | 5.091 |
| Stack (L819–1003) | 32.524 | 12.975 | 19.549 + ~60 (pointer `#Stack`) ≈ 19.609 |
| Structural Seed (L1004–1114) | 6.966 | 0 (không đụng) | 6.966 |
| Capability → Architecture Map (L1115–1129) | 1.515 | 0 (không đụng) | 1.515 |
| Deferred (L1130–1173) | 18.465 | 11.046 | 7.419 |
| **TỔNG** | **158.666** | **≈ 29.798 cắt, +690 thêm (pointer)** | **≈ 129.558 byte (≈ 126,5 KB)** |

**Ngoài khoảng 95–100 KB Ice chốt — ghi rõ vì sao, KHÔNG cắt thêm vào (a)/(b) để ép số:**

Kế hoạch ước tính nhanh trên 3 AD mẫu (AD-47/48/44) rồi suy rộng "12 AD lớn ~55% là (c)/(e)". Đọc hết cả 49 AD ở Pha 1 này cho kết quả khác: sau khi loại mọi câu có thể tách khỏi Rule/Prevents mà KHÔNG đổi nghĩa, phần cắt được thật trong toàn bộ `Invariants & Rules` chỉ là **5.777 byte / 90.806 byte = 6,4%** — không phải ~33% như suy rộng. Lý do đo được: phần lớn "số đo" trong các AD lớn (AD-44, AD-47, AD-48) không tách rời được khỏi Rule/Prevents mà không phá nghĩa — chúng LÀ điều kiện xét lại, LÀ danh mục đóng, hoặc LÀ đối chứng cho một 🔵 bắt buộc giữ (AD-48). `Stack` cắt được NHIỀU hơn kế hoạch ước tính (12.975B so với ước 6.750B — vì phần lớn narrative "Rà NFR15 lượt N" là audit trail thuần, không phải chú thích trong ngoặc). `Deferred` cắt được gần đúng ước tính (11.046B so với ~15.000B ước). Cộng lại, tổng cắt thật (~29,8 KB) chỉ bằng ~59% của tổng cắt kế hoạch giả định (~50,75 KB), nên tệp sau gọn còn **≈126,5 KB**, không nằm trong 95–100 KB. Đây là số đo, không phải lựa chọn cắt non tay — worksheet này KHÔNG đề xuất cắt thêm vào Rule/Prevents để ép số xuống, đúng ràng buộc "KHÔNG cắt lấn sang (a)/(b)".

---

## AD-1 đến AD-9, AD-11, AD-13 đến AD-17, AD-19 đến AD-24, AD-27 đến AD-38, AD-40 đến AD-43, AD-49

**Không cắt.** Đọc đủ toàn văn từng AD này ở Pha 1 (không suy đoán từ mẫu) — không tìm thấy ngày tháng, tên Story/Epic, số đo tách rời được khỏi Rule/Prevents, 🔵, hay chỗ trỏ `deferred-work.md`/`epics.md`. Toàn bộ nội dung là (a) Rule hoặc (b) Prevents thuần, kể cả các bảng, khối mermaid (AD-13), và các đoạn ví dụ minh hoạ ngắn gắn liền với chính mệnh đề (ví dụ AD-44 không nằm trong nhóm này — xem mục riêng; các số trong AD-17 §"AD-44 ③ ghi số đo" chỉ là tham chiếu tên AD, không phải số đo tại chỗ).

AD-49 là ngoại lệ đã chốt trong đề bài: viết theo khuôn gọn từ đầu, không cắt.

Giữ: toàn văn, kể cả bảng AD-7/AD-18/AD-31/AD-37/AD-39/AD-42, mermaid AD-13, và mọi ví dụ số nhỏ gắn liền với mệnh đề (vd. AD-26 mục 3 "headword='running'⇒1; 'Running'⇒0" — đây là AD-44 mục ③... **sửa chú thích:** ví dụ đó thật ra nằm ở AD-44 mục Prevents ③, không phải AD-26; xem mục AD-44 riêng, đã giữ nguyên).

---

## AD-10 — Mỗi lớp từ điển gỡ rời là một file `.db` độc lập

**Cắt:**
- L150 — "toàn bộ payload (nay 343.991.430 byte — NFR6 sửa lần hai 2026-08-05); FR36" — chuỗi cắt chính xác: `(nay 343.991.430 byte — NFR6 sửa lần hai 2026-08-05)` (loại c — số đo + ngày cập nhật, tách rời được khỏi mệnh đề Prevents "toàn bộ payload ... lọt vào git").

**Giữ:** toàn bộ Rule (adapter dùng chung, gỡ 1 lớp = xoá 1 file, nghiệm thu bằng test thật, mỗi `.db` tự mang metadata giấy phép, enum giấy phép cấm).

**Viết lại:** sau khi cắt, câu Prevents đọc: "...biến thành thay đổi mã nguồn + dựng lại toàn bộ payload; FR36 chỉ nghiệm thu được bằng suy luận thay vì bằng test thật." (chỉ xoá cụm trong ngoặc, còn lại nguyên văn, không cần viết câu mới).

---

## AD-12 — Thời điểm checkpoint là quyết định của ứng dụng

**Kiểm `.memlog.md` (theo yêu cầu đề bài):** `.memlog.md` cùng thư mục với spine, dòng 46: *"(version) SỬA LỖI RESEARCH: WAL2 KHÔNG phải tính năng đã phát hành. Diễn đàn SQLite xác nhận vẫn là nhánh thử nghiệm... SQLite 3.53.x (06/2026) không có."* — khớp đúng, còn tồn tại, còn đúng. An toàn để cắt phần diễn giải trong spine và trỏ sang `.memlog.md` (vốn đã là điều spine đang làm).

**Cắt:**
- L162 — chuỗi chính xác: ` *(WAL2 mà báo cáo technical research đề xuất **không tồn tại** như tính năng đã phát hành — xem `.memlog.md`.)*` (loại c/e — tường thuật quyết định + trỏ tài liệu ngoài, đứng thành câu riêng sau câu Prevents chính).

**Giữ:** "auto-checkpoint mặc định (1000 trang) rơi đúng lúc người dùng đang gõ → vi phạm NFR2." và toàn bộ Rule (PRAGMA, checkpoint PASSIVE/TRUNCATE, ngưỡng kích thước WAL).

---

## AD-18 — Một ScopeResolver, ngữ nghĩa khai báo tường minh

**Giữ:** bảng ngữ nghĩa (L242–254); khối "Cưỡng chế" (L278–282, nguyên văn — trỏ file/test thật); khối "Vì sao luật làm sạch là hợp nhất" (L284); đoạn khoá thứ tự TM (L286); đoạn xác nhận segment chỉ ghi TM Tác phẩm (L288).

**Cắt (đều loại c — ngày/Story/trích dẫn tài liệu tách rời được khỏi mệnh đề ngữ nghĩa đã có trong bảng):**
1. L256 — chuỗi chính xác: ` — thêm ở Story 1.8, Ice phê chuẩn 2026-08-04` (trong dòng `**Ngữ nghĩa thứ ba — *chỉ toàn cục* — thêm ở Story 1.8, Ice phê chuẩn 2026-08-04.**`).
2. L258–259 — chuỗi chính xác (nối 2 dòng nguồn):
   ```
   `mockups/settings.html:246` nói thẳng: *"Phím tắt chỉ tồn tại ở tầng Toàn
   cục — một thao tác không nên đổi phím theo từng Tác phẩm."* 
   ```
3. L260–261 — chuỗi chính xác: `nên Story 1.14/1.21 sẽ dựng thanh chuyển phạm vi cho một thứ không nên có; ` (giữa "UX đã cấm," và "*hợp nhất* thì vô nghĩa.").
4. L266–270 — TOÀN BỘ blockquote thứ hai, từ `**Cấu hình AI ghi đè theo từng *trường*, không theo cả struct** *(làm rõ ở Story 1.8,` tới `...một map `khoá trường → giá trị`, y hệt Glossary.` — trùng nguyên văn nội dung đã có trong bảng dòng "Cấu hình AI" (L248), phần còn lại thuần Story/mockups citation.
5. L273–274 — chuỗi chính xác:
   ```
   Story 5.1 định nghĩa nó là trường **bất biến** trong `meta.json` — đặt lúc tạo, không đổi được (`prd.md:765-774`: *"cố định, đặt lúc tạo"*, mệnh đề mà
   `epics.md:296` làm rơi mất) — và nó
   ```

**Viết lại (bắt buộc, để câu 5 không cụt):** dòng L272–276 sau khi cắt mục 5 phải đọc:
```
**`ngôn ngữ nguồn` KHÔNG phải một loại ở bảng này.** FR103 liệt kê nó ở tầng Tác
phẩm, nhưng nó là trường **bất biến** trong `meta.json` — đặt lúc tạo, không đổi được —
không có đối ứng ở tầng Global**, nên không có gì để ghi đè. Nó là thuộc tính của `Work`,
không phải cấu hình hai tầng.
```
(chỉ nối lại hai vế quanh chỗ cắt, không thêm ý mới).

---

## AD-25 — Dữ liệu từ điển là artifact có phiên bản và checksum

**Cắt:**
- L331 — chuỗi chính xác: `(nay 320,3 MB cho cả ba tệp) ` (loại c — số đo cập nhật, tách khỏi câu Prevents "nguồn thô 1,13 GB hoặc `.db` ... lọt vào git").

**Giữ:** toàn bộ Rule (GitHub Release, `dict-manifest.toml`, checksum, parser chỉ ở build tool).

---

## AD-26 — Ba nhánh truy vấn tiếng Trung `[ADOPTED]`

**Cắt (đều loại c — số đo tách rời được khỏi Rule/🔴 đã phát biểu):**
1. L338 — chuỗi chính xác: ` (đo được 20–50 ms)` (sau "`LIKE` **cấm** trên đường nóng").
2. L340 — chuỗi chính xác: ` *(đo: **9** cặp trên **119.039** đầu mục)*` (sau "không áp được cho tiếng Anh").
3. L342 — TOÀN BỘ đoạn ⚠️, nguyên văn từ `⚠️ **Dải hiệu năng mà bản đầu của AD này công bố...` tới `...Mỗi nguồn từ điển thêm vào sẽ làm nó dày thêm.` — cả đoạn là số đo lịch sử (Giai đoạn 0 vs đo lại 2026-08-05), không đổi Rule "LIKE cấm trên đường nóng" đã phát biểu ở trên.

**Giữ:** Rule chính (ba nhánh, ngưỡng 1–2/3+ ký tự), toàn bộ đoạn 🔴 "Phạm vi là TIẾNG TRUNG..." (trừ chuỗi số 2 vừa cắt).

---

## AD-27 — Chỉ mục FTS chính phân biệt dấu `[ADOPTED]`

**Không cắt.** Câu "Chi phí đã biết: ~17 MB mỗi chỉ mục." không mang ngày/Story và đứng độc lập ngắn — nghi ngờ giá trị cắt thấp so với rủi ro, giữ nguyên toàn AD.

---

## AD-39 — Đường nhập là một pipeline có thứ tự cố định, dùng chung cho mọi nguồn

**Cắt:**
- L494 — chuỗi chính xác: `(PRD chốt 2026-08-03) ` (trong "Mẫu phân tách áp lên cột nguồn (PRD chốt 2026-08-03): đầu Chương ở cột gốc...").

**Giữ:** toàn bộ pipeline (khối ```text```), toàn bộ ví dụ "ca hỏng cụ thể nhất" (không có ngày/Story, là minh hoạ gắn liền với Prevents), bảng tách Chương theo hình dạng đầu vào.

---

## AD-44 — Đường tra cứu tiếng Anh: điều phối theo hình dạng truy vấn, và khoá chữ hoa thay cho stemming

**Cắt:**
1. L575 — chuỗi chính xác: `Đo thật: lớp tiếng Anh sinh **9** cặp `char_idx` trên **119.039** đầu mục *(0,0076%)*. ` (đầu mục 1 của Prevents; câu sau "nên một giai đoạn sẽ cho tiếng Anh đi qua `char_idx`." vẫn đứng được, nối thẳng sang "Truy vấn 1–2 ký tự trả rỗng...").
2. L608 — chuỗi chính xác: `— **1.635** đầu mục tiếng Anh mang chữ hoa có nghĩa (`API` · `Wikipedia` · `English`), và **184** nhóm đầu mục chỉ phân biệt nhau bằng chữ hoa` (giữa "không phải THAY khoá gốc" và ". Phép hạ chữ thường...").
3. L610 (đuôi dòng) + L612 + L614 + L616 — TOÀN BỘ, từ `Hai dữ kiện, và chúng **không cùng độ chắc** — đừng trích cái yếu như cái mạnh:` (giữ nguyên câu trước nó "🔴 **Stemming KHÔNG nằm trên đường nóng tra từ điển** — ghi ra để một giai đoạn sau không "vá" nó vào.") cho tới hết blockquote ⚠️ ở L616 ("...đừng chép tiếp.") — cả khối "Dữ kiện MẠNH", "Dữ kiện YẾU HƠN" và blockquote ⚠️ đi liền nhau, không tách được từng phần mà không phá mạch — cắt cả khối.

**Giữ:** mọi mục ①②④⑤⑥ nguyên văn; các 🔴 định nghĩa ký tự Hán, vị từ chạy một lần, không sổ đăng ký; bảng ② (ngưỡng 3 ký tự — đây LÀ Rule); ví dụ "headword='running'⇒1; 'Running'⇒0" ở Prevents mục ③ (minh hoạ gắn liền với chính lỗi, không tách được); L618 (⚠️ "Giới hạn phải khai kèm... xem hàng Deferred" — giữ nguyên, đây là caveat đang sống, không có ngày/Story, trỏ nội bộ tới `## Deferred`, không trỏ `deferred-work.md`).

**Viết lại (bắt buộc — do cắt #3 xoá mất "bảng dưới" mà Prevents mục ④ đang trỏ tới):** L578, chuỗi hiện tại `...đổi một phụ thuộc lấy **0 recall đo được** *(xem bảng dưới)*, rồi ai đọc lại...` → sửa `*(xem bảng dưới)*` thành `*(xem `spine-evidence.md#AD-44`)*`. Đây là thay trỏ nội bộ thành trỏ evidence do nội dung "bảng dưới" đã chuyển chỗ — không thêm ý mới, chỉ sửa đích tham chiếu.

---

## AD-45 — Bản phát hành không mở một cổng LẮNG NGHE nào

**Cắt (đều loại c — ngày/số đo lịch sử, tách rời được khỏi Rule/Prevents):**
1. L638 — chuỗi chính xác: `Bộ lái e2e (Ice chốt 2026-08-11) là ca đầu tiên: `tauri-plugin-wdio-webdriver` kéo `axum` + `tokio` và mở cổng **4445**.` (câu đứng riêng, câu trước nó "...đúng lớp lỗi kho này tồn tại để săn." vẫn đủ nghĩa một mình).
2. L646 — chuỗi chính xác: `Số đo 2026-08-11: cây mặc định **831** dòng · `--features wdio` **948**.` (câu sau "...vắng mặt khỏi `cargo tree` của bộ feature mặc định." — giữ câu đó, cắt câu số đo).
3. L648 — hai chuỗi chính xác trong cùng dòng: `đo được ` và ` (`tokio 1.53.1`)` — dòng gốc `...không gồm `tokio` — đo được nó đã nằm trong cây mặc định từ trước qua `tauri` (`tokio 1.53.1`). Canh nó...` → sau cắt: `...không gồm `tokio` — nó đã nằm trong cây mặc định từ trước qua `tauri`. Canh nó...`.

**Giữ:** toàn bộ Rule (0 cổng lắng nghe, hai lớp chặn — `optional` + `cfg(debug_assertions)`), đoạn "Cưỡng chế bằng lệnh, không bằng kỷ luật" (trừ câu số đo vừa cắt), đoạn cuối về AD mới cho cổng thứ hai.

---

## AD-46 — Cấu trúc đoạn của bản dịch là dữ liệu riêng của bản dịch

**Cắt:**
- L657 — TOÀN BỘ đoạn ⚠️, nguyên văn từ `⚠️ Ghi ra vì nó đã suýt xảy ra: bản ghi phiên thiết kế 2026-08-14 kết luận...` tới `...rồi làm FR121 hỏng ở Epic 8.` (loại c/e — ngày, tên phiên thiết kế, trỏ `epics.md`, tường thuật một sự cố suýt xảy ra; không đổi Rule đã phát biểu ở bullet kế tiếp).

**Giữ:** toàn bộ Prevents còn lại (kể cả câu nhắc "Epic 8, cách đây sáu epic" — gắn liền với mức độ nghiêm trọng, không tách được sạch); toàn bộ Rule; đoạn ⚠️ "Cái mất, ghi ra" ở L673 (không có ngày, là đánh đổi thiết kế đang sống, trỏ UX-DR13).

---

## AD-47 — Mốc so xuất xứ là lượt ghi KHÔNG-PHẢI-NGƯỜI-DÙNG gần nhất, không phải lượt nạp

**Cắt (đều loại c/e):**
1. L678 — chuỗi chính xác: ` (`prd.md:452`)` (sau "người dùng có gõ chữ này không"").
2. Bảng L682–684 — cắt tên Epic khỏi cột "Cơ chế", ba chuỗi chính xác:
   - `, Epic 8` (dòng Review Mode, sau "FR94")
   - `, Epic 7` (dòng TM khớp 100%, sau "FR58")
   - ` (Epic 4)` (dòng "Đưa đề xuất AI sang Editor (Epic 4)" → còn "Đưa đề xuất AI sang Editor")
3. L694 — chuỗi chính xác: ` (đo ở Quyết định #2 của Story 2.7: `commands/segment.rs:1737-1741` so với **đĩa tại lượt flush**, nên `AB` → `A` bật cờ dù văn bản cuối y nguyên)` (sau "phá đúng ca *gõ rồi hoàn tác*").
4. Bảng ③ (L705–713) — cắt CẢ CỘT "Chủ" (Story/Epic sở hữu — thuần lịch sử, không phải một phần ngữ nghĩa xuất xứ):
   - Header L705: cắt ` Chủ |` ở cuối dòng.
   - Separator L706: cắt `---|` cuối cùng (`|---|---|---|` → `|---|---|`).
   - L707: cắt ` Story 2.7 |` cuối dòng.
   - L708: cắt ` Epic 6 |` cuối dòng.
   - L709: cắt ` Epic 8 |` cuối dòng.
   - L710: cắt ` Story 7.4 |` cuối dòng.
   - L711: cắt ` Epic 4 |` cuối dòng.
   - L712: cắt ` Story 2.8 |` cuối dòng.
   - L713: cắt ` Story 2.6 |` cuối dòng.
5. L719 — chuỗi chính xác: ` ngày 2026-08-16` (trong "hệ quả bắt buộc của chữ ký #1(a) ngày 2026-08-16:").
6. L721 — TOÀN BỘ hai câu, nguyên văn: `Món nợ này **cùng gốc** với món nợ bốn nhãn của Story 2.6 (`deferred-work.md:3685-3697`) và đóng cùng lúc với nó — chủ: story nào cho `segment_version` một cột xuất xứ. Nó **không** được tự chấm đạt ở Story 2.7.` (loại e — trùng `deferred-work.md`; câu trước "...thứ có thể thuộc về một phiên bản khác." đứng đủ nghĩa một mình).
7. L734 — chuỗi chính xác: `(`epics.md:5355`)` (sau ""rà lại hoặc dọn sạch phần không phải văn phong của mình"").
8. L738 — chuỗi chính xác: ` Story 2.5 đã cài đúng nó, 372 ca Rust canh.` (sau "AD-31 §bảng máy trạng thái (sáu hàng) — không sửa một chữ.").
9. L742 — chuỗi chính xác: ` từ 2026-08-02` (sau "AD-18 §thứ tự hai khoá đã giả định").
10. L744 — chuỗi chính xác: `(`epics.md:5169-5170`) ` (sau ""giữ nguyên xuất xứ của cặp TM nguồn"" và trước "thêm ở Epic 7").

**Giữ:** toàn bộ Rule ①②③(hai cột còn lại)④⑤(phần "hệ quả bắt buộc"... trừ ngày)⑥⑦⑧ về nội dung; đoạn ⚠️ "Cái mất, ghi ra" ở ④ (L717, ví dụ `''` gộp với "tôi dịch" — gắn liền với Rule, không tách được); mọi tham chiếu AD-khác (AD-31, AD-5, AD-18, AD-14, AD-6) vì đó là (a).

**Viết lại:** không cần — mọi cắt ở AD-47 đều là xoá cụm/câu độc lập, phần còn lại đã đọc thông (đã kiểm từng chỗ nối ở trên).

---

## AD-48 — Hộp thoại chọn tệp gọi TỪ RUST; không một quyền plugin nào ra JavaScript

**Ngoại lệ cố định (đề bài + kế hoạch §4.3):** đoạn 🔵 ở L765 ("2026-08-25 (Story 3.10b) — con số CHÍN... GIỮ trong AD") — **không cắt**. Đoạn L763 ngay trên nó ("Ice chốt 2026-08-24... đường plugin thêm 9 crate...") **cũng phải giữ nguyên** dù mang ngày/Story/số đo — nếu cắt, đoạn 🔵 ở L765 mất đối tượng nó đang sửa ("con số CHÍN ở đoạn ngay trên"), phạm đúng bẫy "không tách được thì giữ nguyên". Đoạn 🔴 "Điều kiện xét lại" (L769, mang `deferred-work.md:777` + `prd.md:946`) cũng **giữ nguyên** — nó là điều kiện xét lại thật của Rule (ngưỡng byte), không phải lịch sử rời, và tách phần trích dẫn ra khỏi phần điều kiện có rủi ro cao hơn lợi ích nhỏ thu được.

**Cắt:**
1. L751 — chuỗi chính xác: `Story 3.10 dựng trọn nửa định dạng CSV/TSV rồi dừng ở đúng chỗ đó — mã chạy được, nghiệm thu được bằng `cargo test`, và **không lối vào nào**. ` (câu đứng giữa hai câu khác vẫn đủ nghĩa: "...không có đường nào cho người dùng chọn một tệp để ghi ra." nối thẳng "FR49/NFR9 hứa...").
2. L771 — chuỗi chính xác: ` *(Ice chốt 2026-08-25)*` (ngay sau "Vị từ của `check-deps.mjs` Kiểm 1 rộng hơn lý do nó tự khai — sửa LÝ DO, giữ VỊ TỪ**").

**Giữ:** mục ① lớp hỏng kiến trúc, toàn bộ Rule 1–3, đoạn "Vì sao `tauri-plugin-dialog` chứ không `rfd` thẳng" (L761–767, TOÀN BỘ kể cả 🔵), đoạn 🔴 "Điều kiện xét lại" (L769), câu kết luận `BANNED_CRATES` (L773).

---

## AD-49

**Không cắt** (ngoại lệ đã chốt trong đề bài — AD đã viết theo khuôn gọn từ đầu).

---

## Stack

Bảng chính (L823–866) **giữ nguyên cấu trúc Name/Version/Giấy phép**; chỉ rút ngắn chú thích dài trong ngoặc ở 11 hàng xuống còn cụm ngắn (module sở hữu / lý do cốt lõi), cắt phần Story/ngày/đo lường lặp lại đã có trong prose bên dưới hoặc trong `spine-evidence.md#Stack`.

### Cắt trong bảng chính (11 hàng, mỗi hàng cắt CHÍNH XÁC cụm sau, giữ phần còn lại của dòng nguyên văn)

| Dòng | Cắt (chuỗi chính xác) | Còn lại trong ngoặc sau cắt |
|---|---|---|
| L843 `keyring` | `Rà NFR15 lượt chín, 2026-09-17, Story 4.3; ` | `*(provenance sửa lại — xem ngay dưới)*` |
| L844 `keyring-core` | `Story 4.3, ` | `*(`[dev-dependencies]` — cài `keyring_core::mock::Store` cho test; production không khai tên nó, xem `Cargo.toml`)*` |
| L845 `reqwest` | ` ở Story 6.1 cho bàn đo` | `*(feature `blocking` bật — 0 crate mới)*` |
| L846 `dom_smoothie` | ` — Story 6.1` | `*(core::webimport::Extractor)*` |
| L847 `chardetng` | `, Story 6.1` | `*(core::webimport — dò bảng mã)*` |
| L848 `encoding_rs` | `, Story 6.1; đã bắc cầu qua `reqwest`/`quick-xml` trước story này` | `*(core::webimport — giải mã theo bảng mã đã dò; khai tường minh thêm 0 byte)*` |
| L849 `regex` | `, Story 6.5; đã có sẵn trong `Cargo.lock` từ trước — bắc cầu qua `jieba-rs`/`tantivy-stemmers` —` | `*(core::cleanup — luật làm sạch dạng biểu thức chính quy; khai tường minh thêm 0 byte)*` |
| L850 `dom_query` | ` — Story 6.9, nâng từ bắc cầu của `dom_smoothie` (dòng `:909`) thành phụ thuộc TRỰC TIẾP: `extractor.rs` cần duyệt lại HTML GỐC bằng CSS selector để dựng mô hình khối giữ/loại — `Article::content` của `dom_smoothie` không phơi ra đủ, xem §Design Notes spec 6.9. Đã có sẵn trong `Cargo.lock` từ Story 6.1, LICENSE đã mở đọc trong nguồn đã tải (`~/.cargo/registry/src/…/dom_query-0.28.0/LICENSE`) — 0 gói MỚI vào `Cargo.lock`, chỉ đổi từ bắc cầu sang khai tường minh` | `*(core::webimport::Extractor — nâng từ bắc cầu thành phụ thuộc trực tiếp; 0 gói MỚI vào `Cargo.lock`)*` |
| L851 `zip` | ` — Story 6.12, đọc kho zip của `.docx`. Đã có sẵn trong `Cargo.lock` từ trước qua `docx-rs` (bắc cầu, `cargo tree -i zip` xác nhận 2026-09-09) — khai tường minh ở đây thêm 0 gói MỚI, chỉ đổi từ bắc cầu sang trực tiếp. LICENSE (MIT) đã mở đọc trong nguồn đã tải, xem Rà NFR15 lượt tám ngay dưới` | `*(core::docx — đọc kho zip của `.docx`; thêm 0 gói MỚI, chỉ đổi từ bắc cầu sang trực tiếp)*` |
| L852 `quick-xml` | ` — Story 6.12, phân tích `word/document.xml`/rels. `Cargo.lock` mang HAI phiên bản của crate này (0.38.4 qua `tauri`→`plist`, 0.41.0 qua `docx-rs`) — khai đúng **0.41.0** để không thêm một phiên bản thứ ba; 0 gói MỚI, LICENSE (MIT) đã mở đọc trong nguồn đã tải, xem Rà NFR15 lượt tám ngay dưới` | `*(core::docx — phân tích `word/document.xml`/rels; khai đúng **0.41.0** để không thêm một phiên bản thứ ba)*` |
| L861 `vitest` | ` — Story 2.3` | `*(bộ chạy test frontend)*` |

(Cách đọc bảng trên: cột giữa là chuỗi CẦN XOÁ nguyên văn khỏi dòng gốc; cột phải là kết quả — dùng để Pha 3 tự kiểm sau khi xoá, không phải một chuỗi cần chèn thêm.)

### Cắt trong prose dưới bảng (L868–1002)

**Cắt nguyên đoạn (loại c/e — audit trail NFR15 thuần, trùng nội dung đã có ở AD-48 hoặc là lịch sử tách rời không đổi nghĩa hiện hành):**
- L874–883 (toàn bộ khối "Ba hàng ⚠️ và một hàng suýt bị chấm sai" + bảng chi tiết jieba-rs/docx-rs/dockview-vue/tantivy-stemmers + câu Vite `LICENSE.md`) — bằng chứng chi tiết, các ⚠️ trong bảng chính đã đủ tín hiệu.
- L885 (đoạn "Rà NFR15 lượt ba — 2026-08-11...").
- L891 (đoạn "Khác ba lượt trước ở đúng một chỗ...").
- L893 (đoạn ⚠️ "vitest mang một bảng giấy phép GỘP...").
- L895 (đoạn ⚠️ "Cây npm đi từ 530 lên 656 gói...").
- L899 (đoạn 🔵 "2026-08-25 (muộn hơn, Story 3.10b) — con số CHÍN sai" — TRÙNG với AD-48 L765, giữ đúng một bản ở AD-48).
- L915 (đoạn ⚠️ "`trash` KHÔNG vào cây phát hành...").
- L917 (đoạn 🔵 "2026-08-25 (Story 3.10b) — số byte payload ĐÃ ĐO" — TRÙNG với AD-48 L765's "156.392 byte").
- L919 (câu mở đầu "**Rà NFR15 lượt sáu — 2026-09-03, Story 6.1**...").
- L941 (câu mở đầu "**Rà NFR15 lượt bảy — 2026-09-05, Story 6.5**...").
- L943–945 (bảng chi tiết `regex` — trùng hàng `regex` đã có trong bảng chính L849).
- L949 (câu mở đầu "**Rà NFR15 lượt tám — 2026-09-09, Story 6.12**...").
- L951–954 (bảng chi tiết `zip`/`quick-xml` — trùng hai hàng đã có trong bảng chính L851–852).
- L996 (đoạn 🔵 "2026-08-25 — `tauri-plugin-dialog` và `tauri-plugin-fs` RỜI danh sách này" — nội dung đã suy ra được từ AD-48 + danh sách `BANNED_CRATES` hiện hành ở L994).

**Cắt một phần (giữ phần còn lại nguyên văn):**
- L868 — GIỮ: `Tương thích GPL v3 theo diện **gộp gói** — font nằm cạnh mã, không liên kết vào mã. Ràng buộc kèm theo: **chỉ `Source Sans 3` khai Reserved Font Name `'Source'`** — subset riêng tệp đó thì bắt buộc đổi tên font nội bộ; hai tệp kia không khai nên subset thoải mái. Cả ba tệp giấy phép gốc phải đi kèm bản phát hành (FR38, FR109).` — CẮT phần còn lại của dòng (câu mở đầu về ngày/Story 1.1/phương pháp, câu về `name ID`/`Source Serif 4` 4.004, câu về `name ID 16`, câu trỏ `research/font-spike-results...`).
- L870 — GIỮ: `Cột `Giấy phép` ở trên nay mang dấu: **✓** = đã tự tay mở tệp và văn bản khớp nhãn; **⚠️** = nhãn đúng nhưng **bằng chứng yếu hơn**, ghi ra để không ai tưởng cả bảng cùng một độ chắc.` — CẮT phần mở đầu ("Rà NFR15 lượt hai — 2026-08-03, Story 1.2...") và hai cụm số liệu `(**16/19** hàng phần mềm)` / `(**3/19**: ...)`.
- L872 — GIỮ CHỈ câu cuối: `Mọi giấy phép trong bảng đều thuộc nhóm dễ dãi (MIT · Apache-2.0 · BSD-3-Clause · BSD-2-Clause · ISC · OFL 1.1) và **tương thích GPL v3 theo chiều đi vào**.` — CẮT toàn bộ phần trước (blockquote về `tantivy-stemmers` bị chấm nhầm BSD-2, "Sửa 2026-08-03 sau rà soát mã: prose cũ ghi 15/19...").
- L887 — CẮT chuỗi chính xác: `Cây npm đi từ 194 lên 530 gói khi bộ lái e2e vào. ` — GIỮ phần còn lại (CC-BY-4.0/`css-value` fact + câu `check-deps.mjs` Kiểm 2).
- L998 — CẮT ba chuỗi chính xác: ` (Story 1.2)` · ` Story 1.3 gắn script này vào CI.` · ` 🔵 *(2026-08-25: sáu → bốn, AD-48.)*` — GIỮ: `**Bốn** tên `tauri-*` trên được **cưỡng chế bằng lệnh**, không bằng kỷ luật: `scripts/check-deps.mjs` chạy `cargo tree` và trả mã thoát khác 0 nếu bất kỳ tên nào xuất hiện.`
- L1002 — CẮT chuỗi chính xác: ` từ 2026-08-11` — GIỮ toàn bộ phần còn lại nguyên văn.

**Không cắt (chứa dữ kiện tuân thủ giấy phép KHÔNG trùng lặp ở đâu khác — rủi ro mất bằng chứng NFR15 nếu cắt):**
- L889 (khối 🔴 "Lượt lật đó KHÔNG bác bỏ luật cũ..." — quy tắc quy trình đang sống, đánh dấu 🔴).
- L897–913 (đoạn mở "Rà NFR15 lượt năm — 2026-08-25, AD-48" + bảng 9 dòng `tauri-plugin-dialog`/`tauri-plugin-fs`/`rfd`/họ `notify`). **Lý do không cắt dù mang ngày/Story/🔵 dày đặc:** `tauri-plugin-dialog`, `tauri-plugin-fs`, `rfd` KHÔNG xuất hiện trong bảng Name/Version/Giấy phép chính (L823–866) — đây là bản ghi giấy phép DUY NHẤT của ba crate đã lên nhị phân phát hành theo AD-48. Xem "Ghi chú cho Pha 2/3" — đây là một bất thường (anomaly) cần Ice/Winston quyết định, không phải một cắt mechanical.
- L919 **loại trừ dòng mở đầu đã liệt ở trên** — nhưng bảng L921–925 (`dom_smoothie`/`chardetng`/`encoding_rs`) và câu kết L927 GIỮ NGUYÊN (dòng đầu ba crate này CÓ trong bảng chính nhưng bảng phụ này còn mang chi tiết "công trình chung Mozilla" không lặp ở đâu khác).
- L929 (đoạn ⚠️ "`dom_smoothie` kéo theo một cây con 10 gói bắc cầu MỚI" — chứa HAI lượt 🔵 tự sửa số đếm lồng trong cùng số liệu thật (12→10, 9→7); không tách được phần 🔵 khỏi phần dữ kiện đúng mà không viết lại nội dung — để nguyên, đánh dấu cần một lượt rà nội dung riêng (không phải cắt cơ học) nếu muốn gọn thêm.
- L931 (đoạn 🔵 "Đo bổ sung 2026-09-03 — mệnh đề MPL-2.0 tương thích GPLv3 ở trên vốn được KHẲNG ĐỊNH, nay được ĐO" — bằng chứng Exhibit B, khớp đúng chính sách NFR15 của AGENTS.md về MPL-2.0).
- Bảng L933–937 (`cssparser`/`cssparser-macros`/`selectors` — bằng chứng Exhibit B chi tiết, không trùng ở đâu khác).
- L939 (giải thích AC "bảng Stack có 37 hàng (34+3)").
- L941 loại trừ dòng mở đầu (đã cắt ở trên); bảng L943–945 và L947 — **mâu thuẫn nội bộ đã lưu ý:** L943–945 (bảng chi tiết `regex`) đã liệt vào "cắt nguyên đoạn" bên trên vì trùng hàng chính L849; L947 (câu kết luận + quy ước ghim `=`) GIỮ NGUYÊN.
- L949 loại trừ dòng mở đầu (đã cắt ở trên); L951–954 đã liệt cắt ở trên (trùng bảng chính); L956 (câu kết luận về `quick-xml` 0.38.4 song song) GIỮ NGUYÊN — giải thích tại sao hai phiên bản `quick-xml` cùng tồn tại, không lặp ở đâu khác.
- L958–990 (toàn bộ "Rà NFR15 lượt chín — Story 4.3", họ `keyring`). **Lý do không cắt:** L843 sau khi rút gọn trỏ "xem ngay dưới" đúng vào đoạn này; đoạn L982–990 còn mang một ràng buộc kỹ thuật thật không lặp ở đâu khác (`keyring_core::mock::Store` phải khởi tạo SAU lượt gọi `keyring::Entry::new` thật đầu tiên — thứ tự bắt buộc, không tuỳ chọn). Gắn cờ cho Winston/Ice: nội dung này có lẽ nên nằm ở AD-29 hoặc một doc-comment trong `tests/aiconfig_contract.rs`, không phải Stack — nhưng đó là việc DỜI nội dung, ngoài phạm vi "cắt cơ học" của Pha 2/3.
- L992 (sàn tối thiểu SQLite — FTS5 trigram ≥3.34, `remove_diacritics 0` ≥3.27 — đây LÀ một ràng buộc kiến trúc, không phải lịch sử).
- L994 (danh sách "Không dùng, đã loại có lý do" — tham chiếu chéo AD-1/AD-29/AD-11/AD-26, ngắn, cấu trúc).
- L1000 (khối ⚠️ "Cổng đó canh MÃ TRONG NHỊ PHÂN, không canh BỀ MẶT IPC" — phân biệt hai cổng đang sống, không có ngày).

---

## Deferred

Quy tắc áp dụng đúng như kế hoạch §3: hàng `✅ ĐÃ ĐÓNG` → rút còn `✅ **ĐÃ ĐÓNG <ngày> (Story x.y nếu có)** — xem `deferred-work.md``, cột "Điều kiện mở lại" giữ `—`. Hàng còn mở (không có `✅ ĐÃ ĐÓNG`, kể cả hàng 🟡 nửa đóng) **giữ nguyên toàn bộ**.

**Cắt — rút gọn 6 hàng đã đóng (mỗi hàng: xoá toàn bộ nội dung ô giữa + mọi đoạn prose phụ lục đi kèm ngay sau hàng, thay bằng dòng rút gọn):**

1. **L1135–1154** ("Thư viện bóc nội dung chính", FR123) — xoá TOÀN BỘ từ ô giữa của hàng L1135 cho tới hết đoạn "Đo bổ sung 2026-09-03" và đoạn 🔵 "SỬA 2026-09-06 (Story 6.7)" (kể cả hai bảng byte-nhị-phân ở L1148–1151) — thay bằng:
   `| ~~**Thư viện bóc nội dung chính** (FR123)~~ | ✅ **ĐÃ ĐÓNG 2026-09-03 (Story 6.1)** — xem `deferred-work.md` | — |`
   (Chi tiết đo lường 7 mẫu, tỉ lệ 72–99%, delta byte nhị phân Story 6.1/6.7 — chép nguyên văn sang `spine-evidence.md#Deferred`; đây là dữ liệu đo trùng với mục "Chi phí byte NFR6 THẬT của `dom_smoothie`" đã có ở `deferred-work.md`, xác nhận bằng chuỗi `→ ✅ ĐÃ ĐÓNG 2026-09-06` trong chính đoạn L1154.)

2. **L1156** ("HTTP client cho `Fetcher`") →
   `| ~~**HTTP client cho `Fetcher`**~~ | ✅ **ĐÃ ĐÓNG 2026-09-03 (Story 6.1)** — xem `deferred-work.md` | — |`

3. **L1157** ("Ranh giới Chương ở đường nhập song ngữ", FR115) →
   `| ~~**Ranh giới Chương ở đường nhập song ngữ** (FR115)~~ | ✅ **ĐÃ ĐÓNG 2026-08-03** — xem `deferred-work.md` | — |`
   (⚠️ mất "Bài học: trước khi ghi một hàng Deferred, soát xem tầng dưới đã trả lời chưa" — chép nguyên văn sang evidence, đây là bài học quy trình có thể còn giá trị chung.)

4. **L1158** ("Hành vi khi một link trong danh sách hỏng") →
   `| ~~**Hành vi khi một link trong danh sách hỏng** (404, timeout, tường chặn)~~ | ✅ **ĐÃ ĐÓNG 2026-09-06 (Story 6.7)** — xem `deferred-work.md` | — |`

5. **L1163** ("HVTĐTD", Q3) →
   `| ~~**HVTĐTD** (Q3)~~ | ✅ **ĐÃ ĐÓNG 2026-08-02** — xem `deferred-work.md` | — |`

6. **L1167** ("Dung lượng và giấy phép font nhúng") →
   `| ~~**Dung lượng và giấy phép font nhúng**~~ | ✅ **ĐÃ ĐÓNG 2026-08-03 (Story 1.1)** — xem `deferred-work.md` | — |`
   (Đoạn gốc rất dài, gồm hai lượt tự sửa số đo — 20,300 MiB, dư địa ~47MB đã lỗi thời, NFR6 sửa lần hai. Chép nguyên văn sang evidence.)

7. **L1168** ("Biến thể vùng cho Source Han Serif") →
   `| ~~**Biến thể vùng cho Source Han Serif**~~ | ✅ **ĐÃ ĐÓNG 2026-08-03 (Story 1.1)** — xem `deferred-work.md` | — |`

**Cắt nhỏ trong hai hàng CÒN MỞ (không đóng hẳn, nhưng có 🔵 đính chính lịch sử tách rời được khỏi câu trả lời hiện hành):**

8. L1164 — chuỗi chính xác cần cắt khỏi tiêu đề hàng: `🔵 *(đổi tên 2026-08-18: "panel Editor" là tên đã chết sau correct-course 2026-08-14; câu hỏi và AD-31 không đổi)* ` — còn lại tiêu đề: `**Thư viện editor cho cột bản dịch của lưới**`.
9. L1165 — chuỗi chính xác cần cắt khỏi cột "Điều kiện mở lại": ` *(Sửa 2026-08-13: bản trước ghi "Giai đoạn 2". CAP-4 dời sang Giai đoạn 2c và 2c nay chạy SAU Giai đoạn 3b — xem `build-sequence.md` cột "Thứ tự". Mỏ neo cũ để lại sẽ bị đọc thành "phải rà giấy phép SSE trước khi làm Editor", sai cả hai vế. Cửa rà NFR15 **không đổi**, chỉ mở muộn hơn.)*` — còn lại: `🔵 **Giai đoạn 2c** — rà giấy phép trước khi thêm.`

**Giữ nguyên toàn bộ (hàng mở, không có ✅ ĐÃ ĐÓNG):** L1134 (docx đọc), L1159 (`similar` vs `dissimilar`), L1160 (segment alignment), L1161 (ngưỡng WAL/flush), L1162 (ngưỡng NFR3/4/5), L1155 (🟡 phát hiện bảng mã — nửa đóng, còn nợ số đo thật, KHÔNG rút gọn), L1166 (ảo hoá danh sách dài), L1169 (stemming AD-44 ③ — dù trùng một phần số liệu đã cắt ở AD-44, đây là hàng MỞ theo đúng quy tắc plan, không đụng), L1170 (cụm nhiều chữ), L1171 (nhiều thư mục gốc), L1172 (chỉ mục FTS Library), L1173 (Q1 vòng phản hồi đứt).

---

## Ghi chú cho Pha 2/3

1. **Bẫy bảng markdown (AD-47 ③, Stack chính):** cắt một CỘT khỏi bảng đòi sửa MỌI hàng kể cả header và separator cùng lúc — xoá lệch một hàng sẽ làm `lint_spine.py` hoặc `markdownlint` (nếu có) báo bảng méo cột. Đã liệt đủ từng hàng cần sửa ở mục AD-47.
2. **Bẫy chuỗi không duy nhất:** nhiều chuỗi cắt ngắn (vd. `, Story 6.1`, ` (Epic 4)`) có thể trùng ở nhiều chỗ trong tệp — Pha 3 PHẢI khớp trong đúng phạm vi dòng đã ghi (L-number tại `d861aab`), không `sed` toàn cục theo chuỗi trần.
3. **Bẫy đoạn 🔵 lồng số liệu đúng (Stack L899/917 vs AD-48 L765, và AD-45's "cổng đó" vs Stack L998/L1000):** một số 🔵 xuất hiện Ở HAI NƠI cho cùng một sự kiện (Stack và AD tương ứng) — chỉ giữ MỘT bản (ở AD, vì đó là nơi Rule sống), cắt bản còn lại ở Stack. Đã áp dụng cho cặp (AD-48 L765) ↔ (Stack L899, L917).
4. **Bẫy dangling reference sau khi cắt bảng/đoạn:** ba chỗ đã xác nhận cần "Viết lại" (AD-18 §③ ngôn ngữ nguồn, AD-44 mục ④ "xem bảng dưới"→ trỏ evidence). Chưa phát hiện chỗ thứ tư — nhưng Pha 3 nên `grep -n 'xem bảng dưới\|xem ngay dưới\|bảng dưới đây'` sau khi cắt xong để tự kiểm không còn con trỏ treo.
5. **Bất thường cần Ice/Winston, KHÔNG tự sửa ở Pha 2/3:** ba crate `tauri-plugin-dialog`/`tauri-plugin-fs`/`rfd` (AD-48) và ba crate MPL-2.0 `cssparser`/`cssparser-macros`/`selectors` (dom_smoothie) không có hàng trong bảng Name/Version/Giấy phép chính — bản ghi giấy phép duy nhất của chúng nằm trong prose "Rà NFR15 lượt năm/sáu" mà worksheet này đã đánh dấu KHÔNG CẮT đúng vì lý do này. Nếu một lượt sau muốn gọn thêm Stack, việc đúng là DỜI các hàng đó vào bảng chính trước, không phải xoá bằng chứng duy nhất của chúng.
6. **AD-12 `.memlog.md`:** đã kiểm — dòng 46 của `.memlog.md` (cùng thư mục spine) còn xác nhận đúng "WAL2 KHÔNG phải tính năng đã phát hành"; an toàn giữ nguyên câu trỏ `xem .memlog.md` sau khi cắt phần diễn giải trong ngoặc (xem mục AD-12).
7. **Không có phát hiện nào phạm quy tắc máy đọc của `lint_spine.py`:** mọi cắt trong worksheet này giữ nguyên heading `### AD-N`, giữ đủ ba nhãn `Binds`/`Prevents`/`Rule` chữ trong mỗi khối AD, không tạo `TBD/TODO/FIXME/XXX`, không tạo `{token}` trống, và bảng Stack chính vẫn giữ cột Version không rỗng ở mọi hàng.
8. **Tổng kích cỡ sau gọn (~126,5 KB) không đạt 95–100 KB Ice chốt** — xem giải trình số ở bảng tóm tắt đầu tệp. Đây là phát hiện của Pha 1 cần báo lại Ice trước khi Pha 3 chạy, không phải lỗi của worksheet.
