#[cfg(test)]

#[tokio::test]
async fn test_compress() {
    use crate::file_compress::compress_file;
    compress_file("outgoing/trades.csv");
}

// #[tokio::test]
// async fn test_decompress() {
//     use crate::file_compress::decompress_file;
//     decompress_file("outgoing/trades.csv.bz2");
// }