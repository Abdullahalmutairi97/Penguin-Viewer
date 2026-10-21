//! Video playback with ffmpeg compiled into the binary.
//!
//! The shape follows door_player (Apache-2.0, https://crates.io/crates/door_player): one thread
//! demuxes and decodes, the sound goes through a cpal output stream and doubles as the clock, and
//! decoded frames wait in a queue until the clock reaches them. Everything else is written for this app.
use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions, Vec2, vec2};
use ffmpeg_next as ff;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, SyncSender, TryRecvError, channel, sync_channel};
use std::sync::{Arc, Mutex, Once};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// The longer side of a decoded frame; bigger videos are scaled down while converting to RGBA.
const MAX_SIDE: f64 = 1920.0;
/// Decoded frames waiting for their turn.
const QUEUE: usize = 6;
/// Seconds of sound buffered ahead of the speakers.
const AUDIO_AHEAD: f64 = 1.0;

struct Frame {
    epoch: u32,
    pts: f64,
    size: [usize; 2],
    /// the size the video is meant to be shown at (pixel aspect ratio applied)
    shown: Vec2,
    rgba: Vec<u8>,
}

/// Interleaved stereo samples on their way to the speakers, and what has been played of them.
struct Audio {
    buf: VecDeque<f32>,
    rate: u32,
    /// media time of the first sample played since the last seek
    base: f64,
    /// frames (one sample per channel) handed to the speakers since then
    played: u64,
    /// no sample has arrived since the last seek: the next one sets `base`
    fresh: bool,
}

struct Shared {
    audio: Mutex<Audio>,
    quit: AtomicBool,
    paused: AtomicBool,
    eof: AtomicBool,
    /// f32 bits
    gain: AtomicU32,
}

enum Cmd {
    Seek { t: f64, epoch: u32 },
}

struct Info {
    dur: f64,
    has_audio: bool,
    /// quarter turns clockwise the picture needs, from the file's rotation metadata
    turn: u8,
}

fn init() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = ff::init();
        ff::log::set_level(ff::log::Level::Quiet);
    });
}

fn gain(volume: f64) -> f32 {
    (volume / 100.0).clamp(0.0, 1.0).powi(2) as f32
}

pub struct Player {
    sh: Arc<Shared>,
    cmds: Option<Sender<Cmd>>,
    frames: Receiver<Frame>,
    thread: Option<JoinHandle<()>>,
    stream: Option<cpal::Stream>,
    tex: Option<TextureHandle>,
    /// a decoded frame that is not due yet
    pending: Option<Frame>,
    /// frames of an older seek are thrown away
    want_gen: u32,
    /// show the next frame at once (first frame, or the first after a seek), even if the clock is not there yet
    need_first: bool,
    shown: Vec2,
    /// the clock when there is no sound
    wall_base: f64,
    wall_since: Option<Instant>,
    audio_on: bool,
    /// seconds
    pub pos: f64,
    pub dur: f64,
    pub paused: bool,
    /// 0..=100
    pub volume: f64,
    pub muted: bool,
    pub ended: bool,
    /// quarter turns clockwise the picture needs
    pub turn: u8,
}

impl Player {
    pub fn new(path: &Path, ctx: &egui::Context) -> Result<Self, String> {
        init();
        let sh = Arc::new(Shared {
            audio: Mutex::new(Audio { buf: VecDeque::new(), rate: 48000, base: 0.0, played: 0, fresh: true }),
            quit: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            eof: AtomicBool::new(false),
            gain: AtomicU32::new(gain(100.0).to_bits()),
        });
        let (cmd_tx, cmd_rx) = channel();
        let (frame_tx, frame_rx) = sync_channel(QUEUE);
        let (info_tx, info_rx) = channel::<Result<Info, String>>();
        let (rate_tx, rate_rx) = channel::<u32>();
        let thread = {
            let (sh, path, ctx) = (sh.clone(), path.to_path_buf(), ctx.clone());
            std::thread::Builder::new()
                .name("video".into())
                .spawn(move || run(sh, path, ctx, cmd_rx, frame_tx, info_tx, rate_rx))
                .map_err(|e| e.to_string())?
        };
        let info = info_rx.recv().map_err(|_| "the decoder stopped".to_string())??;
        let mut stream = None;
        if info.has_audio {
            // 0 tells the decoder thread to go on without sound
            match open_audio(&sh) {
                Some((s, rate)) => {
                    stream = Some(s);
                    let _ = rate_tx.send(rate);
                }
                None => {
                    let _ = rate_tx.send(0);
                }
            }
        }
        Ok(Player {
            sh,
            cmds: Some(cmd_tx),
            frames: frame_rx,
            thread: Some(thread),
            audio_on: stream.is_some(),
            stream,
            tex: None,
            pending: None,
            want_gen: 0,
            need_first: true,
            shown: Vec2::ZERO,
            wall_base: 0.0,
            wall_since: Some(Instant::now()),
            pos: 0.0,
            dur: info.dur,
            paused: false,
            volume: 100.0,
            muted: false,
            ended: false,
            turn: info.turn,
        })
    }

    /// Where the playback is, in seconds: what the speakers have played, or the wall clock without sound.
    fn clock(&self) -> f64 {
        if self.audio_on {
            let a = self.sh.audio.lock().unwrap();
            a.base + a.played as f64 / a.rate as f64
        } else {
            self.wall_base + self.wall_since.map_or(0.0, |t| t.elapsed().as_secs_f64())
        }
    }

    /// Show the frame that is due, refresh the numbers.
    pub fn poll(&mut self, ctx: &egui::Context) {
        let now = self.clock();
        let mut latest = None;
        loop {
            let f = match self.pending.take() {
                Some(f) => f,
                None => match self.frames.try_recv() {
                    Ok(f) => f,
                    Err(_) => break,
                },
            };
            if f.epoch != self.want_gen {
                continue;
            }
            if self.need_first || f.pts <= now + 0.002 {
                self.need_first = false;
                // several frames may be due when we are behind: only the newest is worth drawing
                latest = Some(f);
            } else {
                self.pending = Some(f);
                break;
            }
        }
        if let Some(f) = latest {
            self.shown = f.shown;
            let img = ColorImage::from_rgba_premultiplied(f.size, &f.rgba);
            match &mut self.tex {
                Some(t) => t.set(img, TextureOptions::LINEAR),
                None => self.tex = Some(ctx.load_texture("video", img, TextureOptions::LINEAR)),
            }
        }
        let drained = self.sh.eof.load(Ordering::Relaxed)
            && self.pending.is_none()
            && (!self.audio_on || self.sh.audio.lock().unwrap().buf.is_empty());
        self.ended = drained;
        self.paused = self.sh.paused.load(Ordering::Relaxed);
        self.pos = if self.dur > 0.0 { now.clamp(0.0, self.dur) } else { now.max(0.0) };
        if self.ended && self.dur > 0.0 {
            self.pos = self.dur;
        }
        if !self.paused && !self.ended {
            let due = self.pending.as_ref().map_or(0.016, |f| (f.pts - now).clamp(0.001, 0.016));
            ctx.request_repaint_after(Duration::from_secs_f64(due));
        }
    }

    /// The current frame and the size the video is meant to be shown at.
    pub fn frame(&self) -> Option<(&TextureHandle, Vec2)> {
        self.tex.as_ref().map(|t| (t, self.shown))
    }

    pub fn toggle_pause(&mut self) {
        if self.ended {
            self.seek_to(0.0);
            self.set_paused(false);
        } else {
            self.set_paused(!self.paused);
        }
    }

    fn set_paused(&mut self, p: bool) {
        if p == self.paused {
            return;
        }
        if !self.audio_on {
            if p {
                self.wall_base = self.clock();
                self.wall_since = None;
            } else {
                self.wall_since = Some(Instant::now());
            }
        }
        self.paused = p;
        self.sh.paused.store(p, Ordering::Relaxed);
    }

    pub fn seek_by(&mut self, secs: f64) {
        let t = self.clock() + secs;
        self.seek_to(if self.dur > 0.0 { t.clamp(0.0, self.dur) } else { t.max(0.0) });
    }

    pub fn seek_to(&mut self, secs: f64) {
        self.want_gen += 1;
        self.need_first = true;
        self.pending = None;
        self.pos = secs;
        self.ended = false;
        self.sh.eof.store(false, Ordering::Relaxed);
        self.wall_base = secs;
        self.wall_since = (!self.paused).then(Instant::now);
        {
            // silence until the decoder has caught up; it clears this again when it gets the command
            let mut a = self.sh.audio.lock().unwrap();
            a.buf.clear();
            a.played = 0;
            a.base = secs;
            a.fresh = true;
        }
        if let Some(tx) = &self.cmds {
            let _ = tx.send(Cmd::Seek { t: secs, epoch: self.want_gen });
        }
    }

    pub fn set_volume(&mut self, v: f64) {
        self.volume = v.clamp(0.0, 100.0);
        self.muted = false;
        self.apply_gain();
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        self.apply_gain();
    }

    fn apply_gain(&self) {
        let g = if self.muted { 0.0 } else { gain(self.volume) };
        self.sh.gain.store(g.to_bits(), Ordering::Relaxed);
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.sh.quit.store(true, Ordering::Relaxed);
        // the speakers first, then the decoder thread (its channel closing also wakes it)
        self.stream = None;
        self.cmds = None;
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// An f32 output on the default device, fed from `Shared::audio`; the stream's rate is returned.
fn open_audio(sh: &Arc<Shared>) -> Option<(cpal::Stream, u32)> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    let dev = cpal::default_host().default_output_device()?;
    let mut cfg = dev.default_output_config().ok()?;
    if cfg.sample_format() != cpal::SampleFormat::F32 {
        cfg = dev.supported_output_configs().ok()?.find(|c| c.sample_format() == cpal::SampleFormat::F32)?.with_max_sample_rate();
    }
    let (rate, ch) = (cfg.sample_rate(), cfg.channels() as usize);
    sh.audio.lock().unwrap().rate = rate;
    let s = sh.clone();
    let stream = dev
        .build_output_stream(
            cfg.into(),
            move |data: &mut [f32], _| {
                if s.paused.load(Ordering::Relaxed) {
                    data.fill(0.0);
                    return;
                }
                let g = f32::from_bits(s.gain.load(Ordering::Relaxed));
                let mut a = s.audio.lock().unwrap();
                let mut got = 0;
                for out in data.chunks_mut(ch) {
                    if a.buf.len() < 2 {
                        out.fill(0.0);
                        continue;
                    }
                    let (l, r) = (a.buf.pop_front().unwrap() * g, a.buf.pop_front().unwrap() * g);
                    got += 1;
                    match out.len() {
                        1 => out[0] = (l + r) / 2.0,
                        _ => {
                            out[0] = l;
                            out[1] = r;
                            out[2..].fill(0.0);
                        }
                    }
                }
                a.played += got;
            },
            |_| {},
            None,
        )
        .ok()?;
    stream.play().ok()?;
    Some((stream, rate))
}

// ---- the decoder thread ---------------------------------------------------------------------------

/// What the decoder thread shares with the rest: queues, the clock's audio, seek state.
struct Link {
    sh: Arc<Shared>,
    ctx: egui::Context,
    cmds: Receiver<Cmd>,
    /// a command that arrived while waiting for room in a queue
    next: Option<Cmd>,
    frames: SyncSender<Frame>,
    epoch: u32,
    /// frames and sound before this are decoded (to get to a keyframe) but not shown
    skip_until: f64,
    /// the container's start time, taken off every timestamp
    t0: f64,
    /// the player is gone
    gone: bool,
}

impl Link {
    /// True when the work in hand should be dropped: the player is gone or a seek is waiting.
    fn interrupted(&mut self) -> bool {
        if self.sh.quit.load(Ordering::Relaxed) {
            self.gone = true;
            return true;
        }
        if self.next.is_none() {
            match self.cmds.try_recv() {
                Ok(c) => self.next = Some(c),
                Err(TryRecvError::Disconnected) => {
                    self.gone = true;
                    return true;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        self.next.is_some()
    }

    fn send_frame(&mut self, mut f: Frame) {
        loop {
            match self.frames.try_send(f) {
                Ok(()) => return self.ctx.request_repaint(),
                Err(std::sync::mpsc::TrySendError::Full(back)) => {
                    f = back;
                    if self.interrupted() {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(4));
                }
                Err(_) => return,
            }
        }
    }

    /// Queue interleaved stereo samples that start at `pts`; waits while a second of them is already queued.
    fn push_audio(&mut self, samples: &[f32], pts: f64) {
        loop {
            {
                let mut a = self.sh.audio.lock().unwrap();
                if (a.buf.len() as f64) < a.rate as f64 * 2.0 * AUDIO_AHEAD {
                    let frames = samples.len() / 2;
                    let mut start = 0;
                    if a.fresh {
                        if pts < self.skip_until {
                            start = (((self.skip_until - pts) * a.rate as f64).round() as usize).min(frames);
                        }
                        if start == frames {
                            return;
                        }
                        a.base = pts + start as f64 / a.rate as f64;
                        a.fresh = false;
                    }
                    a.buf.extend(&samples[start * 2..]);
                    return;
                }
            }
            if self.interrupted() {
                return;
            }
            std::thread::sleep(Duration::from_millis(4));
        }
    }
}

struct VideoStream {
    index: usize,
    dec: ff::decoder::Video,
    tb: f64,
    scaler: Option<(ff::software::scaling::Context, (ff::format::Pixel, u32, u32))>,
    last_pts: f64,
}

struct AudioStream {
    index: usize,
    dec: ff::decoder::Audio,
    tb: f64,
    rs: ff::software::resampling::Context,
    layout: ff::ChannelLayout,
    /// output rate over input rate
    ratio: f64,
}

impl VideoStream {
    fn drain(&mut self, link: &mut Link) {
        let mut f = ff::frame::Video::empty();
        while self.dec.receive_frame(&mut f).is_ok() {
            if link.interrupted() {
                return;
            }
            let pts = match f.timestamp().or(f.pts()) {
                Some(t) => t as f64 * self.tb - link.t0,
                None => self.last_pts + 0.04,
            };
            self.last_pts = pts;
            if pts < link.skip_until - 0.001 {
                continue;
            }
            if let Some(frame) = self.convert(&f, pts, link.epoch) {
                link.send_frame(frame);
            }
        }
    }

    fn convert(&mut self, f: &ff::frame::Video, pts: f64, epoch: u32) -> Option<Frame> {
        let (w, h) = (f.width(), f.height());
        if w == 0 || h == 0 {
            return None;
        }
        let sar = f64::from(self.dec.aspect_ratio());
        let dw = if sar.is_finite() && sar > 0.0 { w as f64 * sar } else { w as f64 };
        let k = (MAX_SIDE / dw.max(h as f64)).min(1.0);
        let (ow, oh) = (((dw * k).round() as u32).max(1), ((h as f64 * k).round() as u32).max(1));
        let key = (f.format(), w, h);
        if self.scaler.as_ref().is_none_or(|(_, k)| *k != key) {
            let sc = ff::software::scaling::Context::get(f.format(), w, h, ff::format::Pixel::RGBA, ow, oh, ff::software::scaling::Flags::BILINEAR).ok()?;
            self.scaler = Some((sc, key));
        }
        let mut out = ff::frame::Video::empty();
        self.scaler.as_mut()?.0.run(f, &mut out).ok()?;
        let (data, stride, row) = (out.data(0), out.stride(0), ow as usize * 4);
        let mut rgba = Vec::with_capacity(row * oh as usize);
        for y in 0..oh as usize {
            rgba.extend_from_slice(&data[y * stride..y * stride + row]);
        }
        Some(Frame { epoch, pts, size: [ow as usize, oh as usize], shown: vec2(dw as f32, h as f32), rgba })
    }
}

impl AudioStream {
    fn drain(&mut self, link: &mut Link) {
        let mut f = ff::frame::Audio::empty();
        while self.dec.receive_frame(&mut f).is_ok() {
            if link.interrupted() {
                return;
            }
            if f.channel_layout().is_empty() {
                f.set_channel_layout(self.layout);
            }
            let pts = f.timestamp().or(f.pts()).map_or(0.0, |t| t as f64 * self.tb - link.t0);
            // the output frame has to be big enough up front: left empty, ffmpeg-next sizes it like the
            // input, and when the rate goes up the extra samples would pile up inside the resampler
            let room = (f.samples() as f64 * self.ratio) as usize + 256;
            let mut out = ff::frame::Audio::new(ff::format::Sample::F32(ff::format::sample::Type::Packed), room, ff::ChannelLayout::STEREO);
            if self.rs.run(&f, &mut out).is_err() || out.samples() == 0 {
                continue;
            }
            let bytes = &out.data(0)[..out.samples() * 2 * 4];
            let samples: Vec<f32> = bytes.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
            link.push_audio(&samples, pts);
        }
    }
}

fn rotation(stream: &ff::format::stream::Stream) -> u8 {
    // SAFETY: reads the stream's own parameters, which live as long as the stream.
    unsafe {
        let par = (*stream.as_ptr()).codecpar;
        if par.is_null() || (*par).coded_side_data.is_null() {
            return 0;
        }
        for i in 0..(*par).nb_coded_side_data as usize {
            let d = &*(*par).coded_side_data.add(i);
            if d.type_ == ff::ffi::AVPacketSideDataType::AV_PKT_DATA_DISPLAYMATRIX && d.size >= 36 {
                let deg = ff::ffi::av_display_rotation_get(d.data as *const i32);
                if deg.is_finite() {
                    // ffmpeg reports counter-clockwise degrees
                    return ((-deg / 90.0).round() as i32).rem_euclid(4) as u8;
                }
            }
        }
    }
    0
}

/// Open the file and report what is in it; None when it can't be played.
fn open(path: &PathBuf) -> Result<(ff::format::context::Input, Option<VideoStream>, Option<(usize, ff::decoder::Audio, f64)>, Info), String> {
    let ictx = ff::format::input(path).map_err(|e| e.to_string())?;
    let mut video = None;
    let mut turn = 0;
    if let Some(s) = ictx.streams().best(ff::media::Type::Video) {
        let mut c = ff::codec::context::Context::from_parameters(s.parameters()).map_err(|e| e.to_string())?;
        let mut t = c.threading();
        t.kind = ff::threading::Type::Frame;
        t.count = 0;
        c.set_threading(t);
        let dec = c.decoder().video().map_err(|e| e.to_string())?;
        turn = rotation(&s);
        video = Some(VideoStream { index: s.index(), dec, tb: f64::from(s.time_base()), scaler: None, last_pts: -0.04 });
    }
    let mut audio = None;
    if let Some(s) = ictx.streams().best(ff::media::Type::Audio) {
        if let Ok(c) = ff::codec::context::Context::from_parameters(s.parameters()) {
            if let Ok(dec) = c.decoder().audio() {
                audio = Some((s.index(), dec, f64::from(s.time_base())));
            }
        }
    }
    if video.is_none() && audio.is_none() {
        return Err("no audio or video to play".into());
    }
    let dur = ictx.duration();
    let info = Info {
        dur: if dur > 0 { dur as f64 / ff::ffi::AV_TIME_BASE as f64 } else { 0.0 },
        has_audio: audio.is_some(),
        turn,
    };
    Ok((ictx, video, audio, info))
}

fn run(sh: Arc<Shared>, path: PathBuf, ctx: egui::Context, cmds: Receiver<Cmd>, frames: SyncSender<Frame>, info_tx: Sender<Result<Info, String>>, rate_rx: Receiver<u32>) {
    let (mut ictx, mut video, audio, info) = match open(&path) {
        Ok(o) => o,
        Err(e) => {
            let _ = info_tx.send(Err(e));
            return;
        }
    };
    let has_audio = info.has_audio;
    if info_tx.send(Ok(info)).is_err() {
        return;
    }
    // the sound's output rate is only known once the speakers are open
    let mut audio = match (audio, has_audio) {
        (Some((index, dec, tb)), true) => match rate_rx.recv() {
            Ok(rate) if rate > 0 => {
                let ratio = rate as f64 / dec.rate().max(1) as f64;
                let layout = if dec.channel_layout().is_empty() { ff::ChannelLayout::default(dec.channels() as i32) } else { dec.channel_layout() };
                ff::software::resampling::Context::get(
                    dec.format(),
                    layout,
                    dec.rate(),
                    ff::format::Sample::F32(ff::format::sample::Type::Packed),
                    ff::ChannelLayout::STEREO,
                    rate,
                )
                .ok()
                .map(|rs| AudioStream { index, dec, tb, rs, layout, ratio })
            }
            _ => None,
        },
        _ => None,
    };
    // SAFETY: plain read of the container's start time.
    let start = unsafe { (*ictx.as_ptr()).start_time };
    let t0 = if start == i64::MIN { 0.0 } else { (start as f64 / ff::ffi::AV_TIME_BASE as f64).max(0.0) };
    let mut link = Link { sh, ctx, cmds, next: None, frames, epoch: 0, skip_until: f64::MIN, t0, gone: false };
    let mut eof = false;
    loop {
        if link.gone || link.sh.quit.load(Ordering::Relaxed) {
            return;
        }
        let cmd = match link.next.take() {
            Some(c) => Some(c),
            None if eof => match link.cmds.recv_timeout(Duration::from_millis(20)) {
                Ok(c) => Some(c),
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => return,
            },
            None => match link.cmds.try_recv() {
                Ok(c) => Some(c),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => return,
            },
        };
        if let Some(Cmd::Seek { t, epoch }) = cmd {
            let ts = ((t + link.t0) * ff::ffi::AV_TIME_BASE as f64) as i64;
            let _ = ictx.seek(ts, ..ts);
            if let Some(v) = &mut video {
                v.dec.flush();
            }
            if let Some(a) = &mut audio {
                a.dec.flush();
            }
            link.epoch = epoch;
            link.skip_until = t;
            eof = false;
            link.sh.eof.store(false, Ordering::Relaxed);
            let mut a = link.sh.audio.lock().unwrap();
            a.buf.clear();
            a.played = 0;
            a.base = t;
            a.fresh = true;
            continue;
        }
        if eof {
            continue;
        }
        let mut pkt = ff::Packet::empty();
        if pkt.read(&mut ictx).is_err() {
            // the end (or a file that can't be read any further): let the decoders hand out what they hold
            if let Some(v) = &mut video {
                let _ = v.dec.send_eof();
                v.drain(&mut link);
            }
            if let Some(a) = &mut audio {
                let _ = a.dec.send_eof();
                a.drain(&mut link);
            }
            if link.next.is_none() && !link.gone {
                eof = true;
                link.sh.eof.store(true, Ordering::Relaxed);
                link.ctx.request_repaint();
            }
            continue;
        }
        match (&mut video, &mut audio) {
            (Some(v), _) if pkt.stream() == v.index => {
                let _ = v.dec.send_packet(&pkt);
                v.drain(&mut link);
            }
            (_, Some(a)) if pkt.stream() == a.index => {
                let _ = a.dec.send_packet(&pkt);
                a.drain(&mut link);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2 second 320×180 clip without sound made with ffmpeg; None if there is no ffmpeg to make it.
    fn clip(name: &str, extra: &[&str]) -> Option<PathBuf> {
        let p = std::env::temp_dir().join(format!("penguin-viewer-{name}-{}.mp4", std::process::id()));
        let ok = std::process::Command::new("ffmpeg")
            .args(["-v", "error", "-y"])
            .args(extra)
            .args(["-f", "lavfi", "-i", "testsrc=size=320x180:rate=25", "-t", "2", "-pix_fmt", "yuv420p"])
            .arg(&p)
            .status()
            .is_ok_and(|s| s.success());
        ok.then_some(p)
    }

    fn wait(pl: &mut Player, ctx: &egui::Context, f: impl Fn(&Player) -> bool) {
        for _ in 0..200 {
            pl.poll(ctx);
            if f(pl) {
                return;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        panic!("timed out");
    }

    #[test]
    fn plays_a_clip() {
        let Some(p) = clip("plays", &[]) else { return };
        let ctx = egui::Context::default();
        let mut pl = Player::new(&p, &ctx).unwrap();
        wait(&mut pl, &ctx, |p| p.frame().is_some() && p.pos > 0.2);
        let (tex, size) = pl.frame().expect("a frame");
        assert_eq!(size, vec2(320.0, 180.0));
        assert_eq!(tex.size(), [320, 180]);
        assert!((pl.dur - 2.0).abs() < 0.2, "duration {}", pl.dur);
        pl.toggle_pause();
        pl.poll(&ctx);
        assert!(pl.paused);
        pl.seek_to(1.2);
        wait(&mut pl, &ctx, |p| p.frame().is_some() && (p.pos - 1.2).abs() < 0.1);
        pl.toggle_pause();
        wait(&mut pl, &ctx, |p| p.ended);
        assert!((pl.pos - pl.dur).abs() < 0.3, "pos {} dur {}", pl.pos, pl.dur);
        // pressing play at the end starts over
        pl.toggle_pause();
        wait(&mut pl, &ctx, |p| !p.ended && p.pos < 1.0);
        pl.set_volume(40.0);
        assert_eq!(pl.volume, 40.0);
        pl.toggle_mute();
        assert!(pl.muted);
    }

    #[test]
    fn rotated_clip_reports_its_turn() {
        let Some(base) = clip("rot", &[]) else { return };
        let p = base.with_extension("rotated.mp4");
        let ok = std::process::Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-display_rotation", "90", "-i"])
            .arg(&base)
            .args(["-c", "copy"])
            .arg(&p)
            .status()
            .is_ok_and(|s| s.success());
        if !ok {
            return;
        }
        let ctx = egui::Context::default();
        // ffmpeg's -display_rotation is counter-clockwise: a quarter that way is three clockwise
        assert_eq!(Player::new(&p, &ctx).unwrap().turn, 3);
        assert_eq!(Player::new(&base, &ctx).unwrap().turn, 0);
    }

    #[test]
    fn broken_file_is_an_error() {
        let p = std::env::temp_dir().join(format!("penguin-viewer-broken-{}.mp4", std::process::id()));
        std::fs::write(&p, b"definitely not a video").unwrap();
        let ctx = egui::Context::default();
        assert!(Player::new(&p, &ctx).is_err());
    }

    /// Plays a quiet 3 second tone through the real speakers: cargo test --release audio_clock -- --ignored
    #[test]
    #[ignore]
    fn audio_clock() {
        let base = std::env::temp_dir().join(format!("penguin-viewer-tone-{}.mp4", std::process::id()));
        let ok = std::process::Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc=size=320x180:rate=25", "-f", "lavfi", "-i", "sine=frequency=440:sample_rate=44100", "-t", "3", "-pix_fmt", "yuv420p", "-c:a", "aac"])
            .arg(&base)
            .status()
            .is_ok_and(|s| s.success());
        assert!(ok);
        let ctx = egui::Context::default();
        let mut pl = Player::new(&base, &ctx).unwrap();
        assert!(pl.audio_on, "no audio output opened");
        pl.set_volume(15.0);
        let t = Instant::now();
        wait(&mut pl, &ctx, |p| p.pos > 1.0);
        let real = t.elapsed().as_secs_f64();
        assert!((real - pl.pos).abs() < 0.35, "clock {} vs real {}", pl.pos, real);
        pl.seek_to(2.0);
        wait(&mut pl, &ctx, |p| p.pos > 2.2);
        wait(&mut pl, &ctx, |p| p.ended);
        assert!((pl.pos - pl.dur).abs() < 0.3);
    }

    /// Every codec the viewer should handle, from clips ffmpeg makes: cargo test --release codecs -- --ignored
    #[test]
    #[ignore]
    fn codecs() {
        let cases: &[(&str, &[&str], &str)] = &[
            ("h264", &["-c:v", "libx264"], "mp4"),
            ("hevc", &["-c:v", "libx265", "-tag:v", "hvc1"], "mp4"),
            ("vp9", &["-c:v", "libvpx-vp9"], "webm"),
            ("vp8", &["-c:v", "libvpx"], "webm"),
            ("av1", &["-c:v", "libsvtav1"], "mkv"),
            ("h264-10bit", &["-c:v", "libx264", "-pix_fmt", "yuv420p10le"], "mkv"),
            ("mpeg4", &["-c:v", "mpeg4"], "avi"),
        ];
        let ctx = egui::Context::default();
        for (name, args, ext) in cases {
            let p = std::env::temp_dir().join(format!("pv-codec-{name}-{}.{ext}", std::process::id()));
            let ok = std::process::Command::new("ffmpeg")
                .args(["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc=size=320x180:rate=25", "-t", "1"])
                .args(*args)
                .arg(&p)
                .status()
                .is_ok_and(|s| s.success());
            if !ok {
                eprintln!("{name}: no encoder here, skipped");
                continue;
            }
            let mut pl = Player::new(&p, &ctx).unwrap_or_else(|e| panic!("{name}: {e}"));
            wait(&mut pl, &ctx, |p| p.frame().is_some());
            eprintln!("{name}: ok");
        }
    }
}
