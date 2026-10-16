use crate::files;
use eframe::egui::{
    self, Align2, Color32, ColorImage, CornerRadius, CursorIcon, FontData, FontDefinitions,
    FontFamily, FontId, Key, Painter, Pos2, Rect, Sense, Stroke, TextureHandle, TextureOptions,
    Vec2, ViewportCommand, pos2, vec2,
};
use image::ImageDecoder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

// Penguin Notes' dark palette
const BG: Color32 = Color32::from_rgb(0x14, 0x11, 0x0f);
const BAR: Color32 = Color32::from_rgba_premultiplied(0x22, 0x1d, 0x1a, 0xee);
const INK: Color32 = Color32::from_rgb(0xf0, 0xec, 0xe2);
const INK_3: Color32 = Color32::from_rgb(0x8a, 0x82, 0x78);
const LINE: Color32 = Color32::from_rgb(0x3a, 0x33, 0x2e);
const HOVER: Color32 = Color32::from_rgba_premultiplied(0x3a, 0x33, 0x2e, 0xff);

/// How long the bar stays after the pointer last moved, in seconds.
const BAR_IDLE: f64 = 2.2;

struct Pic {
    tex: TextureHandle,
    /// size of the file's pixels after EXIF rotation; the texture may be smaller
    size: Vec2,
}

enum Slot {
    Loading,
    Ready(Pic),
    Failed(String),
}

type Decoded = Result<(ColorImage, [u32; 2]), String>;

pub struct Viewer {
    files: Vec<PathBuf>,
    idx: usize,
    cache: HashMap<PathBuf, Slot>,
    tx: Sender<(PathBuf, Decoded)>,
    rx: Receiver<(PathBuf, Decoded)>,
    /// pixels on screen per image pixel; None = fit to the window
    zoom: Option<f32>,
    pan: Vec2,
    /// quarter turns clockwise
    rot: u8,
    fonts: bool,
    logo: Option<TextureHandle>,
    title: String,
    moved_at: f64,
    /// show the bar again on the next frame (a new image was opened)
    wake: bool,
}

impl Viewer {
    pub fn new(start: Option<PathBuf>) -> Self {
        let (tx, rx) = channel();
        let mut v = Viewer {
            files: Vec::new(),
            idx: 0,
            cache: HashMap::new(),
            tx,
            rx,
            zoom: None,
            pan: Vec2::ZERO,
            rot: 0,
            fonts: false,
            logo: None,
            title: String::new(),
            moved_at: 0.0,
            wake: true,
        };
        if let Some(p) = start {
            v.open(&p);
        }
        v
    }

    pub fn open(&mut self, path: &Path) {
        let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        (self.files, self.idx) = files::siblings(&path);
        self.cache.clear();
        self.reset_view();
    }

    fn current(&self) -> Option<&PathBuf> {
        self.files.get(self.idx)
    }

    fn reset_view(&mut self) {
        self.zoom = None;
        self.pan = Vec2::ZERO;
        self.rot = 0;
        self.wake = true;
    }

    fn step(&mut self, by: isize) {
        let n = self.files.len() as isize;
        if n < 2 {
            return;
        }
        self.idx = (self.idx as isize + by).rem_euclid(n) as usize;
        self.reset_view();
    }

    fn go(&mut self, i: usize) {
        if i < self.files.len() && i != self.idx {
            self.idx = i;
            self.reset_view();
        }
    }

    /// Start decoding the current image and its neighbours; forget the rest.
    fn load_around(&mut self, ctx: &egui::Context) {
        let n = self.files.len();
        if n == 0 {
            return;
        }
        let want: Vec<PathBuf> = [0isize, 1, -1, 2]
            .iter()
            .map(|d| self.files[(self.idx as isize + d).rem_euclid(n as isize) as usize].clone())
            .collect();
        self.cache.retain(|p, _| want.contains(p));
        let max = ctx.input(|i| i.max_texture_side).min(16384);
        for p in want {
            if self.cache.contains_key(&p) {
                continue;
            }
            self.cache.insert(p.clone(), Slot::Loading);
            let (tx, ctx) = (self.tx.clone(), ctx.clone());
            std::thread::spawn(move || {
                let r = decode(&p, max);
                let _ = tx.send((p, r));
                ctx.request_repaint();
            });
        }
    }

    fn take_decoded(&mut self, ctx: &egui::Context) {
        while let Ok((p, r)) = self.rx.try_recv() {
            // dropped from the cache while decoding: the user has moved on
            if !self.cache.contains_key(&p) {
                continue;
            }
            let slot = match r {
                Ok((img, [w, h])) => Slot::Ready(Pic {
                    tex: ctx.load_texture(p.to_string_lossy(), img, TextureOptions::LINEAR),
                    size: vec2(w as f32, h as f32),
                }),
                Err(e) => Slot::Failed(e),
            };
            self.cache.insert(p, slot);
        }
    }

    /// Blocks until nothing is decoding (tests only).
    #[cfg(test)]
    pub fn wait(&mut self, ctx: &egui::Context) {
        self.load_around(ctx);
        while self.cache.values().any(|s| matches!(s, Slot::Loading)) {
            let (p, r) = self.rx.recv().unwrap();
            let _ = self.tx.send((p, r));
            self.take_decoded(ctx);
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        // edge to edge, whatever margin the host ui has
        let full = ctx.content_rect();
        let mut ui = ui.new_child(egui::UiBuilder::new().max_rect(full));
        ui.set_clip_rect(full);
        let ui = &mut ui;
        // fonts only take effect next frame, so the first frame just installs them
        if !self.fonts {
            install_fonts(&ctx);
            self.logo = Some(ctx.load_texture(
                "logo",
                load_png(include_bytes!("../assets/icon.png")),
                TextureOptions::LINEAR,
            ));
            self.fonts = true;
            ctx.request_repaint();
            return;
        }
        for f in ctx.input(|i| i.raw.dropped_files.clone()) {
            if !f.path().as_os_str().is_empty() {
                self.open(f.path());
            }
        }
        self.take_decoded(&ctx);
        self.load_around(&ctx);
        self.keys(&ctx);
        self.load_around(&ctx);

        ui.painter().rect_filled(full, CornerRadius::ZERO, BG);
        let now = ctx.input(|i| i.time);
        if std::mem::take(&mut self.wake) || ctx.input(|i| i.pointer.delta() != Vec2::ZERO) {
            self.moved_at = now;
        }

        let name = self.current().map(|p| files::name(p)).unwrap_or_default();
        let title = if name.is_empty() {
            "Penguin Viewer".to_string()
        } else {
            format!("{name} — Penguin Viewer")
        };
        if title != self.title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.title = title;
        }

        let mut has_pic = false;
        match self.current().and_then(|p| self.cache.get(p)) {
            Some(Slot::Ready(_)) => {
                has_pic = true;
                self.image(ui, full);
            }
            Some(Slot::Failed(e)) => {
                let e = e.clone();
                message(ui.painter(), full, "Can't open this image", &e);
            }
            Some(Slot::Loading) => {
                ui.painter().text(
                    full.center(),
                    Align2::CENTER_CENTER,
                    "loading…",
                    FontId::proportional(18.0),
                    INK_3,
                );
            }
            None => self.empty(ui.painter(), full),
        }

        let idle = now - self.moved_at;
        let show = !has_pic || idle < BAR_IDLE || ctx.input(|i| i.pointer.any_down());
        let a = ctx.animate_bool_with_time(egui::Id::new("bar"), show, 0.18);
        if a > 0.0 && !self.files.is_empty() {
            self.bar(ui, full, a);
        }
        if show && has_pic {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(BAR_IDLE - idle + 0.05));
        }
        if !show && has_pic {
            ctx.set_cursor_icon(CursorIcon::None);
        }
    }

    fn keys(&mut self, ctx: &egui::Context) {
        let full = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        let (k, shift) = ctx.input(|i| {
            let pressed = |ks: &[Key]| ks.iter().any(|k| i.key_pressed(*k));
            let k = if pressed(&[Key::ArrowRight, Key::ArrowDown, Key::Space, Key::PageDown]) {
                "next"
            } else if pressed(&[Key::ArrowLeft, Key::ArrowUp, Key::Backspace, Key::PageUp]) {
                "prev"
            } else if pressed(&[Key::Home]) {
                "first"
            } else if pressed(&[Key::End]) {
                "last"
            } else if pressed(&[Key::Plus, Key::Equals]) {
                "in"
            } else if pressed(&[Key::Minus]) {
                "out"
            } else if pressed(&[Key::Num0]) {
                "fit"
            } else if pressed(&[Key::Num1]) {
                "actual"
            } else if pressed(&[Key::R]) {
                "rotate"
            } else if pressed(&[Key::F, Key::F11]) {
                "full"
            } else if pressed(&[Key::Escape]) {
                "esc"
            } else if pressed(&[Key::Q]) {
                "quit"
            } else {
                ""
            };
            (k, i.modifiers.shift)
        });
        let center = ctx.content_rect().center();
        match k {
            "next" => self.step(1),
            "prev" => self.step(-1),
            "first" => self.go(0),
            "last" => self.go(self.files.len().saturating_sub(1)),
            "in" => self.zoom_by(ctx, 1.25, center),
            "out" => self.zoom_by(ctx, 0.8, center),
            "fit" => self.fit(),
            "actual" => self.actual(ctx, center),
            "rotate" => self.rotate(if shift { 3 } else { 1 }),
            "full" => ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!full)),
            "esc" if full => ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false)),
            "esc" | "quit" => ctx.send_viewport_cmd(ViewportCommand::Close),
            _ => {}
        }
    }

    fn pic_size(&self) -> Option<Vec2> {
        match self.current().and_then(|p| self.cache.get(p)) {
            Some(Slot::Ready(p)) => Some(if self.rot % 2 == 1 { vec2(p.size.y, p.size.x) } else { p.size }),
            _ => None,
        }
    }

    fn fit_scale(&self, view: Rect) -> f32 {
        let Some(s) = self.pic_size() else { return 1.0 };
        (view.width() / s.x).min(view.height() / s.y).min(1.0)
    }

    fn scale(&self, view: Rect) -> f32 {
        self.zoom.unwrap_or_else(|| self.fit_scale(view))
    }

    fn fit(&mut self) {
        self.zoom = None;
        self.pan = Vec2::ZERO;
    }

    fn actual(&mut self, ctx: &egui::Context, at: Pos2) {
        let s = self.scale(ctx.content_rect());
        self.zoom_by(ctx, 1.0 / s, at);
    }

    fn rotate(&mut self, quarters: u8) {
        self.rot = (self.rot + quarters) % 4;
        self.fit();
    }

    /// Zoom by `f`, keeping the image point under `at` where it is.
    fn zoom_by(&mut self, ctx: &egui::Context, f: f32, at: Pos2) {
        let view = ctx.content_rect();
        let old = self.scale(view);
        let new = (old * f).clamp(0.02, 40.0);
        let fit = self.fit_scale(view);
        if (new - fit).abs() / fit < 0.01 {
            return self.fit();
        }
        let rel = at - (view.center() + self.pan);
        self.pan = at - rel * (new / old) - view.center();
        self.zoom = Some(new);
        self.clamp_pan(view);
    }

    /// Keep a zoomed image from being dragged off screen; a smaller-than-window axis stays centred.
    fn clamp_pan(&mut self, view: Rect) {
        let Some(s) = self.pic_size() else { return };
        let shown = s * self.scale(view);
        let lim = ((shown - view.size()) / 2.0).max(Vec2::ZERO);
        self.pan = self.pan.clamp(-lim, lim);
    }

    fn image(&mut self, ui: &mut egui::Ui, view: Rect) {
        let ctx = ui.ctx().clone();
        let resp = ui.interact(view, egui::Id::new("image"), Sense::click_and_drag());
        if let Some(at) = resp.hover_pos() {
            let (scroll, pinch) = ctx.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
            let f = (scroll / 300.0).exp() * pinch;
            if (f - 1.0).abs() > 1e-4 {
                self.zoom_by(&ctx, f, at);
            }
        }
        if resp.double_clicked() {
            match (self.zoom, resp.interact_pointer_pos()) {
                (Some(_), _) => self.fit(),
                (None, Some(at)) => self.actual(&ctx, at),
                _ => {}
            }
        }
        let zoomed = self.zoom.is_some();
        if resp.dragged() && zoomed {
            self.pan += resp.drag_delta();
            self.clamp_pan(view);
        }
        if zoomed && self.pic_size().is_some_and(|s| s.x * self.scale(view) > view.width() || s.y * self.scale(view) > view.height()) {
            ctx.set_cursor_icon(if resp.dragged() { CursorIcon::Grabbing } else { CursorIcon::Grab });
        }

        let Some(Slot::Ready(pic)) = self.current().and_then(|p| self.cache.get(p)) else { return };
        let scale = self.scale(view);
        let rect = Rect::from_center_size(view.center() + self.pan, pic.size * scale);
        let angle = self.rot as f32 * std::f32::consts::FRAC_PI_2;
        // smooth when shrinking, sharp pixels when blown up past 200%
        let opts = if scale > 2.0 { TextureOptions::NEAREST } else { TextureOptions::LINEAR };
        egui::Image::from_texture(egui::load::SizedTexture::from_handle(&pic.tex))
            .texture_options(opts)
            .rotate(angle, Vec2::splat(0.5))
            .paint_at(ui, rect);
    }

    fn empty(&self, p: &Painter, view: Rect) {
        let c = view.center();
        if let Some(t) = &self.logo {
            p.image(
                t.id(),
                Rect::from_center_size(c - vec2(0.0, 56.0), Vec2::splat(96.0)),
                Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        p.text(c + vec2(0.0, 22.0), Align2::CENTER_CENTER, "Drop an image here", FontId::new(26.0, bold()), INK);
        p.text(
            c + vec2(0.0, 54.0),
            Align2::CENTER_CENTER,
            "or open one from Files  ·  arrow keys to browse",
            FontId::proportional(16.0),
            INK_3,
        );
    }

    /// Floating bar at the bottom: prev/next, name, position, size, zoom and view buttons.
    fn bar(&mut self, ui: &mut egui::Ui, view: Rect, a: f32) {
        let ctx = ui.ctx().clone();
        let p = ui.painter().clone().with_layer_id(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("bar")));
        let fade = |c: Color32| c.gamma_multiply(a);
        let name = self.current().map(|p| files::name(p)).unwrap_or_default();
        let name = ellipsize(&name, ((view.width() - 420.0) / 9.0).clamp(8.0, 60.0) as usize);
        let mut info = format!("{} / {}", self.idx + 1, self.files.len());
        if let Some(s) = self.pic_size() {
            info += &format!("   {}×{}   {}%", s.x as u32, s.y as u32, (self.scale(view) * 100.0).round());
        }
        let name_g = p.layout_no_wrap(name, FontId::new(17.0, bold()), fade(INK));
        let info_g = p.layout_no_wrap(info, FontId::proportional(15.0), fade(INK_3));

        let b = 32.0;
        let gap = 14.0;
        let w = 2.0 * b + gap + name_g.size().x + gap + info_g.size().x + gap + 4.0 * b + 24.0;
        let h = 46.0;
        let rect = Rect::from_center_size(pos2(view.center().x, view.bottom() - 18.0 - h / 2.0 + (1.0 - a) * 14.0), vec2(w.min(view.width() - 16.0), h));
        p.rect(rect, CornerRadius::same(23), fade(BAR), Stroke::new(1.0, fade(LINE)), egui::StrokeKind::Inside);
        // the bar swallows clicks so they don't reach the image underneath
        ui.interact(rect, egui::Id::new("bar-bg"), Sense::click_and_drag());

        let mut x = rect.left() + 12.0;
        let y = rect.center().y;
        let button = |ui: &mut egui::Ui, x: &mut f32, icon: Icon, tip: &str| -> bool {
            let r = Rect::from_center_size(pos2(*x + b / 2.0, y), Vec2::splat(b));
            *x += b;
            let resp = ui.interact(r, egui::Id::new(("bar", tip)), Sense::click()).on_hover_text(tip);
            if resp.hovered() {
                p.circle_filled(r.center(), b / 2.0, fade(HOVER));
                ctx.set_cursor_icon(CursorIcon::PointingHand);
            }
            paint_icon(&p, r.shrink(9.0), icon, fade(if resp.hovered() { INK } else { INK_3 }));
            resp.clicked()
        };
        let many = self.files.len() > 1;
        if button(ui, &mut x, Icon::Prev, "previous  (left arrow)") && many {
            self.step(-1);
        }
        if button(ui, &mut x, Icon::Next, "next  (right arrow)") && many {
            self.step(1);
        }
        x += gap;
        p.galley(pos2(x, y - name_g.size().y / 2.0), name_g.clone(), INK);
        x += name_g.size().x + gap;
        p.galley(pos2(x, y - info_g.size().y / 2.0), info_g.clone(), INK_3);
        x += info_g.size().x + gap;
        let c = view.center();
        if button(ui, &mut x, Icon::Fit, "fit to window  0") {
            self.fit();
        }
        if button(ui, &mut x, Icon::Actual, "actual size  1") {
            self.actual(&ctx, c);
        }
        if button(ui, &mut x, Icon::Rotate, "rotate  R") {
            self.rotate(1);
        }
        let full = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if button(ui, &mut x, Icon::Full, "fullscreen  F") {
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!full));
        }
    }
}

#[derive(Clone, Copy)]
enum Icon {
    Prev,
    Next,
    Fit,
    Actual,
    Rotate,
    Full,
}

fn paint_icon(p: &Painter, r: Rect, icon: Icon, c: Color32) {
    let s = Stroke::new(2.0, c);
    let (l, t, rt, bt, m) = (r.left(), r.top(), r.right(), r.bottom(), r.center());
    match icon {
        Icon::Prev => {
            p.line(vec![pos2(m.x + 3.0, t), pos2(m.x - 4.0, m.y), pos2(m.x + 3.0, bt)], s);
        }
        Icon::Next => {
            p.line(vec![pos2(m.x - 3.0, t), pos2(m.x + 4.0, m.y), pos2(m.x - 3.0, bt)], s);
        }
        Icon::Fit => {
            p.rect_stroke(r.shrink(1.0), CornerRadius::same(3), s, egui::StrokeKind::Middle);
            p.rect_filled(r.shrink(4.5), CornerRadius::same(1), c);
        }
        Icon::Actual => {
            p.text(m, Align2::CENTER_CENTER, "1:1", FontId::new(15.0, bold()), c);
        }
        Icon::Rotate => {
            let rad = r.width() / 2.0 - 0.5;
            let pts: Vec<Pos2> = (0..=20)
                .map(|i| {
                    let a = -1.2 + i as f32 / 20.0 * 4.6;
                    m + vec2(a.cos(), a.sin()) * rad
                })
                .collect();
            let end = pts[0];
            p.line(pts, s);
            // arrow head at the open end, pointing clockwise
            p.line(vec![end + vec2(-5.0, -1.0), end, end + vec2(-1.0, 5.0)], s);
        }
        Icon::Full => {
            let k = 5.0;
            for (cx, cy, dx, dy) in [(l, t, 1.0, 1.0), (rt, t, -1.0, 1.0), (l, bt, 1.0, -1.0), (rt, bt, -1.0, -1.0)] {
                p.line(vec![pos2(cx + dx * k, cy), pos2(cx, cy), pos2(cx, cy + dy * k)], s);
            }
        }
    }
}

fn message(p: &Painter, view: Rect, head: &str, sub: &str) {
    let c = view.center();
    p.text(c - vec2(0.0, 14.0), Align2::CENTER_CENTER, head, FontId::new(24.0, bold()), INK);
    p.text(c + vec2(0.0, 18.0), Align2::CENTER_CENTER, ellipsize(sub, 90), FontId::proportional(15.0), INK_3);
}

fn ellipsize(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let keep = max.saturating_sub(1);
    let head: String = s.chars().take(keep * 2 / 3).collect();
    let tail: String = s.chars().rev().take(keep - keep * 2 / 3).collect::<Vec<_>>().into_iter().rev().collect();
    format!("{head}…{tail}")
}

/// Decode a file, apply its EXIF rotation, and shrink it to fit the GPU's texture limit.
fn decode(path: &Path, max_side: usize) -> Decoded {
    let err = |e: &dyn std::fmt::Display| e.to_string();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if ext == "svg" || ext == "svgz" {
        return svg(path, max_side);
    }
    let mut reader = image::ImageReader::open(path).map_err(|e| err(&e))?.with_guessed_format().map_err(|e| err(&e))?;
    reader.no_limits();
    let mut dec = reader.into_decoder().map_err(|e| err(&e))?;
    let orient = dec.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut img = image::DynamicImage::from_decoder(dec).map_err(|e| err(&e))?;
    img.apply_orientation(orient);
    let full = [img.width(), img.height()];
    if img.width().max(img.height()) as usize > max_side {
        img = img.resize(max_side as u32, max_side as u32, image::imageops::FilterType::Triangle);
    }
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Ok((ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()), full))
}

/// Rasterise an SVG with its longer side at 2048 px (or the texture limit), so zooming in stays sharp.
/// The reported size is the drawing's own, so 100% means its intended size.
fn svg(path: &Path, max_side: usize) -> Decoded {
    use resvg::{tiny_skia, usvg};
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut opt = usvg::Options::default();
    opt.resources_dir = path.parent().map(Path::to_path_buf);
    opt.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_data(&data, &opt).map_err(|e| e.to_string())?;
    let size = tree.size();
    let long = size.width().max(size.height());
    let k = (2048f32.min(max_side as f32) / long).max(1.0).min(max_side as f32 / long);
    let (w, h) = ((size.width() * k).round().max(1.0) as u32, (size.height() * k).round().max(1.0) as u32);
    let mut pix = tiny_skia::Pixmap::new(w, h).ok_or("SVG is too large")?;
    resvg::render(&tree, tiny_skia::Transform::from_scale(k, k), &mut pix.as_mut());
    let full = [size.width().round() as u32, size.height().round() as u32];
    Ok((ColorImage::from_rgba_premultiplied([w as usize, h as usize], pix.data()), full))
}

fn load_png(bytes: &[u8]) -> ColorImage {
    let img = image::load_from_memory(bytes).expect("png").to_rgba8();
    ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw())
}

fn bold() -> FontFamily {
    FontFamily::Name("bold".into())
}

fn install_fonts(ctx: &egui::Context) {
    let mut f = FontDefinitions::default();
    for (k, b) in [
        ("kalam", &include_bytes!("../assets/Kalam-Regular.ttf")[..]),
        ("kalam-bold", &include_bytes!("../assets/Kalam-Bold.ttf")[..]),
        // file names in Arabic: Kalam has no Arabic letters
        ("arabic", &include_bytes!("../assets/NotoNaskhArabic-Regular.ttf")[..]),
    ] {
        f.font_data.insert(k.into(), Arc::new(FontData::from_static(b)));
    }
    let prop = f.families.get_mut(&FontFamily::Proportional).unwrap();
    prop.insert(0, "kalam".into());
    prop.insert(1, "arabic".into());
    let mut b = vec!["kalam-bold".to_string()];
    b.extend(f.families[&FontFamily::Proportional].iter().cloned());
    f.families.insert(bold(), b);
    ctx.set_fonts(f);
}

impl eframe::App for Viewer {
    fn ui(&mut self, ui: &mut egui::Ui, _f: &mut eframe::Frame) {
        self.draw(ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    /// a fresh folder per test: the tests run in parallel
    fn dir(test: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("penguin-viewer-{test}-{}", std::process::id()));
        if !d.exists() {
            std::fs::create_dir_all(&d).unwrap();
            // a wide gradient, a tall one with a stripe, and a broken file
            let wide = image::RgbImage::from_fn(1600, 900, |x, y| image::Rgb([(x / 7) as u8, (y / 4) as u8, 160]));
            wide.save(d.join("img2.png")).unwrap();
            let tall = image::RgbImage::from_fn(300, 600, |x, _| if x < 60 { image::Rgb([217, 77, 42]) } else { image::Rgb([240, 236, 226]) });
            tall.save(d.join("img10.jpg")).unwrap();
            std::fs::write(d.join("img3.png"), b"not a png").unwrap();
            std::fs::write(d.join("notes.txt"), b"skip me").unwrap();
            std::fs::write(
                d.join("logo.svg"),
                br##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32"><rect width="64" height="32" rx="6" fill="#d94d2a"/></svg>"##,
            )
            .unwrap();
        }
        d
    }

    fn harness(v: Viewer) -> Harness<'static, Viewer> {
        let mut h = Harness::builder()
            .with_size([1000.0, 700.0])
            .wgpu()
            .build_ui_state(|ui, v: &mut Viewer| v.draw(ui), v);
        h.run_steps(3);
        let ctx = h.ctx.clone();
        h.state_mut().wait(&ctx);
        h.run_steps(3);
        h
    }

    fn shot(h: &mut Harness<'_, Viewer>, name: &str) {
        let d = format!("{}/tests-shots", env!("CARGO_MANIFEST_DIR"));
        std::fs::create_dir_all(&d).unwrap();
        h.render().expect("render").save(format!("{d}/{name}.png")).unwrap();
    }

    #[test]
    fn lists_folder_in_order() {
        let v = Viewer::new(Some(dir("lists_folder_in_order").join("img2.png")));
        let names: Vec<String> = v.files.iter().map(|p| files::name(p)).collect();
        assert_eq!(names, ["img2.png", "img3.png", "img10.jpg", "logo.svg"]);
        assert_eq!(v.idx, 0);
    }

    #[test]
    fn browse_zoom_rotate() {
        let mut h = harness(Viewer::new(Some(dir("browse_zoom_rotate").join("img2.png"))));
        shot(&mut h, "01-fit");
        assert!(h.state().zoom.is_none());
        // fit: 1600×900 into 1000×700 → 62.5%
        let view = h.ctx.content_rect();
        assert!((h.state().scale(view) - 0.625).abs() < 0.01);

        h.key_press(Key::Num1);
        h.run_steps(2);
        assert_eq!(h.state().zoom, Some(1.0));
        shot(&mut h, "02-actual");

        h.key_press(Key::Num0);
        h.key_press(Key::R);
        h.run_steps(2);
        assert_eq!(h.state().rot, 1);
        assert_eq!(h.state().pic_size(), Some(vec2(900.0, 1600.0)));
        shot(&mut h, "03-rotated");

        // next: the broken file shows an error, rotation is reset
        h.key_press(Key::ArrowRight);
        h.run_steps(2);
        let ctx = h.ctx.clone();
        h.state_mut().wait(&ctx);
        h.run_steps(2);
        assert_eq!(h.state().idx, 1);
        assert_eq!(h.state().rot, 0);
        assert!(matches!(h.state().cache.get(&h.state().files[1]), Some(Slot::Failed(_))));
        shot(&mut h, "04-broken");

        h.key_press(Key::ArrowRight);
        h.run_steps(2);
        let ctx = h.ctx.clone();
        h.state_mut().wait(&ctx);
        h.run_steps(2);
        assert_eq!(h.state().idx, 2);
        shot(&mut h, "05-tall");

        // the SVG keeps its own size, drawn from a sharper texture
        h.key_press(Key::ArrowRight);
        h.run_steps(2);
        let ctx = h.ctx.clone();
        h.state_mut().wait(&ctx);
        h.run_steps(2);
        assert_eq!(h.state().pic_size(), Some(vec2(64.0, 32.0)));
        shot(&mut h, "06-svg");

        // wraps around
        h.key_press(Key::ArrowRight);
        h.run_steps(2);
        assert_eq!(h.state().idx, 0);
    }

    /// PV_REAL=<some image> cargo test real_file  →  tests-shots/07-real.png
    #[test]
    fn real_file() {
        let Ok(p) = std::env::var("PV_REAL") else { return };
        let mut h = harness(Viewer::new(Some(PathBuf::from(p))));
        assert!(matches!(h.state().cache.get(&h.state().files[h.state().idx]), Some(Slot::Ready(_))));
        shot(&mut h, "07-real");
    }

    #[test]
    fn empty_window() {
        let mut h = harness(Viewer::new(None));
        shot(&mut h, "00-empty");
        assert!(h.state().files.is_empty());
    }

    #[test]
    fn zoom_keeps_point_under_cursor() {
        let mut h = harness(Viewer::new(Some(dir("zoom_keeps_point_under_cursor").join("img2.png"))));
        let ctx = h.ctx.clone();
        let view = ctx.content_rect();
        let at = view.center() + vec2(200.0, 100.0);
        let v = h.state_mut();
        let before = (at - (view.center() + v.pan)) / v.scale(view);
        v.zoom_by(&ctx, 2.0, at);
        let after = (at - (view.center() + v.pan)) / v.scale(view);
        assert!((before - after).length() < 0.5, "{before:?} vs {after:?}");
    }
}

#[cfg(test)]
mod decode_tests {
    /// PV_AVIF=<a 320×200 .avif> cargo test decodes_avif
    #[test]
    fn decodes_avif() {
        let Ok(p) = std::env::var("PV_AVIF") else { return };
        let (img, full) = super::decode(std::path::Path::new(&p), 8192).unwrap();
        assert_eq!(full, [320, 200]);
        assert_eq!(img.size, [320, 200]);
    }
}
