use std::fs;

use color_eyre::Result;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize, palette::tailwind::SLATE},
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget,
    },
};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::default().run(terminal);
    ratatui::restore();
    app_result
}

struct App {
    should_exit: bool,
    albums: DefaultList,
    tracks: DefaultList,
}

struct DefaultList {
    items: Vec<String>,
    state: ListState,
}

impl Default for App {
    fn default() -> Self {
        let albums = DefaultList::from_iter(get_albums("/home/guilherme/Musics/".to_string()));
        let tracks = DefaultList::from_iter(["Nothing selected...".to_string()].to_vec());

        Self {
            should_exit: false,
            albums,
            tracks,
        }
    }
}

impl FromIterator<String> for DefaultList {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let items = iter.into_iter().map(|album| album.to_string()).collect();
        let state = ListState::default();
        Self { items, state }
    }
}

impl DefaultList {
    fn select_next(&mut self) {
        self.state.select_next();
    }

    fn select_previous(&mut self) {
        self.state.select_previous();
    }

    fn select_first(&mut self) {
        self.state.select_first();
    }

    fn select_last(&mut self) {
        self.state.select_last();
    }
}

impl App {
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('j') | KeyCode::Down => self.tracks.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.tracks.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.albums.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.albums.select_last(),
            KeyCode::Tab => self.render_selected("next"),
            KeyCode::BackTab => self.render_selected("previous"),
            _ => {}
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [album_area, track_area] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .areas(main_area);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_albums(album_area, buf);
        self.render_tracks(track_area, buf);
    }
}

/// Rendering logic for the app
impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Rusty Player")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to select, ←/→ or TAB to move, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    fn render_albums(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Album List").centered())
            .borders(Borders::all());

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .albums
            .items
            .iter()
            .map(|todo_item| {
                // let color = alternate_colors(i);
                ListItem::new(todo_item.as_str())
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("•")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.albums.state);
    }

    fn render_tracks(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title(Line::raw("Tracks").centered());

        let items: Vec<ListItem> = self
            .tracks
            .items
            .iter()
            .map(|item| {
                // let color = alternate_colors(i);
                ListItem::new(item.as_str())
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("•")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.tracks.state.clone());
    }

    fn render_selected(&mut self, target: &'static str) {
        match target {
            "next" => self.albums.select_next(),
            "previous" => self.albums.select_previous(),
            _ => {}
        }

        let selected: Vec<String> = if let Some(i) = self.albums.state.selected() {
            get_tracks(self.albums.items[i].clone())
        } else {
            vec!["Nothing selected".to_string()]
        };

        self.tracks = DefaultList::from_iter(selected);
    }
}

fn get_albums(path: String) -> Vec<String> {
    let mut albums = Vec::new();
    let dir = fs::read_dir(path);

    match dir {
        Ok(dir) => {
            dir.map(|i| i.unwrap())
                .for_each(|i| albums.push(String::from(i.file_name().to_str().unwrap())));
        }
        Err(dir) => {
            eprintln!("{:?}", dir);
        }
    }

    albums
}

fn get_tracks(album: String) -> Vec<String> {
    let mut tracks = Vec::new();
    let dir = fs::read_dir(format!("/home/guilherme/Music/{}", album));

    match dir {
        Ok(dir) => {
            dir.map(|i| i.unwrap())
                .for_each(|i| tracks.push(String::from(i.file_name().to_str().unwrap())));
        }
        Err(dir) => {
            eprintln!("{:?}", dir);
        }
    }

    tracks
}
