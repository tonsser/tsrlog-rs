use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, stdout, BufRead, Read, Write};
use std::process::exit;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let mut config_file = File::open("tsrlog_config.yaml").unwrap();

    let mut contents = String::new();
    config_file.read_to_string(&mut contents).unwrap();
    let configs = parse_config(contents);

    let command_line_args: Vec<String> = env::args().collect();
    let arg: &str = command_line_args
        .get(1)
        .map(|s| s.as_ref())
        .expect("Missing command line arg");

    let config = configs
        .iter()
        .find(|config| config.alias == arg)
        .expect("No matching config found");

    println!("Capturing logs from {}", config.heroku_app_name);

    run_with(config);
}

fn parse_config(contents: String) -> Vec<Env> {
    let mut acc = Vec::new();

    contents
        .lines()
        .filter(|line| line.len() > 0)
        .for_each(|line| {
            let split: Vec<&str> = line.split(": ").collect();
            let alias = split.get(0).unwrap();
            let heroku_app_name = split.get(1).unwrap();
            let env = Env {
                heroku_app_name: heroku_app_name.to_string(),
                alias: alias.to_string(),
            };
            acc.push(env);
        });

    acc
}

#[derive(Debug)]
struct Env {
    heroku_app_name: String,
    alias: String,
}

type Logs = Arc<Mutex<String>>;

fn run_with(env: &Env) {
    let logs = Arc::new(Mutex::new(String::new()));
    let env = Arc::new(env);

    {
        let app_name: String = env.heroku_app_name.clone();
        let logs = Arc::clone(&logs);
        thread::spawn(move || {
            let child = spawn_tail_logs_command(app_name);
            for s in stream_for_child_process(child) {
                logs.lock().unwrap().push_str(&s);
            }
        });
    }

    run_input_loop(&logs);
}

fn run_input_loop(logs: &Logs) {
    let stdin = io::stdin();
    let mut input = stdin.lock().lines();
    let mut seen_lines: HashSet<String> = HashSet::new();
    loop {
        print!("> ");
        stdout().flush().expect("failed to flush stdout");
        let input: String = input.next().unwrap().unwrap();
        let action = Action::parse(&input);

        match action {
            Action::Skip => {}

            Action::Exit => exit(0),

            Action::Fail => {
                let logs = logs.lock().unwrap();

                let failed_lines = logs
                    .lines()
                    .filter(|line| line.contains("Completed "))
                    .filter(|line| !line.contains("Completed 2"));

                failed_lines.for_each(|line| {
                    if !seen_lines.contains(line.as_ref() as &str) {
                        println!("{}", line);
                    }
                    seen_lines.insert(line.to_string());
                });

                stdout().flush().expect("failed to flush stdout");
            }

            Action::Search(query) => {
                let logs = logs.lock().unwrap();
                logs.lines()
                    .filter(|line| line.to_lowercase().contains(query.as_ref() as &str))
                    .for_each(|line| println!("{}", line));
                stdout().flush().expect("failed to flush stdout");
            }

            Action::Save => {
                let mut file = File::create("logs").unwrap();
                let logs = logs.lock().unwrap();
                file.write_all(logs.as_bytes()).unwrap();
                println!("Saved!");
                stdout().flush().expect("failed to flush stdout");
            }
        }
    }
}

#[derive(Debug)]
enum Action {
    Search(String),
    Fail,
    Exit,
    Save,
    Skip,
}

impl Action {
    fn parse(input: &String) -> Action {
        match input.as_ref() {
            "" => Action::Skip,
            "fail" => Action::Fail,
            "f" => Action::Fail,
            "exit" => Action::Exit,
            "write" => Action::Save,
            "w" => Action::Save,
            "save" => Action::Save,
            "s" => Action::Save,
            s => Action::Search(s.to_string()),
        }
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
        self.child_stdout
            .read(&mut buf)
            .expect("read child out to string");
        let s: String = String::from_utf8_lossy(&buf).into_owned();
        Some(s)
    }
}

fn spawn_tail_logs_command(heroku_app: String) -> Child {
    Command::new("heroku")
        .args(["logs", "-t", "-a", heroku_app.as_ref()].iter())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn thread")
}
