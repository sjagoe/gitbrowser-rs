use std::ffi::OsStr;
use std::path::Path;

use git2::{Blob, Object, Repository};

use ratatui::{
    layout::Rect,
    prelude::{Color, Line, Modifier, Span, Style},
    widgets::{Block, Paragraph},
    Frame,
};

use color_eyre::Result;

use syntect::easy::HighlightLines;
use syntect::highlighting;
use syntect::parsing::SyntaxSet;
use two_face::re_exports::syntect;

use crate::errors::{ErrorKind, GitBrowserError};
use crate::traits::{Drawable, Navigable};

pub struct BlobPager<'repo> {
    top: usize,
    // repo: &'repo Repository,
    pub blob: Blob<'repo>,
    pub name: String,
    background_style: Style,
    // syntax_set: &'syntax SyntaxSet,
    // syntax: Option<SyntaxReference>,
    // theme: &'syntax highlighting::Theme,
    // highlighter: Option<HighlightLines<'syntax>>,
    // raw_lines: Vec<String>,
    lines: Vec<HighlightedLine>,
}

struct HighlightedLine {
    pub components: Vec<(Style, String)>,
}

impl<'a> From<Vec<(highlighting::Style, &'a str)>> for HighlightedLine {
    fn from(value: Vec<(highlighting::Style, &'a str)>) -> HighlightedLine {
        HighlightedLine {
            components: value
                .iter()
                .map(|(style, text)| (to_style(style), text.to_string()))
                .collect(),
        }
    }
}

fn to_color(hcolor: &highlighting::Color) -> Color {
    Color::Rgb(hcolor.r, hcolor.g, hcolor.b)
}

fn to_style(hstyle: &highlighting::Style) -> Style {
    Style::default()
        .fg(to_color(&hstyle.foreground))
        .bg(to_color(&hstyle.background))
}

impl<'repo> BlobPager<'repo> {
    pub fn new(
        _repo: &'repo Repository,
        blob: Blob<'repo>,
        name: String,
        syntax_set: &SyntaxSet,
        theme: &highlighting::Theme,
    ) -> BlobPager<'repo> {
        let content = match std::str::from_utf8(blob.content()) {
            Ok(v) => v,
            Err(e) => panic!("unable to decode utf8 {}", e),
        };
        let raw_lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();

        let syntax = {
            let extension = Path::new(&name).extension().and_then(OsStr::to_str);
            if let Some(ext) = extension {
                syntax_set.find_syntax_by_extension(ext).cloned()
            } else if let Some(line) = raw_lines.first() {
                syntax_set.find_syntax_by_first_line(line).cloned()
            } else {
                None
            }
        };
        let background_style = match syntax {
            Some(_) => {
                if let Some(color) = theme.settings.background {
                    Style::default().bg(to_color(&color))
                } else {
                    Style::default()
                }
            }
            _ => Style::default(),
        };

        let mut highlighter = syntax.as_ref().map(|s| HighlightLines::new(s, theme));

        let lines: Vec<HighlightedLine> = raw_lines
            .iter()
            .map(|text| match &mut highlighter {
                Some(h) => HighlightedLine::from(h.highlight_line(text, syntax_set).unwrap()),
                _ => HighlightedLine {
                    components: vec![(Style::default(), text.to_string())],
                },
            })
            .collect();

        BlobPager {
            top: 0,
            // repo: repo,
            blob: blob.clone(),
            name,
            background_style,
            // syntax_set,
            // syntax,
            // theme,
            // highlighter,
            // raw_lines,
            lines,
        }
    }

    pub fn from_object(
        repo: &'repo Repository,
        object: Object<'repo>,
        name: String,
        syntax_set: &SyntaxSet,
        theme: &highlighting::Theme,
    ) -> Result<Self, GitBrowserError> {
        match object.into_blob() {
            Ok(blob) => {
                if blob.is_binary() {
                    Err(GitBrowserError::Error(ErrorKind::BinaryFile))
                } else {
                    Ok(BlobPager::new(repo, blob, name, syntax_set, theme))
                }
            }
            Err(_) => Err(GitBrowserError::Error(ErrorKind::BlobReference)),
        }
    }
}

impl<'repo> Drawable<'repo> for BlobPager<'repo> {
    fn draw(&self, f: &mut Frame, area: Rect, content_block_ext: Block) -> Rect {
        let content_block = content_block_ext.style(self.background_style);

        let viewport = content_block.inner(area);
        let height: usize = viewport.height.into();
        let bottom = if self.top + height > self.lines.len() {
            self.lines.len()
        } else {
            self.top + height
        };
        let filler: Vec<Line> = if bottom - self.top < height {
            let v: Vec<Line> = vec![Line::styled(
                "~",
                Style::default().add_modifier(Modifier::DIM),
            )];
            let len = height - (bottom - self.top);
            v.iter().cycle().take(len).cloned().collect()
        } else {
            vec![]
        };

        let lines: Vec<Line> = self.lines[self.top..bottom]
            .iter()
            .enumerate()
            .map(|(index, highlighted_line)| {
                let tmp = format!("{}", bottom);
                let width = tmp.len();
                let formatted = format!("{:width$} | ", index + self.top);
                let lineno = Span::styled(formatted, Style::default().add_modifier(Modifier::DIM));

                let mut spans = highlighted_line
                    .components
                    .iter()
                    .map(|(style, text)| Span::styled(text, *style))
                    .collect();
                let mut line = vec![lineno];
                line.append(&mut spans);

                Line::from(line)
            })
            .collect();
        let filled_lines: Vec<Line> = lines
            .iter()
            .cloned()
            .chain(filler.iter().cloned())
            .collect();
        let content =
            Paragraph::new(filled_lines.into_iter().collect::<Vec<Line>>()).block(content_block);
        f.render_widget(content, area);
        viewport
    }

    fn title(&self) -> String {
        self.name.to_string()
    }
}

impl<'repo> Navigable<'repo> for BlobPager<'repo> {
    fn home(&mut self, _page_size: u16) {
        self.top = 0;
    }

    fn end(&mut self, page_size: u16) {
        let h: usize = page_size.into();
        self.top = if !self.lines.is_empty() {
            self.lines.len() - h
        } else {
            0
        };
    }

    fn pagedown(&mut self, page_size: u16) {
        let h: usize = page_size.into();
        let top = self.top + h;
        self.top = if top > self.lines.len() {
            self.lines.len() - 1
        } else {
            top
        }
    }

    fn pageup(&mut self, page_size: u16) {
        let h: usize = page_size.into();
        if self.top < h {
            self.top = 0;
        } else {
            self.top -= h;
        }
    }

    fn next_selection(&mut self) {
        // Always keep the last line on the screen
        if self.top < self.lines.len() - 1 {
            self.top += 1;
        }
    }

    fn previous_selection(&mut self) {
        if self.top > 0 {
            self.top -= 1;
        }
    }

    fn select(&self) -> Option<(Object<'repo>, String)> {
        None
    }

    fn selected_item(&self) -> String {
        "".to_string()
    }

    fn next_tick(&mut self) {
    }
}
