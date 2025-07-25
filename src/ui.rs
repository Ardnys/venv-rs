use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation, StatefulWidget, Widget,
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

        let [venv_layout, venv_details_layout] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .areas(left);

        let footer_block = Block::new()
            .borders(Borders::empty())
            .padding(Padding::left(1));

        let footer_text =
            String::from("Exit: q | Movement: hjkl or arrow keys | Activate: a | Requirements: r");

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
    }
}

// TODO: Is there a way to that while respecting user's terminal colors ?
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
            .map(|venv| ListItem::from(venv.name.clone()))
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

        let mut v = self.get_selected_venv();
        let style = Style::default();
        let no_dependency_style = Style::default().magenta().italic();

        let items: Vec<ListItem> = v
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

        let venv = self.get_selected_venv();
        let style = Style::new().light_blue().italic();
        let details = vec![
            Line::from(Span::styled(format!("Name:     {}", venv.name), style)),
            Line::from(Span::styled(format!("Version:  {}", venv.version), style)),
            Line::from(Span::styled(
                format!("Path:  {}", venv.path.to_string_lossy()),
                style,
            )),
            Line::from(Span::styled(
                format!("# of Pkg: {}", venv.num_dist_info_packages),
                style,
            )),
            Line::from(Span::styled(
                format!("Size:     {}", ParallelReader::formatted_size(venv.size)),
                style,
            )),
        ];

        let p = Paragraph::new(details)
            .block(block)
            .alignment(Alignment::Left);

        p.render(area, buf);
    }
}
