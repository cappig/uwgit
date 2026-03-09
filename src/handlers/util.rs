use std::path::{Component, Path as FsPath};
use std::time::{SystemTime, UNIX_EPOCH};

use askama::Template;
use axum::response::Html;
use time::OffsetDateTime;

use crate::git;
use crate::templates::{DisplayTime, PageChrome, PageLink, Pager, PathComponent, RepoNav};

use super::{AppError, AppState};

pub const PAGE_SIZE: usize = 20;

pub fn is_safe_repo_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut components = FsPath::new(name).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

pub fn is_safe_repo_path(path: &str) -> bool {
    if path.is_empty() {
        return true;
    }

    FsPath::new(path)
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
}

pub fn is_safe_ref_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    !name.chars().any(|ch| ch.is_control() || ch.is_whitespace())
}

pub fn open_repo_checked(
    state: &super::AppState,
    repo_name: &str,
) -> Result<git2::Repository, AppError> {
    if !is_safe_repo_name(repo_name) {
        return Err(AppError::NotFound);
    }

    let repo_path = state.repos_path.join(repo_name);
    let base = std::fs::canonicalize(&state.repos_path).map_err(|_| AppError::NotFound)?;
    let repo = std::fs::canonicalize(&repo_path).map_err(|_| AppError::NotFound)?;

    if !repo.starts_with(&base) {
        return Err(AppError::NotFound);
    }

    git::open_repo(&repo_path).map_err(|_| AppError::NotFound)
}

pub fn parse_ref(ref_name: Option<String>) -> Result<(String, Option<String>), AppError> {
    let ref_name = ref_name.unwrap_or_else(|| "HEAD".to_string());
    if !is_safe_ref_name(&ref_name) {
        return Err(AppError::BadRequest);
    }

    match ref_name.as_str() {
        "HEAD" => Ok((String::new(), None)),
        _ => Ok((ref_name.clone(), Some(ref_name))),
    }
}

pub struct RepoRequestContext {
    pub repo: git2::Repository,
    pub repo_name: String,
    pub display_ref: String,
    pub chrome: PageChrome,
    git_ref: Option<String>,
}

impl RepoRequestContext {
    pub fn load(
        state: &AppState,
        repo_name: String,
        ref_name: Option<String>,
    ) -> Result<Self, AppError> {
        let repo = open_repo_checked(state, &repo_name)?;
        let (display_ref, git_ref) = parse_ref(ref_name)?;
        let header_refs = git::get_header_refs(&repo, git_ref.as_deref());

        Ok(Self {
            repo,
            chrome: PageChrome {
                site_title: state.site_title.clone(),
                css_version: crate::config::css_version(),
                js_version: crate::config::js_version(),
                header_repo: Some(repo_name.clone()),
                header_branch: Some(header_refs.branch),
                header_tag: header_refs.tag,
            },
            repo_name,
            display_ref,
            git_ref,
        })
    }

    pub fn git_ref(&self) -> Option<&str> {
        self.git_ref.as_deref()
    }

    pub fn append_ref(&self, url: String) -> String {
        append_ref(url, &self.display_ref)
    }

    pub fn nav(&self, current_page: &'static str) -> RepoNav {
        RepoNav {
            repo_name: self.repo_name.clone(),
            display_ref: self.display_ref.clone(),
            current_page,
            archive_href: self.append_ref(format!("/{}/archive.tar.gz", self.repo_name)),
        }
    }

    pub fn path_components(&self, path: &str, link_last: bool) -> Vec<PathComponent> {
        build_path_components(&self.repo_name, path, &self.display_ref, link_last)
    }
}

pub fn site_chrome(site_title: String) -> PageChrome {
    PageChrome {
        site_title,
        css_version: crate::config::css_version(),
        js_version: crate::config::js_version(),
        header_repo: None,
        header_branch: None,
        header_tag: None,
    }
}

pub fn render_template<T, F>(build: F) -> Result<Html<String>, AppError>
where
    T: Template,
    F: FnOnce() -> Result<T, AppError>,
{
    Ok(Html(build()?.render()?))
}

pub fn relative_time(timestamp: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let seconds = (now - timestamp).max(0);
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    let plural = |n: i64, unit: &str| -> String {
        if n == 1 {
            format!("1 {} ago", unit)
        } else {
            format!("{} {}s ago", n, unit)
        }
    };

    if seconds < 60 {
        plural(seconds, "second")
    } else if minutes < 60 {
        plural(minutes, "minute")
    } else if hours < 24 {
        plural(hours, "hour")
    } else if days < 30 {
        plural(days, "day")
    } else if days < 365 {
        plural(days / 30, "month")
    } else {
        plural(days / 365, "year")
    }
}

pub fn full_date(timestamp: i64) -> String {
    OffsetDateTime::from_unix_timestamp(timestamp)
        .ok()
        .map(|datetime| {
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
                datetime.year(),
                datetime.month() as u8,
                datetime.day(),
                datetime.hour(),
                datetime.minute(),
                datetime.second()
            )
        })
        .unwrap_or_else(|| format!("unix {}", timestamp))
}

pub fn display_time(timestamp: i64) -> DisplayTime {
    DisplayTime {
        relative: relative_time(timestamp),
        full: full_date(timestamp),
    }
}

fn append_param(mut url: String, key: &str, value: &str) -> String {
    url.reserve(key.len() + value.len() + 2);
    url.push(if url.contains('?') { '&' } else { '?' });
    url.push_str(key);
    url.push('=');
    url.push_str(value);
    url
}

pub fn append_ref(url: String, display_ref: &str) -> String {
    if display_ref.is_empty() {
        url
    } else {
        append_param(url, "ref", display_ref)
    }
}

pub fn append_page(url: String, page: usize) -> String {
    if page <= 1 {
        return url;
    }

    append_param(url, "page", &page.to_string())
}

pub fn total_pages(total: usize, page_size: usize) -> usize {
    let pages = total.div_ceil(page_size);

    pages.max(1)
}

fn page_href(base: &str, display_ref: &str, page: usize) -> String {
    append_page(append_ref(base.to_string(), display_ref), page)
}

fn page_link(base: &str, display_ref: &str, current: usize, page: usize) -> PageLink {
    PageLink {
        label: page.to_string(),
        is_current: page == current,
        is_ellipsis: false,
        href: page_href(base, display_ref, page),
    }
}

fn ellipsis() -> PageLink {
    PageLink {
        label: "...".to_string(),
        is_current: false,
        is_ellipsis: true,
        href: String::new(),
    }
}

pub fn build_page_links(
    base: &str,
    display_ref: &str,
    current: usize,
    total: usize,
) -> Vec<PageLink> {
    if total <= 1 {
        return Vec::new();
    }

    let window = 2usize;
    let start = current.saturating_sub(window).max(1);
    let end = (current + window).min(total).max(start);
    let mut links = Vec::new();

    if start > 1 {
        links.push(page_link(base, display_ref, current, 1));

        if start > 2 {
            links.push(ellipsis());
        }
    }

    for page in start..=end {
        links.push(page_link(base, display_ref, current, page));
    }

    if end < total {
        if end + 1 < total {
            links.push(ellipsis());
        }

        links.push(page_link(base, display_ref, current, total));
    }

    links
}

pub fn build_pager(base: &str, display_ref: &str, current: usize, total: usize) -> Pager {
    let has_prev = current > 1;
    let has_next = current < total;

    Pager {
        page_links: build_page_links(base, display_ref, current, total),
        has_prev,
        has_next,
        prev_href: has_prev
            .then(|| page_href(base, display_ref, current - 1))
            .unwrap_or_default(),
        next_href: has_next
            .then(|| page_href(base, display_ref, current + 1))
            .unwrap_or_default(),
    }
}

pub fn content_type_for_extension(ext: &str) -> &'static str {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "avif" => "image/avif",
        _ => "text/plain",
    }
}

pub fn build_path_components(
    repo_name: &str,
    path: &str,
    display_ref: &str,
    link_last: bool,
) -> Vec<PathComponent> {
    if path.is_empty() {
        return Vec::new();
    }

    let last_index = path.matches('/').count();
    let mut components = Vec::with_capacity(last_index + 1);
    let mut component_path = String::with_capacity(path.len());

    for (idx, part) in path.split('/').enumerate() {
        if idx > 0 {
            component_path.push('/');
        }

        component_path.push_str(part);
        components.push(PathComponent {
            name: part.to_string(),
            href: (link_last || idx < last_index).then(|| {
                append_ref(
                    format!("/{}/tree?path={}", repo_name, component_path),
                    display_ref,
                )
            }),
        });
    }

    components
}
