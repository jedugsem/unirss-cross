#![allow(unreachable_code)]

use std::{path::PathBuf, sync::Arc};

use feed_rs::model::Feed;
#[cfg(not(target_os = "android"))]
pub use iced::Renderer;
pub fn dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        PathBuf::from("/storage/emulated/0/.uni/unirss")
    }
    #[cfg(not(target_os = "android"))]
    {
        PathBuf::from("/home/me/.local/share/unirss")
    }
}
use iced_widget::markdown;
use pages::{
    add_url::AddM, artikle_list::ArtikleListM, feed_list::feeds_unread, feed_view::FeedViewM,
};
use ron::de::from_bytes;
use serde::{Deserialize, Serialize};
pub mod com;
pub mod comps;
pub mod favicons;
pub mod git;
pub mod localize;
mod pages;
pub mod per;
pub mod settings;
use com::Com;
use iced::{
    widget::{column, container, responsive, row, text_editor, themer, Space},
    Length, Theme,
};
use iced_material::{header::header, theme};
use iced_winit::runtime::Task;
use pages::feed_list::FeedListM;
use per::read_online_feeds;
use settings::{PSettings, SettingsM, Them};
//use sys_locale::get_locale;
pub type Element<'a, Message> = iced::Element<'a, Message, theme::Theme, Renderer>;
// State Top Down
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uniquiz {
    pub sett: Settings,
    pub feed: Option<usize>,
    pub artikle: Option<usize>,
    pub unread_expanded: bool,
    pub all_expanded: bool,
    pub url: String,

    pub channels: Option<Vec<(feed_rs::model::Feed, per::Imag)>>,
    // Pages - Optional
    pub modules: Load,
    // Window
    pub window: Window,
    // Sidebar
    pub read: Progress,
    pub online_feeds: OnlineFeeds,
    // Settings
    pub error: Vec<String>,
    pub settings: PSettings,
    // Loading - Modules
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Progress {
    pub read: Vec<(String, Vec<String>)>,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum Settings {
    #[default]
    Normal,
    Repo,
    AddUrl,
    ManageFeeds,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum Nav {
    #[default]
    Mixed,
    All,
    Some(usize),
    None,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnlineFeeds {
    pub urls: Vec<(String, String)>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Window {
    pub keyboard: bool,
    pub title: String,
    pub settings_open: bool,
    pub sideselect: bool,
    pub sidebar: bool,
    pub tab: (usize, Nav),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Load {
    err: Option<String>,
}

impl Default for Uniquiz {
    fn default() -> Self {
        let settings = per::load_settings().unwrap_or_default();

        let mut lang = crate::localize::LANG.lock().unwrap();
        let mut languages = crate::localize::LANGUAGES.lock().unwrap();
        *languages = vec![
            from_bytes(include_bytes!("../i18n/de/unirss.ron")).unwrap(),
            from_bytes(include_bytes!("../i18n/en/unirss.ron")).unwrap(),
        ];

        if let Some(la) = settings.lang {
            *lang = la;
        }

        Self {
            sett: Settings::default(),
            error: vec![],
            unread_expanded: false,
            all_expanded: false,
            online_feeds: OnlineFeeds::default(),
            read: Progress::default(),
            artikle: None,
            feed: None,
            channels: None,
            url: String::new(),
            modules: Load { err: None },
            // Window
            // Sidebar
            window: Window::default(),
            // Settings
            settings,
            // Loading - Modules
            // Pages - Optional
        }
    }
}
type Loaded = (
    (
        Option<Vec<(Feed, per::Imag)>>,
        Option<Vec<(String, String)>>,
    ),
    Option<Progress>,
    OnlineFeeds,
);
#[derive(Debug, Clone)]
pub enum Message {
    Sidebar(comps::sidebar::SidebarM),
    Loaded((Loaded, bool)),
    Error(String),
    Refresh(Result<(), String>),
    Boot,
    Clipboard(String, u8),
    FeedView(FeedViewM),
    ArtikleList(ArtikleListM),
    Side,
    FeedList(FeedListM),
    Add(AddM),
    Back,
    Exit,
    Nothing,
    EditorAction(text_editor::Action, u8),
    ToggleSettings,
    Settings(SettingsM),
}
const BREAKPOINT: f32 = 500.;
impl Clone for Controls {
    fn clone(&self) -> Self {
        Controls {
            url_editor: text_editor::Content::new(),
            name_editor: text_editor::Content::new(),
            markdown: self.markdown.clone(),
            theme: self.theme.clone(),
            state: self.state.clone(),
            #[cfg(target_os = "android")]
            background_color: self.background_color.clone(),
            #[cfg(target_os = "android")]
            proxy: self.proxy.clone(),
        }
    }
}
impl Controls {
    pub fn title(&self) -> String {
        "Unirss".to_string()
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Sidebar(m) => self.update_sidebar(m),
            Message::FeedView(m) => self.update_feed_view(m),
            Message::Error(m) => {
                self.state.error.push(m);
                Com::none()
            }
            Message::ArtikleList(m) => self.update_artikle_list(m),
            Message::Boot => {
                if !dir().join("feeds").exists() {
                    let _ = std::fs::create_dir_all(dir().join("feeds"));
                    self.state.sett = Settings::Repo;
                    self.state.window.settings_open = true;

                    // Update git
                    Com::none()
                } else if !dir().join("git").exists() {
                    self.state.sett = Settings::Repo;
                    self.state.window.settings_open = true;
                    // Update git
                    Com::none()
                } else {
                    Com::perform(
                        self,
                        async move {
                            (
                                per::refresh(None).await,
                                Some(per::read_progress().await),
                                read_online_feeds().await,
                            )
                        },
                        |x| Message::Loaded((x, false)),
                    )

                    // Dont
                }
            }
            Message::Refresh(res) => {
                match res {
                    Ok(_) => {}
                    Err(stf) => {
                        self.state.error.push(stf);
                    }
                }
                //
                let git_state = self.state.settings.git.clone();
                Com::perform(
                    self,
                    async move {
                        (
                            per::refresh(git_state).await,
                            Some(per::read_progress().await),
                            per::read_online_feeds().await,
                        )
                    },
                    |x| Message::Loaded((x, true)),
                )
            }
            Message::Loaded((((channels, icons), progress, online_feeds), git)) => {
                self.state.online_feeds = online_feeds.clone();
                println!("online feeds: {:?}", online_feeds);
                self.state.channels = channels;
                if let Some(prog) = progress {
                    self.state.read = prog;
                }
                if let Some(channels) = &self.state.channels {
                    if feeds_unread(&self.state.read, channels) < 1 {
                        self.state.window.tab.0 = 1;
                    }
                }
                if git {
                    if let Some(down) = icons {
                        Com::perform(
                            self,
                            async move {
                                favicons::favicons(down).await;
                                (per::read_feeds(), None, read_online_feeds().await)
                            },
                            |x| Message::Loaded((x, true)),
                        )
                    } else {
                        Com::none()
                    }
                } else {
                    let git_state = self.state.settings.git.clone();
                    Com::perform(
                        self,
                        async move {
                            (
                                per::refresh(git_state).await,
                                Some(per::read_progress().await),
                                per::read_online_feeds().await,
                            )
                        },
                        |x| Message::Loaded((x, true)),
                    )
                }
            }
            Message::Clipboard(m, n) => {
                match n {
                    0 => self.name_editor.perform(text_editor::Action::Edit(
                        text_editor::Edit::Paste(Arc::new(m)),
                    )),
                    1 => self.url_editor.perform(text_editor::Action::Edit(
                        text_editor::Edit::Paste(Arc::new(m)),
                    )),
                    _ => {}
                }
                Com::none()
            }
            Message::EditorAction(action, editor) => match action {
                text_editor::Action::Click(_) => {
                    #[cfg(target_os = "android")]
                    let _ = self.proxy.send_event(crate::UserEvent::ShowKeyboard);
                    Com::none()
                }
                text_editor::Action::SelectWord => {
                    #[cfg(target_os = "android")]
                    let _ = self.proxy.send_event(crate::UserEvent::HideKeyboard);
                    Com::none()
                }
                text_editor::Action::Edit(_) => match editor {
                    0 => {
                        self.name_editor.perform(action);
                        let text = self.name_editor.text();

                        Com::perform(self, async move { text }, |x| AddM::ChangeName(x).into())
                    }
                    1 => {
                        self.url_editor.perform(action);
                        let text = self.url_editor.text();

                        Com::perform(self, async move { text }, |x| AddM::ChangeUrl(x).into())
                    }
                    _ => Com::none(),
                },

                other => {
                    match editor {
                        0 => self.name_editor.perform(other),
                        1 => self.url_editor.perform(other),
                        _ => {}
                    }
                    Com::none()
                }
            },
            Message::FeedList(m) => self.update_feed_list(m),
            Message::Add(m) => self.update_add_url(m),
            Message::Settings(m) => self.update_settings(m),
            Message::ToggleSettings => {
                let window = &mut self.state.window;
                match (window.sideselect, window.settings_open) {
                    (true, true) => {
                        window.settings_open = false;
                    }
                    (false, true) => {
                        window.sideselect = true;
                    }
                    _ => {
                        window.sideselect = true;
                        window.settings_open = !window.settings_open;
                    }
                }
                Com::none()
            }
            Message::Back => {
                if self.state.window.settings_open {
                    match self.state.sett {
                        Settings::ManageFeeds | Settings::Repo => {
                            //
                            self.state.sett = Settings::Normal;
                        }
                        Settings::Normal => {
                            //
                            self.state.window.settings_open = false;
                        }
                        Settings::AddUrl => {
                            //
                            self.state.sett = Settings::ManageFeeds;
                        }
                    }
                } else if let (0 | 1, _x) = &self.state.window.tab {
                    match &mut self.state {
                        Uniquiz {
                            feed,
                            window,
                            artikle: None,
                            ..
                        } if feed.is_some() => {
                            *feed = None;
                            window.tab.1 = Nav::All;
                        }
                        Uniquiz { feed, artikle, .. } if feed.is_some() && artikle.is_some() => {
                            *artikle = None;
                        }
                        _ => {}
                    }
                }
                //
                //let m = back_message(self.state.window.tab);
                //Com::perform(&self, async move { m }, |x| x)
                Com::none()
            }
            Message::Exit => {
                #[cfg(target_os = "android")]
                {
                    std::process::exit(0);
                    Com::none()
                }
                #[cfg(not(target_os = "android"))]
                iced::window::get_latest().and_then(iced::window::close)
            }
            Message::Side => {
                let window = &mut self.state.window;
                if window.sideselect {
                    window.sideselect = false;
                    window.sidebar = true;
                } else {
                    window.sidebar = !window.sidebar;
                }
                //
                Com::none()
            }

            Message::Nothing => Com::none(),
        }
    }

    pub fn view(&self) -> iced::Element<Message, Theme, Renderer> {
        let window = &self.state.window;
        let sidebar_widget: Element<Message> = responsive(move |size| {
            let content: Element<Message> = if window.settings_open {
                self.view_settings()
            } else {
                self.view_feed_list()
            };
            let sidebar = self.view_sidebar();

            match (size, window.sidebar, window.sideselect) {
                (s, true, _) if s.width > BREAKPOINT => row!(
                    container(sidebar).width(Length::Fixed(300.)),
                    container(content).center_x(Length::Fill)
                )
                .into(),
                (s, _, true) if s.width <= BREAKPOINT => {
                    container(content).center_x(Length::Fill).into()
                }
                (_s, true, false) => container(sidebar).width(Length::Fill).into(),

                _ => container(content).center_x(Length::Fill).into(),
            }
        })
        .into();

        themer(
            self.theme.clone(),
            container(column![
                header(
                    Message::Side,
                    Message::Back,
                    Message::ToggleSettings,
                    Message::Exit,
                    "Unirss"
                ),
                sidebar_widget,
                Space::new(0, if cfg!(target_os = "android") { 17 } else { 0 })
            ])
            .style(theme::container::primary)
            .center(Length::Fill),
        )
        .into()
    }
}

#[cfg(target_os = "android")]
mod android {
    use crate::Message;
    pub use iced::Color;
    pub use iced_wgpu::Renderer;
    pub use iced_winit::winit::event_loop::EventLoopProxy;
    #[derive(Debug)]
    pub enum UserEvent {
        ClipboardRead(u8),
        ClipboardWrite(String),
        ShowKeyboard,
        Boot,
        Task(Message),
        HideKeyboard,
        Back,
    }
}
#[cfg(target_os = "android")]
pub use android::*;
pub struct Controls {
    pub url_editor: text_editor::Content<crate::Renderer>,
    pub name_editor: text_editor::Content<crate::Renderer>,
    pub markdown: Option<Vec<markdown::Item>>,
    pub theme: theme::Theme,
    pub state: Uniquiz,
    #[cfg(target_os = "android")]
    background_color: Color,
    #[cfg(target_os = "android")]
    proxy: EventLoopProxy<UserEvent>,
}

#[cfg(not(target_os = "android"))]
impl Default for Controls {
    fn default() -> Self {
        let uniquiz = Uniquiz::default();
        let theme = if let Some(them) = uniquiz.settings.theme {
            match them {
                Them::Dark => theme::Theme::dark(),
                Them::Light => theme::Theme::light(),
                Them::Default => theme::Theme::default(),
            }
        } else {
            theme::Theme::default()
        };

        Self {
            markdown: None,
            url_editor: text_editor::Content::new(),
            name_editor: text_editor::Content::new(),
            theme,
            state: Uniquiz::default(),
        }
    }
}
#[cfg(target_os = "android")]
impl Controls {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Controls {
        let state = Uniquiz::default();
        let theme = match state.settings.theme {
            Some(Them::Dark) => theme::Theme::dark(),
            Some(Them::Light) => theme::Theme::light(),
            Some(Them::Default) => theme::Theme::default(),
            _ => theme::Theme::default(),
        };
        let url_editor = text_editor::Content::new();
        let name_editor = text_editor::Content::new();

        Controls {
            state,
            theme,
            markdown: None,
            url_editor,
            name_editor,
            background_color: Color::default(),
            proxy,
        }
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }
}
