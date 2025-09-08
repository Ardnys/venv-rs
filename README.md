# venv-rs

venv-rs is a high level Python virtual environment manager specifically developed for [my personal workflow and needs](https://ardnys.github.io/projects/venv-manager/).

> [!WARNING]
> this project is in heavy development. i keep adding, removing, breaking and changing things.
> scope may change as well so even take this README with a generous sprinkle of top quality Himalayan salt. _crunch crunch_

## Features

- Shows virtual environments, their size on disk, number of packages
- Shows packages, versions, and sizes on disk
- Copies activation command on exit
- Prints requirements.txt so you don't have to activate it and print it manually
- Cross platform. I use it in both Command Prompt and Git Bash on Windows.
- Kind of satisfying to use imo

## Roadmap

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
  - [x] automatically detect changes of venvs and update cache
  - [x] check cache updates in a separate thread
- [x] display Package and Venv's last modified dates

## License

Copyright (c) Ardnys

This project is licensed under the MIT license ([LICENSE][LICENSE] or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

[Ratatui]: https://ratatui.rs
[LICENSE]: ./LICENSE
