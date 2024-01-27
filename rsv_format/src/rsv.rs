pub mod rsv_format {
    pub const EOV: u8 = 0xFF; // End-Of-Value separator
    pub const EOR: u8 = 0xFD; // End-Of-Row separator
    pub const NULL: u8 = 0xFE; // Null value encoding

    pub mod rsv_writer {
        use std::fs;
        use std::path::Path;

        use serde_json::{Error, Value};

        use super::{EOR, EOV, NULL};

        fn convert_vec_to_rsv(table_data: Vec<Vec<Option<String>>>) -> Vec<u8> {
            let mut binary_table_encoding = Vec::new();

            for row in table_data.iter() {
                for element in row {
                    if let Some(value) = element {
                        binary_table_encoding.extend_from_slice(value.as_bytes());
                    } else {
                        binary_table_encoding.push(NULL);
                    }
                    binary_table_encoding.push(EOV);
                }
                binary_table_encoding.push(EOR);
            }
            binary_table_encoding
        }

        pub fn from_vec(
            table_data: Vec<Vec<Option<String>>>,
            output_path: &Path,
        ) -> Result<(), std::io::Error> {
            let binary_table_encoding = convert_vec_to_rsv(table_data);

            fs::write(output_path, binary_table_encoding)
        }

        pub fn from_json(json_path: &Path, output_path: &Path) -> Result<(), std::io::Error> {
            let json_content = fs::read_to_string(json_path)?;
            let table_data: Vec<Vec<Option<String>>> = serde_json::from_str(json_content.as_str())?;
            from_vec(table_data, output_path)
        }
    }
    pub mod rsv_reader {
        use std::fs;
        use std::path::Path;

        use super::{EOR, EOV, NULL};

        pub fn to_vec(rsv_path: &Path) -> Result<Vec<Vec<Option<String>>>, std::io::Error> {
            let rsv_content = fs::read(rsv_path)?;
            let mut rsv_table: Vec<Vec<Option<String>>> = Vec::new();
            let mut rsv_row: Vec<Option<String>> = Vec::new();
            let mut curr_bytes_sequence: Vec<u8> = Vec::new();
            let mut last_is_null = false;
            for byte in rsv_content {
                match byte {
                    EOV => {
                        if last_is_null {
                            last_is_null = false;
                            continue;
                        }
                        rsv_row.push(Some(
                            String::from_utf8(curr_bytes_sequence.clone()).map_err(|e| {
                                std::io::Error::new(std::io::ErrorKind::InvalidData, e)
                            })?,
                        ));
                        curr_bytes_sequence.clear();
                    }
                    EOR => {
                        rsv_table.push(rsv_row.clone());
                        rsv_row.clear();
                    }
                    NULL => {
                        last_is_null = true;
                        rsv_row.push(None);
                    }
                    _ => {
                        curr_bytes_sequence.push(byte);
                    }
                }
            }

            Ok(rsv_table)
        }

        pub fn to_json(rsv_path: &Path, json_path: &Path) -> Result<(), std::io::Error> {
            let rsv_table = to_vec(rsv_path)?;
            let json_string = serde_json::to_string(&rsv_table).expect("Failed to serialize to JSON");
            fs::write(json_path, json_string.as_bytes())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::rsv_format::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_from_vec() {
        let table_data = vec![
            vec![
                Some(String::from("A")),
                Some(String::from("B")),
                Some(String::from("Hello")),
                Some(String::from("Word")),
            ],
            vec![],
            vec![Some(String::from("C")), None, Some(String::from("D"))],
        ];

        let path = Path::new("src/from_vec_test.rsv");
        let result = rsv_writer::from_vec(table_data, path);

        assert!(result.is_ok());

        // Check if the file was created and contains the expected content
        let file_content = fs::read(path).expect("Failed to read file");
        assert_eq!(
            file_content,
            vec![
                b'A', EOV, b'B', EOV, b'H', b'e', b'l', b'l', b'o', EOV, b'W', b'o', b'r', b'd',
                EOV, EOR, EOR, b'C', EOV, NULL, EOV, b'D', EOV, EOR
            ]
        );
    }

    #[test]
    fn test_from_json1() {
        let json_path = Path::new("src/from_json_test1.json");
        let output_path = Path::new("src/from_json_test1.rsv");
        let result = rsv_writer::from_json(json_path, output_path);

        assert!(result.is_ok());

        // Check if the file was created and contains the expected content
        let file_content = fs::read(output_path).expect("Failed to read file");
        assert_eq!(
            file_content,
            vec![72, 101, 108, 108, 111, 255, 240, 159, 140, 142, 255, 253]
        );
    }

    #[test]
    fn test_from_json2() {
        let json_path = Path::new("src/from_json_test2.json");
        let output_path = Path::new("src/from_json_test2.rsv");
        let result = rsv_writer::from_json(json_path, output_path);

        assert!(result.is_ok());

        // Check if the file was created and contains the expected content
        let file_content = fs::read(output_path).expect("Failed to read file");
        assert_eq!(
            file_content,
            vec![
                72, 101, 108, 108, 111, 255, 240, 159, 140, 142, 255, 253, 253, 254, 255, 255, 253
            ]
        );
    }

    #[test]
    fn test_to_vec() {
        let rsv_path = Path::new("src/to_vec_test.rsv");
        let result = rsv_reader::to_vec(rsv_path);

        assert!(result.is_ok());

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Some(String::from("Hello")), Some(String::from("🌎"))],
                vec![],
                vec![None, Some(String::from(""))]
            ]
        );
    }

    #[test]
    fn test_to_json() {
        let rsv_path = Path::new("src/to_vec_test.rsv");
        let json_path = Path::new("src/to_json_test.json");
        let result = rsv_reader::to_json(rsv_path, json_path);

        assert!(result.is_ok());
    }
}
