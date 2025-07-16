use std::borrow::Cow;

use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};

pub fn create_comfy_table(path_str: Cow<'_, str>) -> Table {
    let mut table = Table::new();

    let header = vec![Cell::new("Shell"), Cell::new("Command to activate")];
    let rows = vec![
        vec![
            Cell::new("bash/zsh"),
            Cell::new(format!("source {}/bin/activate", path_str))
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
        ],
        vec![
            Cell::new("fish"),
            Cell::new(format!("source {}/bin/activate.fish", path_str))
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
        ],
        vec![
            Cell::new("csh/tsch"),
            Cell::new(format!("source {}/bin/activate.csh", path_str))
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
        ],
        vec![
            Cell::new("pwsh"),
            Cell::new(format!("{}/bin/Activate.ps1", path_str))
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
        ],
        vec![
            Cell::new("cmd.exe"),
            Cell::new(format!("{}\\Scripts\\activate.bat", path_str))
                .fg(Color::Blue)
                .add_attribute(Attribute::Bold),
        ],
        vec![
            Cell::new("PowerShell"),
            Cell::new(format!("{}\\Scripts\\Activate.ps1", path_str))
                .fg(Color::Blue)
                .add_attribute(Attribute::Bold),
        ],
    ];

    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(100)
        .set_header(header)
        .add_rows(rows);

    table
}
