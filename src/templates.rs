use askama::Template;

#[derive(Clone)]
pub struct PageChrome {
    pub site_title: String,
    pub css_version: &'static str,
    pub js_version: &'static str,
    pub header_repo: Option<String>,
    pub header_branch: Option<String>,
    pub header_tag: Option<String>,
}

#[derive(Clone)]
pub struct RepoNav {
    pub repo_name: String,
    pub display_ref: String,
    pub current_page: &'static str,
    pub archive_href: String,
}

#[derive(Clone)]
pub struct DisplayTime {
    pub relative: String,
    pub full: String,
}

#[derive(Clone)]
pub struct PathComponent {
    pub name: String,
    pub href: Option<String>,
}

#[derive(Clone)]
pub struct PageLink {
    pub label: String,
    pub is_current: bool,
    pub is_ellipsis: bool,
    pub href: String,
}

#[derive(Clone)]
pub struct Pager {
    pub page_links: Vec<PageLink>,
    pub has_prev: bool,
    pub has_next: bool,
    pub prev_href: String,
    pub next_href: String,
}

#[derive(Clone)]
pub struct DiffLine {
    pub class: &'static str,
    pub num: String,
    pub text: String,
}

#[derive(Clone)]
pub struct FileDiff {
    pub is_binary: bool,
    pub empty_label: Option<String>,
    pub size_html: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Clone)]
pub struct CommitFileSummary {
    pub file_name: String,
    pub adds: usize,
    pub dels: usize,
    pub is_binary: bool,
    pub is_added: bool,
    pub is_deleted: bool,
    pub empty_label: Option<String>,
    pub diff_href: String,
}

pub struct ReadmeDisplay {
    pub content: String,
    pub is_markdown: bool,
}

#[derive(Clone)]
pub struct RefDisplay {
    pub href: String,
    pub name: String,
    pub ref_type: String,
    pub author: String,
    pub time: DisplayTime,
}

#[derive(Clone)]
pub struct TreeEntryDisplay {
    pub name: String,
    pub is_dir: bool,
    pub size_text: String,
    pub time: Option<DisplayTime>,
    pub href: String,
}

#[derive(Clone)]
pub struct CommitDisplay {
    pub href: String,
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub message: String,
    pub time: DisplayTime,
    pub refs: Vec<CommitRefDisplay>,
}

#[derive(Clone)]
pub struct CommitRefDisplay {
    pub name: String,
    pub class: &'static str,
}

#[derive(Template)]
#[template(path = "repos.html")]
pub struct ReposTemplate {
    pub repos: Vec<crate::git::RepoInfo>,
    pub owner: String,
    pub chrome: PageChrome,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub chrome: PageChrome,
    pub nav: RepoNav,
    pub readme: Option<ReadmeDisplay>,
}

#[derive(Template)]
#[template(path = "refs.html")]
pub struct RefsTemplate {
    pub chrome: PageChrome,
    pub nav: RepoNav,
    pub refs: Vec<RefDisplay>,
    pub pager: Pager,
}

#[derive(Template)]
#[template(path = "tree.html")]
pub struct TreeTemplate {
    pub chrome: PageChrome,
    pub nav: RepoNav,
    pub entries: Vec<TreeEntryDisplay>,
    pub path: String,
    pub parent_href: String,
    pub root_href: String,
    pub path_components: Vec<PathComponent>,
}

#[derive(Template)]
#[template(path = "log.html")]
pub struct LogTemplate {
    pub chrome: PageChrome,
    pub nav: RepoNav,
    pub commits: Vec<CommitDisplay>,
    pub pager: Pager,
}

#[derive(Template)]
#[template(path = "blob.html")]
pub struct BlobTemplate {
    pub chrome: PageChrome,
    pub nav: RepoNav,
    pub file_path: String,
    pub lines: Vec<DiffLine>,
    pub blob_href: String,
    pub root_href: String,
    pub is_binary: bool,
    pub is_image: bool,
    pub path_components: Vec<PathComponent>,
}

#[derive(Template)]
#[template(path = "commit.html")]
pub struct CommitTemplate {
    pub chrome: PageChrome,
    pub nav: RepoNav,
    pub commit_title: String,
    pub commit_message: String,
    pub commit_hash: String,
    pub branch: Option<String>,
    pub tags: Vec<String>,
    pub author: String,
    pub author_email: String,
    pub time: DisplayTime,
    pub files: Vec<CommitFileSummary>,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Template)]
#[template(path = "partials/commit_diff_body.html")]
pub struct CommitDiffBodyTemplate {
    pub diff: FileDiff,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub chrome: PageChrome,
    pub heading: String,
    pub message: String,
}
