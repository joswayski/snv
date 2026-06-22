use std::{
    ffi::OsStr,
    fs::File,
    io::{self, BufRead},
    os::unix::ffi::OsStrExt,
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

pub fn load(file_path: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
    let file_path = file_path.as_ref();
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            println!(
                "An error ocurred loading your file at: '{}'\nError: {e}",
                file_path.display()
            );
            return Err(e);
        }
    };

    let reader = io::BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        match line {
            Ok(line) => {
                if line.len() == 0 {
                    // Skip empty lines
                    continue;
                }

                let Some((key, value)) = line.split_once("=") else {
                    // Warn that we couldn't parse
                    println!(
                        "Unable to parse line number {} with value: '{}'. Did not find a '=' delimiter, make sure you include it like 'key=value'",
                        index + 1,
                        line
                    );

                    continue;
                };

                let normalized_key = key.trim();
                let value = value.trim();
                let mut normalized_value = value.to_string();

                // Remove wrapper quotes
                if let Some(stripped_value) = normalized_value
                    .strip_prefix("\"")
                    .and_then(|v| v.strip_suffix("\""))
                {
                    // If double quoted, remove some escape strings
                    normalized_value = unescape_chars(stripped_value)
                }

                if let Some(stripped_value) = normalized_value
                    .strip_prefix('\'')
                    .and_then(|v| v.strip_suffix('\''))
                {
                    normalized_value = stripped_value.to_string();
                }

                println!("{normalized_key}={normalized_value}");
                unsafe {
                    std::env::set_var(
                        OsStr::from_bytes(normalized_key.as_bytes()),
                        OsStr::from_bytes(normalized_value.as_bytes()),
                    );
                }
            }
            Err(err) => {
                println!("An error ocurred reading line {index}. Error: {err}");

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
}
