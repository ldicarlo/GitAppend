# Git-Append

This program make "links" between local files not in a git index repo and files in a git index repo to keep them in sync

# Requirements

- You need to add a HTTP origin `http-origin` in your git repository. This does not support ssh, and for now `http-origin` is hardcoded

## Use Cases

For a bash history file for example, you would store your history in a git repo shared across your workstations, and Git-Append would keep appending all new lines from `.zsh_history` to `/home/you/zsh-history/.zsh_history`. With `/home/you/zsh-history/` as a git repository.

For a personal log file, such as your daily thoughts, you would `echo` your thoughts to the end of a local file, which would then go to an encrypted file in your repository `journal`.

For debugging purposes you can use the `git-append cat ...` command which show you the content of a file from the config you feed it.

For now, files are sorted per uniques lines.

# Features

- [x] config file
- [ ] logs
- [ ] nixos systemd service
- [ ] config file + systemd service (`git-append run --config=./config.json`)
- [x] local clear text file
- [ ] Appender:
  - [x] .git location
  - clear editable file location
  - sorted:bool ?
  - unique:bool ?
  - [x] password file location
- [x] CLI doc

## V2 Features

- [x] nixos config as expected
- [x] encrypted
- [ ] encrypt lines from nth char

## Someday

- [ ] stat diffs before sending
- [ ] append every 5 seconds to a file
- [ ] make options:
  - [ ] remote name (expected: `http-origin`)
  - [x] branch name
- [ ] ssh support (http only now)
