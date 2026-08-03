# Rubric walker — checklist spine tốt, vòng cập nhật FR122–FR131

**Verdict: đạt sau khi vá bốn lỗ của lens đối kháng.** Bản thân cấu trúc lành; vấn đề nằm ở chi tiết chưa khai, không ở hình dạng.

| Tiêu chí | Kết quả |
|---|---|
| Cố định đúng các điểm phân kỳ của tầng dưới | ⚠️ → ✅ sau vá. Bốn điểm phân kỳ thật bị bỏ sót ở bản đầu (F1–F4) |
| Mỗi Rule cưỡng chế được và thật sự chặn phân kỳ đã nêu | ✅ sau vá. AD-41 là ca đặc biệt: **tự khai mình không được framework cưỡng chế** và bù bằng nghĩa vụ test — đây là cách xử lý đúng, không phải điểm yếu |
| Deferred không để hai đơn vị phân kỳ | ✅ Ba hàng mới đều là **lựa chọn thư viện**, không phải hợp đồng giữa module. AD-39/AD-40 đã cố định phần bất biến |
| Công nghệ nêu tên đã kiểm hiện hành | ✅ Xem `review-version-2026-08-03b.md` |
| Bao phủ năng lực mà PRD giao | ✅ FR122–FR131 đều có AD hoặc quy ước phủ. Bản đồ C1/C2/C8/C9/C10 đã cập nhật |
| Không AD mới nào làm yếu AD cũ | ✅ **AD-15 mạnh lên**: mệnh đề CSP *không ảnh ngoài* giữ nguyên và được FR127 củng cố. **AD-2 giữ nguyên sức nặng**: xét theo đúng thủ tục rồi kết luận không nâng cổng |
| Mọi chiều thuộc altitude đều được quyết, hoãn, hoặc để ngỏ tường minh | ✅ Chiều mới duy nhất là **ra mạng** — quyết ở AD-15, AD-40, AD-41 |

## Ghi nhận riêng

**AD-40 là quyết định tốt nhất của vòng này.** Nó tách một câu hỏi tưởng là nhị phân (*"có port thứ tư không"*) thành hai việc có vòng đời khác nhau, rồi đặt tên chỗ mở rộng sẽ xảy ra **mà không dựng trước bộ khung cho nó**. Ranh giới `Fetcher` không chạm HTML / `Extractor` không chạm mạng cũng chính là thứ làm NFR19 nghiệm thu được — một quyết định phục vụ hai mục đích.

**AD-41 trung thực về giới hạn của chính nó.** Khai thẳng rằng capabilities Tauri là tĩnh nên không diễn đạt được allowlist lúc chạy, rồi biến điều đó thành **nghĩa vụ test bắt buộc**, là cách xử lý đúng. Spine này đã có tiền lệ tốt ở FR39 và FR126: rủi ro chưa giải được vẫn cho ra thiết kế đúng.

**Chỗ mỏng còn lại, không chặn:** đường nhập mới chưa có gì nói về **thất bại từng phần** — dán 50 link, link thứ 30 trả 404 hoặc timeout. Không phải điểm phân kỳ giữa hai đơn vị (nó là hành vi trong một module) nên không thành AD, nhưng nên vào Deferred để không rơi.
