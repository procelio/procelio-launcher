use std::sync::mpsc::Sender;
use std::thread;

use eframe::egui::style::Margin;
use eframe::epaint::TextureHandle;
use eframe::{egui, epi};
use crate::json::*;
use crate::defs;
use crate::patch::PlayGameConfig;
use open;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ProcelioLauncher {
    readme_accepted: i32,
    install_dir: Option<std::path::PathBuf>,
    #[serde(default)]
    channel: String,
    #[serde(default)]
    cdn: String,
    #[serde(default)]
    image: String,

    #[serde(skip)]
    refs: ResourceRefs,
    #[serde(skip)]
    settings: bool,
    #[serde(skip)]
    licenses: bool,
    #[serde(skip)]
    viewed_changelog: usize,
    #[serde(skip)]
    states: Ephemeral,
    #[serde(skip)]
    launcher_name: String,
}

pub struct Ephemeral {
    config: LoadStatus<LauncherConfig>,
    channel: LoadStatus<ConfigResponse>,
    image: LoadStatus<Vec<u8>>,
    launcher_redownload: LoadStatus<()>,
    launching: LoadStatus<()>,
    uninstall: LoadStatus<()>,
    new_version: LoadStatus<InstallManifest>,
    error: Option<Box<anyhow::Error>>,
    processing_status: Option<std::sync::Arc<std::sync::Mutex<(f32, String, Option<Box<anyhow::Error>>)>>>
}

impl Ephemeral {
    pub fn new() -> Ephemeral {
        Ephemeral { 
            config: LoadStatus::AppLoad, 
            channel: LoadStatus::AppLoad,
            image: LoadStatus::AppLoad,
            launcher_redownload: LoadStatus::AppLoad,
            uninstall: LoadStatus::AppLoad,
            launching: LoadStatus::AppLoad,
            new_version: LoadStatus::AppLoad,
            error: None, 
            processing_status: None }
    }

    pub fn ok_to_play(&self) -> bool {
        let launcher = match self.launcher_redownload {
            LoadStatus::AppLoad => true,
            _ => false
        };

        let channel = match self.channel {
            LoadStatus::Loaded(_) => true,
            _ => false
        };
        let game = self.processing_status.is_none();
        launcher && game && channel
    }
}

pub struct ResourceRefs {
    pub procelio_logo: Option<egui::TextureHandle>,
    pub gear_logo: Option<egui::TextureHandle>,
    pub website_logo: Option<egui::TextureHandle>,
    pub discord_logo: Option<egui::TextureHandle>,
    pub twitter_logo: Option<egui::TextureHandle>,
    pub youtube_logo: Option<egui::TextureHandle>,
    pub baseplate_tex: Option<egui::TextureHandle>,
    pub trim_tex: Option<egui::TextureHandle>,

    pub background: Option<egui::TextureHandle>
}

impl ResourceRefs {
    pub fn new() -> Self {
        ResourceRefs {
            gear_logo: None,
            website_logo: None,
            procelio_logo: None,
            discord_logo: None,
            twitter_logo: None,
            youtube_logo: None,
            baseplate_tex: None,
            trim_tex: None,
            background: None,
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

    pub fn get_website_logo(&mut self, ui: &egui::Ui) -> &egui::TextureHandle {
        self.website_logo.get_or_insert_with(|| {
            ui.ctx().load_texture("website-logo", ResourceRefs::load_image_bytes(include_bytes!("resources/procelio_small.png")).unwrap())
        })
    }

    pub fn get_settigns_gear(&mut self, ui: &egui::Ui) -> &egui::TextureHandle {
        self.gear_logo.get_or_insert_with(|| {
            ui.ctx().load_texture("gear-logo", ResourceRefs::load_image_bytes(include_bytes!("resources/gear_logo_small.png")).unwrap())
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

    pub fn get_baseplate_tex(&mut self, ctx: &egui::Context) -> &egui::TextureHandle {
        self.baseplate_tex.get_or_insert_with(|| {
            ctx.load_texture("baseplate", ResourceRefs::load_image_bytes(include_bytes!("resources/baseplate.png")).unwrap())
        })
    }

    pub fn get_trim_tex(&mut self, ctx: &egui::Context) -> &egui::TextureHandle {
        self.trim_tex.get_or_insert_with(|| {
            ctx.load_texture("trim", ResourceRefs::load_image_bytes(include_bytes!("resources/trim.png")).unwrap())
        })
    }

    pub fn get_background(&mut self, img: Option<&Vec<u8>>, ctx: &egui::Context) -> &egui::TextureHandle {
        self.background.get_or_insert_with(|| {
            let bytes = img.as_ref().map(|&x| x.as_slice()).unwrap_or(include_bytes!("resources/background.png"));
            ctx.load_texture("background", ResourceRefs::load_image_bytes(bytes).unwrap())
        })
    }
}

impl Default for ProcelioLauncher {
    fn default() -> Self {
        Self {
            launcher_name: format!("Procelio Launcher"),
            readme_accepted: 0,
            install_dir: None,
            image: "none".to_owned(),
            channel: "prod".to_owned(),
            cdn: "nyc3".to_owned(),
            settings: false,
            licenses: false,
            viewed_changelog: 0,
            refs: ResourceRefs::new(),
            states: Ephemeral::new()
        }
    }
}

impl ProcelioLauncher {
    fn redownload_internal(cdn: String) -> Result<(), anyhow::Error> {
        let url = crate::net::get_launcher_url(&cdn, defs::launcher_name())?;
        let file = crate::net::download_file(&url, None)?;
        let mut data = Vec::new();
        file.as_reader().read_to_end(&mut data)?;
    
        let curr_name = std::env::current_exe()?;
        let mut new_name = curr_name.clone();
        new_name.pop();
        let mut nn = curr_name.components().last().unwrap().as_os_str().to_os_string();
        nn.push(".tmp");
        new_name.push(nn);
    
        std::fs::rename(&curr_name, new_name)?;
        std::fs::write(curr_name, data)?;
        Ok(())
    }

    pub fn redownload_launcher(cdn: String, send: Sender<Result<(), anyhow::Error>>) {
        thread::spawn(move || {//"ProcelioLauncher.exe"
            send.send(ProcelioLauncher::redownload_internal(cdn)).unwrap();
        });
    }

    fn gather_args(&self) -> Option<PlayGameConfig> {
        if let LoadStatus::Loaded(t) = &self.states.channel {
            Some(PlayGameConfig {
                cdn: self.cdn.clone(),
                channel: self.channel.clone(),
                latest_build: t.newest_release_name.clone(),
                args: t.args.clone()
            })
        } else {
            None
        }
    }

    fn uvize(rect: egui::Rect, width: f32, height: f32) -> egui::Rect {
        egui::Rect::from_two_pos(
            egui::pos2(rect.min.x / width, rect.min.y / height),
            egui::pos2(rect.max.x / width, rect.max.y / height)
        )
    }

    fn check_states(&mut self, ctx: &egui::Context, frame: &epi::Frame) -> bool {
        if let LoadStatus::Pending(recv) = &mut self.states.config {
            if let Ok(a) = recv.try_recv() {
                match a {
                    Ok(cfg) => {
                        if !cfg.channels.contains(&self.channel) {
                            self.channel = "prod".to_owned();
                        }
                        if !cfg.cdn_regions.contains(&self.cdn) {
                            self.cdn = "nyc3".to_owned();
                        }
                        if cfg.metadata.version != defs::version() {
                            self.states.launcher_redownload = LoadStatus::AwaitingApproval;
                        }
                        
                        if self.image != cfg.metadata.bg_image {
                            let (ss, rr) = std::sync::mpsc::channel();
                            self.states.image = LoadStatus::Pending(rr);
                            crate::net::get_image(self.image.clone(), cfg.metadata.bg_image.clone(), ss);
                        }

                        self.states.config = LoadStatus::Loaded(cfg);

                        let (s, r) = std::sync::mpsc::channel();
                        self.states.channel = LoadStatus::Pending(r);
                        ctx.request_repaint();
                        crate::net::get_data(self.channel.clone(), s);
                    },
                    Err(e) => {
                        self.states.error = Some(std::boxed::Box::new(e.into()))
                    }
                };
            }
        }

        if let LoadStatus::AwaitingApproval = &mut self.states.launcher_redownload {
            egui::Window::new("Approve Launcher Update?").show(ctx, |ui| {
                if let LoadStatus::Loaded(x) = &self.states.config {
                    ui.label(format!("The launcher will download and update to version {}", x.metadata.version));
                    if ui.button("OK").clicked() {
                        let (s, r) = std::sync::mpsc::channel();
                        self.states.launcher_redownload = LoadStatus::Pending(r);
                        ProcelioLauncher::redownload_launcher(self.cdn.clone(), s);
                    }
                    if ui.button("Quit").clicked() {
                        frame.quit();
                        std::process::exit(1);
                    }
                } else { panic!(); }
            });
            return true;
        }

        if let LoadStatus::Pending(recv) = &mut self.states.channel {
            ctx.request_repaint();
            if let Ok(a) = recv.try_recv() {
                match a {
                    Ok(x) => {
                        self.states.channel = LoadStatus::Loaded(x);
                    },
                    Err(e) => {
                        self.states.error = Some(std::boxed::Box::new(e.into()))
                    }
                }
            }
        }

        if let LoadStatus::Pending(recv) = &mut self.states.image {
            if let Ok(a) = recv.try_recv() {
                match a {
                    Ok(x) => {
                        self.states.image = LoadStatus::Loaded(x);
                        self.refs.background = None;
                    },
                    Err(_) => { }
                }
            }
        }
        
        if let LoadStatus::Pending(recv) = &mut self.states.launcher_redownload {
            if let Ok(a) = recv.try_recv() {
                match a {
                    Ok(_) => {
                        self.states.launcher_redownload = LoadStatus::Loaded(());
                    },
                    Err(e) => {
                        self.states.error = Some(std::boxed::Box::new(e.into()))
                    }
                };
            }
            return true;
        }

        if let LoadStatus::Loaded(()) = self.states.launcher_redownload {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("The launcher will now restart");
                let mut b = false;
                ui.checkbox(&mut b, "OK");
                if b {
                    frame.quit();
                }
            });
            return true;
        }

        if let LoadStatus::AwaitingApproval = self.states.uninstall {
            if let Some(path) = &self.install_dir {
                if self.states.ok_to_play() {
                    egui::Window::new("Confirm Procelio Uninstall?").show(ctx, |ui| {
                        ui.label(format!("The game will be uninstalled at {:?}", path.display()));
                        if ui.button("OK").clicked() {
                            let mutex = std::sync::Arc::new(std::sync::Mutex::new((0., "Uninstalling".to_owned(), None)));
                            self.states.processing_status = Some(mutex.clone());

                            let (send, recv) = std::sync::mpsc::channel();
                            self.states.uninstall = LoadStatus::Pending(recv);
                            crate::patch::uninstall(path.to_owned(), mutex, send);
                        }
                        if ui.button("Cancel").clicked() {
                            self.states.uninstall = LoadStatus::AppLoad;
                        }
                    });
                    return true;
                }
            }
        }
        
        if let LoadStatus::Pending(recv) = &mut self.states.uninstall {
            if let Ok(a) = recv.try_recv() {
                match a {
                    Ok(_) => {
                        self.states.uninstall = LoadStatus::AppLoad;
                        self.states.processing_status = None;
                    },
                    Err(e) => {
                        self.states.error = Some(std::boxed::Box::new(e.into()))
                    }
                };
            }
        }

        if let LoadStatus::Pending(recv) = &mut self.states.launching {
            if let Ok(a) = recv.try_recv() {
                match a {
                    Ok(_) => {
                        self.states.launching = LoadStatus::AppLoad;
                        self.states.processing_status = None;
                    },
                    Err(e) => {
                        self.states.error = Some(std::boxed::Box::new(e.into()))
                    }
                };
            }
        }
        
        if self.readme_accepted != defs::CURRENT_README{
            egui::CentralPanel::default().show(ctx, |ui| {
                let s = "THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS \"AS IS\" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.";
                ui.label(s);
                let mut b = false;
                ui.checkbox(&mut b, "I accept");
                if b {
                    self.readme_accepted = defs::CURRENT_README
                }
            });
            return true;
        }

        if let LoadStatus::Pending(recv) = &mut self.states.new_version {
            if let Ok(a) = recv.try_recv() {
                match a {
                    Ok(_) => { },
                    Err(e) => {
                        self.states.error = Some(std::boxed::Box::new(e.into()))
                    }
                };
            }
        }
        false
    }

    fn image(&mut self, ui: &mut egui::Ui, fill: egui::Color32, name: &str, url: &str, image: &dyn for<'a, 'b> Fn(&'a mut Self, &'b mut egui::Ui) -> &'a TextureHandle) {
        egui::containers::Frame {
            margin: egui::style::Margin { left: 5., right: 5., top: 5., bottom: 5. },
            rounding: egui::Rounding { nw: 0.0, ne: 0.0, sw: 0.0, se: 0.0 },
            shadow: eframe::epaint::Shadow::default(),
            fill,
            stroke: egui::Stroke::default()
        }.show(ui, |ui| {
            let (rect, resp) = ui.allocate_exact_size(egui::Vec2::new(100., 25.), egui::Sense::click());
            ui.allocate_ui_at_rect(rect, |ui| {
                ui.with_layout(egui::Layout::from_main_dir_and_cross_align(egui::Direction::RightToLeft, egui::Align::Center), |ui| {
                    let image = image(self, ui);
                    let size = image.size_vec2() / 1.5;
                    if ui.add(egui::widgets::ImageButton::new(image, size)).clicked() {
                        if let Err(e) = open::that(url) {
                            self.states.error = Some(Box::new(anyhow::Error::new(e)));
                        }
                    }
                    ui.label(name);
                });
            });
            if resp.clicked() {
                if let Err(e) = open::that(url) {
                    self.states.error = Some(Box::new(anyhow::Error::new(e)));
                }
            }
        });
    }

    fn changelog(refs: &mut ResourceRefs, ctx: &egui::Context, ui: &mut egui::Ui, fill: egui::Color32, name: &str, description: &str, url: &str) {
        const WIDTH: f32 = 200.;
        let text_color = egui::Color32::from_rgb(180, 180, 180);
        let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(WIDTH, 300.), egui::Sense::focusable_noninteractive());

        ui.allocate_ui_at_rect(rect, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(0., 0.);
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(WIDTH, 16.), egui::Sense::click());
                ui.allocate_ui_at_rect(rect, |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(32, 32, 32))
                        .margin(Margin::same(5.))
                        .show(ui, |ui| {
                            ui.add_sized([WIDTH - 10., 36.], egui::Label::new(egui::RichText::new(name).color(text_color).size(18.)));
                        });
                });
                let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(WIDTH, 172.), egui::Sense::click());
                ui.allocate_ui_at_rect(rect, |ui| {
                    egui::Frame::none()
                        .fill(fill)
                        .margin(Margin::same(5.))
                        .show(ui, |ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                ui.add(egui::Label::new(egui::RichText::new(description).size(12.)));
                                ui.allocate_space(ui.available_size());
                            });
                        });
                });

                let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(WIDTH, 16.), egui::Sense::click());
                ui.allocate_ui_at_rect(rect, |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(32, 32, 32))
                        .margin(Margin::same(5.))
                        .show(ui, |ui| {
                            let txt = egui::RichText::new("FULL PATCH NOTES").color(text_color).size(16.);
                            if ui.add(egui::widgets::Button::new(txt)).clicked() {
                                let _ = open::that(url);
                            }
                        });
                });

                let trim = refs.get_trim_tex(ctx);
                ui.add(egui::Image::new(trim, trim.size_vec2() * 1. / (trim.size_vec2().x / WIDTH)).tint(egui::Color32::from_rgb(32, 32, 32)));
            });
        });
    }
}

impl epi::App for ProcelioLauncher {

    fn name(&self) -> &str {
       &self.launcher_name
    }

    /// Called once before the first frame.
    fn setup(&mut self, ctx: &egui::Context, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>) {
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        let (s, r) = std::sync::mpsc::channel();
        self.states.config = LoadStatus::Pending(r);
        crate::net::get_config(s);
        if let None = self.install_dir {
            self.settings = true;
        }
        if let Err(e) = crate::patch::delete_old_launcher() {
            self.states.error = Some(Box::new(e));
        }

        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert( "Prime".to_owned(), egui::FontData::from_static(include_bytes!("resources/Prime-Regular.otf")));
        
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Prime".to_owned());

        ctx.set_fonts(fonts);
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        frame.set_window_size(egui::vec2(960.0, 540.0));
        if self.check_states(ctx, frame) {
            return;
        }

        let transparent = egui::Color32::from_rgba_premultiplied(0, 0, 0, 0);

        let col = egui::Color32::from_rgba_premultiplied(8, 8, 8, 225);
        let col2 = egui::Color32::from_rgb(212, 212, 212);

        let nomargin = egui::Frame::default().margin(egui::vec2(0.0, 0.0));

        let bgtex = match &self.states.image {
            LoadStatus::Loaded(x) => {
                self.refs.get_background(Some(x), ctx)
            },
            _ => {
                let img = crate::net::load_image(self.image.clone(), self.image.clone());
                self.refs.get_background(img.as_ref(), ctx)
            }
        };

        let bgwidth = bgtex.size_vec2().x;
        let bgheight = bgtex.size_vec2().y;
        
        let mut top_height = 0.0;
        let mut bottom_height = 0.0;
        let left_width = 0.0;
        let right_width = 0.0;

        let img = egui::Image::new(bgtex, bgtex.size_vec2());

        egui::TopBottomPanel::top("top_panel").resizable(false).frame(nomargin).show(ctx, |ui| {
            top_height = ui.available_height();
            let rect = egui::Rect::from_two_pos(egui::pos2(0.0, 0.0), egui::pos2(bgwidth, top_height));
            img.uv(ProcelioLauncher::uvize(rect, bgwidth, bgheight)).paint_at(ui, rect);

            egui::containers::Frame {
                margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
                rounding: egui::Rounding { nw: 0.0, ne: 0.0, sw: 0.0, se: 0.0 },
                shadow: eframe::epaint::Shadow::default(),
                fill: col,
                stroke: egui::Stroke::default()
            }.show(ui, |ui| {
                ui.columns(2, |ui| {
                    ui[0].with_layout(egui::Layout::left_to_right().with_cross_align(egui::Align::Center), |ui| {
                        let tex = self.refs.get_procelio_logo(ui);
                        ui.image(tex, tex.size_vec2() * 0.5);
                    });

                    egui::Frame::none()
                        .margin(egui::style::Margin { left: 10., right: 10., top: 10., bottom: 10. })
                        .show(&mut ui[1], |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                            ui.label(egui::RichText::new("DAILY MESSAGE").size(18.0).strong().color(col2));
                            if let LoadStatus::Loaded(x) = &self.states.config {
                                ui.label(egui::RichText::new(&x.metadata.message_of_the_day).size(18.0).strong().color(col2));
                                ui.label(egui::RichText::new(&x.metadata.motd_author).size(9.0).color(col2));
                            }
                        });
                     });
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").frame(nomargin).resizable(false).show(ctx, |ui| {
            let base_tex = self.refs.get_baseplate_tex(ctx);
            ui.image(base_tex, base_tex.size_vec2());

            bottom_height = base_tex.size_vec2().y;//ui.available_height();
            let rect = egui::Rect::from_two_pos(egui::pos2(0.0, 540.0 - bottom_height), egui::pos2(960.0, 540.0));
            img.uv(ProcelioLauncher::uvize(rect, bgwidth, bgheight)).paint_at(ui, rect);
            
            let img = egui::Image::new(base_tex, base_tex.size_vec2());
            img.tint(col).paint_at(ui, rect);        

            ui.with_layout(egui::Layout::from_main_dir_and_cross_align(egui::Direction::BottomUp, egui::Align::RIGHT), |ui| {
                ui.with_layout(egui::Layout::from_main_dir_and_cross_align(egui::Direction::RightToLeft, egui::Align::BOTTOM), |ui| {
                    let launch = egui::widgets::Button::new(egui::RichText::new(" PLAY ").size(48.).strong().color(egui::Color32::from_rgb(38, 38, 38))).fill(egui::Color32::from_rgb(255, 117, 0));
                    ui.with_layout(egui::Layout::from_main_dir_and_cross_align(egui::Direction::BottomUp, egui::Align::RIGHT), |ui| {
                        ui.add_space(1.0);
                        if ui.add(launch).clicked() && self.states.ok_to_play() {
                            if let Some(s1) = &self.install_dir {
                                if let LoadStatus::Loaded(_) = &self.states.config {
                                    let (s, r) = std::sync::mpsc::channel();
                                    let (vs, vr) = std::sync::mpsc::channel();
                                    self.states.launching = LoadStatus::Pending(r);
                                    self.states.new_version = LoadStatus::Pending(vr);

                                    let mutex = std::sync::Arc::new(std::sync::Mutex::new((0., "pending".to_owned(), None)));
                                    self.states.processing_status = Some(mutex.clone());

                                    if let Some(c) = self.gather_args() {
                                        crate::patch::play_clicked(s1.to_path_buf(), c, mutex, s, vs);

                                    }
                                }
                            }
                        }
                    });
                });
                if let Some(s) = &self.states.processing_status {
                    ctx.request_repaint();
                    let mut state = s.lock().unwrap();
                    if state.2.is_some() {
                        self.states.error = std::mem::take(&mut state.2);
                    }
                    egui::containers::Frame {
                        margin: egui::style::Margin { left: 5., right: 5., top: 0., bottom: 0. },
                        rounding: egui::Rounding { nw: 5.0, ne: 5.0, sw: 5.0, se: 5.0 },
                        shadow: eframe::epaint::Shadow::default(),
                        fill: col,
                        stroke: egui::Stroke::default()
                    }.show(ui, |ui| {
                        ui.add(egui::widgets::ProgressBar::new(state.0).text(&state.1).animate(true));
                    });
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    let col = egui::Color32::from_rgb(225, 225, 225);
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label(egui::RichText::new(format!("Launcher v{}  |  ", defs::version())).color(col));
                    ui.label(egui::RichText::new("Built with ").color(col));
                    ui.hyperlink_to(egui::RichText::new("egui").strong(), "https://github.com/emilk/egui");
                });
            });
        });
        
        egui::CentralPanel::default().frame(nomargin).show(ctx, |ui| {
            let rect = egui::Rect::from_two_pos(egui::pos2(left_width, top_height), egui::pos2(bgwidth - right_width, bgheight - bottom_height));
            img.uv(ProcelioLauncher::uvize(rect, bgwidth, bgheight)).paint_at(ui, rect);


            ui.with_layout(egui::Layout::right_to_left().with_cross_align(egui::Align::Center), |ui| {
                egui::Frame::none()
                .margin(egui::style::Margin { left: 10., right: 10., top: 10., bottom: 10. })
                .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    self.image(ui, col, "Discord", "https://discord.gg/TDWKZzf",&|x: &mut ProcelioLauncher, ui: &mut egui::Ui| x.refs.get_discord_logo(ui));
                    self.image(ui, col, "Website", "https://proceliogame.com",&|x: &mut ProcelioLauncher, ui: &mut egui::Ui| x.refs.get_website_logo(ui));
                    self.image(ui, col, "YouTube", "https://www.youtube.com/channel/UCb9SlKVDpFMb3_BkcTNv8SQ",&|x: &mut ProcelioLauncher, ui: &mut egui::Ui| x.refs.get_youtube_logo(ui));
                    self.image(ui, col, "Twitter", "https://twitter.com/proceliogame?lang=en",&|x: &mut ProcelioLauncher, ui: &mut egui::Ui| x.refs.get_twitter_logo(ui));
                    ui.label("\n\n");
                    self.image(ui, col, "Settings", "https://discord.gg/TDWKZzf",&|x: &mut ProcelioLauncher, ui: &mut egui::Ui| x.refs.get_settigns_gear(ui));

                });

                egui::Frame::none()
                .show(ui, |ui| {
                    if let LoadStatus::Loaded(x) = &self.states.channel {
                        ui.with_layout(egui::Layout::left_to_right().with_cross_align(egui::Align::Center), |ui| {

                            const WIDTH: usize = 3;
                            let len = x.changelog.len();
                            let mut index = std::cmp::min(usize::saturating_sub(len, WIDTH), self.viewed_changelog);
                            let max_num = std::cmp::min(index + WIDTH, len);

                            if index + WIDTH < len {
                                if ui.button("\n\n\n < \n\n\n").clicked(){
                                    index += 1;
                                }
                            }

                            for i in (index..max_num).rev() {
                                let cl = &x.changelog[len - 1 - i];
                                ProcelioLauncher::changelog(&mut self.refs, ctx, ui, col, &cl.title, &cl.description, &cl.hyperlink);
                            }

                            if index > 0 {
                                if ui.button("\n\n\n > \n\n\n").clicked(){
                                    index -= 1;
                                }
                            }
                            ui.allocate_space(ui.available_size());
                            self.viewed_changelog = index;
                        });
                    }
                    ui.allocate_space(ui.available_size());
                });
            });

            
        });

       
            egui::warn_if_debug_build(ui);
        });

        if let None = self.install_dir {
            egui::Window::new("install-window").show(ctx, |ui| {
                ui.label("Select Procelio Installation Directory:");
                ui.horizontal(|ui| {
                    if ui.button("Install To: ").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            if path.is_dir() {
                                self.install_dir = Some(if path.ends_with("Procelio") { path } else { path.join("Procelio") });
                            }
                        }
                    }
                    ui.code(format!("{}", self.install_dir.as_ref().map(|x|x.as_os_str().to_string_lossy().into_owned()).unwrap_or("".to_owned())));
                });
            });
        }
        if self.settings {
            egui::Window::new("settings-window").show(ctx, |ui| {
                ui.label("Settings:");
                
                egui::ComboBox::from_label("Release Channel")
                    .selected_text(format!("{}", &self.channel))
                    .show_ui(ui, |ui| {
                        if let LoadStatus::Loaded(s) = &self.states.config {
                            let pre = self.channel.clone();

                            s.channels.iter().for_each(|x| {
                                ui.selectable_value(&mut self.channel, x.to_owned(), format!("{}", x));
                            });

                            if self.channel != pre /* Is there a better "wasModified" in egui? */ {
                                let (s, r) = std::sync::mpsc::channel();
                                self.states.channel = LoadStatus::Pending(r);
                                crate::net::get_data(self.channel.clone(), s);
                            }
                        }
                    });

                egui::ComboBox::from_label("CDN")
                    .selected_text(format!("{}", &self.cdn))
                    .show_ui(ui, |ui| {
                        if let LoadStatus::Loaded(s) = &self.states.config {
                            s.cdn_regions.iter().for_each(|x| {
                                ui.selectable_value(&mut self.cdn, x.to_owned(), format!("{}", x));
                            });
                        }
                    }); 
                
                ui.horizontal(|ui| {
                    if ui.button("Install To: ").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            if path.is_dir() {
                                self.install_dir = Some(if path.ends_with("Procelio") { path } else { path.join("Procelio") });
                            }
                        }
                    }
                    ui.code(format!("{}", self.install_dir.as_ref().map(|x|x.as_os_str().to_string_lossy().into_owned()).unwrap_or("".to_owned())));
                });

                if ui.button("View Licenses").clicked() {
                    self.licenses = true;
                }
                if ui.button(egui::RichText::new("Uninstall Procelio").color(egui::Color32::RED)).clicked() && self.states.ok_to_play() {
                    self.states.uninstall = LoadStatus::AwaitingApproval;
                }
                if ui.button("Done").clicked() {
                    self.settings = false;
                }
            });
        }

        if self.licenses {
            egui::Window::new("license-window").show(ctx, |ui| {
                ui.label("Licenses & Dependencies:");

                egui::ScrollArea::both().max_height(256.).show(ui, |ui| {
                    ui.label(defs::LICENSE);
                });

                if ui.button("Done").clicked() {
                    self.licenses = false;
                }
            });
        }

        if let Some(x) = self.states.error.as_ref().map(|x| format!("{:?}", x)) {
            egui::Window::new("error-window").show(ctx, |ui| {
                ui.label("Error:");
                ui.label(format!("{:?}",x));
                if ui.button("OK").clicked() {
                    self.states.error = None;
                }
            });
        }
    }
}
