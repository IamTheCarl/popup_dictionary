use ksni::TrayMethods;

pub fn spawn_tray_icon() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let tray = MyTray;
            let _handle = tray.spawn().await.unwrap();

            std::future::pending::<()>().await;
        });
    });
}

#[derive(Debug)]
struct MyTray;

impl ksni::Tray for MyTray {
    fn id(&self) -> String {
        "popup-dictionary".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let icon_bytes = include_bytes!("./assets/icon_linux_macos.png");
        let image = image::load_from_memory(icon_bytes)
            .expect("Failed to load icon")
            .to_rgba8();

        let (width, height) = image.dimensions();
        let pixels = image.into_raw();

        vec![ksni::Icon {
            width: width as i32,
            height: height as i32,
            data: pixels,
        }]
    }

    fn title(&self) -> String {
        "Popup Dictionary".into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "Exit".into(),
                activate: Box::new(|_| {
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}
