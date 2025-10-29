use std::env;
use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::Path;
use rand::{Rng, thread_rng};

fn parse_size_string(size_str: &str) -> Option<u64> {
    let s = size_str.to_lowercase();
    let (num_str, unit_str) = s.split_at(s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len()));

    let num: u64 = num_str.parse().ok()?;

    match unit_str {
        "b" | "bytes" => Some(num),
        "kb" => Some(num * 1024),
        "mb" => Some(num * 1024 * 1024),
        "gb" => Some(num * 1024 * 1024 * 1024),
        _ => None,
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut target_size: Option<u64> = None;
    let output_filename = "random_bytes.bin";
    let input_filename = "input_bytes.bin";
    // temp_dir and used_random_bytes_path are no longer needed here

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--size" {
            if i + 1 < args.len() {
                target_size = parse_size_string(&args[i + 1]);
                if target_size.is_none() {
                    eprintln!("Error: Invalid size format for --size. Examples: 1gb, 3kb, 39bytes.");
                    return Ok(());
                }
                i += 2;
            } else {
                eprintln!("Error: --size flag requires a value.");
                return Ok(());
            }
        } else {
            eprintln!("Warning: Unknown argument '{}' ignored.", args[i]);
            i += 1;
        }
    }

    let final_size = match target_size {
        Some(size) => size,
        None => {
            // If --size not provided, use input_bytes.bin size
            if !Path::new(input_filename).exists() {
                eprintln!("Error: {} not found. Please provide it or use --size flag.", input_filename);
                return Ok(());
            }
            std::fs::metadata(input_filename)?.len()
        }
    };

    let mut rng = thread_rng();
    let mut file = File::create(output_filename)?;

    let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer for writing
    let mut bytes_written = 0;

    while bytes_written < final_size {
        let bytes_to_write_in_chunk = (final_size - bytes_written).min(buffer.len() as u64);
        rng.fill(&mut buffer[..bytes_to_write_in_chunk as usize]);
        file.write_all(&buffer[..bytes_to_write_in_chunk as usize])?;
        bytes_written += bytes_to_write_in_chunk;
        // Removed: println!("Generated {}/{} bytes...", bytes_written, final_size);
    }

    println!("Successfully generated {} random bytes to {}.", final_size, output_filename);

    Ok(())
}
