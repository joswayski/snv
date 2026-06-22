use std::{
    fs::File,
    io::{self, BufRead},
};

pub enum SnvErrors {
    FileLoadError,
}

fn unescape_chars(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars();

    while let Some(char) = chars.next() {
        // If it's a normal character, continue
        if char != '\\' {
            output.push(char);
            continue;
        }

        // Check the next character
        match chars.next() {
            Some('n') => output.push('\n'),
            Some('t') => output.push('\t'),
            Some('r') => output.push('\r'),
            Some('"') => output.push('"'),
            Some('\'') => output.push('\''),
            Some('\\') => output.push('\\'),
            Some(other_value) => {
                // Unhandled case, push as is
                output.push('\\');
                output.push(other_value)
            }
            // end
            None => output.push('\\'),
        }
    }

    output
}

fn parse_line(index: usize, line: &str) -> Option<(String, String)> {
    if line.trim().is_empty() || line.starts_with("#") {
        // Skip empty lines
        return None;
    }

    let Some((key, value)) = line.split_once('=') else {
        // Warn that we couldn't parse
        println!(
            "Unable to parse line number {} with value: '{}'. Did not find a '=' delimiter, make sure you include it like 'key=value'",
            index + 1,
            line
        );

        return None;
    };

    let normalized_key = key.trim().to_string();
    let normalized_value = normalize_value(value.trim());

    Some((normalized_key, normalized_value))
}

fn strip_wrapped_value(value: &str, wrapper: char, allow_escapes: bool) -> Option<&str> {
    if !value.starts_with(wrapper) {
        return None;
    }
    // Check for inline comments
    let mut escaped = false;
    let mut final_index = None;

    // Get the next unescaped quote, and treat everything after as a comment
    // Skip the first as we know it's not that one

    for (idx, char) in value.char_indices().skip(1) {
        if allow_escapes && escaped {
            escaped = false;
            continue;
        }

        if allow_escapes && char == '\\' {
            // Let it fall through on the next pass
            escaped = true;
            continue;
        }

        if char == wrapper {
            // Not escaped, this is the final char
            final_index = Some(idx);
            break;
        }
    }

    if final_index.is_some() {
        // Get the actual value in between the quotes
        return Some(&value[1..final_index.unwrap()]);
    }

    return None;
}
fn normalize_value(value: &str) -> String {
    let value = value.trim();

    if let Some(stripped_value) = strip_wrapped_value(value, '"', true) {
        return unescape_chars(stripped_value);
    };

    if let Some(stripped_value) = strip_wrapped_value(value, '\'', false) {
        return stripped_value.to_string();
    };

    value.to_string()
}

pub fn load() -> Result<(), std::io::Error> {
    load_from(".env")
}

pub fn load_from(file_path: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
    let file_path = file_path.as_ref();

    let file = File::open(file_path)?;

    let reader = io::BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        match line {
            Ok(line) => {
                if let Some((k, v)) = parse_line(index, &line) {
                    unsafe {
                        std::env::set_var(k, v);
                    }
                };
            }
            Err(err) => {
                println!("An error occurred reading line {index}. Error: {err}");

                continue;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape_chars_newline() {
        let input = r#"hello\nworld"#;
        let expected_output = "hello\nworld";

        assert_eq!(unescape_chars(input), expected_output);
    }

    #[test]
    fn test_unescape_chars_tab() {
        let input = r#"hello\tworld"#;
        let expected_output = "hello\tworld";

        assert_eq!(unescape_chars(input), expected_output);
    }

    #[test]
    fn test_unescape_chars_return() {
        let input = r#"hello\rworld"#;
        let expected_output = "hello\rworld";

        assert_eq!(unescape_chars(input), expected_output);
    }

    #[test]
    fn test_unescape_chars_double_quote() {
        let input = r#"hello\"world"#;
        let expected_output = "hello\"world";

        assert_eq!(unescape_chars(input), expected_output);
    }

    #[test]
    fn test_unescape_chars_single_quote() {
        let input = r#"hello\'world"#;
        let expected_output = "hello'world";

        assert_eq!(unescape_chars(input), expected_output);
    }

    #[test]
    fn test_unescape_chars_backslash() {
        let input = r#"hello\\world"#;
        let expected_output = "hello\\world";

        assert_eq!(unescape_chars(input), expected_output);
    }

    #[test]
    fn test_unescape_chars_no_escape_chars() {
        let input = "hello world";
        let expected_output = "hello world";

        assert_eq!(unescape_chars(input), expected_output);
    }

    #[test]
    fn test_unescape_chars_trailing_backslash() {
        let input = r#"hello world\"#;
        let expected_output = r#"hello world\"#;

        assert_eq!(unescape_chars(input), expected_output);
    }

    #[test]
    fn test_unescape_chars_unknown_left_as_is() {
        let input = r#"hello\-world"#;
        let expected_output = r#"hello\-world"#;

        assert_eq!(unescape_chars(input), expected_output);
    }

    #[test]
    fn test_unescaped_chars_no_newline_on_backslash() {
        let input = r#"c:\\new-folder"#;
        let expected_output = r#"c:\new-folder"#;
        assert_eq!(unescape_chars(input), expected_output);
    }

    #[test]
    fn test_unescaped_chars_multiple_escapes() {
        let input = r#"hello\nworld\"jose here\""#;
        let expected_output = "hello\nworld\"jose here\"";
        assert_eq!(unescape_chars(input), expected_output);
    }

    #[test]
    fn test_parse_line_return_none_if_empty() {
        let input = "";
        assert_eq!(parse_line(0, input), None);
    }

    #[test]
    fn test_parse_line_return_none_starts_with_comment() {
        let input = "# how are you";
        assert_eq!(parse_line(0, input), None);
    }

    #[test]
    fn test_parse_line_return_none_if_no_delimitter() {
        let input = "API_KEYisEMPTY";
        assert_eq!(parse_line(0, input), None);
    }

    #[test]
    fn test_parse_line_happy_path_double_quotes() {
        let input = r#"API_KEY="beans""#;
        assert_eq!(
            parse_line(0, input),
            Some(("API_KEY".into(), "beans".into()))
        );
    }

    #[test]
    fn test_parse_line_happy_path_single_quotes() {
        let input = r#"API_KEY='beans'"#;
        assert_eq!(
            parse_line(0, input),
            Some(("API_KEY".into(), "beans".into()))
        );
    }

    #[test]
    fn test_parse_line_happy_path_unquoted() {
        let input = r#"API_KEY=beans and guac"#;
        assert_eq!(
            parse_line(0, input),
            Some(("API_KEY".into(), "beans and guac".into()))
        );
    }

    #[test]
    fn test_parse_inline_comments_double_quotes() {
        let input = r#"API_KEY="beans" # deprecated actually dont use this"#;
        assert_eq!(
            parse_line(0, input),
            Some(("API_KEY".into(), "beans".into()))
        );
    }
    #[test]
    fn test_parse_inline_comments_within_double_quotes() {
        let input = r#"API_KEY="beans #yeah " # deprecated actually dont use this"#;
        assert_eq!(
            parse_line(0, input),
            Some(("API_KEY".into(), "beans #yeah ".into()))
        );
    }

    #[test]
    fn test_parse_line_inline_comments_single_quotes() {
        let input = r#"API_KEY='beans' # deprecated actually dont use this"#;
        assert_eq!(
            parse_line(0, input),
            Some(("API_KEY".into(), "beans".into()))
        );
    }
}
