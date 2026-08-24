<!-- bmad:context -->
<!-- Verified 2026-08-24 against b290336. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## scripts/ — luật của một CỔNG

Mười ba `check:*` cưỡng chế những mệnh đề khai báo trên TOÀN CÂY (*"không màu viết thẳng ở bất kỳ đâu"*) — vai mà một test đơn lẻ không đảm được. Thêm một cổng là sửa BA danh sách: `package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`, và `check:gates` canh cả ba.

## Conventions that differ from defaults

- Mã thoát là phán quyết. Không cổng nào ghi log rồi đi tiếp.
- 🔴 Lỗi hạ tầng KHÔNG phải một phép kiểm đỏ: không đọc được tệp ⇒ `abort()` và thoát khác 0 kèm câu *"đây là lỗi hạ tầng, không phải đạt"*. Đừng bao giờ báo một kết quả không có thật.
- 🔴 Không phán quyết nào được đọc tham số từ chính thứ nó đang kiểm. Sàn WCAG, danh sách vai, danh sách loại trừ — đóng băng TRONG script. Đã đo: ba đường thoát đều cho exit 0 trong khi sản phẩm mang một cặp tương phản 4,245:1.
- Sàn quần thể: *"cây rỗng không phải cây sạch"*. Cổng đếm số tệp và `abort()` khi dưới sàn; sàn đặt ở ~80–85 % số thật. Sàn là cận DƯỚI, nên tệp thừa không làm cổng đỏ — nó chỉ làm sàn vô nghĩa. Thêm tệp vào `src/**` thì xét lại sàn.
- Node thuần, không bash — `npm run` trên Windows đi qua `cmd.exe`, và một cổng chỉ canh nửa số nền tảng thì không canh được NFR14.
- Không thêm phụ thuộc npm cho một cổng. Parser TOML/CSS trong thư mục này là tập con nghiêm ngặt tự viết, và cú pháp ngoài tập con ⇒ FAIL, không bỏ qua.
- Cổng MỚI phải mang một phép TỰ KIỂM chứng minh nó ĐỎ ĐƯỢC và không đỏ oan — một cổng chưa bao giờ đỏ là một cổng chưa ai biết nó có chạy không.

## Known pitfalls

- ⚠️ Chỉ 4/13 cổng có phép tự kiểm hôm nay (`check-gates` Kiểm C · `check-layout` Kiểm D · `check-panel-refs` Kiểm C · `check-debt-owner` Kiểm B). Tám cổng còn lại chưa chứng minh được là chúng đỏ được — đừng đọc một lượt xanh của chúng như một bảo đảm. Món nợ có chủ trong `deferred-work.md`.
- ⚠️ `abort()` cũng chưa phủ hết: `check-panel-refs.mjs:78` lệch hình dạng (thoát 2, chữ khác), còn `check-scope`, `check-scope-bundled` và `check-dict-manifest` không có `abort()` — chúng `process.exit(1)` trần, nên một lỗi hạ tầng ở đó đọc lên giống một phép kiểm đỏ.
- Cổng không mang tiền tố `check:` phải có mặt ở cả ba danh sách — Kiểm F canh riêng việc đó, vì Kiểm A và D chỉ duyệt tên `check:*`. ⚠️ Bảng `REQUIRED_SCRIPTS` hôm nay có ĐÚNG MỘT mục (`test`). `test:e2e` là cổng thứ hai và cố ý NẰM NGOÀI bảng (`pre-push` loại bộ e2e có chủ ý), nên nó chỉ được canh ở HAI trong ba danh sách: xoá bước `npm run test:e2e` khỏi `ci.yml` thì cả sáu phép kiểm vẫn xanh. Đừng đọc `check:gates` xanh thành "ba danh sách đã khớp". (Chủ: Story 3.9.)

<!-- /bmad:context -->
