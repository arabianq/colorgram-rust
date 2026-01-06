use image;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as fmtResult},
    path::Path,
};

pub struct Hsl {
    pub h: u8,
    pub s: u8,
    pub l: u8,
}

impl Display for Hsl {
    fn fmt(&self, f: &mut Formatter) -> fmtResult {
        write!(f, "hsl({}, {}, {})", self.h, self.s, self.l)
    }
}

pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Display for Rgb {
    fn fmt(&self, f: &mut Formatter) -> fmtResult {
        write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
    }
}

pub struct Color {
    pub rgb: Rgb,
    pub hsl: Hsl,
    pub proportion: f32,
}

impl Color {
    pub fn new(rgb: Rgb, proportion: f32) -> Color {
        let hsl = rgb_to_hsl(&rgb);
        Color {
            rgb,
            hsl,
            proportion,
        }
    }
}

fn rgb_to_hsl(rgb: &Rgb) -> Hsl {
    let r = rgb.r as i32;
    let g = rgb.g as i32;
    let b = rgb.b as i32;

    let most = r.max(g).max(b);
    let least = r.min(g).min(b);

    let l = (most + least) >> 1;

    let (h, s) = if most == least {
        (0, 0)
    } else {
        let diff = most - least;
        let s = if l > 127 {
            diff * 255 / (510 - most - least)
        } else {
            diff * 255 / (most + least)
        };
        let h = if most == r {
            ((g - b) * 255 / diff + if g < b { 1530 } else { 0 }) / 6
        } else if most == g {
            ((b - r) * 255 / diff + 510) / 6
        } else {
            ((r - g) * 255 / diff + 1020) / 6
        };
        (h as u8, s as u8)
    };

    Hsl { h, s, l: l as u8 }
}

pub fn extract<P: AsRef<Path>>(
    path: P,
    number_of_color: usize,
) -> Result<Vec<Color>, Box<dyn Error>> {
    let img = image::open(path)?;
    let img = img.to_rgb8();

    let mut samples = vec![0u32; 4 * 4096];

    for pixel in img.pixels() {
        let rgb = Rgb {
            r: pixel[0],
            g: pixel[1],
            b: pixel[2],
        };
        let hsl = rgb_to_hsl(&rgb);

        let y_val = ((rgb.r as f32 * 0.2126 + rgb.g as f32 * 0.7152 + rgb.b as f32 * 0.0722) as u8)
            & 0b1100_0000;
        let h = hsl.h & 0b1100_0000;
        let l = hsl.l & 0b1100_0000;

        let packed = ((y_val as usize) << 4) | ((h as usize) << 2) | (l as usize);
        let idx = packed * 4;
        samples[idx] += rgb.r as u32;
        samples[idx + 1] += rgb.g as u32;
        samples[idx + 2] += rgb.b as u32;
        samples[idx + 3] += 1;
    }

    let mut used = Vec::new();
    for (_i, chunk) in samples.chunks(4).enumerate() {
        let count = chunk[3];
        if count > 0 {
            let avg_r = (chunk[0] / count) as u8;
            let avg_g = (chunk[1] / count) as u8;
            let avg_b = (chunk[2] / count) as u8;
            let avg_rgb = Rgb {
                r: avg_r,
                g: avg_g,
                b: avg_b,
            };
            used.push((count, avg_rgb));
        }
    }

    used.sort_unstable_by(|a, b| b.0.cmp(&a.0));

    let nmin = number_of_color.min(used.len());
    let top_used = &used[..nmin];
    let sum_counts: u32 = top_used.iter().map(|&(count, _)| count).sum();

    let mut colors = Vec::with_capacity(number_of_color);
    for (count, rgb) in used.into_iter().take(number_of_color) {
        colors.push(Color::new(rgb, count as f32 / sum_counts as f32));
    }

    Ok(colors)
}
