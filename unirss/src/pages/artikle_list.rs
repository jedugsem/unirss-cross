use crate::{
    com::Com,
    dir,
    pages::feed_list::{artikles_unread, feeds_unread},
    per::{self, *},
    Controls, Element, Message, Nav, Renderer, Uniquiz,
};
use chrono::{DateTime, Utc};
use iced::{
    widget::{button, column, row, scrollable, text},
    Alignment::Center,
    Length::Fill,
};
use iced_material::theme::{
    button::{secondary, text_button},
    container::{grey_rounded, secondary_rounded},
};
use iced_widget::{container, image, markdown, Container};
use iced_winit::runtime::Task;
//use quizlib::*;

// Modules to be loaded
#[derive(Clone, Debug)]
pub enum ArtikleListM {
    Select(Option<usize>, usize, bool),
}

impl From<ArtikleListM> for Message {
    fn from(m: ArtikleListM) -> Message {
        Message::ArtikleList(m)
    }
}
impl Controls {
    pub fn view_artikle_list(&self) -> Element<Message> {
        match &self.state {
            Uniquiz {
                channels: Some(channels),
                artikle: None,
                feed: _,
                ..
            } if self.state.window.tab.1 == Nav::Mixed => {
                let mut col: Vec<(String, usize, usize, Option<DateTime<Utc>>, Imag)> = vec![];
                for (feed_num, feed) in channels.iter().enumerate() {
                    let feed_id = feed.0.id.clone();
                    let read: &Vec<String> =
                        if let Some(readd) = self.state.read.read.iter().find(|x| x.0 == feed_id) {
                            &readd.1
                        } else {
                            &vec![]
                        };

                    for (artikle_num, entry) in feed.0.entries.iter().enumerate() {
                        let entry_id = entry.id.clone();
                        let title = entry.title.clone().unwrap().content;

                        if !read.contains(&entry_id) {
                            col.push((title, feed_num, artikle_num, entry.updated, feed.1.clone()));
                        }
                    }
                }
                col.sort_by(|a, b| b.3.cmp(&a.3));

                column!(scrollable(
                    column(col.iter().map(|x| {
                        artikle_cont(
                            x.0.clone(),
                            ArtikleListM::Select(Some(x.1), x.2, true).into(),
                            Some(x.4.clone()),
                            true,
                        )
                        .into()
                    },))
                    .padding(5)
                    .spacing(5)
                ))
                .padding(5)
                .into()
                //
            }
            Uniquiz {
                artikle: Some(_),
                feed: Some(_),
                ..
            } => self.view_feed_view(),
            Uniquiz {
                channels: Some(channels),
                artikle: None,
                feed: Some(feed_num),
                ..
            } => {
                let mut col: Vec<Element<Message>> = vec![];
                let feed = &channels[*feed_num];
                for (i, entry) in feed.0.entries.iter().enumerate() {
                    let feed_id = feed.0.id.clone();
                    let entry_id = entry.id.clone();
                    let read: &Vec<String> =
                        if let Some(readd) = self.state.read.read.iter().find(|x| x.0 == feed_id) {
                            &readd.1
                        } else {
                            &vec![]
                        };
                    let title = entry.title.clone().unwrap().content;
                    if read.contains(&entry_id) {
                        if self.state.window.tab.0 != 0 {
                            col.push(
                                artikle_cont(
                                    title,
                                    ArtikleListM::Select(None, i, false).into(),
                                    None,
                                    false,
                                )
                                .into(),
                            )
                        }
                    } else {
                        col.push(
                            artikle_cont(
                                title,
                                ArtikleListM::Select(None, i, true).into(),
                                None,
                                true,
                            )
                            .into(),
                        );
                    }
                }
                column!(scrollable(column(col).padding(5).spacing(5)))
                    .padding(5)
                    .into()
            }
            _ => text("No artikle found").into(),
        }
    }

    pub fn update_artikle_list(&mut self, m: ArtikleListM) -> Task<Message> {
        match m {
            ArtikleListM::Select(feed_num_prov, artikle_num, unread) => {
                if let Uniquiz {
                    channels: Some(channels),
                    feed: feed_num,
                    settings,
                    read,
                    window,
                    ..
                } = &mut self.state
                {
                    let feed = &channels[if let Some(num) = feed_num_prov {
                        num
                    } else {
                        feed_num.unwrap()
                    }];
                    let artikle = feed.0.entries[artikle_num].clone();
                    if unread {
                        if let Some(read_list) = read.read.iter_mut().find(|x| x.0 == feed.0.id) {
                            println!("old ");
                            read_list.1.push(artikle.id.clone());
                        } else {
                            read.read
                                .push((feed.0.id.clone(), vec![artikle.id.clone()]))
                        }
                        if artikles_unread(read, &feed.0) < 1 && window.tab.0 == 0 {
                            if feeds_unread(read, channels) < 1 {
                                window.tab = (1, Nav::None);
                                *feed_num = None;
                            } else {
                                window.tab.0 = 0;
                                *feed_num = None;
                            }
                        }
                    }

                    if settings.browser {
                        let url = artikle.links[0].clone().href;

                        let _ = webbrowser::open(&url);
                    } else if let Some(content) = artikle.content {
                        let markdown_content = mdka::from_html(&content.body.unwrap());

                        self.markdown = Some(markdown::parse(&markdown_content).collect());
                        self.state.artikle = Some(artikle_num);
                        if let Some(nu) = feed_num_prov {
                            self.state.feed = Some(nu);
                        }
                    } else {
                        let url = artikle.links[0].clone().href;
                        let _ = webbrowser::open(&url);
                    }
                    let progress = self.state.read.clone();
                    let git_state = self.state.settings.git.clone().unwrap();
                    Com::perform(
                        self,
                        async move { per::write_progress(progress, git_state).await },
                        |_x| Message::Nothing,
                    )
                } else {
                    Com::none()
                }
            }
        }
    }
}
fn artikle_cont(
    title: String,
    on_press: Message,
    imag: Option<Imag>,
    read: bool,
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
            button("")
                .height(40)
                .width(40)
                .style(if read { secondary } else { text_button })
                .on_press(on_press)
        )
        .align_y(Center),
    )
    .padding(5)
    .width(Fill)
    .height(50)
    .style(if !read {
        grey_rounded
    } else {
        secondary_rounded
    })
}
