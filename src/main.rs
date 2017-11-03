use std::process::{Command, ChildStdout, Child, Stdio};
use std::io::{self, Read, BufRead, stdout, Write};
use std::env;
use std::process::exit;
use std::collections::HashSet;
use std::thread;
use std::fs::{File};
use std::sync::{Arc, Mutex};

fn main() {
    let command_line_args: Vec<String> = env::args().collect();

    match command_line_args.get(1).map(|s| s.as_ref()) {
        Some("production") =>  { run_on(Env::Production); },
        Some("p") =>           { run_on(Env::Production); },
        Some("staging") =>     { run_on(Env::Staging); },
        Some("s") =>           { run_on(Env::Staging); },
        Some("development") => { run_on(Env::Development); },
        Some("dev") =>         { run_on(Env::Development); },
        Some("d") =>           { run_on(Env::Development); },
        _ => {
            eprintln!("Unknown env");
            exit(1);
        }
    }
}

enum Env {
    Production,
    Staging,
    Development,
}

impl Env {
    fn heroku_app_name(&self) -> &'static str {
        match self {
            &Env::Production => "tonsser-api-production",
            &Env::Staging => "tonsser-api-staging",
            &Env::Development => "tonsser-api-development",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            &Env::Production => "production",
            &Env::Staging => "staging",
            &Env::Development => "dev",
        }
    }
}

type Logs = Arc<Mutex<String>>;

fn run_on(env: Env) {
    let logs = Arc::new(Mutex::new(String::new()));
    let env = Arc::new(env);

    {
        let logs = Arc::clone(&logs);
        let env = Arc::clone(&env);
        thread::spawn(move || {
            let child = spawn_tail_logs_command(env.heroku_app_name());
            for s in stream_for_child_process(child) {
                logs.lock().unwrap().push_str(&s);
            }
        });
    }

    run_input_loop(&logs, &env);
}

fn run_input_loop(logs: &Logs, env: &Env) {
    let stdin = io::stdin();
    let mut input = stdin.lock().lines();
    let mut seen_lines: HashSet<String> = HashSet::new();
    loop {
        print!("tsrlog {} > ", env.name());
        stdout().flush().expect("failed to flush stdout");
        let input: String = input.next().unwrap().unwrap();
        let action = Action::parse(&input);

        match action {
            Action::Skip => {},

            Action::Exit => { exit(0) },

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
            },

            Action::Search(query) => {
                let logs = logs.lock().unwrap();
                logs
                    .lines()
                    .filter(|line| line.to_lowercase().contains(query.as_ref() as &str))
                    .for_each(|line| println!("{}", line));
                stdout().flush().expect("failed to flush stdout");
            },

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
            "" =>      Action::Skip,
            "fail" =>  Action::Fail,
            "f" =>     Action::Fail,
            "exit" =>  Action::Exit,
            "write" => Action::Save,
            "w" =>     Action::Save,
            "save" =>  Action::Save,
            "s" =>     Action::Save,
            s =>       Action::Search(s.to_string()),
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
