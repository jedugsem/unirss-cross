use crate::{
    com::Com,
    comps::searchbar,
    dir, fl,
    git::{self, GitState},
    per, Controls, Element, Message, Settings,
};
use iced::{
    alignment::Horizontal::Right,
    widget::{column, container, row, text, toggler, Space},
    Alignment::Center,
    Length::{self, Fill},
    Task,
};
use iced_material::theme::{self, container::grey_rounded};
use iced_widget::{button, pick_list, scrollable, text_editor};
use serde::{Deserialize, Serialize};

const WIDTH: u16 = 500;
#[derive(Clone, Debug)]
pub enum SettingsM {
    PasteUrl,
    AddUrl,
    ManageFeeds,
    PasteSsh,
    AddRepo,
    RemoveOnline(usize),
    RemoveOffline(String),
    RepoPage,
    ThemeChange(Them),
    ResetFeeds,
    ResetProgress,
    ResetErrors,
    LangChange(Language),
    CheckboxBrowser(bool),
}

impl From<SettingsM> for Message {
    fn from(m: SettingsM) -> Message {
        Message::Settings(m)
    }
}
pub enum Lang {
    Custom(String),
    System(String),
}

impl Controls {
    pub fn update_settings(&mut self, m: SettingsM) -> Task<Message> {
        match m {
            SettingsM::RemoveOnline(num) => {
                self.state.online_feeds.urls.swap_remove(num);
                let git_state = self.state.settings.git.clone().unwrap();
                //
                Com::perform(
                    self,
                    async move {
                        per::write_online_feeds(crate::OnlineFeeds::default(), git_state).await
                    },
                    |_x| Message::Refresh(Ok(())),
                )
            }
            SettingsM::AddUrl => {
                self.state.sett = Settings::AddUrl;
                Com::none()
            }
            SettingsM::RemoveOffline(path) => {
                let _ = std::fs::remove_file(path);
                let git_state = self.state.settings.git.clone().unwrap();
                //
                Com::perform(
                    self,
                    async move {
                        let _ = git::add(vec![]).await;
                        let _ = git::commit("removed feed xml").await;
                        let _ = git::push(git_state).await;
                    },
                    |_x| Message::Refresh(Ok(())),
                )
            }
            SettingsM::ResetErrors => {
                self.state.error = vec![];
                Com::none()
            }
            SettingsM::ResetProgress => {
                let git_state = self.state.settings.git.clone().unwrap();
                //
                Com::perform(
                    self,
                    async move { per::write_progress(crate::Progress::default(), git_state).await },
                    |_x| Message::Refresh(Ok(())),
                )
            }
            SettingsM::ManageFeeds => {
                self.state.sett = Settings::ManageFeeds;
                Com::none()
            }
            SettingsM::ResetFeeds => {
                let git_state = self.state.settings.git.clone().unwrap();

                Com::perform(
                    self,
                    async move {
                        per::write_online_feeds(crate::OnlineFeeds::default(), git_state).await
                    },
                    |_x| Message::Refresh(Ok(())),
                )
            }

            SettingsM::LangChange(lang) => {
                let mut la = crate::localize::LANG.lock().unwrap();
                *la = lang;
                self.state.settings.lang = Some(lang);
                self.write_settings()
            }
            SettingsM::CheckboxBrowser(bool) => {
                self.state.settings.browser = bool;
                self.write_settings()
            }
            SettingsM::ThemeChange(them) => {
                self.state.settings.theme = Some(them);
                match them {
                    Them::Dark => self.theme = theme::Theme::dark(),
                    Them::Light => self.theme = theme::Theme::light(),
                    Them::Default => self.theme = theme::Theme::default(),
                }
                self.write_settings()
            }
            SettingsM::RepoPage => {
                self.state.sett = Settings::Repo;
                Com::none()
            }

            SettingsM::PasteUrl => {
                #[cfg(target_os = "android")]
                {
                    _ = self.proxy.send_event(crate::UserEvent::ClipboardRead(1));
                }

                Com::none()
            }
            SettingsM::PasteSsh => {
                #[cfg(target_os = "android")]
                {
                    _ = self.proxy.send_event(crate::UserEvent::ClipboardRead(0));
                }

                Com::none()
            }
            SettingsM::AddRepo => {
                if dir().join("git").exists() {
                    let _ = std::fs::remove_dir_all(dir().join("git"));
                }
                let ssh = self.name_editor.text();
                let git_url = self.url_editor.text();
                self.url_editor = text_editor::Content::new();
                self.name_editor = text_editor::Content::new();
                self.state.settings.git = Some(GitState {
                    repo: git_url,
                    ssh_priv: ssh,
                });
                self.state.sett = Settings::Normal;
                self.state.window.settings_open = false;
                let settings = self.state.settings.clone();
                let git = self.state.settings.git.clone().unwrap();
                Com::perform(
                    self,
                    async move {
                        crate::per::write_settings(settings);
                        let res = git::clone(git).await;
                        Message::Refresh(res)
                    },
                    |m| m,
                )
            }
        }
    }
    fn write_settings(&self) -> Task<Message> {
        let settings = self.state.settings.clone();
        Com::perform(
            self,
            async move {
                crate::per::write_settings(settings);
                Message::Nothing
            },
            |m| m,
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    System,
    De,
    En,
}

impl Language {
    const ALL: [Language; 3] = [Self::System, Self::En, Self::De];
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::En => fl!("english"),
                Self::De => fl!("german"),
                Self::System => fl!("system"),
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum Them {
    Light,
    Dark,
    #[default]
    Default,
}

impl Them {
    const ALL: [Them; 3] = [Self::Dark, Self::Light, Self::Default];
}

impl std::fmt::Display for Them {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Light => fl!("light-theme"),
                Self::Dark => fl!("dark-theme"),
                Self::Default => fl!("default-theme"),
            }
        )
    }
}

impl Controls {
    pub fn view_settings(&self) -> Element<Message> {
        match self.state.sett {
            Settings::AddUrl => self.view_add_url(),
            Settings::ManageFeeds => {
                let mut online_feeds: Vec<Element<Message>> = self
                    .state
                    .online_feeds
                    .urls
                    .iter()
                    .enumerate()
                    .map(|(num, x)| {
                        container(
                            row!(
                                text!("Online Name: {}", x.0)
                                    .width(Fill)
                                    .height(Fill)
                                    .align_x(Center)
                                    .align_y(Center),
                                button("R")
                                    .height(40)
                                    .width(40)
                                    .on_press(SettingsM::RemoveOnline(num).into())
                            )
                            .align_y(Center),
                        )
                        .padding(5)
                        .width(Fill)
                        .height(50)
                        .style(grey_rounded)
                        .into()
                    })
                    .collect();
                println!("{:?}", self.state.online_feeds.urls);
                let mut offline_feeds: Vec<Element<Message>> =
                    if let Ok(files) = dir().join("git").join("xmls").read_dir() {
                        files
                            .map(|x| {
                                if let Ok(x) = x {
                                    container(
                                        row!(
                                            text!("GitFeed: {}", x.file_name().to_str().unwrap())
                                                .width(Fill)
                                                .height(Fill)
                                                .align_x(Center)
                                                .align_y(Center),
                                            button("R").height(40).width(40).on_press(
                                                SettingsM::RemoveOffline(
                                                    x.path().to_str().unwrap().to_string()
                                                )
                                                .into()
                                            )
                                        )
                                        .align_y(Center),
                                    )
                                    .padding(5)
                                    .width(Fill)
                                    .height(50)
                                    .style(grey_rounded)
                                    .into()
                                    //text!("{}", x.file_name().to_str().unwrap()).into()
                                } else {
                                    text!("not found").into()
                                }
                            })
                            .collect()
                    } else {
                        vec![]
                    };
                online_feeds.append(&mut offline_feeds);
                column!(
                    column!(button("Add").on_press(SettingsM::AddUrl.into()))
                        .align_x(Right)
                        .width(Fill)
                        .padding(5),
                    scrollable(column(online_feeds).padding(5).spacing(5)),
                )
                .padding(5)
                .into()

                //
            }
            Settings::Repo => {
                // test
                column!(
                    text("Git Url:"),
                    searchbar(
                        &self.url_editor,
                        |s| Message::EditorAction(s, 1),
                        Some(|| SettingsM::PasteUrl.into())
                    ),
                    text("Ssh Key:"),
                    searchbar(
                        &self.name_editor,
                        |s| Message::EditorAction(s, 0),
                        Some(|| SettingsM::PasteSsh.into())
                    ),
                    column!(button("Add").on_press(SettingsM::AddRepo.into()))
                        .width(Fill)
                        .align_x(Right),
                )
                .align_x(Center)
                .padding(15)
                .spacing(10)
                .into()
            }
            Settings::Normal => {
                let mut vec: Vec<Element<Message>> = vec![
                    row!(
                        text("Refresh"),
                        Space::with_width(Length::Fill),
                        button("Now").on_press(Message::Refresh(Ok(()))),
                    )
                    .spacing(10)
                    .into(),
                    row!(
                        text("Progress"),
                        Space::with_width(Length::Fill),
                        button("Clear").on_press(SettingsM::ResetProgress.into()),
                    )
                    .spacing(10)
                    .into(),
                    row!(
                        text("Feeds"),
                        Space::with_width(Length::Fill),
                        button("Manage").on_press(SettingsM::ManageFeeds.into()),
                    )
                    .spacing(10)
                    .into(),
                    row!(
                        text("Browser"),
                        Space::with_width(Length::Fill),
                        toggler(self.state.settings.browser)
                            .on_toggle(|boo| Message::Settings(SettingsM::CheckboxBrowser(boo)))
                            .size(20.)
                            .width(100)
                    )
                    .spacing(10)
                    .into(),
                    row!(
                        text(fl!("theme")),
                        Space::with_width(Length::Fill),
                        pick_list(Them::ALL, self.state.settings.theme, |x| {
                            SettingsM::ThemeChange(x).into()
                        }),
                    )
                    .into(),
                    row!(
                        text(fl!("language")),
                        Space::with_width(Length::Fill),
                        pick_list(Language::ALL, self.state.settings.lang, |x| {
                            SettingsM::LangChange(x).into()
                        }),
                    )
                    .into(),
                    row!(
                        text("Change Repo"),
                        Space::with_width(Length::Fill),
                        button("now").on_press(SettingsM::RepoPage.into()),
                    )
                    .into(),
                ];
                if !self.state.error.is_empty() {
                    let err: Vec<Element<Message>> =
                        self.state.error.iter().map(|r| text(r).into()).collect();
                    vec.push(
                        column!(
                            row!(
                                text("Errors"),
                                Space::with_width(Fill),
                                button("Clear").on_press(SettingsM::ResetErrors.into()),
                            ),
                            column(err),
                        )
                        .into(),
                    )
                }
                container(
                    column(vec)
                        .spacing(15)
                        .width(Length::Fixed(WIDTH as f32))
                        .padding(20),
                )
                .center_x(Length::Fill)
                .into()
            }
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PSettings {
    pub git: Option<GitState>,
    pub lang: Option<Language>,
    pub browser: bool,
    pub theme: Option<Them>,
}
impl Default for PSettings {
    fn default() -> Self {
        Self {
            git: Some(GitState::default()),
            lang: Some(Language::De),
            browser: true,
            theme: Some(Them::Default),
        }
    }
}
