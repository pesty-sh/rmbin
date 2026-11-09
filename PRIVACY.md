# Privacy

`rmbin` 0.0.1 does not collect information and does not send information.

What the program does not do:

- It does not open a network socket.
- It does not check for updates, and it does not download data while it runs.
- It does not send telemetry, analytics, crash reports, or a phone-home message.
- It does not read your file contents into a log or into the report. It asks the filesystem for the length, writes the pattern over that many bytes, and deletes the name.
- It does not put environment variables, file contents, or secret values into the report. The report contains paths, sizes, the pattern text, and errors from the operating system.

What is visible on the machine, because that is how a command line works:

- The paths and the `--pattern` text are command-line arguments. A process listing can show them.
- The plan and the report are printed on the terminal. They include those paths and the pattern.
- The pattern bytes are written into the files you asked to remove.

The build tools (`cargo`, GitHub Actions) can use the network when you compile. That is separate from the `rmbin` binary. See [docs/OFFLINE.md](docs/OFFLINE.md).

There is no account, no key, and no remote configuration.
