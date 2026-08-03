use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;

const ROBOTS: &str = "\
User-agent: *
Disallow: /*/blob
Disallow: /*/tree
Disallow: /*/commit/
Disallow: /*/archive.tar.gz
";

pub async fn robots() -> impl IntoResponse {
    return (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        )],
        ROBOTS,
    );
}
