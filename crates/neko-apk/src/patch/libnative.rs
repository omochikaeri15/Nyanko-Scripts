use std::env;
use std::path::{Path, PathBuf};
use tracing::{debug, trace};

pub fn find_local_libnative() -> Option<PathBuf> {
    debug!("Initializing search for local modded libnative-lib.so payload...");

    if let Ok(current_exe) = env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let candidate_path = exe_dir.join("libnative-lib.so");
            trace!(
                candidate = %candidate_path.display(),
                "Probing adjacent to executable for native payload"
            );

            if candidate_path.exists() {
                debug!("Discovered modded libnative-lib.so next to executable");
                return Some(candidate_path);
            }
        }
    }

    let cwd_candidate = Path::new("libnative-lib.so");
    trace!(
        candidate = %cwd_candidate.display(),
        "Probing current working directory for native payload"
    );

    if cwd_candidate.exists() {
        debug!("Discovered modded libnative-lib.so in working directory");
        return Some(cwd_candidate.to_path_buf());
    }

    trace!("No local modded libnative-lib.so found. Proceeding with vanilla binaries.");
    None
}