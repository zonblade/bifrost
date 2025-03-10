use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::stdout;
use std::io::Write;

pub fn printlg(text: String, color: Color) {
    let mut stdout = stdout();

    execute!(
        stdout,
        SetForegroundColor(Color::Green),
        Print("[OPT]"),
        ResetColor,
        Print(" "),
        SetForegroundColor(color),
        Print(text),
        Print("\n"),
        ResetColor
    )
    .unwrap();

    stdout.flush().unwrap();
}

pub fn printlsc(text: String) {
    print!("\r[SCN] {}", text);
    std::io::stdout().flush().unwrap(); // Ensure the output is flushed immediately
}
