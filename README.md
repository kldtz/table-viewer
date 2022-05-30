# Table Viewer

Interactive table viewer for the command line.

## Installation

```
cargo install --git 
```

## Usage

Move between cells using the arrow keys or Vim's `hjkl`. Page up and down. Jump to start via `Home` or `gg`. Jump to end via `End` or `G`. Sort by column under cursor with `a` (ascending) or `d` (descending); return to original order with `o`. Search for substring in column under cursor by typing `/` followed by search term and `Enter`. Repeat last search starting from current cursor position by pressing `Space`. Exit with `q` or `Ctrl-x`.