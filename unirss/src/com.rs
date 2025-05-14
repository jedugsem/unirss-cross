use iced_winit::futures::MaybeSend;
pub struct Com;
impl Com {
    pub fn none() -> iced::Task<crate::Message> {
        iced::Task::none()
    }
    pub fn perform<A: MaybeSend + 'static>(
        _con: &crate::Controls,
        future: impl std::future::Future<Output = A> + 'static + iced_winit::futures::MaybeSend,
        f: impl FnOnce(A) -> crate::Message + 'static + iced_winit::futures::MaybeSend,
    ) -> iced::Task<crate::Message> {
        #[cfg(target_os = "android")]
        {
            use futures::FutureExt;
            let proxy = _con.proxy.clone();
            std::thread::spawn(move || {
                let m = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(future.map(f));
                _ = proxy.send_event(crate::UserEvent::Task(m));
            });
            iced::Task::none()
        }
        #[cfg(not(target_os = "android"))]
        iced::Task::perform(future, f)
    }
}
