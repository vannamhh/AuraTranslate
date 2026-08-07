//! `dict-build` — CLI: `--raw <dir> --out-dir <dir> [--layer <base|thieu-chuu|vietphrase|all>]`.
//! Chỉ gọi `dict_build::build::run_*`; toàn bộ logic sống trong `lib.rs` để `tests/`
//! dùng lại được (xem doc-comment `lib.rs`).
//!
//! Story 1.10, Task 4: thay `--out <file>` (một tệp) bằng `--out-dir <dir>` (nhiều tệp,
//! tên cố định trong mã — `build::output_file_name`). `--layer` mặc định `all`, dựng
//! ĐỦ BA tệp và hỏng nếu bất kỳ lớp nào thiếu nguồn thô — không chế độ bỏ qua (§Bẫy 7).

use std::path::PathBuf;
use std::process::ExitCode;

use dict_build::build;

#[derive(Debug)]
enum Layer {
    All,
    Base,
    Detachable(String),
}

#[derive(Debug)]
struct Args {
    raw: PathBuf,
    out_dir: PathBuf,
    layer: Layer,
}

fn parse_args() -> Result<Args, String> {
    parse_args_from(std::env::args().skip(1))
}

/// Tách khỏi `parse_args` để test được — trước đây lớp CLI không có một test nào dù
/// mở rộng CLI là Task 4 của Story 1.10 (Review Findings).
fn parse_args_from<I: IntoIterator<Item = String>>(args: I) -> Result<Args, String> {
    let mut raw = None;
    let mut out_dir = None;
    let mut layer_str: Option<String> = None;
    let mut it = args.into_iter();

    // Một giá trị bắt đầu bằng `--` gần như luôn là dấu hiệu gõ THIẾU giá trị
    // (`--out-dir --layer base`). Nhận âm thầm cho ra `create_dir_all("--layer")` — một
    // thư mục thật tên `--layer` chứa đủ ba tệp `.db`, với ExitCode::SUCCESS.
    fn value_for(flag: &str, it: &mut impl Iterator<Item = String>) -> Result<String, String> {
        let v = it.next().ok_or_else(|| format!("{flag} cần một giá trị"))?;
        if v.starts_with("--") {
            return Err(format!(
                "{flag} nhận '{v}' làm giá trị — trông như một cờ khác, gõ thiếu giá trị cho {flag}?"
            ));
        }
        Ok(v)
    }

    // Một cờ lặp lại được nhận âm thầm với "lần cuối thắng" là cùng lớp lỗi "ghi nhầm
    // chỗ" mà nhánh `--out` dưới đây tồn tại để chặn.
    fn set_once<T>(slot: &mut Option<T>, flag: &str, value: T) -> Result<(), String> {
        if slot.is_some() {
            return Err(format!("{flag} được khai nhiều hơn một lần"));
        }
        *slot = Some(value);
        Ok(())
    }

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--raw" => set_once(&mut raw, "--raw", PathBuf::from(value_for("--raw", &mut it)?))?,
            "--out-dir" => set_once(&mut out_dir, "--out-dir", PathBuf::from(value_for("--out-dir", &mut it)?))?,
            "--layer" => set_once(&mut layer_str, "--layer", value_for("--layer", &mut it)?)?,
            // 🔴 `--out` (tham số cũ, MỘT tệp) không còn được nhận — lỗi TƯỜNG MINH nêu
            // tên tham số thay thế. Không nhận âm thầm để "tương thích ngược": một
            // lượt build ghi nhầm chỗ là một tệp cũ bị đè.
            "--out" => {
                return Err(
                    "--out đã bị thay bằng --out-dir <thư mục> (Story 1.10) — mỗi lớp giờ ra MỘT tệp riêng trong thư mục đó".to_string(),
                );
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let layer = match layer_str.as_deref() {
        None | Some("all") => Layer::All,
        Some("base") => Layer::Base,
        Some(code) => Layer::Detachable(code.to_string()),
    };

    Ok(Args {
        raw: raw.ok_or("missing --raw <dir>")?,
        out_dir: out_dir.ok_or("missing --out-dir <dir>")?,
        layer,
    })
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Args, String> {
        parse_args_from(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn defaults_to_layer_all() {
        let a = parse(&["--raw", "raw", "--out-dir", "out"]).unwrap();
        assert!(matches!(a.layer, Layer::All));
        assert_eq!(a.raw, PathBuf::from("raw"));
        assert_eq!(a.out_dir, PathBuf::from("out"));
    }

    #[test]
    fn resolves_base_and_detachable_layers() {
        assert!(matches!(parse(&["--raw", "r", "--out-dir", "o", "--layer", "base"]).unwrap().layer, Layer::Base));
        assert!(matches!(
            parse(&["--raw", "r", "--out-dir", "o", "--layer", "vietphrase"]).unwrap().layer,
            Layer::Detachable(ref c) if c == "vietphrase"
        ));
    }

    /// Lời hứa ở README và ở nhánh `--out`: lỗi TƯỜNG MINH nêu tên tham số thay thế.
    #[test]
    fn the_removed_out_flag_names_its_replacement() {
        let err = parse(&["--raw", "r", "--out", "x.db"]).unwrap_err();
        assert!(err.contains("--out-dir"), "lỗi phải nêu tên tham số thay thế: {err}");
    }

    /// 🔴 `--out-dir --layer base`: nhận âm thầm cho ra một thư mục THẬT tên `--layer`
    /// chứa đủ ba tệp `.db`, với ExitCode::SUCCESS.
    #[test]
    fn a_flag_shaped_value_is_rejected_not_used_as_a_path() {
        let err = parse(&["--raw", "r", "--out-dir", "--layer", "base"]).unwrap_err();
        assert!(err.contains("--out-dir"), "{err}");
    }

    #[test]
    fn a_repeated_flag_is_an_error_not_last_one_wins() {
        let err = parse(&["--raw", "a", "--raw", "b", "--out-dir", "o"]).unwrap_err();
        assert!(err.contains("--raw"), "{err}");
    }

    #[test]
    fn missing_required_flags_are_reported() {
        assert!(parse(&["--out-dir", "o"]).unwrap_err().contains("--raw"));
        assert!(parse(&["--raw", "r"]).unwrap_err().contains("--out-dir"));
    }
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("dict-build: {e}");
            eprintln!(
                "cách dùng: dict-build --raw <thư mục raw> --out-dir <thư mục đích> [--layer <base|thieu-chuu|vietphrase|all>]"
            );
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = std::fs::create_dir_all(&args.out_dir) {
        eprintln!("dict-build: không tạo được --out-dir '{}': {e}", args.out_dir.display());
        return ExitCode::FAILURE;
    }

    match args.layer {
        Layer::All => match build::run_all(&args.raw, &args.out_dir) {
            Ok(report) => {
                println!("\n### base — {} ###", report.base.0);
                build::print_report(&report.base.1);
                for (name, r) in &report.detachable {
                    println!("\n### lớp gỡ rời — {name} ###");
                    build::print_report(r);
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("dict-build: lỗi khi dựng: {e}");
                ExitCode::FAILURE
            }
        },
        Layer::Base => {
            let out_path = args.out_dir.join(build::output_file_name("base"));
            match build::run_base(&args.raw, &out_path) {
                Ok(report) => {
                    build::print_report(&report);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("dict-build: lỗi khi dựng: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Layer::Detachable(code) => match build::run_detachable_by_code(&args.raw, &args.out_dir, &code) {
            Ok((_name, report)) => {
                build::print_report(&report);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("dict-build: lỗi khi dựng: {e}");
                ExitCode::FAILURE
            }
        },
    }
}
