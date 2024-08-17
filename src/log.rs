use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::{io::{stdout, Stdout}, process::ChildStdout};
use std::io::Write;
use std::io::Read;

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
}

pub fn printopt(stdout: &mut Stdout,text: String) {
    // print with color
    execute!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print("[CMD]"),
        ResetColor,
        Print(" "),
        Print(text),
        Print("\n"),
        ResetColor
    ).unwrap();
}


pub fn printlsc(text: String) {
    print!("\r[SCN] {}", text);
    std::io::stdout().flush().unwrap(); // Ensure the output is flushed immediately
}
