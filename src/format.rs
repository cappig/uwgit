use crate::git::CommitDiffChange;

pub fn human_readable_size(bytes: u64) -> String {
    let mut size = bytes as f64;
    let units = ["B", "KB", "MB", "GB", "TB"];

    let mut unit = 0usize;
    while size >= 1024.0 && unit < units.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{} {}", bytes, units[unit])
    } else {
        format!("{:.1} {}", size, units[unit])
    }
}

pub fn size_html_from_sizes(old_size: Option<u64>, new_size: Option<u64>) -> String {
    match (old_size, new_size) {
        (Some(old_size), Some(new_size)) => format!(
            "<span class=\"size-old\">{}</span> <span class=\"size-arrow\">→</span> <span class=\"size-new\">{}</span>",
            human_readable_size(old_size),
            human_readable_size(new_size)
        ),
        (None, Some(new_size)) => format!(
            "<span class=\"size-new\">+{}</span>",
            human_readable_size(new_size)
        ),
        (Some(old_size), None) => format!(
            "<span class=\"size-old\">-{}</span>",
            human_readable_size(old_size)
        ),
        (None, None) => String::new(),
    }
}

pub fn empty_diff_label(
    is_binary: bool,
    has_lines: bool,
    old_size: Option<u64>,
    new_size: Option<u64>,
    change: CommitDiffChange,
) -> Option<&'static str> {
    if is_binary || has_lines {
        return None;
    }

    match change {
        CommitDiffChange::Added if new_size == Some(0) => Some("empty added"),
        CommitDiffChange::Deleted if old_size == Some(0) => Some("empty deleted"),
        CommitDiffChange::Modified if old_size == Some(0) && new_size == Some(0) => Some("empty"),
        _ => None,
    }
}
