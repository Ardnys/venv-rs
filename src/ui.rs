use chrono::{DateTime, Local};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, HighlightSpacing, LineGauge, List, ListItem, Padding, Paragraph,
        Scrollbar, ScrollbarOrientation, StatefulWidget, Widget, Wrap,
    },
};
use venv_rs::dir_size::{Chonk, ParallelReader};

use crate::app::App;

const PANEL_STYLE: Style = Style::new().fg(Color::White);
const FOCUSED_PANEL_STYLE: Style = Style::new().fg(Color::Green);
const SELECTED_STYLE: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::Blue)
    .add_modifier(Modifier::BOLD);

impl Widget for &mut App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [main, footer_chunk] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(97), Constraint::Percentage(3)])
            .areas(area);

        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(main);

        let [packages_layout, pkg_details_layout] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .areas(right);

        let [pkg_details, pkg_dependencies] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(pkg_details_layout);

        let sync_size = if self.syncing { 3 } else { 0 };

        let [pkg_dependencies, progress] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Min(sync_size)])
            .areas(pkg_dependencies);

        let [venv_layout, venv_details_layout] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .areas(left);

        let footer_block = Block::new()
            .borders(Borders::empty())
            .padding(Padding::left(1));

        let footer_text = String::from(
            "Exit: q | Movement: hjkl or ↓ ↑ ← → | Activate: a | Requirements: r | Update: u | Help: ?",
        );

        let footer = Paragraph::new(footer_text)
            .block(footer_block)
            .fg(Color::Blue)
            .left_aligned();

        footer.render(footer_chunk, buf);

        self.render_venvs(venv_layout, buf);
        self.render_packages(packages_layout, buf);
        self.render_package_details(pkg_details, buf);
        self.render_package_dependencies(pkg_dependencies, buf);
        self.render_venv_details(venv_details_layout, buf);

        if self.syncing {
            self.render_sync_text(progress, buf);
        }

        if self.show_help {
            self.render_help(area, buf);
        }

        if self.maybe_error.is_some() {
            self.render_error(area, buf);
        }
    }
}

// const fn alternate_colors(i: usize) -> Color {
//     if i % 2 == 0 {
//         NORMAL_ROW_BG
//     } else {
//         ALT_ROW_BG_COLOR
//     }
// }

// rendering for app
impl App {
    fn render_venvs(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Virtual Environments").centered())
            .borders(Borders::ALL)
            .border_style(match self.current_focus {
                crate::app::Panel::Venv => FOCUSED_PANEL_STYLE,
                _ => PANEL_STYLE,
            });

        let items: Vec<ListItem> = self
            .venv_list
            .venvs
            .iter()
            .map(|vui| ListItem::from(vui.venv.name.clone()))
            .collect();

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = self.venv_list.scroll_state.position(self.venv_index);

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // render the list before rendering its scroll
        StatefulWidget::render(list, area, buf, &mut self.venv_list.list_state);
        StatefulWidget::render(
            scrollbar,
            area.inner(Margin {
                // using an inner vertical margin of 1 unit makes the scrollbar inside the block
                vertical: 1,
                horizontal: 0,
            }),
            buf,
            &mut scrollbar_state,
        );
    }

    fn render_packages(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Packages").centered())
            .borders(Borders::ALL)
            .border_style(match self.current_focus {
                crate::app::Panel::Packages => FOCUSED_PANEL_STYLE,
                _ => PANEL_STYLE,
            });

        let mut v = self.get_selected_venv_ui();
        let style = Style::default();
        let no_dependency_style = Style::default().magenta().italic();

        let items: Vec<ListItem> = v
            .venv
            .packages
            .iter()
            .map(|pack| {
                let mut item = ListItem::from(pack.name.clone());
                if pack.metadata.dependencies.is_none() {
                    item = item.style(no_dependency_style);
                } else {
                    item = item.style(style);
                }
                item
            })
            .collect();

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = v.scroll_state.position(self.packages_index);

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // render the list before rendering its scroll
        StatefulWidget::render(list, area, buf, &mut v.list_state);
        StatefulWidget::render(
            scrollbar,
            area.inner(Margin {
                // using an inner vertical margin of 1 unit makes the scrollbar inside the block
                vertical: 1,
                horizontal: 0,
            }),
            buf,
            &mut scrollbar_state,
        );
    }

    fn render_package_details(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Package Details").centered())
            .borders(Borders::ALL)
            .border_style(PANEL_STYLE);

        let package = self.get_selected_package();
        let style = Style::new().yellow().italic();

        let datetime: DateTime<Local> = package.last_modified.into();
        let fmt_date = datetime.format("%Y-%m-%d %H:%M");

        let details = vec![
            Line::from(Span::styled(format!("Name:     {}", package.name), style)),
            Line::from(Span::styled(
                format!("Version:  {}", package.version),
                style,
            )),
            Line::from(Span::styled(
                format!("Summary:  {}", package.metadata.summary),
                style,
            )),
            Line::from(Span::styled(
                format!("Size: {}", ParallelReader::formatted_size(package.size)),
                style,
            )),
            Line::from(Span::styled(format!("Last Modified: {}", fmt_date), style)),
            if package.metadata.dependencies.is_some() {
                Line::from(Span::styled(
                    format!(
                        "Num Dependencies: {}",
                        package.metadata.dependencies.unwrap().len()
                    ),
                    style,
                ))
            } else {
                Line::from("")
            },
        ];

        let p = Paragraph::new(details)
            .block(block)
            .wrap(Wrap::default())
            .alignment(Alignment::Left);

        p.render(area, buf);
    }

    fn render_package_dependencies(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Package Depedencies").centered())
            .borders(Borders::ALL)
            .border_style(PANEL_STYLE);

        let package = self.get_selected_package();
        let style = Style::new().red().bold();
        let no_dep_style = Style::new().magenta().italic();

        let deps: Vec<Line> = package
            .metadata
            .dependencies
            .unwrap_or_default()
            .iter()
            .map(|d| Line::from(Span::styled(d.to_string(), style)))
            .collect();

        let p = if deps.is_empty() {
            Paragraph::new(Text::styled("! No Dependencies !", no_dep_style))
        } else {
            Paragraph::new(deps)
        };
        let p = p.block(block).alignment(Alignment::Left);

        p.render(area, buf);
    }

    fn render_venv_details(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Venv Details").centered())
            .borders(Borders::ALL)
            .border_style(PANEL_STYLE);

        let venv = self.get_selected_venv_ui().venv;
        let style = Style::new().light_blue().italic();

        let datetime = self.get_selected_venv_ui_ref().last_modified;
        let fmt_date = datetime.format("%Y-%m-%d %H:%M");

        let details = vec![
            Line::from(Span::styled(
                format!("Name:           {}", venv.name),
                style,
            )),
            Line::from(Span::styled(
                format!("Version:        {}", venv.version),
                style,
            )),
            Line::from(Span::styled(
                format!("Path:           {}", venv.path.to_string_lossy()),
                style,
            )),
            Line::from(Span::styled(
                format!("# of Pkg:       {}", venv.num_dist_info_packages),
                style,
            )),
            Line::from(Span::styled(
                format!(
                    "Size:           {}",
                    ParallelReader::formatted_size(venv.size)
                ),
                style,
            )),
            Line::from(Span::styled(format!("Last Modified:  {}", fmt_date), style)),
        ];

        let p = Paragraph::new(details)
            .block(block)
            .alignment(Alignment::Left);

        p.render(area, buf);
    }

    fn render_help(&mut self, area: Rect, buf: &mut Buffer) {
        // Create centered rect: 60% width, 70% height
        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(15), // top padding
                Constraint::Percentage(70), // help box
                Constraint::Percentage(15), // bottom padding
            ])
            .split(area)[1];

        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // left padding
                Constraint::Percentage(60), // help box
                Constraint::Percentage(20), // right padding
            ])
            .split(popup_area)[1];

        // Clear the part where help popup is rendered
        Clear.render(popup_area, buf);

        let block = Block::new()
            .title(Line::styled(
                " Help / Keybinds ",
                Style::new().bold().yellow(),
            ))
            .borders(Borders::ALL)
            .border_style(FOCUSED_PANEL_STYLE);

        let actions: Vec<(&str, &str)> = vec![
            ("q", "Exit"),
            ("a", "Activate selected venv"),
            ("r", "Print requirements and exit"),
            ("u", "Parse the venv and update cache"),
            ("?", "Toggle keybinds"),
        ];

        let navigations: Vec<(&str, &str)> = vec![
            ("j / ↓", "Scroll down"),
            ("k / ↑", "Scroll up"),
            ("h / ←", "Switch to left pane"),
            ("l / →", "Switch to right pane"),
            ("Ctrl+d / PgDn", "Half page down"),
            ("Ctrl+u / PgUp", "Half page up"),
            ("J / Ctrl+↓", "Scroll last"),
            ("K / Ctrl+↑", "Scroll first"),
        ];

        /* layout kinda looks like this
                * ┌──────────────┐
                  │action title  │
                  └──────────────┘
                  ┌──────┐┌──────┐
                  │ key  ││ desc │
                  │      ││      │
                  │      ││      │
                  │      ││      │
                  └──────┘└──────┘
                  ┌──────────────┐
                  │navig title   │
                  └──────────────┘
                  ┌──────┐┌──────┐
                  │ key  ││ desc │
                  │      ││      │
                  │      ││      │
                  │      ││      │
                  │      ││      │
                  └──────┘└──────┘
        */

        let [
            action_title_layout,
            action_keybind_layout,
            navigation_title_layout,
            navigation_keybind_layout,
            _,
        ] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Fill(1),
            ])
            .areas(popup_area.inner(Margin {
                vertical: 1,
                horizontal: 2,
            }));

        let [action_key_layout, action_desc_layout] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .areas(action_keybind_layout);

        let [navigation_key_layout, navigation_desc_layout] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .areas(navigation_keybind_layout);

        // === Actions ===
        let action_title = Paragraph::new(Text::styled(
            "--- Actions ---",
            Style::new().green().italic(),
        ))
        .alignment(Alignment::Left)
        .block(Block::default().padding(Padding {
            left: 1,
            right: 1,
            top: 1,
            bottom: 0,
        }));

        let action_key_lines: Vec<Line> = actions
            .iter()
            .map(|(k, _)| Line::from(Span::styled(*k, Style::new().cyan())))
            .collect();

        let action_keys = Paragraph::new(action_key_lines)
            .alignment(Alignment::Left)
            .block(Block::default().padding(Padding::left(1)));

        let action_desc_lines: Vec<Line> = actions
            .iter()
            .map(|(_, d)| Line::from(Span::raw(*d)))
            .collect();

        let action_desc = Paragraph::new(action_desc_lines)
            .alignment(Alignment::Left)
            .block(Block::default().padding(Padding::left(1)));

        // === Navigations ===
        let navigation_title = Paragraph::new(Text::styled(
            "--- Navigations ---",
            Style::new().green().italic(),
        ))
        .alignment(Alignment::Left)
        .block(Block::default().padding(Padding {
            left: 1,
            right: 1,
            top: 1,
            bottom: 0,
        }));

        let navigation_key_lines: Vec<Line> = navigations
            .iter()
            .map(|(k, _)| Line::from(Span::styled(*k, Style::new().cyan())))
            .collect();

        let navigation_keys = Paragraph::new(navigation_key_lines)
            .alignment(Alignment::Left)
            .block(Block::default().padding(Padding::left(1)));

        let navigation_desc_lines: Vec<Line> = navigations
            .iter()
            .map(|(_, d)| Line::from(Span::raw(*d)))
            .collect();

        let navigation_desc = Paragraph::new(navigation_desc_lines)
            .alignment(Alignment::Left)
            .block(Block::default().padding(Padding::left(1)));

        // Render outer block
        block.render(popup_area, buf);

        // Render actions
        action_title.render(action_title_layout, buf);
        action_keys.render(action_key_layout, buf);
        action_desc.render(action_desc_layout, buf);

        // Render navigations
        navigation_title.render(navigation_title_layout, buf);
        navigation_keys.render(navigation_key_layout, buf);
        navigation_desc.render(navigation_desc_layout, buf);
    }

    fn render_error(&mut self, area: Rect, buf: &mut Buffer) {
        // Create centered rect: 60% width, 70% height
        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(15), // top padding
                Constraint::Percentage(70), // help box
                Constraint::Percentage(15), // bottom padding
            ])
            .split(area)[1];

        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // left padding
                Constraint::Percentage(60), // help box
                Constraint::Percentage(20), // right padding
            ])
            .split(popup_area)[1];

        // Clear the part where help popup is rendered
        Clear.render(popup_area, buf);

        let block = Block::new()
            .title(Line::styled(
                "! An Error Occured !",
                Style::new().bold().red(),
            ))
            .borders(Borders::ALL)
            .border_style(FOCUSED_PANEL_STYLE);

        if let Some(rep) = &self.maybe_error {
            let error_message = rep.to_string();
            let error_text = Text::styled(error_message.to_string(), Style::new().on_black().red());
            let error_paragraph = Paragraph::new(error_text).block(block);
            error_paragraph.render(area, buf);
        }
    }

    fn render_sync_text(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title("Syncing")
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center);

        let current = self.venv_sync_progress as usize;
        let total = self.total_venvs.max(1) as usize;

        // how wide is the inner area we can use for the label?
        let inner_width = area.width.saturating_sub(2) as usize;

        // right-side counter string
        let counter = format!("{}/{}", current, total);

        let avail = inner_width.saturating_sub(2 + counter.len());

        // build the label with formatting:
        // "[{name:-<avail}]{counter}"
        //   → left-align name inside a field of `avail` filled with '-'
        let label = format!(
            "[{:-<width$}]{}",
            self.current_syncing_venv,
            counter,
            width = avail
        );

        let label_line = Line::from(Span::styled(label, Style::new().italic().light_green()));
        let ratio = current as f64 / total as f64;

        LineGauge::default()
            .block(block)
            .ratio(ratio)
            .label(label_line)
            .render(area, buf);
    }
}
