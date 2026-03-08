# μwGit

> A tiny web frontend for git

`uwGit` is a small read-only git browser built with Rust. Point it at a folder of local repos and it gives you clean pages for browsing code, history, refs, and README files. 

### Run

`uwGit` reads configuration from `.env`.

```env
REPOS_PATH=/path/to/repos
SITE_TITLE=git.example.dev
OWNER=Your Name
HOST=0.0.0.0
PORT=3000
```

Then start it:

```bash
cargo run
```

Open `http://127.0.0.1:3000`

### Stack

Rust, Axum, Askama, and git2
