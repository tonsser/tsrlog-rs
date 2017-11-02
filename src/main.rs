use std::process::{Command, ChildStdout, Child, Stdio};
use std::io::Read;

fn main() {
    let child = spawn_tail_logs_command("tonsser-api-production");

    for s in stream_for_child_process(child) {
        print!("{}", s);
    }
}

fn stream_for_child_process(child: Child) -> ChildStream {
    ChildStream {
        child_stdout: child.stdout.expect("failed getting stdout for child"),
    }
}

struct ChildStream {
    child_stdout: ChildStdout,
}

impl Iterator for ChildStream {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let mut buf = [0; 10];
        self.child_stdout.read(&mut buf).expect("read child out to string");
        let s: String = String::from_utf8_lossy(&buf).into_owned();
        Some(s)
    }
}

fn spawn_tail_logs_command(heroku_app: &'static str) -> Child {
    Command::new("heroku")
        .args(["logs", "-t", "-a", heroku_app.clone()].iter())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn thread")
}
