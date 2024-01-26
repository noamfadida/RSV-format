pub mod rsv_format {
    pub const EOV: u8 = 0xFF; // End-Of-Value separator
    pub const EOR: u8 = 0xFD; // End-Of-Row separator
    pub const NULL: u8 = 0xFE; // Null value encoding

    pub mod rsv_writer {
        use std::fs;
        use std::path::Path;

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
}
