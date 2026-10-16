use std::cmp::Ordering;
use std::path::{Path, PathBuf};

/// Extensions the `image` crate decodes (avif through dav1d), plus SVG through resvg.
const EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "jfif", "webp", "gif", "bmp", "ico", "tif", "tiff", "tga", "qoi",
    "pnm", "pbm", "pgm", "ppm", "hdr", "exr", "dds", "ff", "avif", "svg", "svgz",
];

pub fn is_image(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTS.contains(&e.to_ascii_lowercase().as_str()))
}

/// The images next to `path` (or inside it, for a folder) in natural order, and where `path` sits.
pub fn siblings(path: &Path) -> (Vec<PathBuf>, usize) {
    let (dir, target) = if path.is_dir() {
        (path.to_path_buf(), None)
    } else {
        let dir = path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
        (dir.to_path_buf(), Some(path))
    };
    let mut list: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file() && is_image(p))
                .collect()
        })
        .unwrap_or_default();
    list.sort_by(|a, b| natural(&name(a), &name(b)));
    let Some(t) = target else { return (list, 0) };
    if let Some(i) = list.iter().position(|p| p.file_name() == t.file_name()) {
        return (list, i);
    }
    // a file with an odd extension that was opened by hand still gets shown
    list.insert(0, t.to_path_buf());
    (list, 0)
}

pub fn name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// "img2" before "img10", case ignored.
pub fn natural(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let num = |it: &mut std::iter::Peekable<std::str::Chars>| {
                    let mut s = String::new();
                    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
                        s.push(c);
                        it.next();
                    }
                    s
                };
                let (na, nb) = (num(&mut a), num(&mut b));
                let (ta, tb) = (na.trim_start_matches('0'), nb.trim_start_matches('0'));
                let o = ta.len().cmp(&tb.len()).then(ta.cmp(tb));
                if o != Ordering::Equal {
                    return o;
                }
            }
            (Some(x), Some(y)) => {
                let o = x.to_lowercase().cmp(y.to_lowercase());
                if o != Ordering::Equal {
                    return o;
                }
                a.next();
                b.next();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn natural_order() {
        let mut v = vec!["img10.png", "IMG2.png", "img1.png", "a.png", "img02b.png"];
        v.sort_by(|a, b| natural(a, b));
        assert_eq!(v, ["a.png", "img1.png", "IMG2.png", "img02b.png", "img10.png"]);
    }

    #[test]
    fn extensions() {
        assert!(is_image(Path::new("x/Photo.JPG")));
        assert!(is_image(Path::new("a.webp")));
        assert!(!is_image(Path::new("notes.md")));
        assert!(!is_image(Path::new("noext")));
    }
}
