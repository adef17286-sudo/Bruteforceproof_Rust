use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write, BufReader, BufWriter, BufRead};
use std::path::Path;

fn load_reverse_table(filename: &str) -> HashMap<u8, String> {
    let mut reverse_table = HashMap::new();
    let mut plus_table = HashMap::new(); // To handle precedence for '+' entries

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

            if key.starts_with('+') {
                let current_plus_val = plus_table.get(&byte_val)
                    .and_then(|s: &String| u8::from_str_radix(&s[1..], 16).ok());
                let new_plus_val = u8::from_str_radix(&key[1..], 16).expect("Invalid hex in key");

                if current_plus_val.is_none() || new_plus_val < current_plus_val.unwrap() {
                    plus_table.insert(byte_val, key);
                }
            } else {
                if !plus_table.contains_key(&byte_val) {
                    plus_table.insert(byte_val, key);
                }
            }
        }
    }
    reverse_table.extend(plus_table);
    reverse_table
}

fn recover_original_byte(random_byte: u8, encrypted_byte: u8, reverse_table: &HashMap<u8, String>) -> u8 {
    let op_val = reverse_table.get(&encrypted_byte).expect("Encrypted byte not found in reverse table");
    let op = op_val.chars().next().expect("Empty op_val string");
    let value = u8::from_str_radix(&op_val[1..], 16).expect("Invalid hex value in op_val");

    let original: u8;
    if op == '+' {
        original = random_byte.wrapping_add(value);
    } else if op == '-' {
        original = random_byte.wrapping_sub(value);
    } else {
        panic!("Invalid operator in op_val");
    }
    original
}

fn main() -> io::Result<()> {
    let random_file_path = Path::new("random_bytes.bin");
    let changes_file_path = Path::new("changes.bin");
    let table_file = "conversionTable.txt";
    let output_file = "reversed_bytes.bin";
    let temp_dir = Path::new(".temp_random_bytes");
    let used_random_bytes_path = temp_dir.join("used_random_bytes.bin");

    // 1. Clean temporary folder
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir)?;
    }
    fs::create_dir(temp_dir)?; // Recreate the directory

    // 2. Determine changes.bin size
    let changes_metadata = fs::metadata(changes_file_path)?;
    let changes_size = changes_metadata.len();

    // 3. Read random_bytes.bin entirely
    let mut random_bytes_full = fs::read(random_file_path)?;
    let random_bytes_full_len = random_bytes_full.len() as u64;

    // 4. Check size
    if random_bytes_full_len < changes_size {
        eprintln!("Error: random_bytes.bin is smaller than changes.bin. Cannot decrypt.");
        return Ok(());
    }

    // 5. Extract used random bytes
    let used_random_bytes: Vec<u8> = random_bytes_full.drain(0..changes_size as usize).collect();

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

    // Decryption logic using used_random_bytes
    let reverse_table = load_reverse_table(table_file);

    let mut changes_stream = BufReader::new(File::open(changes_file_path)?);
    let mut output_stream = BufWriter::new(File::create(output_file)?);

    let mut changes_buffer = vec![0; changes_size as usize];
    changes_stream.read_exact(&mut changes_buffer)?;

    let mut decrypted_buffer = vec![0; changes_size as usize];

    for i in 0..changes_size as usize {
        decrypted_buffer[i] = recover_original_byte(
            used_random_bytes[i], // Use bytes from used_random_bytes
            changes_buffer[i],
            &reverse_table,
        );
    }
    output_stream.write_all(&decrypted_buffer)?;
    println!("Processed {} bytes... Done!", changes_size);

    Ok(())
}
