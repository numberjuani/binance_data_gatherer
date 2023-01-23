
use log::error;
use log::info;
use std::fs::File;
use std::io::copy;
use std::io::BufReader;
use bzip2::Compression;
use bzip2::read::BzEncoder;

/// Compresses a file using bzip2, returns the compressed filename
pub fn compress_file(filepath: &str) -> Result<String, std::io::Error> {
    let input = BufReader::new(File::open(filepath).unwrap());
    let mut output = File::create(format!("{}.bz2", filepath)).unwrap();
    let mut encoder = BzEncoder::new(input, Compression::best());
    match copy(&mut encoder, &mut output) {
        Ok(_) => {
            info!("Succesfully Compressed file {}", filepath);
            match std::fs::remove_file(filepath) {
                Ok(_) => {
                    info!("Succesfully Deleted uncompressed file {}", filepath);
                    Ok(format!("{}.bz2", filepath))
                },
                Err(e) => {
                    error!("Error deleting uncompressed file {}: {}", filepath, e);
                    Err(e)
                },
            }
        },
        Err(e) => {
            error!("Error compressing file: {} {}", e, filepath);
            Err(e)
        },
    }
}

// pub fn decompress_file(filepath:&str) {
//     let input = BufReader::new(File::open(filepath).unwrap());
//     let mut output = File::create(format!("{}.csv", filepath)).unwrap();
//     let mut decoder = bzip2::read::BzDecoder::new(input);
//     match copy(&mut decoder, &mut output) {
//         Ok(_) => {
//             info!("Succesfully Decompressed file {}", filepath);
//             match std::fs::remove_file(filepath) {
//                 Ok(_) => info!("Succesfully Deleted compressed file {}", filepath),
//                 Err(e) => error!("Error deleting compressed file {}: {}", filepath, e),
//             }
//         },
//         Err(e) => {
//             error!("Error decompressing file: {} {}", e, filepath);
//         },
//     }
// }