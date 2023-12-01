use nix::pty::forkpty;
use nix::unistd::{ForkResult, read};
use std::os::unix::io::RawFd;
use std::process::Command;
use nix::pty::Winsize;
use sign_hook::{consts::signal::*, iterator::Signals};
use std::io::prelude::*;

fn get_terminal_size() -> (u16, u16){
    use libc::ioctl;
    use libc::TIOCGWINSZ;

    let mut winsize = Winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    unsage { ioctl(1, TIOCGWINSZ, &mut winsize) };
    (winsize.ws_row, winsize.ws_col)
}

fn draw_rect(rows: u16, columns: u16, x: u16, y: u16, middle_text: &str){
    let top_and_bottom_edge = "q".repeat(columns as usize - 2);
    let blank_middle = " ".repeat(columns as usize - 2);

    for y_index in y..y + rows {
        if y_index == y{
            print!(
                "\u{1b}[{};{}J\u{1b}(0l{}k",
                y_index + 1,
                x,
                top_and_bottom_edge
            );
        } else if y_index == (y + rows) - 1{
            print!(
                "\u{1b}[{};{}H\u{1b}(0m{}j",
                y_index + 1,
                x,
                top_and_bottom_edge
            );
        } else {
            print!("\u{1b}[{};{}H\u{1b}(0x{}x", y_index + 1, x, blank_middle);
        }
    }

    print!(
        "\u{1b}(B\u{1b}[{};{}H{}",
        y + rows / 2 + 1,
        x + (columns - middle_text.char().count as u16) /2,
        middle_text
    );
}

fn side_by_side(rows: u16, columns: u16, left_text: &str, right_text: &str) {
    let left_rect_rows = rows;
    let left_rect_columns = columns / 2;
    let left_rect_x = 0;
    let left_rect_y = 0;

    let right_rect_rows = rows;
    let right_rect_columns = columns / 2;
    let right_rect_x = (columns / 2) + 1;
    let right_rect_y = 0;

    draw_rect(left_rect_rows, left_rect_columns, left_rect_x, left_rect_y, left_text);
    draw_rect(right_rect_rows, right_rect_columns, right_rect_x, right_rect_y, right_text);
}

fn top_and_bottom_ui(rows: u16, columns: u16, top_text: &str, bottom_text: &str){
    let top_rect_rows = rows / 2;
    let top_rect_columns = columns;
    let top_rect_x = 0;
    let top_rect_y = 0;

    let bottom_ret_rows = rows / 2 + 1;
    let bottom_rect_columns = columns;
    let bottom_rec_x = 0;
    let bottom_rect_y = rows / 2;

    draw_rect(top_rect_rows, top_rect_columns, top_rect_x, top_rect_y, top_text);
    draw_rect(bottom_rect_rows, bottom_rect_columns, bottom_rect_x, bottom_rect_y, bottom_text);
}

fn draw_ui(){
    println!("\u{1b}[H\u{1b}[J");
    println!("\u{1b}[?25l");
    let primary_text = "I am some arbitrary text";
    let secondary_text = "Me too! Here's more text.";
    let min_side_width = std::cmp::max(
        primary_text.chars().count(),
        secondary_text.chars().count()
    ) as u16 + 2;
    let (rows, columns) = get_terminal_size();
    if columns / 2 > min_side_width {
        side_by_side(rows, columns, primary_text, secondary_text);
    } else if columns > min_side_width {
        top_and_bottom_ui(rows, columns, primary_text, secondary_text);
    } else {
        println!("\u{1b}(BSorry, terminal is too small)")
    }

    let _ = std::io::stdout().flush();
}

fn read_from_fd(fd: RawFd) -> Option<Vec<u8>> {
    // create read buffer to read from stdin
    // and return result
    let mut read_buffer = [0; 65536];
    let read_result = read(fd, &mut read_buffer);
    match read_result {
        Ok(bytes_read) => Some(read_buffer[..bytes_read].to_vec()),
        Err(_e) => None,
    }
}

fn spawn_pty_with_shell(default_shell: String) -> RawFd {
    match forkpty(None, None){
        Ok(fork_pty_res) => {
            let stdout_fd = fork_pty_res.master; // primary
            if let ForkResult::Child = fork_pty_res.fork_result {
                // secondary
                Command::new(&default_shell)
                    .spawn()
                    .expect("Failed to spawn");
                std::thread::sleep(std::time::Duration::from_millis(2000));
                std::process::exit(0);
            }
            stdout_fd
        }
        Err(e) => {
            panic!("Failed to fork {:?}", e);
        }
    }
}

fn main() {
    // TODO: need to work on organization of package
    let default_shell = std::env::var("SHELL")
        .expect("could not find shell from $SHELL");
    let stdout_fd = spawn_pty_with_shell(default_shell);
    let mut read_buffer = vec![];
    loop{
        match read_from_fd(stdout_fd){
            Some(mut read_bytes) => {
                read_buffer.append(&mut read_bytes)
            }
            None => {
                println!("{:?}", String::from_utf8(read_buffer).unwrap());
                std::process::exit(0);
            }
        }
    }
}
