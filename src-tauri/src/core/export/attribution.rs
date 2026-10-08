use crate::core::store::{ReadHandle, SqlResult};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChapterOrigin {
    pub author: Option<String>,
    pub site_name: Option<String>,
    pub url: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attribution {
    pub translator: Option<String>,
}

pub(super) fn load_origin(conn: ReadHandle<'_>, chapter_id: i64) -> SqlResult<ChapterOrigin> {
    conn.query_row(
        "SELECT origin_author, origin_site_name, origin_url, origin_published_at FROM chapter WHERE id = ?1",
        [chapter_id],
        |row| {
            Ok(ChapterOrigin {
                author: row.get(0)?,
                site_name: row.get(1)?,
                url: row.get(2)?,
                published_at: row.get(3)?,
            })
        },
    )
}

fn present(value: &Option<String>) -> Option<String> {
    value.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(str::to_owned)
}

pub(super) fn lines_for(
    conn: ReadHandle<'_>,
    chapter_id: i64,
    attribution: Option<&Attribution>,
) -> SqlResult<Vec<String>> {
    match attribution {
        None => Ok(Vec::new()),
        Some(attribution) => Ok(attribution_lines(&load_origin(conn, chapter_id)?, attribution.translator.as_deref())),
    }
}

pub fn attribution_lines(chapter_origin: &ChapterOrigin, translator: Option<&str>) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(author) = present(&chapter_origin.author) {
        // aura-allow-text: label written into the exported file, not UI text
        lines.push(format!("Tác giả: {author}"));
    }
    let source: Vec<String> = [present(&chapter_origin.site_name), present(&chapter_origin.url)].into_iter().flatten().collect();
    if !source.is_empty() {
        // aura-allow-text: label written into the exported file, not UI text
        lines.push(format!("Nguồn: {}", source.join(" · ")));
    }
    if let Some(published) = present(&chapter_origin.published_at) {
        // aura-allow-text: label written into the exported file, not UI text
        lines.push(format!("Ngày đăng gốc: {published}"));
    }
    if let Some(name) = translator.map(str::trim).filter(|n| !n.is_empty()) {
        // aura-allow-text: label written into the exported file, not UI text
        lines.push(format!("Người dịch: {name}"));
    }
    lines
}

/// Whether a line is one of the labelled lines [`attribution_lines`] writes.
pub(super) fn is_attribution_line(line: &str) -> bool {
    // aura-allow-text: labels read back from the exported file, kept next to the writer above
    ["Tác giả: ", "Nguồn: ", "Ngày đăng gốc: ", "Người dịch: "].iter().any(|label| line.starts_with(label))
}
