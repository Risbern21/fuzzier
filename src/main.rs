use std::path::PathBuf;

use iced::{
    Alignment, Background, Border, Color, Element, Font, Length, Settings, Task, Theme,
    border::Radius,
    theme,
    widget::{
        self, Grid, Scrollable, button, center, column, container, pick_list, row,
        scrollable::{Scroller, Status as ScrollableStatus},
        space::horizontal,
        text, text_input,
    },
};
use vecstore::Neighbor;

use crate::images::{Error, find_similar_images};

mod images;

#[tokio::main]
pub async fn main() -> iced::Result {
    // tokio::spawn(embed_image_directory(PathBuf::from(
    //     "/home/risbern21/Pictures",
    // )));

    iced::application(Fuzzier::new, Fuzzier::update, Fuzzier::view)
        .title("Fuzzier")
        .theme(Fuzzier::theme)
        .settings(Settings {
            default_font: Font::MONOSPACE,
            fonts: vec![
                include_bytes!("../fonts/fuzzier-icons.ttf")
                    .as_slice()
                    .into(),
            ],
            ..Default::default()
        })
        .run()
}

struct Fuzzier {
    file_name: String,
    file_type: FileType,
    files_found: Option<Vec<Neighbor>>,
    theme: theme::Theme,
    grid_columns: usize,
    selected_file: Option<usize>,
    error: Option<Error>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileType {
    All,
    Text,
    Images,
}

#[derive(Debug, Clone)]
enum Message {
    ChangeTheme(theme::Theme),
    LoadAllFiles,
    LoadTextFiles,
    LoadImages,
    FileNameEntered(String),
    FilesFound(Result<Vec<Neighbor>, Error>),
    FileSelected(usize),
    SearchFile,
}

impl Fuzzier {
    fn new() -> Self {
        Self {
            file_name: String::from(""),
            file_type: FileType::All,
            files_found: None,
            theme: theme::Theme::SolarizedDark,
            grid_columns: 6,
            selected_file: None,
            error: None,
        }
    }

    fn theme(fuzzier: &Fuzzier) -> Option<Theme> {
        Some(fuzzier.theme.clone())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadAllFiles => {
                self.file_type = FileType::All;
                Task::none()
            }
            Message::LoadTextFiles => {
                self.file_type = FileType::Text;
                Task::none()
            }
            Message::LoadImages => {
                self.file_type = FileType::Images;
                Task::none()
            }
            Message::FileNameEntered(name) => {
                self.file_name = name;
                Task::none()
            }
            Message::SearchFile => {
                self.error = None;
                self.selected_file = None;
                Task::perform(
                    find_similar_images(self.file_name.clone(), 5),
                    Message::FilesFound,
                )
            }
            Message::FilesFound(Ok(result)) => {
                self.files_found = Some(result);
                Task::none()
            }
            Message::FilesFound(Err(error)) => {
                self.error = Some(error);
                Task::none()
            }
            Message::FileSelected(index) => {
                self.selected_file = Some(index);
                Task::none()
            }
            Message::ChangeTheme(theme) => {
                self.theme = theme;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Toolbar
        let toolbar = {
            let title = text("Fuzzier").size(18).font(Font::MONOSPACE);

            let search_bar = text_input("Search images by description…", &self.file_name)
                .on_input(Message::FileNameEntered)
                .on_submit(Message::SearchFile)
                .padding(8)
                .width(Length::Fixed(340.0));

            let search_button = button(text("Search").size(14).center())
                .on_press(Message::SearchFile)
                .padding([8, 16])
                .style(primary_button_style);

            let theme_picker = pick_list(Theme::ALL, Some(&self.theme), Message::ChangeTheme)
                .padding(8)
                .width(Length::Fixed(170.0));

            container(
                row![title, horizontal(), search_bar, search_button, theme_picker,]
                    .spacing(12)
                    .align_y(Alignment::Center),
            )
            .padding(12)
            .width(Length::Fill)
            .style(toolbar_style)
        };

        // Sidebar
        let sidebar = {
            let items = column![
                sidebar_item(
                    "All Files",
                    folder_icon(18.0),
                    self.file_type == FileType::All,
                    Message::LoadAllFiles,
                ),
                sidebar_item(
                    "Text",
                    text_file_icon(18.0),
                    self.file_type == FileType::Text,
                    Message::LoadTextFiles,
                ),
                sidebar_item(
                    "Images",
                    image_file_icon(18.0),
                    self.file_type == FileType::Images,
                    Message::LoadImages,
                ),
            ]
            .spacing(4);

            container(column![text("Locations").size(12), items].spacing(8))
                .padding(12)
                .width(Length::Fixed(190.0))
                .height(Length::Fill)
                .style(sidebar_style)
        };

        // File grid
        let grid_view: Element<'_, Message> = match &self.files_found {
            Some(files_found) if !files_found.is_empty() => {
                let mut file_grid = Grid::new().columns(self.grid_columns).spacing(18);

                for (index, file) in files_found.iter().enumerate() {
                    let file_val = match file.metadata.fields.get("file_name") {
                        Some(v) => match v.as_str() {
                            Some(s) => PathBuf::from(s),
                            None => continue,
                        },
                        None => continue,
                    };

                    let file_name = file_val
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    let file_extension = file_val
                        .extension()
                        .map(|e| e.to_string_lossy().to_lowercase())
                        .unwrap_or_default();

                    let selected = self.selected_file == Some(index);

                    let tile_content = column![
                        container(icon_for_extension(&file_extension)).center(56),
                        text(file_name).size(12).center(),
                    ]
                    .align_x(Alignment::Center)
                    .spacing(6)
                    .width(Length::Fixed(96.0));

                    let tile = button(tile_content)
                        .on_press(Message::FileSelected(index))
                        .padding(10)
                        .style(move |theme: &Theme, status| {
                            file_tile_style(theme, status, selected)
                        });

                    file_grid = file_grid.push(tile);
                }

                Scrollable::new(container(file_grid).padding(20).width(Length::Fill))
                    .style(scrollable_grid_style)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            Some(_) => center(
                column![
                    text("No results").size(16),
                    text("Try a different search term").size(12),
                ]
                .align_x(Alignment::Center)
                .spacing(6),
            )
            .into(),
            None => center(
                column![
                    text("Search for something").size(16),
                    text("Results will appear here as icons, just like your file manager").size(12),
                ]
                .align_x(Alignment::Center)
                .spacing(6),
            )
            .into(),
        };

        let content = container(grid_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(content_style);

        // Status bar
        let status_bar = {
            let status_text = if self.error.is_some() {
                "Something went wrong while searching".to_string()
            } else {
                let count = self.files_found.as_ref().map(|f| f.len()).unwrap_or(0);
                format!("{count} item{}", if count == 1 { "" } else { "s" })
            };

            container(text(status_text).size(12))
                .padding([6, 12])
                .width(Length::Fill)
                .style(status_bar_style)
        };

        let body = row![sidebar, content].height(Length::Fill);

        column![toolbar, body, status_bar].into()
    }
}

fn toolbar_style(theme: &Theme) -> widget::container::Style {
    let palette = theme.extended_palette();
    widget::container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            color: palette.background.strong.color,
            width: 0.0,
            radius: Radius::new(0.0),
        },
        text_color: Some(palette.background.base.text),
        ..Default::default()
    }
}

fn sidebar_style(theme: &Theme) -> widget::container::Style {
    let palette = theme.extended_palette();
    widget::container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: Radius::new(0.0),
        },
        text_color: Some(palette.background.base.text),
        ..Default::default()
    }
}

fn content_style(theme: &Theme) -> widget::container::Style {
    let palette = theme.extended_palette();
    widget::container::Style {
        background: Some(Background::Color(palette.background.base.color)),
        ..Default::default()
    }
}

fn status_bar_style(theme: &Theme) -> widget::container::Style {
    let palette = theme.extended_palette();
    widget::container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: Radius::new(0.0),
        },
        text_color: Some(palette.background.base.text.scale_alpha(0.7)),
        ..Default::default()
    }
}

fn sidebar_button_style(
    theme: &Theme,
    status: widget::button::Status,
    selected: bool,
) -> widget::button::Style {
    let palette = theme.extended_palette();

    let background = if selected {
        Some(Background::Color(palette.primary.base.color))
    } else {
        match status {
            widget::button::Status::Hovered => {
                Some(Background::Color(palette.background.strong.color))
            }
            _ => None,
        }
    };

    let text_color = if selected {
        palette.primary.base.text
    } else {
        palette.background.base.text
    };

    widget::button::Style {
        background,
        text_color,
        border: Border {
            radius: Radius::new(6.0),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn file_tile_style(
    theme: &Theme,
    status: widget::button::Status,
    selected: bool,
) -> widget::button::Style {
    let palette = theme.extended_palette();

    let background = if selected {
        Some(Background::Color(palette.primary.weak.color))
    } else {
        match status {
            widget::button::Status::Hovered => {
                Some(Background::Color(palette.background.weak.color))
            }
            _ => None,
        }
    };

    let border_color = if selected {
        palette.primary.base.color
    } else {
        Color::TRANSPARENT
    };

    widget::button::Style {
        background,
        text_color: palette.background.base.text,
        border: Border {
            color: border_color,
            width: 1.5,
            radius: Radius::new(10.0),
        },
        ..Default::default()
    }
}

fn primary_button_style(theme: &Theme, status: widget::button::Status) -> widget::button::Style {
    let palette = theme.extended_palette();

    let background = match status {
        widget::button::Status::Hovered | widget::button::Status::Pressed => {
            palette.primary.strong.color
        }
        _ => palette.primary.base.color,
    };

    widget::button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.primary.base.text,
        border: Border {
            radius: Radius::new(6.0),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn scrollable_grid_style(theme: &Theme, status: ScrollableStatus) -> widget::scrollable::Style {
    let palette = theme.extended_palette();
    let mut style = widget::scrollable::default(theme, status);

    style.vertical_rail.scroller = Scroller {
        background: Background::Color(palette.primary.base.color),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::new(10.0),
        },
    };

    style
}
fn sidebar_item<'a>(
    label: &'a str,
    icon: Element<'a, Message>,
    selected: bool,
    on_press: Message,
) -> Element<'a, Message> {
    let content = row![
        container(icon).width(Length::Fixed(24.0)).center_y(24),
        text(label).size(13),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    button(content)
        .on_press(on_press)
        .padding([8, 10])
        .width(Length::Fill)
        .style(move |theme: &Theme, status| sidebar_button_style(theme, status, selected))
        .into()
}

fn icon_for_extension<'a>(extension: &str) -> Element<'a, Message> {
    match extension {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "tiff" | "heic" => {
            image_file_icon(36.0)
        }
        "txt" | "md" | "rs" | "toml" | "json" | "yaml" | "yml" | "csv" | "log" | "conf" => {
            text_file_icon(36.0)
        }
        _ => generic_file_icon(36.0),
    }
}

#[allow(dead_code)]
fn generic_file_icon<'a>(size: f32) -> Element<'a, Message> {
    icon('\u{E800}', size)
}

fn image_file_icon<'a>(size: f32) -> Element<'a, Message> {
    icon('\u{E802}', size)
}

fn text_file_icon<'a>(size: f32) -> Element<'a, Message> {
    icon('\u{F0F6}', size)
}

fn folder_icon<'a>(size: f32) -> Element<'a, Message> {
    icon('\u{F114}', size)
}

#[allow(dead_code)]
fn empty_folder_icon<'a>(size: f32) -> Element<'a, Message> {
    icon('\u{F115}', size)
}

fn icon<'a>(codepoint: char, size: f32) -> Element<'a, Message> {
    const FONT_ICON: Font = Font::with_name("fuzzier-icons");
    text(codepoint).font(FONT_ICON).size(size).center().into()
}
