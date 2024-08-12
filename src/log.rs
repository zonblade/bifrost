use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::stdout;
use std::io::Write;

pub fn printlg(text: String) {
    let mut stdout = stdout();

    execute!(
        stdout,
        SetForegroundColor(Color::Green),
        Print("[OPT]"),
        ResetColor,
        Print(" "),
        SetForegroundColor(Color::White),
        Print(text),
        Print("\n"),
        ResetColor
    )
    .unwrap();
}

pub fn printlsc(text: String) {
    print!("\r[SCN] {}", text);
    std::io::stdout().flush().unwrap(); // Ensure the output is flushed immediately
}
