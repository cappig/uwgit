use anyhow::Result;
use git2::{Delta, Diff, DiffDelta, DiffOptions, Patch, Repository};

use crate::format::{empty_diff_label, size_html_from_sizes};
use crate::templates::{DiffLine, FileDiff};

use super::{commit_for_hash, parent_tree};

pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Clone)]
pub struct CommitDiffSummary {
    pub path: String,
    pub adds: usize,
    pub dels: usize,
    pub is_binary: bool,
    pub empty_label: Option<&'static str>,
    pub change: CommitDiffChange,
    pub old_size: Option<u64>,
    pub new_size: Option<u64>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CommitDiffChange {
    Added,
    Deleted,
    Modified,
}

pub fn get_commit_diff_summaries(
    repo: &Repository,
    commit_hash: &str,
) -> Result<(Vec<CommitDiffSummary>, DiffStats)> {
    let commit = commit_for_hash(repo, commit_hash)?;
    let diff = commit_diff(repo, &commit, None)?;
    let stats = diff.stats()?;

    let diff_stats = DiffStats {
        files_changed: stats.files_changed(),
        insertions: stats.insertions(),
        deletions: stats.deletions(),
    };

    let mut summaries = Vec::new();
    for (idx, delta) in diff.deltas().enumerate() {
        let patch = Patch::from_diff(&diff, idx)?;
        summaries.push(diff_summary(&delta, patch.as_ref())?);
    }

    Ok((summaries, diff_stats))
}

pub fn get_commit_diff_for_path(
    repo: &Repository,
    commit_hash: &str,
    path: &str,
) -> Result<Option<FileDiff>> {
    let commit = commit_for_hash(repo, commit_hash)?;
    let diff = commit_diff(repo, &commit, Some(path))?;

    let Some((idx, delta)) = diff.deltas().enumerate().next() else {
        return Ok(None);
    };

    let patch = Patch::from_diff(&diff, idx)?;
    let summary = diff_summary(&delta, patch.as_ref())?;
    let mut lines = patch
        .as_ref()
        .map(render_patch_lines)
        .transpose()?
        .unwrap_or_default();

    if summary.is_binary {
        lines.clear();
    }

    Ok(Some(FileDiff {
        is_binary: summary.is_binary,
        empty_label: summary.empty_label.map(str::to_string),
        size_html: size_html_from_sizes(summary.old_size, summary.new_size),
        lines,
    }))
}

pub fn is_binary_bytes(content: &[u8]) -> bool {
    if content.is_empty() {
        return false;
    }

    let threshold = content.len() / 100;
    let mut non_text = 0usize;

    for &byte in content {
        if byte == 0 {
            return true;
        }

        if byte < 32 && byte != b'\n' && byte != b'\r' && byte != b'\t' {
            non_text += 1;
            if non_text > threshold {
                return true;
            }
        }
    }

    false
}

fn diff_summary(delta: &DiffDelta<'_>, patch: Option<&Patch<'_>>) -> Result<CommitDiffSummary> {
    let old_file = delta.old_file();
    let new_file = delta.new_file();

    let old_size = old_file.exists().then_some(old_file.size());
    let new_size = new_file.exists().then_some(new_file.size());

    let path = delta_path(delta);
    let (has_lines, adds, dels) = patch_stats(patch)?;
    let is_binary = old_file.is_binary() || new_file.is_binary();
    let change = change_from_delta(delta.status());

    Ok(CommitDiffSummary {
        path,
        adds,
        dels,
        is_binary,
        empty_label: empty_diff_label(is_binary, has_lines, old_size, new_size, change),
        change,
        old_size,
        new_size,
    })
}

fn patch_stats(patch: Option<&Patch<'_>>) -> Result<(bool, usize, usize)> {
    let Some(patch) = patch else {
        return Ok((false, 0, 0));
    };

    let (context, additions, deletions) = patch.line_stats()?;
    Ok((context + additions + deletions > 0, additions, deletions))
}

fn render_patch_lines(patch: &Patch<'_>) -> Result<Vec<DiffLine>> {
    let mut lines = Vec::new();

    for hunk_idx in 0..patch.num_hunks() {
        let (hunk, line_count) = patch.hunk(hunk_idx)?;
        let mut old_line = hunk.old_start() as i32;
        let mut new_line = hunk.new_start() as i32;

        for line_idx in 0..line_count {
            let line = patch.line_in_hunk(hunk_idx, line_idx)?;
            let text = diff_line_text(&line);

            match line.origin() {
                '+' | '>' => {
                    lines.push(DiffLine {
                        class: "line-add",
                        num: new_line.to_string(),
                        text,
                    });
                    new_line += 1;
                }
                '-' | '<' => {
                    lines.push(DiffLine {
                        class: "line-remove",
                        num: old_line.to_string(),
                        text,
                    });
                    old_line += 1;
                }
                ' ' | '=' => {
                    lines.push(DiffLine {
                        class: "",
                        num: new_line.to_string(),
                        text,
                    });
                    old_line += 1;
                    new_line += 1;
                }
                _ => {}
            }
        }
    }

    Ok(lines)
}

fn diff_line_text(line: &git2::DiffLine<'_>) -> String {
    String::from_utf8_lossy(line.content())
        .trim_end_matches('\n')
        .trim_end_matches('\r')
        .to_string()
}

fn commit_diff<'repo>(
    repo: &'repo Repository,
    commit: &git2::Commit<'repo>,
    path: Option<&str>,
) -> Result<Diff<'repo>> {
    let tree = commit.tree()?;
    let parent_tree = parent_tree(&commit);
    let mut opts = DiffOptions::new();

    if let Some(path) = path {
        opts.pathspec(path).disable_pathspec_match(true);
    }

    repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut opts))
        .map_err(Into::into)
}

fn change_from_delta(delta: Delta) -> CommitDiffChange {
    match delta {
        Delta::Added => CommitDiffChange::Added,
        Delta::Deleted => CommitDiffChange::Deleted,
        _ => CommitDiffChange::Modified,
    }
}

fn delta_path(delta: &DiffDelta<'_>) -> String {
    delta
        .new_file()
        .path()
        .or_else(|| delta.old_file().path())
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default()
}
