//! Markdown rendering, ported from `render.ts`.
//!
//! Post bodies are authored by us (trusted), so we render markdown straight to
//! HTML without sanitizing — same trust model as the Node service, which used
//! `marked` with GFM. `pulldown-cmark` is a pure-Rust CommonMark/GFM parser with
//! no C dependencies.

use pulldown_cmark::{html, Options, Parser};

/// Render a post's raw markdown body to HTML for the email content box.
pub fn render_markdown(md: &str) -> String {
    // GFM feature set to match `marked({ gfm: true })`: tables, strikethrough,
    // task lists, and autolinking. `breaks: false` is the default (soft line
    // breaks stay as newlines, not <br>).
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_GFM);

    let parser = Parser::new_ext(md, opts);
    let mut out = String::with_capacity(md.len() + md.len() / 2);
    html::push_html(&mut out, parser);
    out
}
