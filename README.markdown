# tsrlogs

Tool for working with streaming logs from Heroku.

Should work with logs from any web framework, but we use it with our Rails API.

## Features

Running the program will start tailing the logs in the background and open a prompt:

```
> type command here
```

The supported commands are:

- `exit`: Stop the program.
- `f`, `fail`: Print lines that match `Completed [^2]`. Used to find failed requests.
- `save`, `s`, `write`, `w`: Write the logs to a file named `logs`.
- Any other input: Search the logs for the input (ignoring case) and print matching lines.

## Install

For the time being you have to compile it yourself, but that should be very straight forward.

1. [Install Rust](https://www.rust-lang.org/en-US/install.html)
2. Download the source: `git clone https://github.com/tonsser/tsrlogs-rs`
3. Compile: `cargo build --release`
4. Make a configuration file. See chapter below 👇

You can now run it with `./target/release/tsrlog-rs ARG`.

## Configuration

The code expects a file named `tsrlog_config.yaml` in the current directory. The file should look like this:

```yaml
production: some-heroku-app-production
staging: some-heroku-app-staging
```

Running the command `./target/release/tsrlog-rs production` will look for the key `production` in the configuration file, and tail logs from the Heroku app with the corresponding name.

If you feel like it you can also make some shorthands:

```yaml
production: some-heroku-app-production
p: some-heroku-app-production

staging: some-heroku-app-staging
s: some-heroku-app-staging
```

Empty lines in the configuration file are ignored.

## Readline support

The code doesn't support tracking history and using the arrow keys to move up and down. For that we recommend you use `rlwrap`. It can be installed with `brew install rlwrap`.

I recommend adding a shell alias like:

```
alias tsrlogs='rlwrap PATH_TO_BINARY'
```
