use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write, BufReader, BufWriter, BufRead};
use std::path::Path;

fn load_conversion_table(filename: &str) -> HashMap<String, u8> {
    let mut table = HashMap::new();
    let file = File::open(filename).expect("Unable to open conversionTable.txt");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Failed to read line").trim().to_string();
        if line.is_empty() || !line.contains("=") {
            continue;
        }
        let parts: Vec<&str> = line.split("=").collect();
        if parts.len() == 2 {
            let key = parts[0].trim().to_string();
            let value_str = parts[1].trim();
            let byte_val = u8::from_str_radix(value_str, 16).expect("Invalid hex value in conversionTable.txt");
            table.insert(key, byte_val);
        }
    }
    table
}

fn get_op_value(hex1: u8, result: u8) -> Option<String> {
    for i in 0..=255 {
        if (hex1.wrapping_add(i)) == result {
            return Some(format!("+{:02X}", i));
        }
        if (hex1.wrapping_sub(i)) == result {
            return Some(format!("-{:02X}", i));
        }
    }
    None
}

fn encrypt_byte(random_byte: u8, input_byte: u8, table: &HashMap<String, u8>) -> u8 {
    let op_val = get_op_value(random_byte, input_byte)
        .expect("Could not find op_value for encryption");

    *table.get(&op_val).unwrap_or(&0xFF) // Default to 0xFF if key not found, similar to Java
}

fn main() -> io::Result<()> {
    let random_file_path = Path::new("random_bytes.bin");
    let input_file_path = Path::new("input_bytes.bin");
    let table_file = "conversionTable.txt";
    let encrypted_file = "changes.bin";
    let temp_dir = Path::new(".temp_random_bytes");
    let used_random_bytes_path = temp_dir.join("used_random_bytes.bin");

    // 1. Determine input_bytes.bin size
    let input_metadata = fs::metadata(input_file_path)?;
    let input_size = input_metadata.len();

    // 2. Read random_bytes.bin entirely
    let mut random_bytes_full = fs::read(random_file_path)?;
    let random_bytes_full_len = random_bytes_full.len() as u64;

    // 3. Check size
    if random_bytes_full_len < input_size {
        eprintln!("Error: random_bytes.bin is smaller than input_bytes.bin. Cannot encrypt.");
        return Ok(());
    }

    // 4. Extract used random bytes
    let used_random_bytes: Vec<u8> = random_bytes_full.drain(0..input_size as usize).collect();

    // 5. Create .temp_random_bytes directory
    if !temp_dir.exists() {
        fs::create_dir(temp_dir)?;
    }

    // 6. Move used random bytes to temp folder
    fs::write(&used_random_bytes_path, &used_random_bytes)?;
    println!("Moved {} used random bytes to {:?}", used_random_bytes.len(), used_random_bytes_path);

    // 7. Update random_bytes.bin with remaining bytes
    if random_bytes_full.is_empty() {
        fs::remove_file(random_file_path)?;
        println!("random_bytes.bin is now empty and has been removed.");
    } else {
        fs::write(random_file_path, &random_bytes_full)?;
        println!("Updated random_bytes.bin with {} remaining bytes.", random_bytes_full.len());
    }

    // Encryption logic using used_random_bytes
    let conversion_table = load_conversion_table(table_file);

    let mut input_stream = BufReader::new(File::open(input_file_path)?);
    let mut output_stream = BufWriter::new(File::create(encrypted_file)?);

    let mut input_buffer = vec![0; input_size as usize];
    input_stream.read_exact(&mut input_buffer)?;

    let mut encrypted_buffer = vec![0; input_size as usize];

    for i in 0..input_size as usize {
        encrypted_buffer[i] = encrypt_byte(
            used_random_bytes[i],
            input_buffer[i],
            &conversion_table,
        );
    }
    output_stream.write_all(&encrypted_buffer)?;
    println!("Processed {} bytes... Done!", input_size);

    Ok(())
}