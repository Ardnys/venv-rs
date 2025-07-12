# venv-rs

venv-rs is a high level Python virtual environment manager specifically developed for my personal workflow and needs.

My python virtual environments easily get out of hand, especially when I work on AI projects. I get confused and end up with 5 pytorch installations taking half of my disk space. It's difficult to manage manually with terminal and file explorer. Thus, this weird thing has been developed!

Basically this will show me an overview of my virtual environments so I can keep them tidy.

> [!WARNING]
this project is in heavy development. i keep adding, removing, breaking and changing things.
scope may change as well so even take this README with a generous sprinkle of top quality Himalayan salt. _crunch crunch_

## Features
- Shows virtual environments, their size on disk, number of packages
- Shows packages, versions, and sizes on disk
- Prints activation command on exit
- Prints requirements.txt so you don't have to activate it and print it manually
- Kind of satisfying to use imo

## Roadmap
- [x] show packages in the venv
- [x] show package versions and sizes
- [x] human readable byte sizes
- [x] venv details (python version, size, num packages)
- [x] _encourage_ activation on exit
- [x] print the requirements.txt
- [ ] popup confirmation as a flag
- [ ] custom activation encouragement with shell detection 
- [ ] windows compatibility
- [ ] fix anyhow & color_eyre situation
- [ ] config file
- [ ] the directory tree and look for .env folders. that way i don't have to limit to this particular workflow.

[Ratatui]: https://ratatui.rs

## License

Copyright (c) Ardnys

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
