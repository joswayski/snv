use std::{
    fmt::format,
    fs::{self, File},
    io::{self, BufRead},
};

use sjl::{Logger, LoggerOptions, json};

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
                logger.info(format!("{line}"), ());
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
