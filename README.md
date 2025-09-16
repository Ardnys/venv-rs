# venv-rs

venv-rs is a high level Python virtual environment manager specifically developed for [my personal workflow and needs](https://ardnys.github.io/projects/venv-manager/).
<div align="center">
  <img src="https://github.com/Ardnys/venv-rs/blob/main/images/venv_rs_logo.png" width=300 height=300 />
</div>

# Demo
## Inspect your virtualenvs directory
<img src="https://github.com/Ardnys/venv-rs/blob/main/images/venvs_demo.gif" />

## Inspect a single virtual environment
<img src="https://github.com/Ardnys/venv-rs/blob/main/images/venv_demo.gif" />

## Search for virtual environments
<img src="https://github.com/Ardnys/venv-rs/blob/main/images/search_demo.gif" />

# Features
- Shows virtual environments, their size on disk, number of packages
- Shows packages, versions, and sizes on disk
- Copies activation command on exit
- Prints requirements.txt so you don't have to activate it and print it manually
- Cross platform. I use it in both Command Prompt and Git Bash on Windows.
- Kind of satisfying to use imo

# Usage
```
Usage: venv-rs [OPTIONS] <COMMAND>

Commands:
  venv         Inspect a single virtual environment
  search       Search virtual environments recursively
  venvs        Directory containing virtual environments
  list-shells  List available shells [aliases: ls]
  help         Print this message or the help of the given subcommand(s)

Options:
  -s, --shell <SHELL>  Shell for the activation command
  -h, --help           Print help
  -V, --version        Print version
```
Press "?" in TUI for the help screen.

# Configuration
Currently there's minimal configuration mostly to set preferences to shorten the commands. An example config is below:
```yaml
# put it in $XDG_CONFIG_HOME/venv-rs/config.yaml if it doesn't exist already
shell: "zsh" 
venvs_dir: "~/.virtualenvs"
extra:
  xclip: true # for linux
```
> [!Tip]
Check supported shells with `venv-rs ls` command.


With the config above the command
```bash
$ venv-rs venvs
```
is equivalent to
```bash
$ venv-rs -s zsh venvs ~/.virtualenvs
```
> [!Tip]
CLI arguments have priority over configuration options.

# Roadmap
- [x] show packages in the venv
- [x] show package versions and sizes
- [x] human readable byte sizes
- [x] venv details (python version, size, num packages)
- [x] _encourage_ activation on exit
- [x] print the requirements.txt
- [x] remove anyhow
- [x] walk the directory tree and look for .env folders. that way i don't have to limit to this particular workflow.
- [x] parse package dependencies
  - [ ] add them to the package size
  - [ ] implement petgraph for dependencies
  - [x] show dependencies in UI
  - [ ] consider extra features and which dependencies they add
- [x] windows compatibility
- [x] copy activation command on exit
- [x] config file
  - [x] shell to use for activation command
  - [x] default path to look for virtual environmnts
- [x] Cache the parsing results to improve startup times
  - [x] reload venvs to update caches
    - [ ] reload a single venv ("u" key)
    - [ ] sync on command ("U" key)
  - [ ] cache with unique ids so venvs with same names don't collide
  - [x] automatically detect changes of venvs and update cache
  - [x] check cache updates in a separate thread
  - [ ] command to clean up cache
- [x] display Package and Venv's last modified dates

# License

Copyright (c) Ardnys

This project is licensed under the MIT license ([LICENSE][LICENSE] or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

[Ratatui]: https://ratatui.rs
[LICENSE]: ./LICENSE
