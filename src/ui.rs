use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{
        Color, Modifier, Style, Stylize,
        palette::tailwind::{BLUE, GREEN, SLATE},
    },
    symbols,
    text::Line,
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, List, ListItem, Padding, Paragraph,
        StatefulWidget, Widget,
    },
};

use crate::app::App;

const TODO_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;

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
            .constraints([Constraint::Percentage(90), Constraint::Percentage(4)])
            .areas(area);

        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(main);

        fn create_block(title: String) -> Block<'static> {
            Block::bordered()
                .title(title)
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
        }

        let body_block = create_block("venv".to_string());
        let packages_block = create_block("packages".to_string());
        let footer_block = Block::new()
            .borders(Borders::empty())
            .padding(Padding::left(2));

        let text = format!(
            "This is a tui template.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                Counter: {}",
            self.counter
        );

        let packages_text = String::from("This is where packages will be shown");

        let footer_text = String::from(
            "Exit: q | Movement: hjkl or arrow keys | Activate: a | Install: i | Requirements: r",
        );

        let paragraph = Paragraph::new(text)
            .block(body_block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered();

        let packages = Paragraph::new(packages_text)
            .block(packages_block)
            .fg(Color::LightRed)
            .bg(Color::Black)
            .centered();

        let footer = Paragraph::new(footer_text)
            .block(footer_block)
            .fg(Color::Yellow)
            .bg(Color::Black)
            .left_aligned();

        // paragraph.render(left, buf);
        // packages.render(right, buf);
        footer.render(footer_chunk, buf);

        self.render_venvs(left, buf);
        self.render_packages(right, buf);
    }
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

// rendering for app
impl App {
    fn render_venvs(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Virtual Environments").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        let items: Vec<ListItem> = self
            .venvs
            .venvs
            .iter()
            .enumerate()
            .map(|(i, venv)| {
                let color = alternate_colors(i);
                ListItem::from(venv.name.clone()).bg(color)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.venvs.state);
    }

    fn render_packages(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Packages").centered())
            .borders(Borders::ALL)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // TODO: get selected venv and get them packages
        let mut v = self.get_selected_venv();

        let items: Vec<ListItem> = v
            .packages
            .iter()
            .enumerate()
            .map(|(i, pack)| {
                let color = alternate_colors(i);
                ListItem::from(pack.clone()).bg(color)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut v.state);
    }
}
