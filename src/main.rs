use std::process::Command;
use copypasta::{ClipboardContext, ClipboardProvider};
use qr_code::{QrCode, bmp_monochrome};
use eframe::NativeOptions;
use eframe::egui::{self, vec2, Image, CentralPanel, Color32, ColorImage, TextureHandle, TextureOptions};
use eframe::egui::load::SizedTexture;

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions::default();
    eframe::run_native(
        "QR Clipboard Copy - Preview",
        options,
        Box::new(|_cc| {
            egui_extras::install_image_loaders(&_cc.egui_ctx);
            Ok(Box::<QRVwr>::default())
        }),
    )
}

struct QRVwr {
    copied_content: String,
    generated_bmp: bmp_monochrome::Bmp,
    image_texture: Option<TextureHandle>,
    zoom: f32
}

impl Default for QRVwr {
    fn default() -> Self {
        let mut ctx = ClipboardContext::new().unwrap();
        let mut content = match ctx.get_contents() {
            Ok(cc) => cc,
            Err(_) => panic!("Fatal: No clipboard content was found.")
        };

        // x11 was likely not available or being used, attempting wayland workaround using `wl-paste`
        if content.is_empty() {
            let wl_output = Command::new("wl-paste").output().expect("failed to execute command: unable to check wayland clipboard");
            
            if !wl_output.status.success() {
                panic!("Fatal: No clipboard content was found - tried x11 and wayland");
            }

            content = String::from_utf8_lossy(&wl_output.stdout).to_string();
        }

        let qr_code = QrCode::new(&content).unwrap();
        
        Self {
            generated_bmp: qr_code.to_bmp(),
            copied_content: content,
            image_texture: None,
            zoom: 2.0,
        }
    }
}

impl eframe::App for QRVwr {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.image_texture.is_none() {
            if let Ok(img) = load_bmp_image(&self.generated_bmp) {
                let texture = ctx.load_texture("bmp", img, TextureOptions::NEAREST);
                self.image_texture = Some(texture);
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            let scroll = ctx.input(|i| i.smooth_scroll_delta.y);

            if scroll != 0.0 {
                let zoom_speed = 0.1;
                self.zoom *= (1.0_f32 + zoom_speed * scroll).max(0.5_f32).min(5.0_f32);
                self.zoom = self.zoom.clamp(0.5, 5.0);
            }

            ui.centered_and_justified(|ui| {
                if let Some(texture) = &self.image_texture {
                    let zoomed_size = texture.size_vec2() * self.zoom;
                    let sized_texture = SizedTexture::new(texture.id(), zoomed_size);
                    ui.add(Image::new(sized_texture)).on_hover_text_at_pointer(&self.copied_content);
                } else {
                    ui.label("Failed to load image.");
                }
            });
        });
    }
}

fn load_bmp_image(bmp: &bmp_monochrome::Bmp) -> Result<ColorImage, String> {
    let width = bmp.width() as usize;
    let height = bmp.height() as usize;

    let mut pixels = Vec::with_capacity(width * height);

    for y in 0..height {
        for x in 0..width {
            let is_white = bmp.get(x as u16, (height - 1 - y) as u16);
            let color = if is_white { Color32::WHITE } else { Color32::BLACK };
            pixels.push(color);
        }
    }

    Ok(ColorImage {
        size: [width, height],
        pixels,
        source_size: vec2(width as f32, height as f32),
    })
}
