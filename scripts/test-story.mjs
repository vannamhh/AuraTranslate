#!/usr/bin/env node
/**
 * VÒNG LẶP DEV CÓ PHẠM VI — chạy đúng những bộ test mà một story chạm tới.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO CÓ TỆP NÀY — SỐ ĐO, KHÔNG PHẢI CẢM GIÁC (đo 2026-09-12, máy Ice 16 nhân,
 *    cây sạch, cache ấm)
 * ═════════════════════════════════════════════════════════════════════════════════
 * Điều kiện đo GIỐNG NHAU cho cả ba lượt: `touch src-tauri/src/lib.rs` (một sửa chạm
 * crate gốc — ca thường của một story), cache đã ổn định, cây đứng yên:
 *
 *   đủ bộ   `cargo test --locked` + `npx vitest run`     140,4 s
 *   10 target Rust + 2 tệp frontend (phạm vi THẬT của 6.17)  37,4 s
 *    3 target Rust + 2 tệp frontend (spec 6.17 khai)         21,3 s
 *
 * Gấp **3,7 lần** ở phạm vi thật. (Không phải 14 lần — con số đó đến từ một lượt đo
 * so 1 target với 52 target, hai thứ không cùng công việc. Nó đã bị gỡ.)
 *
 * ⚠️ Hai cái bẫy đo, ghi ra vì chúng đã bẫy chính lượt dựng tệp này:
 *   - `cargo test --no-run` trên cây ĐỨNG YÊN tốn **0,58 s**, không phải 21 s. Con số
 *     21 s đo được ngay sau một lượt `cargo clean` của phiên trước, lúc cache còn đang
 *     dựng lại — nó đo cái máy, không đo cái lệnh.
 *   - Cùng lệnh A chạy hai lần cho 66,6 s rồi 21,3 s, chỉ vì lượt đầu nối tiếp ngay sau
 *     một lượt xây khác. Hai lượt đo phải cùng trạng thái cache, nếu không thì đừng so.
 *
 * ⚠️ CÁI NÀY KHÔNG THAY CỔNG. `pre-push` giữ nguyên phạm vi đầy đủ, vì chạy hẹp không
 * xoá rủi ro hồi quy ở module hàng xóm — nó DỜI rủi ro đó tới lúc push. Đổi lại: nhiều
 * lượt rẻ trong lúc code, một lượt đầy đủ trước khi đẩy. Ai dùng tệp này để xin bớt
 * `pre-push` là đang đọc ngược nó.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 DIFF LÀ NGUỒN CHÍNH. SPEC LÀ LƯỚI BẮT CÁI DIFF CHƯA TỚI. ĐỪNG ĐẢO HAI VAI NÀY.
 * ═════════════════════════════════════════════════════════════════════════════════
 * Bản đầu của tệp này lấy §Code Map → **Tests that move** của spec làm nguồn chính và
 * dùng diff để đối chứng. Một phép đo lật ngược nó ngay trong lượt dựng:
 *
 *   Story 6.17 (`3c25e9a`) — spec khai **3** tệp test; commit chạm **10**.
 *   Bảy tệp KHÔNG khai: asset · chapter_origin · cleanup · docx · project ·
 *   segment_role · webimport (`_contract.rs`). Tỉ lệ trượt **70%**, trên story
 *   gần nhất, viết bởi một agent chạy 512 lượt gọi công cụ.
 *
 * Lý do trượt có cấu trúc, không phải cẩu thả: spec viết **trước** khi code, nên nó
 * không thể biết một đổi chữ ký sẽ kéo theo bao nhiêu lời gọi định vị-theo-thứ-tự ở
 * các tệp test khác. Diff thì không đoán — nó là dấu vết của việc ĐÃ làm.
 *
 * Nên: **diff của cây làm việc là nguồn chính** (chính xác tuyệt đối cho cái đã sửa),
 * **spec là lưới** bắt tệp test story SẼ phải sửa mà chưa sửa. Lệnh này chạy HỢP của
 * hai tập — thà chạy thừa một target 0,8 giây còn hơn bỏ sót một hồi quy.
 *
 * ⚠️ Nếu diff chạm một tệp test mà spec không nêu, lệnh báo **THIẾU KHAI**. Đó là
 * tín hiệu đi sửa §Code Map của spec, không phải lỗi của lượt chạy. `--strict` biến
 * nó thành mã thoát 2 cho ai muốn cưỡng chế.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * Dùng:
 *   node scripts/test-story.mjs 6.17            chạy phần Rust + phần frontend của story
 *   node scripts/test-story.mjs 6.17 --list     chỉ in ra sẽ chạy gì, không chạy
 *   node scripts/test-story.mjs --diff          suy ra từ diff, không cần spec
 *   node scripts/test-story.mjs 6.17 --rust     chỉ Rust
 *   node scripts/test-story.mjs 6.17 --front    chỉ frontend
 *   node scripts/test-story.mjs 6.17 --strict   THIẾU KHAI là lỗi (mã thoát 2)
 *
 * `--diff <ref>` đổi mốc so sánh (mặc định `HEAD`, tức gồm cả thay đổi chưa commit).
 */

import { execFileSync, spawnSync } from 'node:child_process';
import { readdirSync, readFileSync, existsSync } from 'node:fs';
import { basename, join } from 'node:path';

const ROOT = new URL('..', import.meta.url).pathname.replace(/\/$/, '');
const RUST_TEST_DIR = join(ROOT, 'src-tauri', 'tests');
const FRONT_TEST_DIR = join(ROOT, 'tests', 'frontend');
const SPEC_DIR = join(ROOT, '_bmad-output', 'implementation-artifacts');

// ── quần thể thật: đừng đoán, hãy đếm ────────────────────────────────────────────
const rustTargets = readdirSync(RUST_TEST_DIR)
  .filter((f) => f.endsWith('.rs'))
  .map((f) => basename(f, '.rs'));
const frontTests = readdirSync(FRONT_TEST_DIR)
  .filter((f) => f.endsWith('.test.ts'))
  .map((f) => `tests/frontend/${f}`);

// ── tham số ──────────────────────────────────────────────────────────────────────
const argv = process.argv.slice(2);
const flag = (n) => argv.includes(`--${n}`);
const flagVal = (n, dflt) => {
  const i = argv.indexOf(`--${n}`);
  return i >= 0 && argv[i + 1] && !argv[i + 1].startsWith('--') ? argv[i + 1] : dflt;
};
const storyArg = argv.find((a) => !a.startsWith('--') && a !== flagVal('diff', null));

const onlyRust = flag('rust');
const onlyFront = flag('front');
const listOnly = flag('list');
const strict = flag('strict');
const diffRef = flagVal('diff', 'HEAD');

// ── ① lời khai của spec ──────────────────────────────────────────────────────────
function findSpec(id) {
  const slug = id.replace(/\./g, '-');
  const hit = readdirSync(SPEC_DIR).find(
    (f) => f.startsWith(`spec-${slug}-`) || f === `spec-${slug}.md`,
  );
  return hit ? join(SPEC_DIR, hit) : null;
}

function declaredFrom(specPath) {
  const text = readFileSync(specPath, 'utf8');
  const rust = new Set();
  const front = new Set();
  // Bắt cả `src-tauri/tests/x.rs`, `tests/x.rs` lẫn `x_contract.rs` đứng trần trong văn xuôi —
  // rồi LỌC lại theo quần thể thật ở trên, nên một tên bịa không lọt qua được.
  for (const m of text.matchAll(/([A-Za-z0-9_]+)\.rs\b/g)) {
    if (rustTargets.includes(m[1])) rust.add(m[1]);
  }
  for (const m of text.matchAll(/tests\/frontend\/([A-Za-z0-9_.-]+\.test\.ts)\b/g)) {
    const p = `tests/frontend/${m[1]}`;
    if (frontTests.includes(p)) front.add(p);
  }
  return { rust, front };
}

// ── ② đối chứng: diff thật sự chạm tệp test nào ──────────────────────────────────
function touchedFrom(ref) {
  let out = '';
  try {
    out = execFileSync('git', ['diff', '--name-only', ref], { cwd: ROOT, encoding: 'utf8' });
    out += execFileSync('git', ['ls-files', '--others', '--exclude-standard'], {
      cwd: ROOT,
      encoding: 'utf8',
    });
  } catch {
    return { rust: new Set(), front: new Set(), ok: false };
  }
  const rust = new Set();
  const front = new Set();
  for (const line of out.split('\n')) {
    const f = line.trim();
    if (!f) continue;
    const mr = f.match(/^src-tauri\/tests\/([A-Za-z0-9_]+)\.rs$/);
    if (mr && rustTargets.includes(mr[1])) rust.add(mr[1]);
    if (frontTests.includes(f)) front.add(f);
  }
  return { rust, front, ok: true };
}

// ── gom lại ──────────────────────────────────────────────────────────────────────
let declared = { rust: new Set(), front: new Set() };
let specPath = null;

if (storyArg) {
  specPath = findSpec(storyArg);
  if (!specPath) {
    console.error(`KHÔNG tìm thấy spec cho story "${storyArg}" trong ${SPEC_DIR}`);
    process.exit(1);
  }
  declared = declaredFrom(specPath);
}

const touched = touchedFrom(diffRef);

const undeclaredRust = [...touched.rust].filter((t) => !declared.rust.has(t));
const undeclaredFront = [...touched.front].filter((t) => !declared.front.has(t));

const runRust = [...new Set([...declared.rust, ...touched.rust])].sort();
const runFront = [...new Set([...declared.front, ...touched.front])].sort();

// ── báo cáo ──────────────────────────────────────────────────────────────────────
console.log('─'.repeat(78));
if (specPath) console.log(`spec       : ${specPath.slice(ROOT.length + 1)}`);
console.log(`mốc diff   : ${diffRef}${touched.ok ? '' : '  (git không đọc được — bỏ qua đối chứng)'}`);
console.log(
  `quần thể   : ${rustTargets.length} target Rust · ${frontTests.length} tệp frontend`,
);
console.log('─'.repeat(78));
console.log(`Rust     (${runRust.length}/${rustTargets.length}) : ${runRust.join(' ') || '(không có)'}`);
console.log(
  `Frontend (${runFront.length}/${frontTests.length}) : ${
    runFront.map((f) => basename(f)).join(' ') || '(không có)'
  }`,
);

if (undeclaredRust.length || undeclaredFront.length) {
  console.log('');
  console.log('⚠️ THIẾU KHAI — diff chạm những tệp test này mà spec không nêu:');
  for (const t of undeclaredRust) console.log(`     src-tauri/tests/${t}.rs`);
  for (const t of undeclaredFront) console.log(`     ${t}`);
  console.log('   Bổ sung chúng vào §Code Map → "Tests that move" của spec.');
}

if (!runRust.length && !runFront.length) {
  console.log('');
  console.log('Không suy ra được bộ test nào. Chạy đủ bộ: npm run test && cargo test');
  process.exit(1);
}

if (listOnly) process.exit(0);
if (strict && (undeclaredRust.length || undeclaredFront.length)) {
  console.error('\n--strict: THIẾU KHAI là lỗi.');
  process.exit(2);
}

// ── chạy ─────────────────────────────────────────────────────────────────────────
let rc = 0;
const t0 = Date.now();

if (!onlyFront && runRust.length) {
  const args = ['test', '--locked', ...runRust.flatMap((t) => ['--test', t])];
  console.log(`\n$ cargo ${args.join(' ')}`);
  const r = spawnSync('cargo', args, { cwd: join(ROOT, 'src-tauri'), stdio: 'inherit' });
  if (r.status !== 0) rc = 1;
}

if (!onlyRust && runFront.length) {
  console.log(`\n$ npx vitest run ${runFront.join(' ')}`);
  const r = spawnSync('npx', ['vitest', 'run', ...runFront], { cwd: ROOT, stdio: 'inherit' });
  if (r.status !== 0) rc = 1;
}

console.log(`\n── xong trong ${((Date.now() - t0) / 1000).toFixed(1)}s · mã thoát ${rc}`);
console.log('   Đây là vòng lặp dev, KHÔNG phải cổng. Trước khi push: git push (pre-push chạy đủ).');
process.exit(rc);
