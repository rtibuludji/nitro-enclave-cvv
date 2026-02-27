
pub fn hexdump(bytes: &[u8]) {
    hexdump_with_options(bytes, 16, true)
}

pub fn hexdump_cols(bytes: &[u8], bytes_per_line: usize) {
    hexdump_with_options(bytes, bytes_per_line, true)
}

pub fn hexdump_no_ascii(bytes: &[u8]) {
    hexdump_with_options(bytes, 16, false)
}

pub fn hexdump_with_options(bytes: &[u8], bytes_per_line: usize, show_ascii: bool) {
    for (i, chunk) in bytes.chunks(bytes_per_line).enumerate() {
        print!("{:08x}  ", i * bytes_per_line);
        
        for (j, byte) in chunk.iter().enumerate() {
            print!("{:02x}", byte);

            if bytes_per_line == 16 && j == 7 {
                print!(" ");
            }
            print!(" ");
        }

        let padding = bytes_per_line - chunk.len();
        for j in 0..padding {
            if bytes_per_line == 16 && chunk.len() + j == 8 {
                print!(" ");
            }
            print!("   ");
        }

        if show_ascii {
            print!(" |");
            for byte in chunk {
                if byte.is_ascii_graphic() || *byte == b' ' {
                    print!("{}", *byte as char);
                } else {
                    print!(".");
                }
            }
            print!("|");
        }
        
        println!();
    }
}

pub fn hexdump_string(bytes: &[u8]) -> String {
    let mut result = String::new();
    
    for (i, chunk) in bytes.chunks(16).enumerate() {

        result.push_str(&format!("{:08x}  ", i * 16));

        for (j, byte) in chunk.iter().enumerate() {
            result.push_str(&format!("{:02x}", byte));
            if 16 == 16 && j == 7 {
                result.push(' ');
            }
            result.push(' ');
        }

        let padding = 16 - chunk.len();
        for j in 0..padding {
            if chunk.len() + j == 8 {
                result.push(' ');
            }
            result.push_str("   ");
        }

        result.push_str(" |");
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                result.push(*byte as char);
            } else {
                result.push('.');
            }
        }
        result.push_str("|\n");
    }
    
    result
}
