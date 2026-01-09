//! Custom Markdown Parser
//!
//! Extends pulldown-cmark with:
//! - Custom color syntax: %r%red%r%
//! - Math support: $E=mc^2$ (Katex)
//! - Syntax highlighting (syntect)
//! - Enhanced media:
//!   - Videos (<video> tag for mp4/webm/mov/mkv)
//!   - Local file access (asset:// protocol)

use pulldown_cmark::{Parser, Options, Event, CowStr, Tag, TagEnd, CodeBlockKind, html::push_html};
use std::sync::OnceLock;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::html::highlighted_html_for_string;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC, CONTROLS, AsciiSet};

/// Syntax highlighter resources (lazy loaded)
static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(|| SyntaxSet::load_defaults_newlines())
}

fn get_theme() -> &'static Theme {
    THEME_SET.get_or_init(ThemeSet::load_defaults).themes.get("InspiredGitHub").expect("Theme not found")
}

/// Color codes and their hex values
const COLORS: &[(&str, &str)] = &[
    ("r", "#e74c3c"), // red
    ("g", "#27ae60"), // green
    ("b", "#3498db"), // blue
    ("y", "#f1c40f"), // yellow
    ("o", "#e67e22"), // orange
    ("p", "#9b59b6"), // purple
];

/// Parse markdown with all extensions enabled
pub fn parse_markdown(text: &str) -> String {
    let parser = Parser::new_ext(text, get_options());
    let events = transform_events(parser);
    let mut html_output = String::new();
    push_html(&mut html_output, events.into_iter());
    html_output
}

/// Parse markdown for inline use (strips outer <p> tags)
pub fn parse_markdown_inline(text: &str) -> String {
    let html = parse_markdown(text);
    
    html.trim()
        .strip_prefix("<p>")
        .and_then(|s| s.strip_suffix("</p>"))
        .map(|s| s.to_string())
        .unwrap_or(html)
}

fn get_options() -> Options {
    Options::ENABLE_STRIKETHROUGH 
        | Options::ENABLE_TABLES 
        | Options::ENABLE_TASKLISTS
}

// State for the event transformer
enum State {
    Normal,
    InCodeBlock { lang: Option<String>, content: String },
    InVideo { dropped_depth: usize },
}

/// Transform parser events to handle all custom features
fn transform_events<'a>(parser: Parser<'a>) -> Vec<Event<'a>> {
    let mut events = Vec::new();
    let mut state = State::Normal;
    
    for event in parser {
        match state {
            State::Normal => {
                match event {
                    // --- Code Blocks (Highlighting) ---
                    Event::Start(Tag::CodeBlock(kind)) => {
                        let lang = match kind {
                            CodeBlockKind::Fenced(l) => Some(l.to_string()),
                            CodeBlockKind::Indented => None,
                        };
                        state = State::InCodeBlock { lang, content: String::new() };
                    }
                    
                    // --- Media (Images & Videos) ---
                    Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
                        let url = convert_local_url(&dest_url);
                        
                        if is_video_url(&url) {
                            // Render <video> tag
                            let html = format!(
                                r#"<video controls src="{}" style="max-width: 100%; max-height: 400px; display: block; border-radius: 4px;"></video>"#, 
                                url
                            );
                            events.push(Event::Html(CowStr::from(html)));
                            state = State::InVideo { dropped_depth: 0 };
                        } else {
                            // Render image with max-width constraint
                            let html = format!(
                                r#"<img src="{}" style="max-width: 100%; max-height: 400px; display: block; border-radius: 4px; cursor: pointer;" />"#,
                                url
                            );
                            events.push(Event::Html(CowStr::from(html)));
                            state = State::InVideo { dropped_depth: 0 }; // Drop the alt text events
                        }
                    }
                    
                    // --- Custom Colors (%r%) AND Math ($$) ---
                    Event::Text(text) => {
                         if contains_special_syntax(&text) {
                            events.extend(process_text_events(&text));
                        } else {
                            events.push(Event::Text(text));
                        }
                    }
                    
                    other => events.push(other),
                }
            }
            
            State::InCodeBlock { ref mut lang, ref mut content } => {
                match event {
                    Event::Text(t) => content.push_str(&t),
                    Event::End(TagEnd::CodeBlock) => {
                        let html = highlight_code(content, lang.as_deref());
                        events.push(Event::Html(CowStr::from(html)));
                        state = State::Normal;
                    }
                    _ => {} // Ignore likely
                }
            }
            
            State::InVideo { ref mut dropped_depth } => {
                match event {
                    Event::Start(_) => *dropped_depth += 1,
                    Event::End(_) => {
                        if *dropped_depth == 0 {
                            state = State::Normal;
                        } else {
                            *dropped_depth -= 1;
                        }
                    }
                    _ => {} 
                }
            }
        }
    }
    
    events
}

fn highlight_code(code: &str, lang: Option<&str>) -> String {
    let ss = get_syntax_set();
    let theme = get_theme();
    
    let syntax = lang
        .and_then(|l| ss.find_syntax_by_token(l))
        .unwrap_or_else(|| ss.find_syntax_plain_text());
        
    highlighted_html_for_string(code, ss, syntax, theme)
        .unwrap_or_else(|_| format!("<pre><code>{}</code></pre>", escape_html(code)))
}

// function convert_local_url
const PATH_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}');

fn convert_local_url(url: &str) -> String {
    let lower = url.to_lowercase();
    // Check if it's a local file path
    let is_local = lower.starts_with("c:") || lower.starts_with("d:") || lower.starts_with("e:") 
        || lower.starts_with("f:") || lower.starts_with("g:") || lower.starts_with("z:") 
        || (url.starts_with('/') && !url.starts_with("//"));
        
    if is_local {
        // Use https scheme for Windows compatibility now that assetProtocol is enabled
        // Handles C:/Users... -> https://asset.localhost/C:/Users...
        let normalized = url.replace('\\', "/");
        
        let encoded = utf8_percent_encode(&normalized, PATH_ENCODE_SET).to_string();
        format!("http://asset.localhost/{}", encoded)
    } else {
        url.to_string()
    }
}

fn is_video_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.ends_with(".mp4") || lower.ends_with(".webm") || lower.ends_with(".mov") || lower.ends_with(".mkv")
}

fn contains_special_syntax(text: &str) -> bool {
    text.contains('$') || COLORS.iter().any(|(code, _)| {
        text.contains(&format!("%{}%", code))
    })
}

// Process text for colors and math
fn process_text_events(text: &str) -> Vec<Event<'static>> {
    let mut events = Vec::new();
    let mut remaining = text.to_string();
    
    while !remaining.is_empty() {
        let mut earliest_match: Option<(usize, String, MatchType)> = None; // pos, pattern, type
        
        // 1. Check for Display Math $$
        if let Some(pos) = remaining.find("$$") {
             if earliest_match.as_ref().map_or(true, |m| pos < m.0) {
                earliest_match = Some((pos, "$$".to_string(), MatchType::DisplayMath));
            }
        }
        
        // 2. Check for Inline Math $ (if not overridden by $$)
        if let Some(pos) = remaining.find('$') {
            let is_start_of_display = remaining[pos..].starts_with("$$");
            if !is_start_of_display {
                 if earliest_match.as_ref().map_or(true, |m| pos < m.0) {
                    earliest_match = Some((pos, "$".to_string(), MatchType::InlineMath));
                }
            }
        }
        
        // 3. Check for Colors
        for (code, color) in COLORS {
            let pattern = format!("%{}%", code);
            if let Some(pos) = remaining.find(&pattern) {
                if earliest_match.as_ref().map_or(true, |m| pos < m.0) {
                    earliest_match = Some((pos, pattern.clone(), MatchType::Color(color.to_string(), pattern)));
                }
            }
        }
        
        match earliest_match {
            Some((pos, _pattern, match_type)) => {
                // Add text before marker
                if pos > 0 {
                    events.push(Event::Text(CowStr::from(remaining[..pos].to_string())));
                }
                
                match match_type {
                    MatchType::DisplayMath => {
                         remaining = remaining[pos + 2..].to_string();
                         if let Some(end_pos) = remaining.find("$$") {
                             // Emit as HTML to prevent escaping
                             let content = &remaining[..end_pos];
                             events.push(Event::Html(CowStr::from(format!("$${}$$", content))));
                             remaining = remaining[end_pos + 2..].to_string();
                         } else {
                             events.push(Event::Text(CowStr::from("$$")));
                         }
                    }
                    MatchType::InlineMath => {
                         remaining = remaining[pos + 1..].to_string();
                         if let Some(end_pos) = remaining.find('$') {
                             let content = &remaining[..end_pos];
                             events.push(Event::Html(CowStr::from(format!("${}$", content))));
                             remaining = remaining[end_pos + 1..].to_string();
                         } else {
                             events.push(Event::Text(CowStr::from("$")));
                         }
                    }
                    MatchType::Color(color, pattern) => {
                        remaining = remaining[pos + pattern.len()..].to_string();
                        if let Some(end_pos) = remaining.find(&pattern) {
                            events.push(Event::Html(CowStr::from(format!("<span style=\"color: {}\">", color))));
                            if end_pos > 0 {
                                events.push(Event::Text(CowStr::from(remaining[..end_pos].to_string())));
                            }
                            events.push(Event::Html(CowStr::from("</span>".to_string())));
                            remaining = remaining[end_pos + pattern.len()..].to_string();
                        } else {
                            events.push(Event::Text(CowStr::from(pattern)));
                        }
                    }
                }
            }
            None => {
                events.push(Event::Text(CowStr::from(remaining)));
                break;
            }
        }
    }
    events
}

#[derive(Clone)]
enum MatchType {
    DisplayMath,
    InlineMath,
    Color(String, String), // color_hex, pattern
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Trigger Katex rendering (calls window.renderMathInElement)
pub fn trigger_math_render(selector: &str) {
    use leptos::task::spawn_local;
    let selector = selector.to_string();
    
    spawn_local(async move {
        let js_code = format!(r#"
            (function() {{
                var attempts = 0;
                var maxAttempts = 50; 
                
                function tryRender() {{
                    if (window.renderMathInElement) {{
                        var el = document.querySelector('{}');
                        if (!el) return;
                        
                        try {{
                            window.renderMathInElement(el, {{
                                delimiters: [
                                    {{left: '$$', right: '$$', display: true}},
                                    {{left: '$', right: '$', display: false}},
                                    {{left: '\\(', right: '\\)', display: false}},
                                    {{left: '\\[', right: '\\]', display: true}}
                                ],
                                throwOnError: false
                            }});
                            // console.log('Katex rendered successfully on {}');
                        }} catch(e) {{
                            console.error('Katex render error:', e);
                        }}
                    }} else {{
                        attempts++;
                        if (attempts < maxAttempts) {{
                            setTimeout(tryRender, 200);
                        }}
                    }}
                }}
                tryRender();
            }})();
        "#, selector, selector);
        
        let _ = js_sys::eval(&js_code);
    });
}
