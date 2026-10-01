//! Email HTML/text templates, ported from `email-templates.ts`. The markup and
//! copy are kept identical so rendered emails look the same as before.

use chrono::{DateTime, NaiveDate};

use crate::config::{Config, ListName};

pub struct OutgoingEmail {
    pub subject: String,
    pub html: String,
    pub text: String,
    /// Extra headers (e.g. one-click List-Unsubscribe). Empty for confirmations.
    pub headers: Vec<(String, String)>,
}

fn list_label(list: ListName) -> &'static str {
    match list {
        ListName::Blog => "the blog",
        ListName::Notes => "notes",
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Percent-encode a value for use in a URL query component — the equivalent of
/// JavaScript's `encodeURIComponent`. Our tokens are base64url (already URL-safe),
/// but this stays faithful and defends against anything unexpected in the value.
pub fn encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Format a date the way the TS service did: `new Date(value).toLocaleDateString`
/// with `{ year:'numeric', month:'long', day:'numeric' }` → e.g. "September 29, 2026".
/// Falls back to the raw string when the value can't be parsed.
fn format_date(value: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return dt.format("%B %-d, %Y").to_string();
    }
    if let Ok(d) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return d.format("%B %-d, %Y").to_string();
    }
    value.to_string()
}

/// Shared email shell (see the TS original for the design rationale). Typography
/// lives in a `<head><style>` block; structural bits are also inlined.
fn shell(
    title: &str,
    title_href: Option<&str>,
    byline_html: Option<&str>,
    forward_html: Option<&str>,
    content_html: &str,
    footer_html: Option<&str>,
) -> String {
    // Escape the href too: `title_href` is the admin-supplied post URL, and while
    // the admin is trusted, an unescaped value could break out of the attribute.
    let title_inner = match title_href {
        Some(href) => format!(
            "<a href=\"{}\">{}</a>",
            escape_html(href),
            escape_html(title)
        ),
        None => escape_html(title),
    };
    let byline = byline_html
        .map(|b| format!("<p class=\"byline\">{b}</p>"))
        .unwrap_or_default();
    let forward = forward_html
        .map(|f| format!("<p class=\"forward\">{f}</p>"))
        .unwrap_or_default();
    let footer = footer_html
        .map(|f| format!("<p class=\"footer\">{f}</p>"))
        .unwrap_or_default();

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>
  body {{ margin:0; padding:0; background:#f4f4f2; }}
  .container {{ max-width:660px; margin:0 auto; padding:44px 24px 64px; font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif; color:#1a1a1a; }}
  .title {{ text-align:center; font-size:34px; line-height:1.15; font-weight:800; letter-spacing:-0.02em; margin:0 0 18px; }}
  .title a {{ color:inherit; text-decoration:none; }}
  .byline {{ text-align:center; text-transform:uppercase; letter-spacing:0.08em; font-size:13px; color:#8a8a8a; margin:0 0 40px; }}
  .forward {{ text-align:right; font-size:15px; color:#6b7280; margin:0 0 14px; }}
  .forward a, .content a {{ color:#2563eb; }}
  .content {{ border:1px solid #e3e3e0; border-radius:12px; padding:8px 30px; background:#ffffff; font-size:18px; line-height:1.62; }}
  .content p {{ margin:20px 0; }}
  .content strong {{ font-weight:700; }}
  .content em {{ font-style:italic; }}
  .content h1,.content h2,.content h3 {{ line-height:1.25; margin:28px 0 12px; }}
  .content ul,.content ol {{ padding-left:22px; }}
  .content li {{ margin:6px 0; }}
  .content img {{ max-width:100%; height:auto; border-radius:8px; }}
  .content hr {{ border:none; border-top:1px solid #e3e3e0; margin:28px 0; }}
  .content blockquote {{ margin:24px 0; background:#f3f3f1; border-radius:12px; padding:2px 24px; color:#6b7280; }}
  .content pre {{ background:#f3f3f1; border-radius:8px; padding:14px 16px; overflow:auto; font-size:14px; line-height:1.5; }}
  .content code {{ background:#f3f3f1; border-radius:4px; padding:1px 5px; font-size:0.9em; overflow-wrap:anywhere; }}
  .content pre code {{ background:none; padding:0; }}
  .button {{ display:inline-block; background:#28665f; color:#ffffff !important; text-decoration:none; padding:12px 24px; border-radius:8px; font-weight:600; }}
  .footer {{ margin-top:30px; text-align:center; font-size:13px; color:#8a8a8a; }}
  .footer a {{ color:#8a8a8a; }}
  .fallback {{ font-size:13px; color:#8a8a8a; word-break:break-all; }}
  /* Phones: the desktop padding costs ~110px of a 390px screen, which leaves the
     measure too narrow to read. Clients that ignore this still get the desktop rules. */
  @media only screen and (max-width: 520px) {{
    .container {{ padding:28px 14px 44px; }}
    .title {{ font-size:26px; margin-bottom:14px; }}
    .byline {{ font-size:12px; margin-bottom:26px; }}
    .forward {{ text-align:center; font-size:14px; }}
    .content {{ padding:4px 18px; font-size:17px; }}
    .content p {{ margin:16px 0; }}
    .content blockquote {{ padding:2px 16px; margin:20px 0; }}
    .content pre {{ padding:12px; font-size:13px; }}
    .content ul,.content ol {{ padding-left:20px; }}
    .button {{ display:block; text-align:center; }}
  }}
</style>
</head>
<body style="margin:0;padding:0;background:#f4f4f2;">
  <div class="container">
    <h1 class="title">{title_inner}</h1>
    {byline}
    {forward}
    <div class="content">{content_html}</div>
    {footer}
  </div>
</body>
</html>"#
    )
}

pub fn confirmation_email(
    config: &Config,
    lists: &[ListName],
    confirm_token: &str,
) -> OutgoingEmail {
    // Lands on the site's confirm page, which confirms on a button click. A link
    // that confirmed on GET would be "clicked" by mail scanners and link
    // prefetchers, subscribing addresses nobody opted in.
    let confirm_url = format!(
        "{}/newsletter/confirm?t={}",
        config.site_origin,
        encode_component(confirm_token)
    );
    let what = lists
        .iter()
        .map(|l| list_label(*l))
        .collect::<Vec<_>>()
        .join(" and ");

    let content = format!(
        r#"
      <p>You (or someone using this address) asked to subscribe to {what} on emnudge.dev.</p>
      <p>Confirm below — if this wasn't you, just ignore this email.</p>
      <p style="text-align: center; margin: 26px 0;"><a class="button" href="{confirm_url}">Confirm subscription</a></p>
      <p class="fallback">Or paste this link into your browser: {confirm_url}</p>"#
    );

    let html = shell(
        "Confirm your subscription",
        None,
        Some("One quick step"),
        None,
        &content,
        Some("You received this because this address was entered at emnudge.dev."),
    );

    let text = format!(
        "Confirm your subscription to {what} on emnudge.dev.\n\nConfirm: {confirm_url}\n\nIf this wasn't you, ignore this email."
    );

    OutgoingEmail {
        subject: "Confirm your subscription".to_string(),
        html,
        text,
        headers: Vec::new(),
    }
}

pub struct PostArgs<'a> {
    pub title: &'a str,
    pub author: &'a str,
    pub date: &'a str,
    pub list: ListName,
    pub url: &'a str,
    pub content_html: &'a str,
    pub content_text: &'a str,
    pub unsub_token: &'a str,
}

pub fn post_email(config: &Config, post: &PostArgs) -> OutgoingEmail {
    let subscribe_url = format!("{}/{}", config.site_origin, post.list.as_str());
    let unsub_url = format!(
        "{}/newsletter/unsubscribe?t={}",
        config.site_origin,
        encode_component(post.unsub_token)
    );
    let one_click_unsub = format!(
        "{}/unsubscribe?t={}",
        config.public_base,
        encode_component(post.unsub_token)
    );

    let byline = format!(
        "{} &middot; {}",
        escape_html(post.author),
        escape_html(&format_date(post.date))
    );
    let forward = format!(
        "Did someone forward you this? <a href=\"{}\">Subscribe to this newsletter</a>.",
        escape_html(&subscribe_url)
    );
    let footer = format!(
        "You're subscribed to {} on emnudge.dev. <a href=\"{}\">Unsubscribe</a>.",
        list_label(post.list),
        escape_html(&unsub_url)
    );

    let html = shell(
        post.title,
        Some(post.url),
        Some(&byline),
        Some(&forward),
        post.content_html,
        Some(&footer),
    );

    let text = format!(
        "{}\n{} · {}\n\n{}\n\n—\nRead online: {}\nUnsubscribe: {}",
        post.title,
        post.author,
        format_date(post.date),
        post.content_text,
        post.url,
        unsub_url
    );

    // One-click unsubscribe (RFC 8058) — POSTed directly to this service.
    let headers = vec![
        (
            "List-Unsubscribe".to_string(),
            format!("<{one_click_unsub}>"),
        ),
        (
            "List-Unsubscribe-Post".to_string(),
            "List-Unsubscribe=One-Click".to_string(),
        ),
    ];

    OutgoingEmail {
        subject: post.title.to_string(),
        html,
        text,
        headers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post<'a>(title: &'a str, author: &'a str, unsub_token: &'a str) -> PostArgs<'a> {
        PostArgs {
            title,
            author,
            date: "2026-09-29",
            list: ListName::Notes,
            url: "https://site.test/notes/p",
            content_html: "<p>body</p>",
            content_text: "body",
            unsub_token,
        }
    }

    #[test]
    fn encodes_like_encode_uri_component() {
        assert_eq!(encode_component("aZ09-_.!~*'()"), "aZ09-_.!~*'()");
        assert_eq!(encode_component("a b/+=&?#"), "a%20b%2F%2B%3D%26%3F%23");
        assert_eq!(encode_component("é"), "%C3%A9");
    }

    #[test]
    fn formats_iso_dates_and_passes_through_the_rest() {
        assert_eq!(format_date("2026-09-29"), "September 29, 2026");
        assert_eq!(format_date("2026-09-29T10:00:00Z"), "September 29, 2026");
        // new-post.sh writes this shape; it must survive untouched.
        assert_eq!(format_date("Sep 18, 2026"), "Sep 18, 2026");
    }

    #[test]
    fn post_email_escapes_admin_supplied_text() {
        let config = Config::for_tests();
        let mail = post_email(&config, &post("<b>T</b> & co", "<i>me</i>", "tok"));
        assert!(mail.html.contains("&lt;b&gt;T&lt;/b&gt; &amp; co"));
        assert!(mail.html.contains("&lt;i&gt;me&lt;/i&gt;"));
        assert!(!mail.html.contains("<b>T</b>"));
        assert!(
            mail.html.contains("<p>body</p>"),
            "rendered body is not escaped"
        );
        assert_eq!(mail.subject, "<b>T</b> & co", "subject is plain text");
    }

    #[test]
    fn post_email_carries_working_unsubscribe_links() {
        let config = Config::for_tests();
        let mail = post_email(&config, &post("T", "me", "a+b/c"));
        let unsub = "https://site.test/newsletter/unsubscribe?t=a%2Bb%2Fc";
        assert!(mail.html.contains(unsub));
        assert!(mail.text.contains(unsub));
        assert!(mail.text.contains("Read online: https://site.test/notes/p"));

        let header = |name: &str| {
            mail.headers
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(
            header("List-Unsubscribe"),
            Some("<https://api.site.test/unsubscribe?t=a%2Bb%2Fc>")
        );
        assert_eq!(
            header("List-Unsubscribe-Post"),
            Some("List-Unsubscribe=One-Click")
        );
    }

    #[test]
    fn confirmation_links_to_the_site_page_not_the_service() {
        let config = Config::for_tests();
        let mail = confirmation_email(&config, &[ListName::Blog, ListName::Notes], "tok");
        let link = "https://site.test/newsletter/confirm?t=tok";
        assert!(mail.html.contains(link));
        assert!(mail.text.contains(link));
        assert!(!mail.text.contains(&config.public_base));
        assert!(mail.text.contains("the blog and notes"));
        assert!(mail.headers.is_empty());
    }
}
