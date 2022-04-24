use eframe::{egui, epi};
use egui_extras::RetainedImage;
use crate::json::*;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ProcelioLauncher {
    readmeAccepted: bool,
    installDir: Option<std::path::PathBuf>,
    useDevBuilds: bool,
    #[serde(skip)]
    config: LauncherConfigStatus,
    #[serde(skip)]
    error: Option<Box<dyn std::error::Error>>,
    #[serde(skip)]
    refs: ResourceRefs,
}

pub struct ResourceRefs {
    pub procelio_logo: Option<egui::TextureHandle>,
    pub discord_logo: Option<egui::TextureHandle>,
    pub twitter_logo: Option<egui::TextureHandle>,
    pub youtube_logo: Option<egui::TextureHandle>
}

impl ResourceRefs {
    pub fn new() -> Self {
        ResourceRefs {
            procelio_logo: None,
            discord_logo: None,
            twitter_logo: None,
            youtube_logo: None,
        }
    }

    pub fn load_image_bytes(image_bytes: &[u8]) -> Result<egui::ColorImage, String> {
        let image = image::load_from_memory(image_bytes).map_err(|err| err.to_string())?;
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        Ok(egui::ColorImage::from_rgba_unmultiplied(
            size,
            pixels.as_slice(),
        ))
    }

    pub fn get_procelio_logo(&mut self, ui: &egui::Ui) -> &egui::TextureHandle {
        self.procelio_logo.get_or_insert_with(|| {
            ui.ctx().load_texture("procelio-logo", ResourceRefs::load_image_bytes(include_bytes!("resources/Procelio_Light.png")).unwrap())
        })
    }

    pub fn get_discord_logo(&mut self, ui: &egui::Ui) -> &egui::TextureHandle {
        self.discord_logo.get_or_insert_with(|| {
            ui.ctx().load_texture("discord-logo", ResourceRefs::load_image_bytes(include_bytes!("resources/discord_logo_small.png")).unwrap())
        })
    }

    pub fn get_twitter_logo(&mut self, ui: &egui::Ui) -> &egui::TextureHandle {
        self.twitter_logo.get_or_insert_with(|| {
            ui.ctx().load_texture("twitter-logo", ResourceRefs::load_image_bytes(include_bytes!("resources/twitter_logo_small.png")).unwrap())
        })
    }

    pub fn get_youtube_logo(&mut self, ui: &egui::Ui) -> &egui::TextureHandle {
        self.youtube_logo.get_or_insert_with(|| {
            ui.ctx().load_texture("youtube-logo", ResourceRefs::load_image_bytes(include_bytes!("resources/youtube_logo_small.png")).unwrap())
        })
    }
}

impl Default for ProcelioLauncher {
    fn default() -> Self {
        Self {
            readmeAccepted: false,
            installDir: None,
            useDevBuilds: false,
            config: LauncherConfigStatus::AppLoad,
            error: None,
            refs: ResourceRefs::new()
        }
    }
}

impl epi::App for ProcelioLauncher {
    fn name(&self) -> &str {
        "Procelio Launcher"
    }

    /// Called once before the first frame.
    fn setup(&mut self, _ctx: &egui::Context, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>) {
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        let (s, r) = std::sync::mpsc::channel();
        self.config = LauncherConfigStatus::Pending(r);
        crate::net::get_config(s);
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        if let LauncherConfigStatus::Pending(recv) = &mut self.config {
            if let Ok(a) = recv.try_recv() {
                match a {
                    Ok(cfg) => {
                        self.config = LauncherConfigStatus::Loaded(cfg);
                    },
                    Err(e) => {
                        println!("{:?}", e);
                        self.error = Some(std::boxed::Box::new(e))
                    }
                };
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
              /*   ui.columns(3, |ui| {
                    ui[0].with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        let twitter = self.refs.get_twitter_logo(ui);
                        ui.add(egui::widgets::ImageButton::new(twitter, twitter.size_vec2()));
    
                        let youtube = self.refs.get_youtube_logo(ui);
                        ui.add(egui::widgets::ImageButton::new(youtube, youtube.size_vec2()));
                    });

                    let tex = self.refs.get_procelio_logo(&ui[1]);
                    ui[1].set_width(tex.size_vec2().x);
                    ui[1].with_layout(egui::Layout::top_down(egui::Align::Center),|ui| {
                        
                        ui.image(tex, tex.size_vec2());
    
                        ui.hyperlink("https://www.proceliogame.com");
                    });

                    ui[2].with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                        let discord = self.refs.get_discord_logo(ui);
                        ui.add(egui::widgets::ImageButton::new(discord, discord.size_vec2()));
                    });
                });*/
                
                ui.vertical(|ui| {
                    let twitter = self.refs.get_twitter_logo(ui);
                    ui.add(egui::widgets::ImageButton::new(twitter, twitter.size_vec2()));

                    let youtube = self.refs.get_youtube_logo(ui);
                    ui.add(egui::widgets::ImageButton::new(youtube, youtube.size_vec2()));
                });

                ui.vertical(|ui| {
                    let tex = self.refs.get_procelio_logo(ui);
                    ui.image(tex, tex.size_vec2());

                    ui.hyperlink("https://www.proceliogame.com");
                });
               
                ui.vertical(|ui| {
                    let discord = self.refs.get_discord_logo(ui);
                    ui.add(egui::widgets::ImageButton::new(discord, discord.size_vec2()));
                    let discord = self.refs.get_discord_logo(ui);
                    ui.add(egui::widgets::ImageButton::new(discord, discord.size_vec2()));
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
           //     ui.text_edit_singleline(label);
            });

            //ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
           //     *value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });

        if let Some(x) = self.error.as_ref().map(|x| format!("{:?}", x)) {
            egui::Window::new("error-window").show(ctx, |ui| {
                ui.label("Error:");
                ui.label(format!("{:?}",x));
                if ui.button("OK").clicked() {
                    self.error = None;
                }
            });
        }
    }
}
