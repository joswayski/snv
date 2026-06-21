use std::{
    ffi::OsStr,
    fs::File,
    io::{self, BufRead},
    os::unix::ffi::OsStrExt,
};

use sjl::{LogLevel::Info, Logger, json};

pub enum SnvErrors {
    FileLoadError,
}

pub fn load(file_name: Option<&str>) -> Result<(), SnvErrors> {
    let logger = Logger::new();
    let file_name = file_name.unwrap_or(".env");

    let file = match File::open(file_name) {
        Ok(file) => {
            logger.info(format!("Found your {file_name} file!"), ());
            file
        }
        Err(e) => {
            logger.error(
                format!("An error ocurred loading your {file_name} file!"),
                json!({"error": e.to_string(), "file_name": file_name}),
            );
            return Err(SnvErrors::FileLoadError);
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
                    logger.warn(
                        format!("Unable to parse line number {} with value: '{}'. Did not find a '=' delimiter, make sure you include it like 'key=value'", index + 1, line), ()
                    );
                    continue;
                };

                let normalized_key = key.trim();
                let mut normalized_value = value.trim();

                // Remove wrapper quotes
                if let Some(stripped_value) = normalized_value
                    .strip_prefix('"')
                    .and_then(|v| v.strip_suffix('"'))
                {
                    normalized_value = stripped_value
                }

                if let Some(stripped_value) = normalized_value
                    .strip_prefix("'")
                    .and_then(|v| v.strip_suffix("'"))
                {
                    normalized_value = stripped_value
                }

                logger.info("ENV LINE", json!({normalized_key: normalized_value}));
                unsafe {
                    std::env::set_var(
                        OsStr::from_bytes(normalized_key.as_bytes()),
                        OsStr::from_bytes(normalized_value.as_bytes()),
                    );
                }
            }
            Err(err) => {
                logger.error(
                    format!("An error ocurred reading line {}", index),
                    json!({"error": err.to_string()}),
                );

                continue;
            }
        }
    }

    Ok(())
}
