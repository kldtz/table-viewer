# Table Viewer

Interactive table viewer for the command line.

## Installation

Install the tool from this repo via Cargo.

```bash
cargo install --git https://github.com/kldtz/table-viewer
```

## Usage

Check the help for correct invocation and navigation.

```bash
tv --help
```

In the simplest case (when the delimiter matches the extension and the quote character is `"`), you only need to provide the file path:

```bash
tv table.csv
```


Move between cells using the arrow keys or Vim's `hjkl`. Page up and down. Jump to start via `Home` or `gg`. Jump to end via `End` or `G`. Sort by column under cursor with `a` (ascending) or `d` (descending); return to original order with `o`. Search for substring in column under cursor by typing `/` followed by search term and `Enter`. Repeat last search starting from current cursor position by pressing `Space`. Exit with `q` or `Ctrl-x`.

The tool loads the whole file into memory. If you're dealing with huge files, you can peek at just a few rows like this:

```bash
head table.csv | tv
```