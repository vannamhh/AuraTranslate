#!/bin/zsh
# Bộ lái gõ của Story 2.4 — Quyết định #2 đường (c): bơm phím ở TẦNG HỆ ĐIỀU HÀNH.
# Sự kiện phím THẬT đi vào cửa sổ thật ⇒ engine chạy trọn keydown → beforeinput → input.
#
# Quyết định #4: mỗi lượt bơm mang một CHỈ SỐ ĐƠN ĐIỆU TĂNG `⟦n⟧`, và thời điểm bơm ra nó
# được ghi vào nhật ký. Sau `SIGKILL` + mở lại, chỉ số LỚN NHẤT còn sống trong project.db
# tra ngược ra thời điểm bơm ⇒ cửa sổ mất dữ liệu = (mốc kill) − (mốc bơm ra nó).
#
# 🔴 NOI DUNG LA ASCII, VA DO LA MOT GIOI HAN DA DO, KHONG MOT LUA CHON.
# `osascript ... keystroke` di qua bo cuc ban phim hien hanh nen MOI ky tu ngoai ASCII bi be:
# go `⟦42⟧ Trời hôm nay...` thi kho nhan `a42a Trai ham nay...`. Do duoc 2026-08-13.
# ⇒ Viet chuoi mo bang ASCII de NHAT KY KHOP voi thu that su toi noi. Ban truoc ghi chuoi
# tieng Viet vao nhat ky trong khi app nhan mot chuoi khac han va DAI KHAC — tuc chinh phep
# truy nguoc thoi diem bom dung tren mot ban ghi sai.
# ⚠️ Cai gia, ghi ra chu khong nuot: ban do KHONG cham duong IME va KHONG cham ca *xoa lui
# qua dau cau* — ca thung cao nhat theo `deferred-work.md`. Chu: Ice. Xem §Can Ice quyet.
# Do dai cau van thay doi that, va luot xoa van con.
#
# Dùng: type-driver.sh <giây> <tệp-nhật-ký>
set -u
DUR="${1:-1800}"
LOG="${2:-/tmp/at-bench-typing.log}"

# Câu mồi — độ dài thay đổi thật, dấu thật, không một dòng ký tự đồng nhất nào.
PHRASES=(
  "Troi hom nay trong xanh la thuong."
  "Ong lao nhin ra bien, long nang triu nhung dieu chua noi."
  "Nang cuoi."
  "Trong khoanh khac ay, moi tieng dong cua pho phuong nhu lui het ve phia sau lung."
  "Khong ai tra loi."
  "Gio thoi qua rang tre, mang theo mui bun va mui lua non cua canh dong phia dong."
  "Han buoc di cham rai."
  "Dem xuong, anh den vang hat len buc tuong loang lo nhung vet mua cu ky."
)

now() { perl -MTime::HiRes=time -e 'printf "%.3f\n", time' }

: > "$LOG"
echo "# type-driver bắt đầu $(now) · thời lượng ${DUR}s" >> "$LOG"

START=$(now)
n=0
while :; do
  T=$(now)
  ELAPSED=$(perl -e "printf '%.0f', $T - $START")
  [ "$ELAPSED" -ge "$DUR" ] && break

  n=$((n+1))
  P="${PHRASES[$(( (n % ${#PHRASES[@]}) + 1 ))]}"
  TXT="[$n] $P"

  # Mốc bơm ghi TRƯỚC lượt bơm — nó là cận trên an toàn cho "thời điểm gõ ra ký tự đó".
  echo "$n $(now) ${#TXT}" >> "$LOG"

  # Cứ 7 lượt thì có một lượt XOÁ LÙI — ca thủng cao nhất theo dự đoán.
  if [ $((n % 7)) -eq 0 ]; then
    osascript -e "tell application \"System Events\" to keystroke \"$TXT\"" \
              -e 'tell application "System Events" to key code 51' \
              -e 'tell application "System Events" to key code 51' \
              -e 'tell application "System Events" to key code 51' >/dev/null 2>&1
  else
    osascript -e "tell application \"System Events\" to keystroke \"$TXT\"" >/dev/null 2>&1
  fi

  # Nhịp người: 0,6–1,4 s giữa hai câu.
  perl -e 'select(undef,undef,undef, 0.6 + rand(0.8))'
done
echo "# type-driver xong $(now) · $n lượt" >> "$LOG"
echo "$n"
