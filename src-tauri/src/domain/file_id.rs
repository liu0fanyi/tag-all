//! File Identification Logic
//! 
//! Handles quick hash (metadata-based) and content hash calculation.

use std::path::Path;
use std::fs;
use std::io::Read;

pub struct FileIdentifier;

impl FileIdentifier {
    /// Compute a quick hash based on metadata (filename, size, created time).
    /// Used for fast move detection.
    /// Format: blake3(filename|size|created_ms)
    pub fn compute_quick_hash(path: &Path) -> Result<String, String> {
        let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
        let file_name = path.file_name().ok_or("No filename")?.to_string_lossy();
        let size = metadata.len();
        
        // On some platforms created time might not be available, fallback to modified
        let created = metadata.created().or_else(|_| metadata.modified())
            .map_err(|e| format!("Could not get file time: {}", e))?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis();
            
        let input = format!("{}|{}|{}", file_name, size, created);
        let hash = blake3::hash(input.as_bytes());
        Ok(hash.to_hex().to_string())
    }

    /// Compute full content hash.
    /// Used for definitive identity.
    pub fn compute_content_hash(path: &Path) -> Result<String, String> {
        // Use a buffer to read file in chunks to avoid loading entire file into memory
        let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0; 65536]; // 64KB buffer

        loop {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    hasher.update(&buffer[..n]);
                }
                Err(e) => return Err(e.to_string()),
            }
        }
        
        Ok(hasher.finalize().to_hex().to_string())
    }
}
