use nix::pty::forkpty;
use nix::unistd::{ForkResult, read};
use std::os::unix::io::RawFd;
use std::process::Command;

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
