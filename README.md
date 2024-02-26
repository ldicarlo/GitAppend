# Features

- [ ] append every 5 seconds to a file
- [ ] config file
  - [ ] JSON
  ```json
  {
    "/etc/file/to/sync": {
        "every_seconds": 10,
        "git_folder_location": "/projects/synced_git_project"
    }
  }
  ```
- [ ] stat diffs before sending
- [ ] systemd service
- [ ] logs
- config file + systemd service (`just-append run --config=./config.json`)
- local clear text file
- Appender:
  - .git location
  - clear editable file location
  - sorted:bool ?
  - unique:bool ?
  - password file location ?


## V2 Features
