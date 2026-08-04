//! `dict-build` — CLI: `--raw <dir> --out <file>`. Chỉ gọi `dict_build::build::run`;
//! toàn bộ logic sống trong `lib.rs` để `tests/` dùng lại được (xem doc-comment `lib.rs`).

use std::path::PathBuf;
use std::process::ExitCode;

use dict_build::build;

struct Args {
    raw: PathBuf,
    out: PathBuf,
}

fn parse_args() -> Result<Args, String> {
    let mut raw = None;
    let mut out = None;
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--raw" => raw = Some(PathBuf::from(it.next().ok_or("--raw needs a value")?)),
            "--out" => out = Some(PathBuf::from(it.next().ok_or("--out needs a value")?)),
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(Args {
        raw: raw.ok_or("missing --raw <dir>")?,
        out: out.ok_or("missing --out <file>")?,
    })
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("dict-build: {e}");
            eprintln!("cách dùng: dict-build --raw <thư mục raw> --out <đường dẫn .db>");
            return ExitCode::FAILURE;
        }
    };

    match build::run(&args.raw, &args.out) {
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
