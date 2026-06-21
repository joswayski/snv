use std::{
    ffi::OsStr,
    fs::File,
    io::{self, BufRead},
    os::unix::ffi::OsStrExt,
};

pub enum SnvErrors {
    FileLoadError,
}

pub fn load(file_name: Option<&str>) -> Result<(), SnvErrors> {
    let file_name = file_name.unwrap_or(".env");

    let file = match File::open(file_name) {
        Ok(file) => {
            println!("Found your {file_name} file!"); // ! TODO remove
            file
        }
        Err(e) => {
            println!("An error ocurred loading your {file_name} file! Error: {e}");
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
                    .strip_prefix('"')
                    .and_then(|v| v.strip_suffix('"'))
                {
                    // If double quoted, remove some escape strings
                    normalized_value = stripped_value
                        .replace("\\n", "\n")
                        .replace("\\t", "\t")
                        .replace("\\r", "\r")
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
