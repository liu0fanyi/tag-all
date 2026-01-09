//! Custom Markdown Parser
//!
//! Extends pulldown-cmark with custom color syntax: %r%red%r%, %g%green%g%, etc.
//! Uses Event stream interception to inject HTML spans for colored text.
//! 
//! Syntax uses %X% delimiters to avoid conflict with standard Markdown emphasis (* or _).

use pulldown_cmark::{Parser, Options, Event, CowStr, html::push_html};

/// Color codes and their hex values
const COLORS: &[(&str, &str)] = &[
    ("r", "#e74c3c"), // red
    ("g", "#27ae60"), // green
    ("b", "#3498db"), // blue
    ("y", "#f1c40f"), // yellow
    ("o", "#e67e22"), // orange
    ("p", "#9b59b6"), // purple
];

/// Parse markdown with custom color syntax support
/// 
/// Supports standard markdown (bold, italic, strikethrough, code, etc.)
/// Plus custom color syntax: %r%red text%r%, %g%green%g%, etc.
pub fn parse_markdown(text: &str) -> String {
    let options = Options::ENABLE_STRIKETHROUGH 
        | Options::ENABLE_TABLES 
        | Options::ENABLE_TASKLISTS;
    
    // Parse the raw text directly - no pre-processing needed
    // because %X% is not standard markdown syntax
    let parser = Parser::new_ext(text, options);
    
    // Transform events to handle custom color syntax
    let events = transform_color_syntax(parser);
    
    let mut html_output = String::new();
    push_html(&mut html_output, events.into_iter());
    html_output
}

/// Parse markdown for inline use (strips outer <p> tags)
pub fn parse_markdown_inline(text: &str) -> String {
    let html = parse_markdown(text);
    
    // Strip outer <p> tags for inline display
    html.trim()
        .strip_prefix("<p>")
        .and_then(|s| s.strip_suffix("</p>"))
        .map(|s| s.to_string())
        .unwrap_or(html)
}

/// Transform parser events to handle custom color syntax in text nodes
fn transform_color_syntax<'a>(parser: Parser<'a>) -> Vec<Event<'a>> {
    let mut events = Vec::new();
    
    for event in parser {
        match event {
            Event::Text(text) => {
                // Check if text contains any color markers
                if contains_color_marker(&text) {
                    // Split the text event into multiple events (HTML, Text, HTML...)
                    let new_events = process_colored_text_events(&text);
                    events.extend(new_events);
                } else {
                    events.push(Event::Text(text));
                }
            }
            other => events.push(other),
        }
    }
    
    events
}

/// Check if text contains any color markers
fn contains_color_marker(text: &str) -> bool {
    COLORS.iter().any(|(code, _)| {
        let pattern = format!("%{}%", code);
        text.contains(&pattern)
    })
}

/// Process text with color markers and return a sequence of Events
fn process_colored_text_events(text: &str) -> Vec<Event<'static>> {
    let mut events = Vec::new();
    let mut remaining = text.to_string();
    
    while !remaining.is_empty() {
        // Find the earliest color marker
        let mut earliest_match: Option<(usize, &str, &str)> = None;
        
        for (code, color) in COLORS {
            let pattern = format!("%{}%", code);
            if let Some(pos) = remaining.find(&pattern) {
                if earliest_match.is_none() || pos < earliest_match.unwrap().0 {
                    earliest_match = Some((pos, *code, *color));
                }
            }
        }
        
        match earliest_match {
            Some((pos, code, color)) => {
                let pattern = format!("%{}%", code);
                
                // Add text before the marker
                if pos > 0 {
                    events.push(Event::Text(CowStr::from(remaining[..pos].to_string())));
                }
                remaining = remaining[pos + pattern.len()..].to_string();
                
                // Find closing marker
                if let Some(end_pos) = remaining.find(&pattern) {
                    // Start color span
                    events.push(Event::Html(CowStr::from(format!("<span style=\"color: {}\">", color))));
                    
                    // Add inner content (as text for now, could recurse if needed but keeping simple)
                    if end_pos > 0 {
                         events.push(Event::Text(CowStr::from(remaining[..end_pos].to_string())));
                    }
                    
                    // End color span
                    events.push(Event::Html(CowStr::from("</span>".to_string())));
                    
                    remaining = remaining[end_pos + pattern.len()..].to_string();
                } else {
                    // No closing marker, treat opening marker as text
                    events.push(Event::Text(CowStr::from(pattern)));
                }
            }
            None => {
                // No more markers, match remaining text
                events.push(Event::Text(CowStr::from(remaining)));
                break;
            }
        }
    }
    
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_color_syntax() {
        let input = "%r%red text%r% normal %g%green%g%";
        let output = parse_markdown_inline(input);
        
        // Output should contain spans with colors
        assert!(output.contains("color: #e74c3c")); // red
        assert!(output.contains("color: #27ae60")); // green
        assert!(output.contains(">red text<"));
        assert!(!output.contains("%r%")); // markers removed
    }
    
    #[test]
    fn test_bold_italic() {
        // Standard markdown should still work
        let input = "**bold** and *italic*";
        let output = parse_markdown_inline(input);
        assert!(output.contains("<strong>bold</strong>"));
        assert!(output.contains("<em>italic</em>"));
    }
    
    #[test]
    fn test_color_and_bold() {
        // Mixing should work seamlessly because pulldown handles structure
        // Note: nesting color inside bold works like: **%r%text%r%**
        let input = "**%r%bold red%r%**";
        let output = parse_markdown_inline(input);
        assert!(output.contains("<strong>"));
        assert!(output.contains("color: #e74c3c"));
        assert!(output.contains("bold red"));
    }
}
