//! Update check (FR111): network exit (2) of AD-15. Runs only when the user asks. It reads
//! the release list and compares versions; it never downloads, writes a file or starts a process.

use std::time::Duration;

pub const RELEASES_API_BASE: &str = "https://api.github.com";
const RELEASES_PATH: &str = "/repos/vannamhh/AuraTranslate/releases?per_page=100";
const RELEASE_PAGE_PREFIX: &str = "https://github.com/vannamhh/AuraTranslate/releases/tag/";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u64, pub u64, pub u64);

impl Version {
    /// Parses `X.Y.Z` (no leading `v`), digits only.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split('.');
        let a = parse_number(parts.next()?)?;
        let b = parse_number(parts.next()?)?;
        let c = parse_number(parts.next()?)?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self(a, b, c))
    }

    pub fn parse_tag(tag: &str) -> Option<Self> {
        Self::parse(tag.strip_prefix('v')?)
    }

    pub fn display(&self) -> String {
        format!("{}.{}.{}", self.0, self.1, self.2)
    }

    pub fn release_page_url(&self) -> String {
        format!("{RELEASE_PAGE_PREFIX}v{}", self.display())
    }
}

fn parse_number(part: &str) -> Option<u64> {
    if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    part.parse().ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateOutcome {
    UpToDate,
    Available(Version),
    NoAppRelease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateError {
    Connect,
    Timeout,
    RateLimited,
    BadResponse,
}

/// Highest `vX.Y.Z` tag among published (non-draft, non-prerelease) releases.
pub fn highest_app_version(body: &[u8]) -> Result<Option<Version>, UpdateError> {
    let value: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| UpdateError::BadResponse)?;
    let releases = value.as_array().ok_or(UpdateError::BadResponse)?;
    Ok(releases
        .iter()
        .filter(|r| r.get("draft").and_then(serde_json::Value::as_bool) != Some(true))
        .filter(|r| r.get("prerelease").and_then(serde_json::Value::as_bool) != Some(true))
        .filter_map(|r| r.get("tag_name")?.as_str())
        .filter_map(Version::parse_tag)
        .max())
}

pub fn decide(current: Version, highest: Option<Version>) -> UpdateOutcome {
    match highest {
        None => UpdateOutcome::NoAppRelease,
        Some(latest) if latest > current => UpdateOutcome::Available(latest),
        Some(_) => UpdateOutcome::UpToDate,
    }
}

fn map_transport(e: &reqwest::Error) -> UpdateError {
    if e.is_timeout() {
        UpdateError::Timeout
    } else if e.is_connect() {
        UpdateError::Connect
    } else {
        UpdateError::BadResponse
    }
}

/// Blocking fetch of the release list from `base_url` (the fixed GitHub host in production, a
/// local fake in tests) followed by [`decide`]. Run it off the async runtime.
pub fn check(base_url: &str, current: Version) -> Result<UpdateOutcome, UpdateError> {
    check_within(base_url, current, REQUEST_TIMEOUT)
}

pub fn check_within(
    base_url: &str,
    current: Version,
    timeout: Duration,
) -> Result<UpdateOutcome, UpdateError> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("AuraTranslate/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(CONNECT_TIMEOUT.min(timeout))
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| UpdateError::BadResponse)?;
    let url = format!("{}{RELEASES_PATH}", base_url.trim_end_matches('/'));
    let response = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| map_transport(&e))?;
    let status = response.status();
    if status.as_u16() == 403 || status.as_u16() == 429 {
        return Err(UpdateError::RateLimited);
    }
    if !status.is_success() {
        return Err(UpdateError::BadResponse);
    }
    let mut body = Vec::new();
    use std::io::Read as _;
    response
        .take(MAX_BODY_BYTES as u64 + 1)
        .read_to_end(&mut body)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::TimedOut {
                UpdateError::Timeout
            } else {
                UpdateError::BadResponse
            }
        })?;
    if body.len() > MAX_BODY_BYTES {
        return Err(UpdateError::BadResponse);
    }
    Ok(decide(current, highest_app_version(&body)?))
}
