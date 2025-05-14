use crate::{com::Com, pages::feed_list::artikles_unread, Controls, Element, Message, Nav};

use iced::{
    widget::{button, column, row, scrollable, text, Space},
    Alignment::{self, Center},
    Length::{self, Fill},
};
use iced_material::theme::container::grey_rounded;
use iced_widget::container;

// Modules to be loaded
#[derive(Clone, Debug)]
pub enum SidebarM {
    ExpandUnread,
    ExpandAll,
    Select((usize, Nav)),
}

impl From<SidebarM> for Message {
    fn from(m: SidebarM) -> Message {
        Message::Sidebar(m)
    }
}

impl Controls {
    pub fn view_sidebar(&self) -> Element<Message> {
        let mut unread_num = 0;
        if let Some(channels) = &self.state.channels {
            for i in channels {
                unread_num += artikles_unread(&self.state.read, &i.0)
            }
        }

        let unread_button: Element<Message> = row!(container(
            row!(
                button(if self.state.all_expanded {
                    "↓"
                } else {
                    "→"
                })
                .height(30)
                .width(30)
                .on_press(SidebarM::ExpandUnread.into()),
                text!("Unread")
                    .width(Fill)
                    .height(Fill)
                    .align_x(Center)
                    .align_y(Center),
                button(
                    text!("All")
                        .width(Fill)
                        .height(Fill)
                        .align_x(Center)
                        .align_y(Center)
                )
                .height(30)
                .width(40)
                .on_press(SidebarM::Select((0, Nav::All)).into()),
                Space::with_width(5),
                button(
                    text!("{}", unread_num)
                        .width(Fill)
                        .height(Fill)
                        .align_x(Center)
                        .align_y(Center)
                )
                .height(30)
                .width(40)
                .on_press(SidebarM::Select((0, Nav::Mixed)).into())
            )
            .align_y(Center),
        )
        .padding(5)
        .width(Fill)
        .height(40)
        .style(grey_rounded))
        .into();

        let unread: Element<Message> = if self.state.unread_expanded {
            if let Some(channels) = &self.state.channels {
                let mut unread_feeds: Vec<Element<Message>> = vec![];
                for (i, feed) in channels.iter().enumerate() {
                    let unread = artikles_unread(&self.state.read, &feed.0);
                    if unread != 0 {
                        unread_feeds.push(
                            row!(
                                Space::with_width(40),
                                container(
                                    row!(
                                        text!("{}", feed.0.title.clone().unwrap().content)
                                            .width(Fill)
                                            .height(Fill)
                                            .align_x(Center)
                                            .align_y(Center),
                                        button(
                                            text!("{}", unread)
                                                .width(Fill)
                                                .height(Fill)
                                                .align_x(Center)
                                                .align_y(Center)
                                        )
                                        .height(30)
                                        .width(40)
                                        .on_press(SidebarM::Select((0, Nav::Some(i))).into())
                                    )
                                    .align_y(Center),
                                )
                                .padding(5)
                                .width(Fill)
                                .height(40)
                                .style(grey_rounded)
                            )
                            .into(),
                        );
                    }
                }
                column!(unread_button, column(unread_feeds).spacing(5))
                    .spacing(5)
                    .into()
            } else {
                column!(unread_button).spacing(5).into()
            }
        } else {
            column!(unread_button).spacing(5).into()
        };
        let all_button: Element<Message> = row!(container(
            row!(
                button(if self.state.all_expanded {
                    "↓"
                } else {
                    "→"
                })
                .height(30)
                .width(30)
                .on_press(SidebarM::ExpandAll.into()),
                text!("All Feeds")
                    .width(Fill)
                    .height(Fill)
                    .align_x(Center)
                    .align_y(Center),
                button("")
                    .height(30)
                    .width(40)
                    .on_press(SidebarM::Select((1, Nav::None)).into())
            )
            .align_y(Center),
        )
        .padding(5)
        .width(Fill)
        .height(40)
        .style(grey_rounded))
        .into();

        let feeds: Element<Message> = if self.state.all_expanded {
            if let Some(channels) = &self.state.channels {
                let mut all_feeds: Vec<Element<Message>> = vec![];
                for (i, feed) in channels.iter().enumerate() {
                    let title = feed.0.title.clone().unwrap().content;
                    all_feeds.push(
                        row!(
                            Space::with_width(40),
                            container(
                                row!(
                                    text!("{}", title)
                                        .width(Fill)
                                        .height(Fill)
                                        .align_x(Center)
                                        .align_y(Center),
                                    button("")
                                        .height(30)
                                        .width(40)
                                        .on_press(SidebarM::Select((1, Nav::Some(i))).into())
                                )
                                .align_y(Center),
                            )
                            .padding(5)
                            .width(Fill)
                            .height(40)
                            .style(grey_rounded)
                        )
                        .into(),
                    );
                }

                column!(all_button, column(all_feeds).spacing(5))
                    .spacing(5)
                    .into()
            } else {
                column!(all_button).spacing(5).into()
            }
        } else {
            column!(all_button).spacing(5).into()
        };
        let sidebar: Element<Message> = if unread_num > 0 {
            column!(unread, feeds)
                .spacing(5)
                .align_x(Alignment::Center)
                .into()
        } else {
            column!(feeds).spacing(5).align_x(Alignment::Center).into()
        };

        container(scrollable(sidebar))
            .padding(10)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .style(iced_material::theme::container::light_grey)
            .into()
    }
    pub fn update_sidebar(&mut self, m: SidebarM) -> iced::Task<Message> {
        match m {
            SidebarM::ExpandAll => {
                self.state.all_expanded = !self.state.all_expanded;
                Com::none()
            }
            SidebarM::ExpandUnread => {
                self.state.unread_expanded = !self.state.unread_expanded;
                Com::none()
            }
            SidebarM::Select(tab) => {
                let window = &mut self.state.window;
                //if let Some(_loaded) = &mut self.state.loaded {
                window.settings_open = false;
                window.sideselect = true;
                window.tab = tab.clone();

                match tab {
                    (_, feed) => {
                        self.state.artikle = None;
                        self.state.feed = if let Nav::Some(s) = feed {
                            Some(s)
                        } else {
                            None
                        };
                        Com::none()
                    }
                }
            }
        }
    }
}
