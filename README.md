# Git-Append

This program make "links" between local files not in a git index repo and files in a git index repo to keep them in sync

## Use Cases

For a bash history file for example, you would store your history in a git repo shared across your workstations, and Git-Append would keep appending all new lines from `.zsh_history` to `/home/you/zsh-history/.zsh_history`. With `/home/you/zsh-history/` as a git repository.

For a personal log file, such as your daily thoughts, you would `echo` your thoughts to the end of a local file, which would then go to an encrypted file in your repository `journal`.

# Features

- [x] config file
- [ ] systemd service
- [ ] logs
- [ ] config file + systemd service (`just-append run --config=./config.json`)
- [ ] local clear text file
- [ ] Appender:
  - .git location
  - clear editable file location
  - sorted:bool ?
  - unique:bool ?
  - [x] password file location
- [ ] nixos service
- [ ] CLI doc

## V2 Features

- [ ] nixos config as expected
- [ ] encrypted
- [ ] encrypt lines from nth char

## Someday

- [ ] stat diffs before sending
- [ ] append every 5 seconds to a file
- [ ] make options:
  - [ ] remote name (expected: `http-origin`)
  - [ ] branch name
- [ ] ssh support (http only now)
