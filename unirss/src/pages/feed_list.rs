use crate::{com::Com, dir, per::*, Controls, Element, Message, Nav, Progress, Renderer};
use feed_rs::model::Feed;
use iced::{
    widget::{button, column, row, scrollable, text, Space},
    Alignment::Center,
    Length::Fill,
};
use iced_material::theme::container::grey_rounded;
use iced_widget::{container, image, Container};
use iced_winit::runtime::Task;
//use quizlib::*;

// Modules to be loaded
#[derive(Clone, Debug)]
pub enum FeedListM {
    Select(usize),
    ReadAll(usize),
    Remove(usize),
}

impl From<FeedListM> for Message {
    fn from(m: FeedListM) -> Message {
        Message::FeedList(m)
    }
}
pub fn feeds_unread(read: &Progress, feeds: &[(Feed, Imag)]) -> usize {
    feeds
        .iter()
        .filter(|feed| artikles_unread(read, &feed.0) > 0)
        .count()
}
pub fn artikles_unread(read: &Progress, feed: &Feed) -> usize {
    if let Some(already_read_feed) = read
        .read
        .iter()
        .find(|(feed_id, _artikles)| feed_id == &feed.id)
    {
        feed.entries.len() - already_read_feed.1.len()
    } else {
        feed.entries.len()
    }
}
impl Controls {
    pub fn view_feed_list(&self) -> Element<Message> {
        if let Some(_feed) = self.state.feed {
            self.view_artikle_list()
        } else {
            match self.state.window.tab {
                (0, Nav::All) => {
                    if let Some(channels) = &self.state.channels {
                        let mut unread_feeds: Vec<Element<Message>> = vec![];
                        for (i, feed) in channels.iter().enumerate() {
                            let unread: usize = artikles_unread(&self.state.read, &feed.0);
                            if unread != 0 {
                                let title = feed.0.title.clone().unwrap().content;
                                unread_feeds.push(
                                    feed_cont(
                                        title,
                                        FeedListM::Select(i).into(),
                                        Some(FeedListM::ReadAll(i).into()),
                                        Some(unread),
                                        Some(&feed.1),
                                    )
                                    .into(),
                                );
                            }
                        }
                        column!(scrollable(column(unread_feeds).padding(5).spacing(5)))
                            .padding(5)
                            .into()
                    } else {
                        text("hm").into()
                    }
                }
                (1, _) => {
                    let list: Element<Message> = if let Some(channels) = &self.state.channels {
                        let mut col: Vec<Element<Message>> = vec![];
                        for (i, feed) in channels.iter().enumerate() {
                            let title = feed.0.title.clone().unwrap().content;
                            col.push(
                                feed_cont(
                                    title,
                                    FeedListM::Select(i).into(),
                                    None,
                                    None,
                                    Some(&feed.1),
                                )
                                .into(),
                            );
                        }
                        column!(scrollable(column(col).padding(5).spacing(5)))
                            .padding(5)
                            .into()
                    } else {
                        column!(text("nothing found")).into()
                    };
                    column!(list).into()
                }
                (0, Nav::Mixed) => self.view_artikle_list(),
                _ => text("hm").into(),
            }
        }
    }

    pub fn update_feed_list(&mut self, m: FeedListM) -> Task<Message> {
        match m {
            FeedListM::ReadAll(num) => {
                if let Some(channels) = &self.state.channels {
                    let id = channels[num].0.id.clone();
                    if let Some(channel) = self.state.read.read.iter_mut().find(|d| d.0 == id) {
                        for i in channels[num].0.entries.iter() {
                            if !channel.1.contains(&i.id) {
                                channel.1.push(i.id.clone());
                            }
                        }
                    } else {
                        let mut ve = vec![];
                        for i in channels[num].0.entries.iter() {
                            ve.push(i.id.clone());
                        }
                        self.state.read.read.push((id, ve))
                    }
                    if feeds_unread(&self.state.read, channels) < 1 {
                        self.state.window.tab = (1, Nav::None);
                    }
                }
                Com::none()
            }
            FeedListM::Select(num) => {
                self.state.feed = Some(num);
                Com::none()
            }
            _ => Com::none(),
        }
    }
}

fn feed_cont(
    title: String,
    on_press: Message,
    on_press2: Option<Message>,
    unread: Option<usize>,
    imag: Option<&Imag>,
) -> Container<'static, Message, iced_material::theme::Theme, Renderer> {
    container(
        row!(
            if let Some(Imag {
                id,
                name: Some(name),
                ..
            }) = imag
            {
                column!(image(dir().join("cache").join(id).join(name))).padding(5)
            } else {
                column!()
            },
            text!(" {}", title)
                .width(Fill)
                .height(Fill)
                .align_x(Center)
                .align_y(Center),
            if let Some(m) = on_press2 {
                row!(
                    button("rA").on_press(m).height(40).width(40),
                    Space::with_width(10),
                )
            } else {
                row!()
            },
            button(if let Some(unread) = unread {
                text!("{}", unread)
                    .width(Fill)
                    .height(Fill)
                    .align_x(Center)
                    .align_y(Center)
            } else {
                text!("")
            })
            .height(40)
            .width(40)
            .on_press(on_press)
        )
        .align_y(Center),
    )
    .style(grey_rounded)
    .padding(5)
    .width(Fill)
    .height(50)
}
