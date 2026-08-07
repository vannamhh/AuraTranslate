/**
 * Nạp bốn tệp font LÚC CHẠY, từ `$RESOURCE/fonts/**` qua asset protocol. Story 1.4.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * BA ĐƯỜNG KHÔNG ĐI ĐƯỢC — cả ba đều *trông* hợp lý và cả ba đã được đo là hỏng
 * ─────────────────────────────────────────────────────────────────────────────────
 *
 * 1. `@font-face { src: url('./fonts/…') }` trong CSS. Font KHÔNG nằm trong `src/` —
 *    bốn tệp sống ở `src-tauri/resources/fonts/` và đi vào bản cài qua `bundle.resources`
 *    (`tauri.conf.json:35-40`). Một `url()` tương đối sẽ được Vite giải thành asset của
 *    bundle, tức font bị NHÂN BẢN vào `dist/`: cộng thẳng ~26 MiB vào payload trong khi
 *    NFR6 chỉ còn ~47 MB dư địa. Bẫy này *chạy được* trên máy dev, nên nó chỉ lộ ra ở
 *    phép đo dung lượng của Story 1.9.
 *
 * 2. `fetch()` tệp font rồi dựng `FontFace` từ `ArrayBuffer`. CSP hiện tại là
 *    `connect-src 'self' ipc: http://ipc.localhost` — KHÔNG có `asset:`, trong khi
 *    `font-src` thì CÓ (`tauri.conf.json:25`). Đo thật trên bản `.app` debug 2026-08-03:
 *    bốn sự kiện `securitypolicyviolation` nêu đích danh `connect-src`. Ice đã chốt giữ
 *    nguyên CSP. ⇒ `FontFace` với nguồn `url()` chạy, `fetch` không.
 *
 * 3. Bỏ descriptor `weight` cho hai tệp biến thiên. Xem khối ⚠️ ngay dưới.
 *
 * Đường DUY NHẤT đã được chứng minh chạy dưới CSP là `resolveResource()` →
 * `convertFileSrc()` → `new FontFace(family, 'url(…)')` — chính đường mà
 * `src/selftest/scopeCheck.ts:220-242` đo được `LOADED` ở chế độ bundled.
 */
import { convertFileSrc } from '@tauri-apps/api/core'
import { resolveResource } from '@tauri-apps/api/path'

export interface FontLoadResult {
  family: string
  file: string
  status: 'loaded' | 'failed'
  detail: string
}

interface FontSpec {
  family: string
  file: string
  descriptors: FontFaceDescriptors
  why: string
}

/**
 * ⚠️ `weight: '200 900'` là BẮT BUỘC cho hai tệp biến thiên, không phải trang trí.
 *
 * `ARCHITECTURE-SPINE.md §Stack` ghi thẳng: `SourceSans3[wght].ttf` có
 * `name ID 1 = Source Sans 3 ExtraLight` vì **mặc định trục `wght` là 200**. Thiếu
 * descriptor thì trình duyệt coi cả tệp là một nét 200 duy nhất, và token `ui-label`
 * (700) hoặc ra chữ mảnh, hoặc bị TỔNG HỢP NÉT ĐẬM GIẢ. Ở cỡ 10px với
 * `letter-spacing: 0.1em`, nét giả trông *gần đúng* — đủ gần để không ai nhận ra trong
 * sáu tháng. `scopeCheck.ts:165,223` đã dùng đúng descriptor này; chép lại, đừng phát minh.
 */
const FONTS: readonly FontSpec[] = [
  {
    family: 'Source Serif 4',
    file: 'fonts/SourceSerif4[opsz,wght].ttf',
    descriptors: { weight: '200 900', style: 'normal', display: 'swap' },
    why: 'Họ `read` — chữ nội dung. Nét 600 của `read-title` nằm trong dải, là nét THẬT.',
  },
  {
    family: 'Source Serif 4',
    file: 'fonts/SourceSerif4-Italic[opsz,wght].ttf',
    // Cùng `family`, khác `style` — đó là cách khai một họ có hai tệp. Khai thành hai
    // họ riêng ("Source Serif 4 Italic") thì `font-style: italic` của `source-hanviet`
    // và `lookup-example` không tìm thấy nó, và trình duyệt nghiêng giả tệp Regular.
    descriptors: { weight: '200 900', style: 'italic', display: 'swap' },
    why: 'Nghiêng THẬT cho phần Latin của `source-hanviet` và `lookup-example`.',
  },
  {
    family: 'Source Sans 3',
    file: 'fonts/SourceSans3[wght].ttf',
    descriptors: { weight: '200 900', style: 'normal', display: 'swap' },
    why: 'Họ `ui` — chữ bộ máy. `ui-label` khai 700; dải 200–900 phủ trọn.',
  },
  {
    family: 'Noto Serif CJK TC',
    file: 'fonts/NotoSerifCJKtc-Regular.otf',
    // ⚠️ CHỈ Regular, có chủ ý: bản nghiêng CJK là ~23 MiB — một phần ba ngân sách font,
    // và dư địa NFR6 chỉ còn ~47 MB. Hệ quả là chữ Hán dưới một token nghiêng sẽ bị
    // trình duyệt nghiêng giả; lời giải KHÔNG phải thêm tệp, mà là
    // `font-synthesis: none` khai ở chính hai token đó. Xem `src/tokens/README.md`.
    descriptors: { weight: '400', style: 'normal', display: 'swap' },
    why: 'Họ `read-cjk`, và là chặng dự phòng CJK của họ `read`.',
  },
]

/**
 * Hạn giờ cho MỘT tệp. Bốn tệp là ~26 MiB đọc từ đĩa qua asset protocol; một máy chậm
 * vẫn xong dưới vài giây. Con số này không phải để tối ưu — nó tồn tại vì một promise
 * treo và một promise thành công trông GIỐNG HỆT NHAU trong log.
 */
const LOAD_TIMEOUT_MS = 20_000

const withTimeout = <T>(p: Promise<T>, ms: number, what: string): Promise<T> =>
  new Promise<T>((resolve, reject) => {
    const t = setTimeout(() => reject(new Error(`quá ${ms} ms mà ${what} chưa xong (treo, không phải lỗi)`)), ms)
    p.then(
      (v) => {
        clearTimeout(t)
        resolve(v)
      },
      (e) => {
        clearTimeout(t)
        reject(e)
      },
    )
  })

/**
 * ⚠️ Luỹ đẳng. `document.fonts.add()` KHÔNG khử trùng lặp: gọi `loadFonts()` lần thứ hai
 * — Vite HMR chạy lại `main.ts`, hay một lượt khởi tạo lại trong tương lai — nhét thêm
 * bốn `FontFace` cùng family/style vào `document.fonts`. Chốt ở tầng module, và trả lại
 * đúng promise cũ để nơi gọi thứ hai vẫn đọc được kết quả.
 */
let inflight: Promise<FontLoadResult[]> | null = null

/**
 * Đăng ký cả bốn tệp. KHÔNG ném: một tệp font thiếu phải làm chữ rơi về font hệ thống,
 * không làm trắng cửa sổ. Kết quả trả về để nơi gọi (và mũi thăm dò thị giác của Task 9)
 * còn đọc được cái gì đã nạp, cái gì không.
 *
 * ⚠️ SONG SONG, không nối tiếp. Bản trước là `for … await`: nếu tệp đầu **treo** thay vì
 * reject thì ba tệp sau không bao giờ được dựng, promise trả về không bao giờ settle, và
 * khối `.then` ở `main.ts` không chạy — một lượt nạp treo không phân biệt được với một
 * lượt nạp thành công. `allSettled` + hạn giờ cho từng tệp làm mỗi tệp tự chịu trách
 * nhiệm cho chính nó.
 */
export function loadFonts(): Promise<FontLoadResult[]> {
  if (inflight) return inflight

  inflight = Promise.allSettled(
    FONTS.map(async (spec): Promise<FontLoadResult> => {
      try {
        const path = await withTimeout(resolveResource(spec.file), LOAD_TIMEOUT_MS, `resolveResource(${spec.file})`)
        const url = convertFileSrc(path)
        const face = new FontFace(spec.family, `url("${url}")`, spec.descriptors)
        await withTimeout(face.load(), LOAD_TIMEOUT_MS, `nạp ${spec.file}`)
        document.fonts.add(face)
        return {
          family: spec.family,
          file: spec.file,
          status: 'loaded',
          detail: `${spec.descriptors.style} ${spec.descriptors.weight} — ${url}`,
        }
      } catch (err) {
        // ⚠️ `FontFace` trả CÙNG một `NetworkError` cho "scope chặn", cho "tệp có thật
        // nhưng không phải font", và cho "404" — đo thật ở Story 1.3. Nên chẩn đoán ở đây
        // phải liệt kê cả ba khả năng thay vì đoán một cái.
        return {
          family: spec.family,
          file: spec.file,
          status: 'failed',
          detail:
            `${String(err)} — ba nguyên nhân có thể và FontFace KHÔNG phân biệt được: ` +
            '`assetProtocol.scope` chặn · tệp thiếu trong `bundle.resources` · `font-src` ' +
            'không còn cho `asset:`. Kiểm `bundle.resources` trước.',
        }
      }
    }),
  ).then((settled) =>
    settled.map((s, i) =>
      s.status === 'fulfilled'
        ? s.value
        : {
            family: FONTS[i].family,
            file: FONTS[i].file,
            status: 'failed' as const,
            detail: `lỗi ngoài dự kiến trong đường nạp: ${String(s.reason)}`,
          },
    ),
  )

  return inflight
}

export { FONTS }
