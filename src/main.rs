use table_viewer::renderer::TerminalTableRenderer;
use std::path::Path;

use clap::Parser;
use table_viewer::viewer::TableViewer;
use table_viewer::csv::{read_csv_from_file, read_csv_from_stdin};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
/// Interactive table viewer for the command line.
///
/// Move between cells using the arrow keys or Vim's hjkl. Page up and down.
/// Jump to start via Home or gg. Jump to end via End or G. Sort by column
/// under cursor with a (ascending) or d (descending); return to original
/// order with o. Search for substring in column under cursor by typing /
/// followed by search term and Enter. Repeat last search starting from
/// current cursor position by typing Space. Exit with q or Ctrl-x.
struct Args {
    /// Path to CSV/TSV file
    #[clap()]
    file: Option<String>,

    /// Field delimiter (default based on file extension)
    #[clap(short, long)]
    delimiter: Option<char>,

    /// Quote character
    #[clap(short, long)]
    quote: Option<char>,
}

fn main() {
    let args = Args::parse();
    let delimiter = match args.delimiter {
        Some(c) => c as u8,
        None => match args.file {
            Some(ref file) if file.ends_with(".tsv") => b'\t',
            _ => b',',
        },
    };
    let quote = match args.quote {
        Some(c) => c as u8,
        None => b'"',
    };
    let (header, rows) = match args.file {
        Some(ref file) => match read_csv_from_file(Path::new(file), delimiter, quote) {
            Ok(viewer) => viewer,
            Err(err) => {
                eprintln!("Error reading file '{:?}': {}", file, err);
                std::process::exit(1);
            }
        },
        None => match read_csv_from_stdin(delimiter, quote) {
            Ok(viewer) => viewer,
            Err(err) => {
                eprintln!("Error reading from stdin: {}", err);
                std::process::exit(1);
            }
        },
    };
    let mut table_viewer = TableViewer::new(TerminalTableRenderer {}, header, rows);
    match table_viewer.run() {
        Ok(_) => (),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };
}
