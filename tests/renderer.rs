use std::path::Path;
use table_viewer::csv::read_csv_from_file;
use table_viewer::renderer::{RenderingAction, TableRenderer, TerminalTableRenderer};
use table_viewer::state::{CharCoord, TableState};

fn small_table_state_fixture() -> TableState {
    let (header, rows) =
        read_csv_from_file(Path::new("tests/resources/small_table.csv"), b',', b'"').unwrap();
    TableState::new(header, rows, CharCoord { x: 9, y: 4 })
}

fn pretty_print(value: &str) -> String {
    let value: String = value.chars().skip(10).collect();
    value
        .replace("\x1B[1m", "")
        .replace("\x1B[m", "")
        .replace("\x1B[", "\n<goto>")
        .replace("H", "</goto>")
        .replace("\r", "")
}

fn render(renderer: &TerminalTableRenderer, state: &TableState) -> String {
    pretty_print(&renderer.render(&state, &RenderingAction::Rerender).unwrap())
}

#[test]
fn test_move_down() {
    let mut state = small_table_state_fixture();
    let renderer = TerminalTableRenderer {};

    let mut actual = render(&renderer, &state);

    // Move cursor down within displayed window
    for row in 1..4 {
        let expected = vec![
            "#  a   bb",
            "1  1a  1…",
            "2  2a  2…",
            "3  3a  3…",
            &format!("<goto>{};1</goto>", row),
        ]
        .join("\n");
        assert_eq!(actual, expected);
        state.move_down();
        actual = render(&renderer, &state);
    }

    // Shift displayed window to the end
    state.move_down();
    actual = render(&renderer, &state);
    let expected = vec![
        "#  a   bb",
        "2  2a  2…",
        "3  3a  3…",
        "4  4a  4…",
        "<goto>4;1</goto>",
    ]
    .join("\n");
    assert_eq!(actual, expected);

    state.move_down();
    actual = render(&renderer, &state);
    let expected = vec![
        "#  a   bb",
        "3  3a  3…",
        "4  4a  4…",
        "5  5a  5…",
        "<goto>4;1</goto>",
    ]
    .join("\n");
    assert_eq!(actual, expected);

    // We reached the end, so move down doesn't have any effect
    state.move_down();
    actual = render(&renderer, &state);
    assert_eq!(actual, expected);
}

#[test]
fn test_move_up() {
    let mut state = small_table_state_fixture();
    let renderer = TerminalTableRenderer {};
    state.offsets.row = 2;
    state.cur_pos.row = 3;

    let mut actual = render(&renderer, &state);

    // Move cursor up within displayed window
    for row in (2..5).rev() {
        let expected = vec![
            "#  a   bb",
            "3  3a  3…",
            "4  4a  4…",
            "5  5a  5…",
            &format!("<goto>{};1</goto>", row),
        ]
        .join("\n");
        assert_eq!(actual, expected);
        state.move_up();
        actual = render(&renderer, &state);
    }

    let expected = vec![
        "#  a   bb",
        "2  2a  2…",
        "3  3a  3…",
        "4  4a  4…",
        &format!("<goto>{};1</goto>", 2),
    ]
    .join("\n");
    assert_eq!(actual, expected);

    state.move_up();
    actual = render(&renderer, &state);
    let expected = vec![
        "#  a   bb",
        "1  1a  1…",
        "2  2a  2…",
        "3  3a  3…",
        &format!("<goto>{};1</goto>", 2),
    ]
    .join("\n");
    assert_eq!(actual, expected);

    // Move to header
    state.move_up();
    actual = render(&renderer, &state);
    let expected = vec![
        "#  a   bb",
        "1  1a  1…",
        "2  2a  2…",
        "3  3a  3…",
        &format!("<goto>{};1</goto>", 1),
    ]
    .join("\n");
    assert_eq!(actual, expected);

    // We are already at the top, so nothing happens
    state.move_up();
    actual = render(&renderer, &state);
    assert_eq!(actual, expected);
}

#[test]
fn test_move_right() {
    let mut state = small_table_state_fixture();
    let renderer = TerminalTableRenderer {};

    let actual = render(&renderer, &state);
    let expected = vec![
        "#  a   bb",
        "1  1a  1…",
        "2  2a  2…",
        "3  3a  3…",
        "<goto>1;1</goto>",
    ]
    .join("\n");
    assert_eq!(actual, expected);

    state.move_right();
    let actual = render(&renderer, &state);
    let expected = vec![
        "#  a   bb",
        "1  1a  1…",
        "2  2a  2…",
        "3  3a  3…",
        "<goto>1;4</goto>",
    ]
    .join("\n");
    assert_eq!(actual, expected);

    // Window needs to shift right
    state.move_right();
    let actual = render(&renderer, &state);
    let expected = vec![
        "a   bb   ",
        "1a  1bb  ",
        "2a  2bb  ",
        "3a  3bb  ",
        "<goto>1;5</goto>",
    ]
    .join("\n");
    assert_eq!(actual, expected);

    state.move_right();
    let actual = render(&renderer, &state);
    let expected = vec![
        "bb   c   ",
        "1bb  1c  ",
        "2bb  2c  ",
        "3bb  3c  ",
        "<goto>1;6</goto>",
    ]
    .join("\n");
    assert_eq!(actual, expected);

    // Already at the end, nothing happens
    state.move_right();
    let actual = render(&renderer, &state);
    let expected = vec![
        "bb   c   ",
        "1bb  1c  ",
        "2bb  2c  ",
        "3bb  3c  ",
        "<goto>1;6</goto>",
    ]
    .join("\n");
    assert_eq!(actual, expected);
}
