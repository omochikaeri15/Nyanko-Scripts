use colored::Colorize;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use tracing::{debug, error, info, trace, warn};

use crate::io::get_local_dir;
use crate::keys::UserKeys;
use crate::scanner::scan_and_resolve;
use nyanko::pack::cryptology::{self, check_integrity};

pub fn execute(input_target: &str, force: bool, output_dir: Option<&str>) {
    debug!(target = input_target, "Starting decryption process");
    let input_path = Path::new(input_target);
    let keys = UserKeys::load();
    let validations = keys.validate();
    let all_valid = validations.iter().all(|&(key, iv)| key && iv);

    if !all_valid {
        if force {
            warn!("Bypassing key validation failures due to --force flag");
        } else {
            warn!(
                "\n{}: Invalid or missing keys detected in 'keys' file, continue anyways? [Y/n]: ",
                "WARNING".yellow().bold()
            );
            let _ = std::io::stdout().flush();
            let mut choice = String::new();
            let _ = std::io::stdin().read_line(&mut choice);

            if choice.trim().to_lowercase() != "y" {
                error!("\nFAILURE: Session aborted!\n");
                return;
            }
            warn!("You can create a 'keys.json' file by running the 'bcc-pack keys load' command.");
        }
    }

    let nyanko_keys = match keys.to_nyanko_keys() {
        Ok(valid_keys) => valid_keys,
        Err(err) => {
            error!("  {} ERROR: Failed to parse keys for decryption: {}", "✗".red(), err);
            return;
        }
    };

    let pairs = match scan_and_resolve(input_path) {
        Ok(resolved_pairs) => resolved_pairs,
        Err(err) => {
            error!(error = %err, "Scan and resolve failed");
            return;
        }
    };

    let output_base = if let Some(custom_dir) = output_dir {
        Path::new(custom_dir).to_path_buf()
    } else {
        let mut default_dir = get_local_dir();
        default_dir.push("decrypted");
        default_dir
    };

    let display_base = output_base
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("decrypted");

    let mut total_extracted_count = 0;

    for pair in pairs {
        debug!(pack = %pair.name, output_dir = %output_base.display(), "Processing pack pair");

        let Ok(list_data) = fs::read(&pair.list_path) else {
            error!(
                "  {} Failed to extract files from {} (Could not read .list file)",
                "✗".red(),
                pair.name.cyan()
            );

            continue;
        };

        let decoded_list_content = match cryptology::decrypt_list(&list_data) {
            Ok(content) => content,
            Err(_) => {
                error!(pack = %pair.name, "List decryption failed");
                continue;
            }
        };

        if decoded_list_content.trim().is_empty() {
            warn!(pack = %pair.name, "No files found in list");
            continue;
        }

        let mut pack_file = match fs::File::open(&pair.pack_path) {
            Ok(file) => file,
            Err(_) => {
                error!(pack = %pair.name, "Could not open .pack file");
                continue;
            }
        };

        let pack_output_dir = output_base.join(&pair.name);
        let mut extracted_count = 0;
        let mut corrupted_count = 0;

        for line in decoded_list_content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 3 {
                continue;
            }

            let asset_name = parts[0].trim();
            let Ok(offset): Result<u64, _> = parts[1].trim().parse() else {
                continue;
            };
            let Ok(size): Result<usize, _> = parts[2].trim().parse() else {
                continue;
            };

            if size == 0 {
                continue;
            }

            let memory_aligned_size = if size % 16 == 0 { size } else { ((size / 16) + 1) * 16 };
            let mut encrypted_buffer = vec![0u8; memory_aligned_size];

            if pack_file.seek(SeekFrom::Start(offset)).is_err() {
                continue;
            }
            if pack_file.read_exact(&mut encrypted_buffer).is_err() {
                continue;
            }

            let (decrypted_data, _) = cryptology::decrypt_chunk(&encrypted_buffer, asset_name, &nyanko_keys);
            let strict_limit = std::cmp::min(size, decrypted_data.len());
            let clean_data = &decrypted_data[..strict_limit];

            if !check_integrity(clean_data, asset_name) {
                trace!(asset = %asset_name, pack = %pair.name, "Integrity check failed");
                corrupted_count += 1;
                continue;
            }

            if extracted_count == 0 {
                let _ = fs::create_dir_all(&pack_output_dir);
            }

            let final_path = pack_output_dir.join(asset_name);
            if let Some(parent_dir) = final_path.parent() {
                let _ = fs::create_dir_all(parent_dir);
            }

            if fs::write(&final_path, clean_data).is_ok() {
                extracted_count += 1;
                trace!(asset = %asset_name, pack = %pair.name, size = strict_limit, dest = %final_path.display(), "File successfully extracted to disk");
            } else {
                trace!(asset = %asset_name, pack = %pair.name, "Failed to write extracted file to disk");
            }
        }

        drop(pack_file);
        total_extracted_count += extracted_count;

        if extracted_count > 0 {
            if corrupted_count > 0 {
                warn!(pack = %pair.name, corrupted = corrupted_count, "Skipped corrupted files");
            }
            info!(pack = %pair.name, extracted = extracted_count, dest = %pack_output_dir.display(), "Files extracted successfully");
        } else if corrupted_count > 0 {
            error!(pack = %pair.name, corrupted = corrupted_count, "Skipped completely corrupted pack");
        } else {
            warn!(pack = %pair.name, "No files found in pack");
        }
    }

    let mut temp_apk_dir = get_local_dir();
    temp_apk_dir.push("apk");

    if temp_apk_dir.exists() {
        if let Err(err) = fs::remove_dir_all(&temp_apk_dir) {
            error!(error = %err, "Could not delete temporary apk directory");
        } else {
            debug!("Cleaned up temporary APK files");
        }
    }

    if total_extracted_count > 0 {
        info!(total_extracted = total_extracted_count, "Decryption complete");
    } else {
        error!("Decrypted no files");
        std::process::exit(1);
    }
}
