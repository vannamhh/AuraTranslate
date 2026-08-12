//! Crate root của AuraTranslate.
//!
//! Bố cục `lib.rs` + `main.rs` là quy ước của chính framework Tauri v2, và nó là
//! điều kiện để `tests/` `use auratranslate_lib::…` được. Đừng viết test vào `main.rs`.
//!
//! ⚠️ `mod core` là module miền của dự án, KHÔNG phải crate `core` của Rust.
//! Trong crate này luôn viết `crate::core::…`; đừng viết `use core::…` — nó nhập
//! nhằng với crate chuẩn.

pub mod commands;
pub mod core;
pub mod ports;

/// Tên biến môi trường bật Kiểm 3 của Story 1.2 (phạm vi asset protocol).
///
/// Bật thì ứng dụng chạy self-check trong webview, in kết quả ra stdout rồi **thoát
/// với mã 0/1**. Đó là thứ Story 1.3 gắn vào pipeline được; một phép kiểm chỉ hiện
/// kết quả trên màn hình thì không cưỡng chế được gì.
///
/// ⚠️ Móc này **chỉ tồn tại trong bản debug** (`#[cfg(debug_assertions)]`). Nó đóng
/// toàn bộ cửa sổ rồi thoát tiến trình — một móc như vậy không có việc gì trong bản
/// phát hành, và mã self-check phía frontend cũng không vào bundle release
/// (`App.vue` chỉ `import()` động khi `VITE_SCOPE_SELFTEST=1`). Hai đầu phải đối xứng.
pub const SCOPE_SELFTEST_ENV: &str = "AURA_SCOPE_SELFTEST";

/// Tên event mà self-check của frontend phát về. Khớp với `src/selftest/scopeCheck.ts`.
pub const SCOPE_SELFTEST_EVENT: &str = "selftest:scope-check";

/// Tên tệp kho toàn cục dưới `$APPDATA`. Xem [`open_global_store`].
const GLOBAL_DB_FILE: &str = "global.db";

/// Tên biến môi trường chỉ `$APPDATA` của tiến trình này sang một thư mục khác — **chỉ e2e**.
///
/// 🔴 Giá trị là **chính thư mục dữ liệu**, không phải thư mục cha của nó:
/// `app_data_dir()` của Tauri là `data_dir()/<định danh bundle>`, còn biến này **thay thế
/// trọn** kết quả đó. Nên `global.db` nằm thẳng trong giá trị được đặt. Bản đầu của phép
/// tự kiểm ở `e2e/wdio.conf.mjs` nối thêm định danh vào và ĐỎ ngay lượt chạy thật đầu
/// tiên, dù móc hoạt động đúng — ghi ra đây để lượt sau không mất một vòng vì cùng lý do.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// VÌ SAO MÓC NÀY TỒN TẠI
/// ─────────────────────────────────────────────────────────────────────────────
/// Bộ lái e2e dựng một cửa sổ **thật**, nên nó đọc và ghi đúng `$APPDATA` mà ứng dụng
/// thật của người chạy dùng. Story 1.21 ghi phím tắt xuống `global.db`
/// (`ScopeKind::Shortcut`), nên một ca gán phím **sửa cấu hình thật của người chạy**. Đo
/// được ở lượt dựng: một lượt chẩn đoán để lại `⌥⌘K` trên `layout.toggle_source`, và
/// lượt sau đọc nó thành trạng thái đầu rồi ĐỎ với một câu đổ lỗi cho sản phẩm — tức một
/// bộ đo tự làm hỏng phép đo của chính nó.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO MỘT BIẾN MÔI TRƯỜNG ĐỌC TRONG RUST, CHỨ KHÔNG PHẢI CHỈ `HOME`
/// ─────────────────────────────────────────────────────────────────────────────
/// Cách rẻ hơn là đổi `HOME` của tiến trình con. Đo trên chính cây phụ thuộc đang ghim
/// *(2026-08-11)*, và nó **chỉ đúng một nửa**:
///
/// | Nền tảng | `dirs::data_dir()` đi qua | Đổi được bằng biến môi trường? |
/// |---|---|---|
/// | macOS   | `home_dir()/Library/Application Support`, `home_dir()` đọc `$HOME` trước | **CÓ** |
/// | Windows | `known_folder(FOLDERID_RoamingAppData)` — Known Folder API | **KHÔNG** |
///
/// `dirs-sys-0.5.0` gọi thẳng Shell API trên Windows và **bỏ qua** `%APPDATA%`. Nên một
/// bản vá chỉ đổi `HOME` là một bản vá **chạy trên macOS và hỏng im lặng trên Windows** —
/// đúng lớp lệch nền tảng NFR14 tồn tại để chặn, và hôm nay nửa Windows không có đường
/// nghiệm thu nào để phát hiện ra. Móc này phân giải **giống hệt nhau** trên hai nền tảng.
///
/// ⚠️ Chặn bằng ĐÚNG HAI LỚP của AD-45, cùng khuôn với chính plugin WebDriver:
/// `debug_assertions` **và** `feature = "wdio"`. Bản phát hành không có một dòng mã nào
/// đọc biến này — [`data_dir_override`] ở nhánh `not(...)` trả thẳng `None`. Một móc chỉ
/// hiển thị trong bản debug **và** chỉ khi bật feature là một móc không phải là bề mặt
/// tấn công; một móc chỉ có `debug_assertions` thì có.
#[cfg(all(debug_assertions, feature = "wdio"))]
pub const E2E_DATA_DIR_ENV: &str = "AURATRANSLATE_E2E_DATA_DIR";

/// Tên biến môi trường chỉ **thư mục gốc Library** sang một thư mục khác — **chỉ e2e**.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO CẦN BIẾN THỨ HAI — `$APPDATA` KHÔNG phải bề mặt dữ liệu thật duy nhất
/// ─────────────────────────────────────────────────────────────────────────────
/// Phát hiện 2026-08-11 (Story 1.22, C2): [`E2E_DATA_DIR_ENV`] đóng đúng **một** đường ghi.
/// Thư mục gốc Library đi một đường **hoàn toàn khác** — `app.path().document_dir()` ⇒
/// `~/Documents/AuraTranslate/`, phân giải ở `commands::project::default_library_root`
/// (AD-23, scope động). Nên một bàn đo tạo Tác phẩm sẽ ghi vào thư mục **Documents THẬT**
/// của người chạy, tức tái lập nguyên vẹn lớp lỗi mà [`E2E_DATA_DIR_ENV`] vừa đóng, chỉ ở
/// một thư mục khác.
///
/// ⚠️ Đây là lý do biến này ra đời **TRƯỚC** fixture đầu tiên tạo Tác phẩm, không sau: bề
/// mặt thứ hai tìm ra bằng cách đọc mã, không bằng cách mất dữ liệu một lần nữa.
///
/// Hai biến chứ không một, vì hai kho là **hai vai khác nhau** trong AD-7 — `global.db` là
/// nguồn sự thật của tầng Global, `.atproj/` là nguồn sự thật của một Tác phẩm — và ranh
/// giới sở hữu ở đó là **cứng**. Gộp chúng vào một biến là dạy người đọc rằng chúng cùng
/// một chỗ.
///
/// Chặn bằng ĐÚNG HAI LỚP của AD-45, giống hệt [`E2E_DATA_DIR_ENV`].
#[cfg(all(debug_assertions, feature = "wdio"))]
pub const E2E_LIBRARY_ROOT_ENV: &str = "AURATRANSLATE_E2E_LIBRARY_ROOT";

/// Chính sách CHUNG của hai móc e2e, tách khỏi phép **đọc** biến môi trường.
///
/// Tách ra vì hai lý do đo được, không phải vì gu kiến trúc:
/// 1. `std::env::set_var` là `unsafe` từ edition 2024, và một ca test đặt biến môi trường
///    còn đua với mọi ca chạy song song trong cùng nhị phân. Hàm thuần thì test được mà
///    không chạm tiến trình.
/// 2. Nó **không** bị `cfg` gác, nên chính sách được kiểm ở **mọi** bộ feature — kể cả bộ
///    mặc định mà `cargo test` của hook pre-push chạy. Thứ bị gác là phép đọc, không phải
///    luật.
///
/// 🔴 **Đường dẫn tương đối bị TỪ CHỐI, không phải được phân giải.** Một đường tương đối
/// phân giải theo thư mục làm việc của tiến trình con — thứ bộ lái đặt, không thứ người
/// viết ca test nghĩ — nên nó sẽ đẻ một `global.db` ở một chỗ bất kỳ trong kho mà không
/// ai báo. Rỗng cũng bị từ chối: một biến đặt thành chuỗi rỗng là một lượt đặt hỏng, và
/// coi nó như *"không đặt"* là im lặng nuốt một lỗi cấu hình.
pub fn absolute_dir_override_from_raw(raw: Option<&std::ffi::OsStr>) -> Option<std::path::PathBuf> {
    let raw = raw?;
    if raw.is_empty() {
        return None;
    }
    let path = std::path::PathBuf::from(raw);
    if path.is_absolute() { Some(path) } else { None }
}

/// Đọc [`E2E_DATA_DIR_ENV`] rồi áp [`absolute_dir_override_from_raw`].
///
/// Nhánh `not(...)` trả `None` **theo kiểu**, nên bản phát hành không có đường mã nào
/// chạm biến môi trường đó.
#[cfg(all(debug_assertions, feature = "wdio"))]
fn data_dir_override() -> Option<std::path::PathBuf> {
    absolute_dir_override_from_raw(std::env::var_os(E2E_DATA_DIR_ENV).as_deref())
}

#[cfg(not(all(debug_assertions, feature = "wdio")))]
fn data_dir_override() -> Option<std::path::PathBuf> {
    None
}

/// Đọc [`E2E_LIBRARY_ROOT_ENV`] rồi áp [`absolute_dir_override_from_raw`].
///
/// `pub(crate)` vì chỗ dùng là `commands::project::default_library_root` — mọi phép **đọc**
/// biến môi trường của bộ e2e cố ý sống chung ở tệp này, sau cùng một cặp `cfg`. Rải chúng
/// ra từng module là làm bất biến *"chỉ tồn tại sau hai lớp gác"* thành một thứ phải kiểm
/// ở N chỗ.
#[cfg(all(debug_assertions, feature = "wdio"))]
pub(crate) fn library_root_override() -> Option<std::path::PathBuf> {
    absolute_dir_override_from_raw(std::env::var_os(E2E_LIBRARY_ROOT_ENV).as_deref())
}

#[cfg(not(all(debug_assertions, feature = "wdio")))]
pub(crate) fn library_root_override() -> Option<std::path::PathBuf> {
    None
}

/// Nhãn cửa sổ duy nhất — khớp `tauri.conf.json::windows[0].label`.
const MAIN_WINDOW_LABEL: &str = "main";

/// Tên event forward-tới-JS khi người dùng kéo-thả tệp vào cửa sổ — Story 1.15, Quyết
/// định #1(b). Khớp `src/modes/libraryImport.ts::DRAG_DROP_EVENT`.
pub const DRAG_DROP_EVENT: &str = "aura://file-dropped";

/// Một thao tác kéo vừa **vào** cửa sổ. Khớp `src/modes/libraryImport.ts`.
///
/// 🔴 Vì sao cần một event RIÊNG cho việc này, thay vì để webview tự bắt `dragenter` của
/// DOM: `drag_drop_enabled` mặc định `true` ở Tauri v2 (`tauri.conf.json` không override),
/// nghĩa là bộ xử lý kéo-thả tầng **hệ điều hành** giành lấy thao tác và webview **không
/// bao giờ** thấy `dragenter`/`dragover`/`dragleave` của DOM. Vùng kéo-thả vì thế không
/// có một tín hiệu trực quan nào — người dùng không biết vùng đó còn sống. Lỗi tìm ra ở
/// lượt code review 2026-08-06.
///
/// ⚠️ Đi cùng đường với [`DRAG_DROP_EVENT`] — cùng `on_window_event`, **0** permission
/// mới, 0 phụ thuộc mới.
pub const DRAG_ENTER_EVENT: &str = "aura://file-drag-enter";

/// Thao tác kéo đã **rời** cửa sổ hoặc bị huỷ. Khớp `src/modes/libraryImport.ts`.
pub const DRAG_LEAVE_EVENT: &str = "aura://file-drag-leave";

/// Thư mục con của `$RESOURCE` chứa các tệp `.db` từ điển. Xem [`open_dict_layers`].
///
/// 🔴 Đây là một **THƯ MỤC**, không phải một danh sách tên tệp — và khác biệt đó là cả
/// FR36: tập lớp là **mọi** tệp `*.db` tìm thấy trong nó, nên *"gỡ một lớp = xoá một file"*
/// đúng theo nghĩa đen. `tests/dict_boundary.rs::the_layer_set_never_hardcodes_a_db_filename`
/// canh vế đó bằng máy.
const DICT_RESOURCE_DIR: &str = "dict";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let selftest = std::env::var(SCOPE_SELFTEST_ENV).as_deref() == Ok("1");

    // ─────────────────────────────────────────────────────────────────────────────
    // Bộ lái cửa sổ THẬT — Ice chốt 2026-08-11
    // ─────────────────────────────────────────────────────────────────────────────
    // Máy chủ W3C WebDriver nhúng, chạy TRONG webview của sản phẩm. Nó tồn tại để bàn
    // đo chạy tay của Epic 1 có một đường nghiệm thu bằng máy — và quan trọng hơn, để
    // lớp lỗi ĐẶC THÙ ENGINE nghiệm thu được: khuyết tật hạng cao nhất của lượt code
    // review Story 1.21 là *"WKWebView không đặt tiêu điểm cho `<button>`"*, và không bộ
    // chạy nào trong Chrome tái lập nổi nó.
    //
    // 🔴 HAI lớp gác, và cần cả hai — chúng gác hai thứ khác nhau:
    //   1. `feature = "wdio"` (KHÔNG trong `default`) giữ `axum`/`tokio` khỏi **cây phụ
    //      thuộc**, tức khỏi nhị phân phát hành. `check-deps.mjs` Kiểm 1b canh vế này.
    //   2. `debug_assertions` giữ nó khỏi **bản release** kể cả khi ai đó bật feature.
    // Bỏ lớp 1 mà giữ lớp 2 vẫn nhét một máy chủ HTTP vào cây của bản phát hành; bỏ lớp
    // 2 mà giữ lớp 1 thì một lượt `cargo build --release --features wdio` mở cổng 4445
    // trên máy người dùng. Cùng khuôn đối xứng mà `SCOPE_SELFTEST_ENV` đã dựng ở Story
    // 1.2 — hai đầu phải khớp.
    //
    // ⚠️ Plugin khai `permissions = []` và phơi **0** invoke command (đo trên
    // `permissions/default.toml` của chính crate 1.3.0), nên nó KHÔNG cần một mục nào
    // trong `capabilities/`. Hai bất biến của `tests/config_invariants.rs` —
    // `capabilities_directory_holds_exactly_the_one_reviewed_file` và
    // `main_capability_grants_the_minimum_and_no_plugin_permission` — giữ nguyên nghĩa
    // gốc, không bị nới một chữ nào.
    #[cfg(all(debug_assertions, feature = "wdio"))]
    let builder = tauri::Builder::default().plugin(tauri_plugin_wdio_webdriver::init());
    #[cfg(not(all(debug_assertions, feature = "wdio")))]
    let builder = tauri::Builder::default();

    let builder = builder
        // ─────────────────────────────────────────────────────────────────────────
        // 🔴 BỀ MẶT IPC ĐẦU TIÊN CỦA DỰ ÁN — Story 1.8
        // ─────────────────────────────────────────────────────────────────────────
        // ⚠️ **Không** thêm mục ACL vào `capabilities/main.json` cho hai command này.
        // Trong Tauri v2, command do **chính ứng dụng** khai không cần quyền — ACL canh
        // command của **plugin**. `tests/config_invariants.rs:333` khoá tệp đó ở đúng ba
        // quyền, và nới nó ra là nới đúng thứ AD-23 tồn tại để siết.
        //
        // ⚠️ Tên trên dây là tên hàm, nên đường dẫn trỏ vào module `wire` — xem
        // `commands/config.rs`.
        .invoke_handler(tauri::generate_handler![
            crate::commands::config::wire::bootstrap_config,
            crate::commands::config::wire::put_config,
            crate::commands::config::wire::delete_config,
            crate::commands::project::wire::create_work_from_text,
            crate::commands::project::wire::create_work_from_file,
            crate::commands::chapter::wire::read_open_chapter,
            crate::commands::dict::wire::read_han_viet,
            crate::commands::dict::wire::lookup_dictionary,
            crate::commands::dict::wire::list_dict_sources,
            // Story 1.20 — muc da ghim, pham vi TOAN UNG DUNG (`global.db`, Ice ky lai
            // 2026-08-11): khong ton tai duong mo lai mot `.atproj`, nen mot bo ghim o
            // `project.db` khong co duong nao de doc toi sau khi dong app.
            crate::commands::pinned::wire::list_pinned_entries,
            crate::commands::pinned::wire::pin_entry,
            crate::commands::pinned::wire::unpin_entry,
            // Story 2.1 — tach segment TUONG MINH cho Chuong da co tren dia
            // (`segment_count = 0` cua moi Chuong Epic 1). Chuong MOI nhap duoc tach tu
            // dong trong `create_work`, cung giao dich; lenh nay chi phuc vu duong cu.
            crate::commands::segment::wire::split_chapter_into_segments,
            // Story 2.2 — nap segment cua Chuong dang mo len Panel Editor. Doc theo
            // `(chapter_id, ord)`, tuc dung index `idx_segment_chapter_ord` cua buoc 5.
            crate::commands::segment::wire::read_open_chapter_segments,
        ])
        .setup(move |app| {
            #[cfg(debug_assertions)]
            if selftest {
                wire_scope_selftest(app.handle());
            }

            open_global_store(app);
            open_dict_layers(app);
            open_work_slot(app);
            wire_drag_drop(app);
            Ok(())
        });

    // ⚠️ `build(ctx)?.run(callback)` thay cho `run(ctx)`, và phép đổi này KHÔNG có tác
    // dụng phụ nào: `Builder::run` trong `tauri-2.11.5/src/app.rs:2449-2452` **chính là**
    // `self.build(context)?.run(|_, _| {})`. Thứ duy nhất thêm vào là callback dưới đây.
    let app = tauri::Builder::build(builder, tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|handle, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            close_global_store(handle);
            close_dict_layers(handle);
            close_open_work(handle);
        }
    });
}

/// Mở `$APPDATA/global.db` và đưa nó vào state của ứng dụng (AD-11, AC3).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// VÌ SAO MỞ Ở `setup()` CHỨ KHÔNG PHẢI LÚC DÙNG LẦN ĐẦU
/// ─────────────────────────────────────────────────────────────────────────────
/// AC3 nói *"`global.db` **khi khởi tạo**"*. Và mở sớm làm một `$APPDATA` không ghi được
/// lộ ra **ngay lúc khởi động** thay vì lúc người dùng đang gõ dở một câu — đó là toàn
/// bộ khác biệt giữa một thông báo khó chịu và một đoạn công sức đã mất.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỞ KHO TRƯỢT ⇒ GHI CHẨN ĐOÁN RÕ RỒI **ĐI TIẾP**, KHÔNG CHẶN KHỞI ĐỘNG
/// ─────────────────────────────────────────────────────────────────────────────
/// Hai lý do, và lý do thứ hai là thứ sẽ đỏ ngay hôm nay nếu làm khác:
/// 1. ⚠️ **Cập nhật Story 1.8 — lý do này nay đã ĐỔI, và mệnh lệnh thì không.** Lúc viết,
///    chưa có bề mặt nào để **nói** với người dùng, nên một `return Err(…)` ở đây làm cửa
///    sổ không mở và người dùng nhận đúng thứ tệ nhất: im lặng. Nay đã có bề mặt —
///    `commands::config::bootstrap_config` trả `store.open_failed` khi `try_state` rỗng,
///    và `src/App.vue` vẽ nó thành một dải báo lỗi **không chặn**. Nên việc *"đi tiếp"* ở
///    đây từ hôm nay là **đúng** thay vì chỉ là ít tệ nhất: ứng dụng lên bằng cấu hình mặc
///    định và nói ra rằng nó không đọc được kho, thay vì không lên.
/// 2. `scripts/check-scope.mjs` và `check-scope-bundled.mjs` chạy nhị phân rồi đọc dòng
///    `VERDICT:`. Một `setup()` trả `Err` làm **hai cổng của Story 1.2/1.3 đỏ** vì tầng
///    ghi dữ liệu, không vì phạm vi mà chúng canh.
///
/// ⚠️ Chuỗi chẩn đoán viết KHÔNG DẤU: `scripts/check-i18n.mjs` Kiểm A quét
/// `src-tauri/**/*.rs` và `lib.rs` **không** nằm trong danh sách miễn trừ.
fn open_global_store(app: &tauri::App) {
    use tauri::Manager as _;

    // Không viết cứng `$APPDATA` — `app.path()` là đường duy nhất, và đây là chỗ NFR14
    // (hành vi tương đương hai nền tảng) hỏng đầu tiên nếu ai đó ghép chuỗi bằng tay.
    //
    // Móc e2e đứng TRƯỚC lời gọi đó và chỉ tồn tại trong bản debug + feature `wdio`
    // (AD-45). Bản phát hành đi thẳng xuống nhánh dưới. Xem `E2E_DATA_DIR_ENV`.
    let dir = match data_dir_override() {
        Some(dir) => {
            // In ra vì một lượt chuyển hướng IM LẶNG là thứ tệ nhất ở đây: nếu biến bị
            // đặt nhầm trong một phiên tay, người chạy phải thấy ngay rằng app đang đọc
            // một kho khác, thay vì tưởng cấu hình của mình vừa bốc hơi.
            // Chuoi khong dau: `check-i18n.mjs` Kiem A quet `lib.rs`.
            println!("store[global] data dir override: {}", dir.display());
            dir
        }
        None => match app.path().app_data_dir() {
            Ok(dir) => dir,
            Err(err) => {
                eprintln!("store[global] cannot resolve the app data directory: {err}");
                return;
            }
        },
    };

    if let Err(err) = std::fs::create_dir_all(&dir) {
        eprintln!(
            "store[global] cannot create {}: {err}",
            dir.display()
        );
        return;
    }

    let spec = crate::core::store::StoreSpec::global(dir.join(GLOBAL_DB_FILE));
    match crate::core::store::Store::open(spec) {
        Ok(store) => {
            app.manage(store);
        }
        Err(err) => {
            // `Display` của `StoreError` mang sẵn tên kho và lỗi thô. Story 1.8 nối cùng
            // giá trị đó lên giao diện qua `IpcError`; ở đây nó ra stderr.
            eprintln!("store[global] open failed, running without a data layer: {err}");
        }
    }
}

/// `RunEvent::Exit` ⇒ TRUNCATE lần cuối rồi dừng các luồng của kho (AD-12, AC4).
///
/// ⚠️ Đây là đường DUY NHẤT trong bản release mà `.db-wal` được cắt về 0. Nó có trần
/// thời gian (`Tuning::close_truncate_budget`) vì `AppHandle::exit` đi **qua vòng lặp sự
/// kiện** (`tauri-2.11.5/src/app.rs:574-580`) — nghĩa là móc self-check `#[cfg(debug_assertions)]`
/// cũng chạy callback này, và một `close()` treo ở đây làm `check:scope` /
/// `check:scope:bundled` đỏ vì một lý do không liên quan tới phạm vi chúng canh.
///
/// Vẫn còn hở, và ghi thẳng ra thay vì đánh dấu đạt: `panic = "abort"` nghĩa là một
/// lần thoát cứng **không** đi qua đây. Xem `deferred-work.md`.
fn close_global_store(handle: &tauri::AppHandle) {
    use tauri::Manager as _;

    if let Some(store) = handle.try_state::<crate::core::store::Store>() {
        store.close();
    }
}

/// Mở **tập lớp từ điển** dưới `$RESOURCE/dict/` và đưa nó vào state (Story 1.13, AC3/AC4).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO MỞ Ở `setup()` CHỨ KHÔNG PHẢI Ở PHÍM ĐẦU TIÊN NGƯỜI DÙNG GÕ
/// ─────────────────────────────────────────────────────────────────────────────
/// Mở một lớp là mở một **pool kết nối SQLite**. Làm việc đó lúc tra cứu lần đầu là đặt N
/// lượt mở tệp lên đúng đường nóng của NFR1 *(100 ms đầu-cuối, backend giữ ≤ 10 ms)* — một
/// hình dạng **chắc chắn** vỡ ngân sách đó, và vỡ đúng vào lần tra đầu tiên của mỗi phiên,
/// tức đúng ấn tượng đầu tiên. Mở một lần lúc khởi động là cùng khuôn [`open_global_store`].
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 KHÔNG CÓ LỚP NÀO LÀ MỘT TRẠNG THÁI **BÌNH THƯỜNG CÓ TÊN**
/// ─────────────────────────────────────────────────────────────────────────────
/// `src-tauri/resources/dict/` hôm nay **rỗng** trong git *(không tệp `.db` nào — AD-25)*
/// và `bundle.resources` **chưa** mang thư mục đó *(Story 10.1)*. Nên một bản dựng hôm nay
/// lên với **không lớp nào**, và đó **không** phải một lỗi — nó là chính hình dạng FR36
/// đòi hỏi: *"gỡ một lớp = xoá một file"*, và trường hợp giới hạn của mệnh đề đó là **gỡ
/// hết**.
///
/// [`DictLayers::open`] vì thế **không bao giờ** trả lỗi; thứ nó trả về là một tập lớp
/// rỗng cộng một danh sách `skipped` **có tên**. Ở đây danh sách đó ra stderr; Story 1.17
/// nối nó lên giao diện qua bề mặt IPC của Panel Lookup.
///
/// ⚠️ Chuỗi chẩn đoán viết **KHÔNG DẤU** — cùng bài học `lib.rs:99-100`:
/// `scripts/check-i18n.mjs` Kiểm A quét `src-tauri/**/*.rs` và tệp này không được miễn trừ.
fn open_dict_layers(app: &tauri::App) {
    use tauri::Manager as _;

    // Không ghép chuỗi bằng tay — `app.path()` là đường duy nhất, và đây là chỗ NFR14
    // (hành vi tương đương hai nền tảng) hỏng đầu tiên nếu ai đó tự dựng đường dẫn.
    let dir = match app.path().resource_dir() {
        Ok(dir) => dir.join(DICT_RESOURCE_DIR),
        Err(err) => {
            // ⚠️ Vẫn phải `app.manage(...)` một tập lớp — RỖNG là được, nhưng CHƯA quản lý
            // thì không: một chỗ gọi sau này lấy `DictLayers` ra từ state (vd. một
            // `#[tauri::command]` của Story 1.17) mà state trống hẳn sẽ panic thay vì đọc
            // một tập lớp rỗng có tên.
            eprintln!("dict[layers] cannot resolve the resource directory: {err}");
            app.manage(crate::core::dict::DictLayers::empty());
            return;
        }
    };

    let layers = crate::core::dict::DictLayers::open(&dir);

    // Một dòng cho **mỗi** lớp bị bỏ qua: `SkipReason` mang sẵn đường dẫn và lý do, nên
    // người đọc stderr biết **tệp nào** và **vì sao**, không phải *"từ điển không lên"*.
    for skipped in layers.skipped() {
        eprintln!(
            "dict[layers] skipping {}: {}",
            skipped.path.display(),
            skipped.reason
        );
    }
    eprintln!(
        "dict[layers] {} layer(s) loaded from {}",
        layers.layers().len(),
        dir.display()
    );

    app.manage(layers);
}

/// `RunEvent::Exit` ⇒ đóng **mọi** tệp từ điển đang mở (NFR14, FR112).
///
/// 🔴 Vế này **không** bỏ được, và lý do không phải là gọn gàng: trên Windows một tệp
/// còn mở là một tệp **không thay được** — tức một bản cập nhật không ghi đè nổi
/// `dict-*.db`, và **FR112** *(chính sách gỡ bỏ dữ liệu)* đứng trên đúng khả năng xoá được
/// tệp. Cùng bài học NFR14 đã học ở [`close_global_store`].
///
/// Vẫn còn hở, và ghi thẳng ra thay vì đánh dấu đạt: `panic = "abort"` nghĩa là một lần
/// thoát cứng **không** đi qua đây. Cùng món nợ đã ghi cho [`close_global_store`].
fn close_dict_layers(handle: &tauri::AppHandle) {
    use tauri::Manager as _;

    if let Some(layers) = handle.try_state::<crate::core::dict::DictLayers>() {
        layers.close();
    }
}

/// Đăng ký state cho kho **thứ hai** — Tác phẩm đang mở (Story 1.15, Task 7).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO ĐÂY LÀ MỘT KHO **THỨ HAI**, KHÔNG PHẢI MỘT NHÁNH CỦA KHO TOÀN CỤC
/// ─────────────────────────────────────────────────────────────────────────────
/// `global.db` mở **một lần** ở `setup()` và sống suốt vòng đời tiến trình
/// ([`open_global_store`]). `.atproj/project.db` mở/đóng theo thao tác của người dùng —
/// **N** Tác phẩm có thể mở rồi đóng trong một phiên, không phải một lần lúc khởi động.
/// ⇒ state ở đây là `Mutex<Option<OpenWork>>` (rỗng lúc khởi động), không phải
/// `OpenWork` trần — mọi `#[tauri::command]` sau này lấy nó qua `try_state`, cùng khuôn
/// [`crate::commands::config`].
///
/// ⚠️ **Hệ quả đã biết, ghi ra thay vì giấu** (`deferred-work.md`): mở một kho thứ hai
/// nghĩa là **luồng checkpoint thứ hai + pool đọc thứ hai** (4 kết nối nữa) mỗi khi một
/// Tác phẩm mở — sáu số `Tuning` vẫn TẠM (chủ: Story 2.4), chưa cái nào được đo với hai
/// kho cùng chạy song song. Và mục `Checkpointer::shutdown()` của `deferred-work.md`
/// (treo lửng lúc thoát) đổi từ "vô hại" sang "rủi ro thật" đúng từ story này: đây là
/// story ĐẦU TIÊN khởi động lại một kho (mở Tác phẩm khác) mà không thoát tiến trình.
fn open_work_slot(app: &tauri::App) {
    use tauri::Manager as _;
    app.manage(crate::commands::project::OpenWorkState::new(None));
}

/// `RunEvent::Exit` ⇒ đóng Tác phẩm đang mở (nếu có), cùng khuôn [`close_global_store`].
///
/// ⚠️ Trên Windows một tệp `project.db` còn mở là một `remove_dir_all` thất bại (NFR14) —
/// đúng lớp lỗi mà [`close_global_store`] đã học, áp y hệt cho kho thứ hai.
///
/// Vẫn còn hở, và ghi thẳng ra thay vì đánh dấu đạt: `panic = "abort"` nghĩa là một lần
/// thoát cứng **không** đi qua đây — cùng món nợ đã ghi cho [`close_global_store`].
fn close_open_work(handle: &tauri::AppHandle) {
    use tauri::Manager as _;

    if let Some(state) = handle.try_state::<crate::commands::project::OpenWorkState>() {
        let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(open) = guard.take() {
            open.store.close();
        }
    }
}

/// Nối `WindowEvent::DragDrop` gốc tới một event JS — Story 1.15, Quyết định #1(b).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO ĐÂY LÀ ĐƯỜNG DUY NHẤT, VÀ VÌ SAO NÓ CẦN **ĐÚNG 0** PERMISSION MỚI
/// ─────────────────────────────────────────────────────────────────────────────
/// `WebviewWindow::on_window_event` đăng ký một callback THẲNG trên dispatcher runtime —
/// hoàn toàn KHÔNG đi qua hệ thống invoke/ACL/capabilities (đó không phải một
/// `#[tauri::command]`). Mũi thăm dò của Task 0 (đọc mã nguồn `tauri-runtime-2.11.3` và
/// `tauri-2.11.5` đã ghim, xem story `1-15…md` §Debug Log References) xác nhận: nhận
/// `WindowEvent::DragDrop` ở đây cần 0 permission — không phải "ba permission hiện có có
/// đủ không", câu hỏi đó không áp dụng cho đường này.
///
/// `app.emit(...)` **CÓ** đi qua hệ thống event, và đó là chỗ `core:event:default` (đã
/// cấp từ Story 1.2) thật sự cần — để JS *nghe* được, không phải để Rust *nhận* được.
///
/// Rust chỉ chuyển tiếp **đường dẫn**, **không đọc nội dung tệp** — AD-1/AD-16 đòi mọi
/// nội dung ngoài do Rust phân tích; phía JS gọi lại `create_work_from_file` với đường
/// dẫn này, và Rust đọc tệp ở đó (`core::segment::import::import_file`), không phải ở
/// đây.
fn wire_drag_drop(app: &tauri::App) {
    use tauri::Emitter as _;
    use tauri::Manager as _;

    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        eprintln!("drag-drop: main window not found, drag-drop import disabled");
        return;
    };

    let handle = app.handle().clone();
    window.on_window_event(move |event| {
        let tauri::WindowEvent::DragDrop(drag) = event else {
            return;
        };

        // ⚠️ `DragDropEvent` là `#[non_exhaustive]` — nhánh `_` bắt buộc, và nó cũng là
        // chỗ `Over` rơi vào: `Over` bắn liên tục theo từng chuyển động chuột, forward nó
        // qua IPC là một trận lụt event không ai dùng tới (`Enter` đã đủ để bật cờ).
        let result = match drag {
            tauri::DragDropEvent::Drop { paths, .. } => {
                let payload: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
                handle.emit(DRAG_DROP_EVENT, payload)
            }
            tauri::DragDropEvent::Enter { .. } => handle.emit(DRAG_ENTER_EVENT, ()),
            tauri::DragDropEvent::Leave => handle.emit(DRAG_LEAVE_EVENT, ()),
            _ => return,
        };

        if let Err(err) = result {
            eprintln!("drag-drop: emit failed: {err}");
        }
    });
}

/// Nghe kết quả Kiểm 3 từ webview, in ra stdout, rồi thoát với mã tương ứng.
///
/// Chỉ nối khi `AURA_SCOPE_SELFTEST=1`, và chỉ tồn tại trong bản debug.
#[cfg(debug_assertions)]
fn wire_scope_selftest(handle: &tauri::AppHandle) {
    use tauri::{Listener as _, Manager as _};

    let app = handle.clone();
    handle.listen(SCOPE_SELFTEST_EVENT, move |event| {
        let payload = event.payload();

        // Đọc verdict từ JSON đã parse, KHÔNG so chuỗi con trên payload thô: một
        // `detail` chứa đúng chuỗi đó, hay một serializer chèn khoảng trắng
        // (`{"verdict": "PASS"}`), là đủ để mã thoát nói ngược lại dòng in ra.
        let parsed = serde_json::from_str::<serde_json::Value>(payload).ok();

        let text = parsed
            .as_ref()
            .and_then(|v| v.get("text").and_then(|t| t.as_str()).map(str::to_owned))
            .unwrap_or_else(|| payload.to_owned());

        // Không parse được ⇒ FAIL. Một payload không đọc được không bao giờ là "đạt".
        let verdict_is_pass = parsed
            .as_ref()
            .and_then(|v| v.get("verdict").and_then(|t| t.as_str()))
            .map(|v| v == "PASS")
            .unwrap_or(false);

        println!("{text}");

        let code = if verdict_is_pass { 0 } else { 1 };
        for (_, window) in app.webview_windows() {
            let _ = window.close();
        }
        app.exit(code);
    });
}
