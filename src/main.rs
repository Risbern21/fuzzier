use iced::{
    Background, Border, Color, Element, Font, Task, Theme,
    border::Radius,
    theme,
    widget::{
        self, Grid, Scrollable, button, column, container, pick_list, row,
        scrollable::{Scroller, Status},
        space::{self, horizontal},
        text, text_input,
    },
};
use walkdir::DirEntry;

use crate::images::{Error, find_similar_images};

mod images;

pub fn main() -> iced::Result {
    iced::application(Fuzzier::new, Fuzzier::update, Fuzzier::view)
        .title("Fuzzier")
        .theme(Fuzzier::theme)
        .run()
}

struct Fuzzier {
    file_name: String,
    file_type: FileType,
    files_found: Option<Vec<DirEntry>>,
    theme: theme::Theme,
    viewport_width: u32,
    viewport_height: u32,
    grid_columns: usize,
    grid_width: u32,
    error: Option<Error>,
}

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
    FilesFound(Result<Vec<DirEntry>, Error>),
    SearchFile,
}

impl Fuzzier {
    fn new() -> Self {
        Self {
            file_name: String::from(""),
            file_type: FileType::All,
            files_found: Some(vec![]),
            theme: theme::Theme::SolarizedDark,
            viewport_width: 3000,
            viewport_height: 3000,
            grid_columns: 6,
            grid_width: 3000,
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
                println!("searching");
                Task::perform(
                    find_similar_images(self.file_name.clone()),
                    Message::FilesFound,
                )
            }
            Message::FilesFound(Ok(result)) => {
                println!("files were found");
                self.files_found = Some(result);
                Task::none()
            }
            Message::FilesFound(Err(error)) => {
                self.error = Some(error);
                Task::none()
            }
            Message::ChangeTheme(theme) => {
                self.theme = theme;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let control_bar = {
            let search_bar = {
                text_input(
                    "enter the file name or anything that closely resembles the file",
                    &self.file_name,
                )
                .on_input(Message::FileNameEntered)
            };

            let search_controls =
                row![search_bar, action("Search", Message::SearchFile)].spacing(10);

            row![
                pick_list(Theme::ALL, Some(&self.theme), Message::ChangeTheme),
                horizontal(),
                search_controls
            ]
        };

        let controls = {
            let buttons = column![
                action("All", Message::LoadAllFiles),
                action("Text", Message::LoadTextFiles),
                action("Images", Message::LoadImages),
            ]
            .spacing(10);

            column![buttons, space::vertical(),]
        };

        let mut my_grid = Grid::new()
            .columns(self.grid_columns)
            .width(self.grid_width)
            .spacing(10);

        match self.files_found {
            Some(ref files_found) => {
                for file in files_found {
                    let file_icon = container(column![
                        image_file_icon(),
                        text!("{}", file.file_name().to_os_string().into_string().unwrap())
                    ]);

                    my_grid = my_grid.push(file_icon);
                }
            }
            None => eprintln!("no files found"),
        }

        let scrollable_grid = Scrollable::new(my_grid).style(scrollable_grid_style);

        let viewport = container(scrollable_grid)
            .padding(20)
            .width(self.viewport_width)
            .height(self.viewport_height)
            .style(viewport_style);

        container(column![control_bar, row![controls, viewport].spacing(10)].spacing(10))
            .padding(10)
            .into()
    }
}

fn viewport_style(theme: &Theme) -> widget::container::Style {
    widget::container::Style {
        border: Border {
            color: theme.palette().primary.into(),
            width: 2.0.into(),
            radius: Radius::new(5.0),
        },
        ..Default::default()
    }
}

fn scrollable_grid_style<'a>(theme: &'a Theme, status: Status) -> widget::scrollable::Style {
    let mut style = widget::scrollable::default(&theme, status);

    style.vertical_rail.scroller = Scroller {
        background: Background::Color(Color::WHITE),
        border: Border {
            color: Color::BLACK,
            width: 0.0,
            radius: Radius {
                top_left: 10.0,
                top_right: 10.0,
                bottom_right: 10.0,
                bottom_left: 10.0,
            },
        },
    };

    style
}

fn file_icon_style(theme: &Theme) -> widget::container::Style {
    widget::container::Style {
        border: Border {
            color: theme.palette().primary.into(),
            width: 2.0.into(),
            radius: Radius::new(5.0),
        },
        ..Default::default()
    }
}

fn action<'a>(content: &'a str, on_press: Message) -> Element<'a, Message> {
    button(container(content).center_x(80).center_y(20))
        .on_press(on_press)
        .into()
}

fn generic_file_icon<'a>() -> Element<'a, Message> {
    icon('\u{E800}')
}

fn image_file_icon<'a>() -> Element<'a, Message> {
    icon('\u{E802}')
}

fn text_file_icon<'a>() -> Element<'a, Message> {
    icon('\u{F0F6}')
}

fn folder_icon<'a>() -> Element<'a, Message> {
    icon('\u{F114}')
}

fn empty_folder_icon<'a>() -> Element<'a, Message> {
    icon('\u{F115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const FONT_ICON: Font = Font::with_name("text-editor-fonts");

    text(codepoint).font(FONT_ICON).into()
}
