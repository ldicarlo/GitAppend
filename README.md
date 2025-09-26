# GitAppend

This program makes "links" between local files not in a git index repo and files in a git index repo to keep them in sync.

# Requirements / Installation

- for the configuration see [./tests/example-config.json](./tests/example-config.json) for a full example
- You need to add a HTTP origin `http-origin` in your git repository. This does not support ssh, and for now `http-origin` is hardcoded
- If you don't use nixos, you need to set up any CRON/systemd, to run `git-append run --config-path=/some/path.json`
- If you use nixos, after importing it, you have access to options, see in `./flake.nix`

## Use Cases

For a bash history file for example, you would store your history in a git repo shared across your workstations, and Git-Append would keep appending all new lines from `.zsh_history` to `/home/you/zsh-history/.zsh_history`. With `/home/you/zsh-history/` as a git repository.

For a personal log file, such as your daily thoughts, you would `echo` your thoughts to the end of a local file, which would then go to an encrypted file in your repository `journal`.

For debugging purposes you can use the `git-append cat ...` command which show you the content of a file from the config you feed it.

For now, files are sorted per uniques lines.

### Folder links

If you use [Per-Directory-History](https://github.com/jimhester/per-directory-history) for example, you can also declare a synced folder, using `folder_links` (see [per-directory-history config example](./tests/example-per-directory-history-config.json)).

# Features

- [x] config file
- [ ] logs (there is still a bug here, the logs don't appear in systemd)
- [x] nixos systemd service
- [x] config file + systemd service (`git-append run --config=./config.json`)
- [x] local clear text file
- [ ] Appender:
  - [x] .git location
  - [x] clear editable file location
  - [ ] sorted:bool ?
  - [ ] unique:bool ?
  - [x] password file location
- [x] CLI doc
- [x] Whole folder sync (`folder_links`)

## V2 Features

- [x] nixos config as expected
- [x] encrypted

## Someday

- [ ] per line encryption
- [ ] encrypt lines from nth char
- [ ] stat diffs before sending
- [ ] append every 5 seconds to a file
- [ ] make options:
  - [ ] remote name (expected: `http-origin`)
  - [x] branch name
- [ ] ssh support (http only now)
