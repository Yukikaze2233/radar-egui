use std::sync::{Arc, Mutex};

use crate::video_stream::VideoFrame;

#[derive(Default)]
pub(super) struct VideoTextureCache {
    texture: Option<egui::TextureHandle>,
    rgba_scratch: Vec<u8>,
    frame_seq: Option<u32>,
    dimensions: Option<(u32, u32)>,
}

impl VideoTextureCache {
    pub(super) fn refresh(&mut self, ctx: &egui::Context, shared: &Arc<Mutex<Option<VideoFrame>>>) {
        let Ok(video) = shared.lock() else {
            return;
        };

        let Some(frame) = video.as_ref() else {
            self.texture = None;
            self.frame_seq = None;
            self.dimensions = None;
            return;
        };

        let dimensions = (frame.width, frame.height);
        let needs_upload = self.texture.is_none()
            || self.frame_seq != Some(frame.seq)
            || self.dimensions != Some(dimensions);

        if !needs_upload {
            return;
        }

        fill_rgba_scratch(&frame.data, &mut self.rgba_scratch);
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [frame.width as usize, frame.height as usize],
            &self.rgba_scratch,
        );

        if let Some(texture) = self.texture.as_mut() {
            texture.set(image, egui::TextureOptions::LINEAR);
        } else {
            self.texture =
                Some(ctx.load_texture("video_frame", image, egui::TextureOptions::LINEAR));
        }

        self.frame_seq = Some(frame.seq);
        self.dimensions = Some(dimensions);
    }

    pub(super) fn texture(&self) -> Option<&egui::TextureHandle> {
        self.texture.as_ref()
    }
}

fn fill_rgba_scratch(bgr: &[u8], rgba: &mut Vec<u8>) {
    let pixel_count = bgr.len() / 3;
    let required_len = pixel_count * 4;
    if rgba.len() != required_len {
        rgba.resize(required_len, 0);
    }

    for (index, chunk) in bgr.chunks_exact(3).enumerate() {
        let offset = index * 4;
        rgba[offset] = chunk[2];
        rgba[offset + 1] = chunk[1];
        rgba[offset + 2] = chunk[0];
        rgba[offset + 3] = 255;
    }
}
