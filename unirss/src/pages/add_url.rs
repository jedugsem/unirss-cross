use crate::{com::Com, comps::searchbar, per, Controls, Element, Message};
use iced::{
    alignment::Horizontal::Right,
    widget::{button, column, text},
    Alignment::Center,
    Length::Fill,
};
use iced_widget::text_editor;
use iced_winit::runtime::Task;
#[derive(Clone, Debug)]
pub enum AddM {
    Add,
    ChangeUrl(String),
    PasteUrl,
    PasteName,
    ChangeName(String),
}

impl From<AddM> for Message {
    fn from(m: AddM) -> Message {
        Message::Add(m)
    }
}
impl Controls {
    pub fn view_add_url(&self) -> Element<Message> {
        let _list: Element<Message> = if let Some(channels) = &self.state.channels {
            let mut col: Vec<Element<Message>> = vec![];
            for i in channels {
                let title = i.0.title.clone().unwrap().content;
                col.push(button(text(title)).into());
            }
            column!(text("found zero"), column(col)).into()
        } else {
            column!(text("nothing found")).into()
        };
        column!(
            text("Name:"),
            searchbar(
                &self.name_editor,
                |s| Message::EditorAction(s, 0),
                Some(|| AddM::PasteName.into())
            ),
            text("Url:"),
            searchbar(
                &self.url_editor,
                |s| Message::EditorAction(s, 1),
                Some(|| AddM::PasteUrl.into())
            ),
            column!(button("Add").on_press(Message::Add(AddM::Add)))
                .width(Fill)
                .align_x(Right),
        )
        .align_x(Center)
        .padding(15)
        .spacing(10)
        .into()
    }

    pub fn update_add_url(&mut self, m: AddM) -> Task<Message> {
        match m {
            AddM::PasteName => {
                #[cfg(target_os = "android")]
                {
                    _ = self.proxy.send_event(crate::UserEvent::ClipboardRead(0));
                }
                Com::none()
            }

            AddM::PasteUrl => {
                #[cfg(target_os = "android")]
                {
                    _ = self.proxy.send_event(crate::UserEvent::ClipboardRead(1));
                }

                Com::none()
            }
            AddM::Add => {
                let name = self.name_editor.text();
                let url = self.url_editor.text();
                self.url_editor = text_editor::Content::new();
                self.name_editor = text_editor::Content::new();
                self.state
                    .online_feeds
                    .urls
                    .push((name, url.trim().to_string()));
                let feeds = self.state.online_feeds.clone();
                let git_state = self.state.settings.git.clone().unwrap();

                Com::perform(
                    self,
                    async move { per::write_online_feeds(feeds, git_state).await },
                    |_s| Message::Refresh(Ok(())),
                )
            }
            AddM::ChangeUrl(_cont) => Com::none(),
            AddM::ChangeName(_cont) => Com::none(),
        }
    }
}
