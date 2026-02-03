mod sidebar;
mod theme;

use iced::widget::{container, row, Text};
use iced::{Element, Length, Theme, Task};
use sidebar::{Sidebar, SidebarMessage};

pub fn main() -> iced::Result {
    iced::application(
        || (FeatherAlloy::default(), Task::perform(theme::detect_system_theme(), Message::ThemeDetected)),
        FeatherAlloy::update,
        FeatherAlloy::view,
    )
    .title(|_: &FeatherAlloy| "Feather Alloy".to_string())
    .theme(|_: &FeatherAlloy| Theme::Dark)
    .run()
}

struct FeatherAlloy {
    sidebar: Sidebar,
    theme_colors: theme::ThemeColors,
}

#[derive(Debug, Clone)]
enum Message {
    Sidebar(SidebarMessage),
    ThemeDetected(theme::ThemeColors),
}

impl FeatherAlloy {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Sidebar(sidebar_msg) => {
                self.sidebar.update(sidebar_msg);
            }
            Message::ThemeDetected(colors) => {
                self.theme_colors = colors;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let colors = self.theme_colors;
        let content = container(
            container(Text::new(format!("Conteúdo da Webview - Accent: {:?}", colors.accent)))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |theme| theme::content_container(theme, colors));

        row![
            self.sidebar.view(colors).map(Message::Sidebar),
            content,
        ]
        .into()
    }
}

impl Default for FeatherAlloy {
    fn default() -> Self {
        Self {
            sidebar: Sidebar::new(),
            theme_colors: theme::ThemeColors::default(),
        }
    }
}