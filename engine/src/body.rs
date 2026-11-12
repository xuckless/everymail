//! Message bodies as complete HTML documents for a sandboxed iframe. Raw bytes, never protobuf.
//!
//! A body is asked for as `<message id>?theme=<system|dark|light>`, so the iframe reloads when the
//! theme changes and the document follows the engine's theme, not the system's. The content policy
//! allows nothing external and only the engine's own script, by a nonce new on every response. That
//! script posts the height of the body's content to the embedding page as a number of CSS pixels, on
//! load and on every resize, since a sandboxed iframe cannot be measured from outside.

use crate::proto::ui::v1::Theme;

// The app's text colour from design/tokens.json, as `TEXT_DARK` and `TEXT_LIGHT`.
include!(concat!(env!("OUT_DIR"), "/body_tokens.rs"));

/// The response header that sandboxes a body when its iframe carries no sandbox attribute.
pub const SANDBOX: &str = "sandbox allow-scripts";

const THEMES: [(Theme, &str); 3] = [
    (Theme::System, "system"),
    (Theme::Dark, "dark"),
    (Theme::Light, "light"),
];

const BASE: &str = "body{margin:0;display:flow-root;background:transparent;font:15px/1.7 system-ui,sans-serif;overflow-wrap:anywhere}p{margin:0 0 1em}p:last-child{margin-bottom:0}";
const HEIGHT: &str = "new ResizeObserver(()=>parent.postMessage(Math.ceil(document.body.getBoundingClientRect().height),'*')).observe(document.body)";

/// What `body_url` carries after its origin: `m-slides-4?theme=dark`.
pub fn request(message_id: &str, theme: Theme) -> String {
    let slug = THEMES
        .iter()
        .find(|(t, _)| *t == theme)
        .map_or("system", |(_, slug)| slug);
    format!("{message_id}?theme={slug}")
}

/// The message id and theme of a request; a missing or unknown theme is System.
pub fn parse(request: &str) -> (&str, Theme) {
    let (id, query) = request.split_once('?').unwrap_or((request, ""));
    let theme = query
        .strip_prefix("theme=")
        .and_then(|slug| THEMES.iter().find(|(_, s)| *s == slug))
        .map_or(Theme::System, |(theme, _)| *theme);
    (id, theme)
}

fn colours(theme: Theme) -> String {
    match theme {
        Theme::Dark => format!(":root{{color-scheme:dark;color:{TEXT_DARK}}}"),
        Theme::Light => format!(":root{{color-scheme:light;color:{TEXT_LIGHT}}}"),
        Theme::System | Theme::Unspecified => format!(
            ":root{{color-scheme:light dark;color:{TEXT_LIGHT}}}@media(prefers-color-scheme:dark){{:root{{color:{TEXT_DARK}}}}}"
        ),
    }
}

/// A document of plain-text paragraphs; a line break inside a paragraph is kept.
pub fn document(paragraphs: &[&str], theme: Theme, nonce: &str) -> Vec<u8> {
    let mut html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; script-src 'nonce-{nonce}'; base-uri 'none'; form-action 'none'\"><style>{}{BASE}</style></head><body>",
        colours(theme)
    );
    for paragraph in paragraphs {
        html.push_str("<p>");
        for (i, line) in paragraph.split('\n').enumerate() {
            if i > 0 {
                html.push_str("<br>");
            }
            escape(line, &mut html);
        }
        html.push_str("</p>");
    }
    html.push_str(&format!(
        "<script nonce=\"{nonce}\">{HEIGHT}</script></body></html>"
    ));
    html.into_bytes()
}

fn escape(text: &str, html: &mut String) {
    for c in text.chars() {
        match c {
            '&' => html.push_str("&amp;"),
            '<' => html.push_str("&lt;"),
            '>' => html.push_str("&gt;"),
            '"' => html.push_str("&quot;"),
            _ => html.push(c),
        }
    }
}

/// 128 random bits, hex encoded.
pub fn nonce() -> Option<String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).ok()?;
    Some(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_round_trip() {
        for (theme, _) in THEMES {
            assert_eq!(parse(&request("m-1", theme)), ("m-1", theme));
        }
        assert_eq!(parse("m-1"), ("m-1", Theme::System));
        assert_eq!(parse("m-1?theme=sepia"), ("m-1", Theme::System));
    }
}
