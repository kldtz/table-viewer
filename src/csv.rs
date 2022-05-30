use std::error::Error;
use std::fs::File;
use std::path::Path;

use std::io::{self, BufReader, Read};
use std::iter::once;

pub fn read_csv_from_file(
    path: &Path,
    delimiter: u8,
    quote: u8,
) -> Result<(Vec<String>, Vec<Vec<String>>), Box<dyn Error>> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    read_csv(reader, delimiter, quote)
}

pub fn read_csv_from_stdin(
    delimiter: u8,
    quote: u8,
) -> Result<(Vec<String>, Vec<Vec<String>>), Box<dyn Error>> {
    read_csv(io::stdin(), delimiter, quote)
}

fn read_csv<R: Read>(
    reader: R,
    delimiter: u8,
    quote: u8,
) -> Result<(Vec<String>, Vec<Vec<String>>), Box<dyn Error>> {
    // TODO: add row numbers
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .quote(quote)
        .from_reader(reader);
    let header = once("#".to_string())
        .chain(csv_reader.headers()?.iter().map(|value| value.to_string()))
        .collect();
    let mut rows: Vec<Vec<String>> = Vec::new();
    for (i, result) in csv_reader.records().enumerate() {
        let record = result?;
        let row: Vec<String> = once(format!("{}", i + 1))
            .chain(record.iter().map(|value| value.to_string()))
            .collect();
        rows.push(row);
    }
    Ok((header, rows))
}
