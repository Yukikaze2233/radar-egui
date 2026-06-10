use super::{RadarApp, FONT_ONCE, LOGO_PATH, MINIMAP_BG_PATH};

impl RadarApp {
    pub(super) fn ensure_minimap_texture(&mut self, ctx: &egui::Context) {
        if self.minimap_texture.is_some() || self.minimap_texture_failed {
            return;
        }

        match image::open(MINIMAP_BG_PATH) {
            Ok(image) => {
                let rgba = image.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                self.minimap_texture = Some(ctx.load_texture(
                    "unity_minimap_bg",
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));
                log::info!("Loaded minimap background from {}", MINIMAP_BG_PATH);
            }
            Err(err) => {
                self.minimap_texture_failed = true;
                log::warn!(
                    "Failed to load minimap background from {}: {}",
                    MINIMAP_BG_PATH,
                    err
                );
            }
        }
    }

    pub(super) fn ensure_logo_texture(&mut self, ctx: &egui::Context) {
        if self.logo_texture.is_some() || self.logo_texture_failed {
            return;
        }

        match image::open(LOGO_PATH) {
            Ok(image) => {
                let rgba = image.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                self.logo_texture =
                    Some(ctx.load_texture("rail_logo", color_image, egui::TextureOptions::LINEAR));
                log::info!("Loaded rail logo from {}", LOGO_PATH);
            }
            Err(err) => {
                self.logo_texture_failed = true;
                log::warn!("Failed to load rail logo from {}: {}", LOGO_PATH, err);
            }
        }
    }

    pub(super) fn setup_fonts(&self, ctx: &egui::Context) {
        FONT_ONCE.call_once(|| {
            let mut fonts = egui::FontDefinitions::default();

            if let Ok(data) = std::fs::read(
                "/usr/share/fonts/TTF/JetBrains-Maple-Mono-NF-XX-XX/JetBrainsMapleMono-Regular.ttf",
            ) {
                log::info!("Loaded JetBrains Maple Mono (Latin + CJK)");
                fonts
                    .font_data
                    .insert("maple".to_owned(), egui::FontData::from_owned(data).into());
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "maple".to_owned());
            }

            if let Ok(data) = std::fs::read("/usr/share/fonts/TTF/JetBrainsMono-Regular.ttf") {
                log::info!("Loaded JetBrains Mono (monospace)");
                fonts.font_data.insert(
                    "jb_mono".to_owned(),
                    egui::FontData::from_owned(data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "jb_mono".to_owned());
            }

            if let Ok(data) = std::fs::read("/usr/share/fonts/TTF/LXGWWenKaiMonoGBScreen.ttf") {
                log::info!("Loaded LXGW WenKai Mono GB Screen (mono CJK fallback)");
                fonts.font_data.insert(
                    "lxgw_mono".to_owned(),
                    egui::FontData::from_owned(data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push("lxgw_mono".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("lxgw_mono".to_owned());
            }

            ctx.set_fonts(fonts);
        });
    }
}
