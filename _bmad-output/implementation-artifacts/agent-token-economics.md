# Kinh tế token của agent — số đo, và chỗ số đo còn thiếu

Đo **2026-09-12** trên toàn bộ transcript của dự án (`~/.claude/projects/…AuraTranslate/`):
130 phiên, 560 subagent, **~31 tỉ token**. Tệp này giữ BẰNG CHỨNG; ba luật rút ra từ
nó sống trong `AGENTS.md` §Policy. Sửa số ở đây thì sửa luật ở kia, và ngược lại.

🔴 **Tệp này là nguồn cho `bmad-project-context`.** Nó được khai trong
`_bmad/custom/bmad-project-context.toml` → `external_sources`, nên mỗi lượt
`refresh` đọc lại nó và dựng lại ba luật đó từ đây. Đừng xoá nó để "gọn kho" —
xoá nó là gỡ nguồn của ba dòng trong tệp được nạp mọi phiên.

---

## 1. Tiền nằm ở đâu

| | token | |
|---|---:|---:|
| Phiên chính (130) | 8,6 tỉ | 28% |
| **Subagent (560)** | **22,7 tỉ** | **72%** |

Phân bố lệch cực mạnh trong nhóm subagent:

| Nhóm | Số agent | Token | |
|---|---:|---:|---:|
| 0-50 M | 502 (90%) | 2,4 tỉ | 10% |
| 50-200 M | 19 | 2,5 tỉ | 11% |
| **> 200 M** | **39** (7%) | **17,8 tỉ** | **79%** |

**38 trên 39** con nặng nhận cùng một câu giao việc: *"đọc spec và thi công story"*.
Trung vị **779 lượt**, đỉnh ngữ cảnh trung vị **813k**. Con tệ nhất: 2.217 lượt ·
16,5 giờ · **1,2 tỉ token** (trong đó 11,1 giờ là khoảng nghỉ, một cú 481 phút).

## 2. Cơ chế: chi phí là DIỆN TÍCH DƯỚI ĐƯỜNG NGỮ CẢNH

Con 1,2 tỉ: output của chính nó **0,459 M = 0,04%**. 99,96% còn lại là đọc lại thứ
nó đã nói. Giá trung bình một lượt của nhóm nặng: **516k token** — bất kể lượt đó
trả về gì.

**Ba giả thuyết rẻ tiền, đo và chết cả ba:**

| Giả thuyết | Phép đo | |
|---|---|---|
| "agent chạy lặp, lãng phí" | 22 lệnh Bash trùng / 898 | chết (2,4%) |
| "kết quả test đổ đầy ngữ cảnh" | toàn bộ kết quả Bash của 39 agent = 16,1 MB | chết (0,02% chi phí) |
| "ngữ cảnh phình vì nội dung cồng kềnh" | tốc độ phình 906 token/lượt | chết (đã rất gọn) |

Nội dung tích lại ~1,3 triệu token; trả 1.199 triệu. **Khuếch đại ~900 lần.**

Bóc `g` (trung vị 906 token/lượt, tứ phân vị 813-968, biên 609-1163):

| Thành phần | token/lượt | % |
|---|---:|---:|
| Kết quả công cụ trả về | 650 | 66,6% |
| Lời gọi `Edit` (old/new string) | 123 | 12,6% |
| Nhắc hệ thống / attachment | 79 | 8,1% |
| Lời gọi `Bash` | 59 | 6,1% |
| Lời gọi `Write` | 34 | 3,4% |
| Văn bản trả lời của agent | 14 | 1,4% |
| Lời gọi `Read` | 7 | 0,7% |

## 3. Cần gạt ① — chẻ story thành ~4 pha: **2,9×**

Mô hình: `chi phí ≈ lượt × (ngữ cảnh khởi động + trần) / 2`.

⚠️ **Mô hình phải tự kiểm trước khi được phép dự đoán.** Dùng B, g, C đo được của
từng agent để dự đoán lại chi phí THẬT của chính nó: 38/39 con sai ≤25% (trung vị
**−18,8%**, lệch cùng chiều ở 36/39). Một con sai 27% — **bị loại** khỏi phép ngoại suy.

Phản thực, cộng trên từng agent với B, g, N của **chính nó**:

| Trần | bàn giao 0k | **15k** | 40k |
|---:|---:|---:|---:|
| 150k | 4,7× | **4,3×** | 3,9× |
| **250k** | 3,1× | **2,9×** | 2,7× |
| 400k | 2,0× | **1,9×** | 1,8× |
| 600k | 1,4× | **1,3×** | 1,3× |

Ở trần **250k**, phí bàn giao **15k**: **2,9×** — trên thang thật 17,1 tỉ → 5,9 tỉ,
**tiết kiệm ≈ 11,2 tỉ token**, giá là **~3,8 lần bàn giao mỗi story**.

🔴 **Con số này đã sai HAI lần trước khi đúng** — ghi ra để không ai lặp lại:

1. `3,7×` — ngoại suy từ MỘT agent, dùng `g` mượn của chính nó. Con đó có `g = 813`
   so với trung vị nhóm `906`, tức nó là **ca thuận lợi nhất**.
2. `3,4×` — đo cả quần thể, nhưng chia chi phí **thật** cho chi phí **mô hình**.
   Mô hình lệch thấp 15% một cách hệ thống ⇒ mẫu số nhỏ đi ⇒ tỉ số phồng lên.
3. `2,9×` — **mô hình so mô hình**, sai lệch triệt tiêu; `g` đo từng con; có phí
   bàn giao; đã loại con mô hình không tả nổi.

⚠️ Bốn lần bàn giao là bốn chỗ đánh rơi một chi tiết. Cái giá đó **không định lượng
được** ở lượt đo này.

## 4. Cần gạt ② — dồn việc nặng về CUỐI đời agent: **27×**, chưa có ước lượng tiết kiệm

Đổi đơn vị đo: byte không phải chi phí; **byte × số lượt còn phải cõng** mới là.

```
chi phí cõng sinh trong 25% lượt ĐẦU   :  69%
chi phí cõng sinh trong 25% lượt CUỐI  :   3%
⇒ một byte nạp SỚM đắt hơn cùng byte đó nạp MUỘN ~27 lần
```

Theo đơn vị đó: `Read` **61,9%** chi phí cõng · `Bash` 36,0% · `Edit` 1,9%.

Hệ quả khó chịu: **pha "đọc để hiểu bối cảnh" là pha đắt nhất của một agent** — đúng
cái pha mọi hướng dẫn bảo phải làm cho kỹ. Lối ra là làm nó trong một agent **NGẮN**
rồi ghi kết quả ra đĩa.

🔴 **KHÔNG có con số tiết kiệm cho cần gạt này, và đó là cố ý.** Chưa ai đo bao nhiêu
phần của pha đọc đầu là **bắt buộc** và bao nhiêu là **hoãn được**. Đừng bịa một con số
vào ô trống này; hãy đo nó. Chủ: Ice.

## 5. Cần gạt ③ — kích thước tệp nguồn là một khoản chi token: **9,4%**

| Tệp | Lượt Read | Byte | % chi phí cõng |
|---|---:|---:|---:|
| **`src-tauri/src/commands/project.rs`** (6.630 dòng) | **377** | 2,30 MB | **9,4%** |
| `src-tauri/src/core/segment/pipeline.rs` (1.676) | 135 | 1,15 MB | 5,0% |
| `src-tauri/AGENTS.md` | 34 | 0,44 MB | 1,8% |
| `src/ImportPreviewOverlay.vue` (2.433) | 116 | 0,46 MB | 1,3% |

⚠️ **Lời giải hiển nhiên đã bị phép đo BÁC.** "Agent đọc cả tệp thay vì đọc một đoạn"
— sai: **80%** lượt Read đã có `offset`/`limit`, trung vị một lượt đọc `project.rs`
là **2,9 KB**. Vấn đề ngược lại: **377 lượt ngó, mỗi lượt một ô cửa sổ khác, ~10 lượt
mỗi agent**, vì không ai — người hay máy — giữ nổi bản đồ 6.630 dòng. Bản vá là chẻ
**TỆP**, không phải đọc ít hơn.

`project.rs` lớn gấp **1,8×** tệp kế tiếp (`commands/segment.rs`, 3.755 dòng).
Không cổng nào đo kích thước tệp hôm nay.

## 6. Giá của chính tệp này

Cả lượt sửa `AGENTS.md` hôm nay làm nó phình **13.004 → 17.710 byte (+36%)** — gồm
ba luật này, luật chống-treo, bản sửa 🔵 về LuLu, và hai dòng về vòng lặp dev có
phạm vi. Riêng ba luật token: bản đầu tốn ~1.800 byte, sau khi dời bằng chứng
xuống tệp này còn **~1.100 byte ≈ 280 token**, nạp ở **mọi** phiên và **mọi**
subagent.

Với ~65.000 lượt đã đo trong kho này, nếu ba dòng đó tồn tại từ đầu thì chúng đã
tốn **≈ 18 triệu token** — đổi lấy **11,2 tỉ**. Tỉ lệ **~620 : 1**. (Bản chưa dời
bằng chứng: ~29 triệu, tỉ lệ ~390:1. Phép dời xuống tệp này tự nó đáng ~11 triệu
token.)

Ghi ra vì một tài liệu về tiết kiệm token mà không tự tính giá của mình thì không
đáng tin. Nếu một lượt `audit` thấy §Policy vượt ngân sách, ba dòng đó vẫn đứng —
tỉ lệ 127:1 là căn cứ — nhưng phần **bằng chứng** thì thuộc về tệp này, không thuộc
về khối được nạp mọi phiên.
