use crate::{com::Com, Controls, Element, Message, Uniquiz};
use iced::{
    widget::{button, column, row, scrollable, text},
    Alignment::Center,
    Length::Fill,
    Padding,
};
use iced_winit::runtime::Task;
//use quizlib::*;

// Modules to be loaded
#[derive(Clone, Debug)]
pub enum FeedViewM {
    Open,
    LinkClicked(markdown::Url),
}

impl From<FeedViewM> for Message {
    fn from(m: FeedViewM) -> Message {
        Message::FeedView(m)
    }
}
use iced_widget::markdown;

impl Controls {
    pub fn view_feed_view(&self) -> Element<Message> {
        match &self.state {
            Uniquiz {
                artikle: Some(artikle),
                feed: Some(feed),
                channels: Some(channels),
                ..
            } => {
                let entry = channels[*feed].0.entries[*artikle].clone();
                let title = entry.title.clone().unwrap().content;

                if let Some(mark) = &self.markdown {
                    column!(
                        row!(
                            text!("{}", title)
                                .width(Fill)
                                .height(Fill)
                                .align_y(Center)
                                .align_x(Center),
                            button("open").on_press(FeedViewM::Open.into()).height(30)
                        )
                        .height(40)
                        .padding(5),
                        scrollable(
                            column!(markdown(mark, &self.theme)
                                .map(|url| FeedViewM::LinkClicked(url).into()),)
                            .padding(Padding {
                                top: 0.,
                                bottom: 0.,
                                left: 10.,
                                right: 10.,
                            })
                        )
                    )
                    .padding(Padding {
                        top: 0.,
                        bottom: 0.,
                        left: 5.,
                        right: 5.,
                    })
                    .into()
                } else {
                    let content = entry.content.unwrap().body.unwrap();

                    column!(scrollable(
                        column!(text!("Title: {}", title), text!("Content: {}", content),)
                            .padding(10)
                    ))
                    .padding(10)
                    .into()
                }
            }
            _ => text("no artikle found").into(),
        }
    }

    pub fn update_feed_view(&mut self, m: FeedViewM) -> Task<Message> {
        match m {
            FeedViewM::LinkClicked(url) => {
                let _ = webbrowser::open(url.as_str());
                Com::none()
            }
            FeedViewM::Open => match &self.state {
                Uniquiz {
                    artikle: Some(artikle),
                    feed: Some(feed),
                    channels: Some(channels),
                    ..
                } => {
                    let entry = channels[*feed].0.entries[*artikle].clone();
                    let link = &entry.links[0].href;
                    let _ = webbrowser::open(link);
                    Com::none()
                }
                _ => Com::none(),
            },
        }
    }
}
