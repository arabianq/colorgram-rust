use image;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as fmtResult},
};

/// Represents a color in the HSL (Hue, Saturation, Lightness) color space.
///
/// Note: In this implementation, values are stored as `u8` (0-255) rather than
/// the standard degrees (0-360) and percentages (0-100), following the
/// integer-based logic inherited from `colorgram.py`.
#[derive(PartialEq, Eq, Debug)]
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

/// Represents a color in the RGB (Red, Green, Blue) color space.
#[derive(PartialEq, Eq, Debug)]
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

/// A structure representing an extracted color, its HSL equivalent,
/// and its prevalence in the image.
#[derive(PartialEq, Debug)]
pub struct Color {
    /// The average RGB color of the cluster.
    pub rgb: Rgb,
    /// The HSL representation (calculated using the bug-compatible `rgb_to_hsl`).
    pub hsl: Hsl,
    /// The proportion of this color in the image (range 0.0 to 1.0).
    pub proportion: f32,
}

impl Color {
    /// Creates a new `Color` instance and automatically calculates its HSL value.
    ///
    /// ### Arguments:
    /// * `rgb` - The base RGB color.
    /// * `proportion` - The weight of this color relative to others.
    pub fn new(rgb: Rgb, proportion: f32) -> Color {
        let hsl = rgb_to_hsl(&rgb);
        Color {
            rgb,
            hsl,
            proportion,
        }
    }
}

/// Converts RGB color space to HSL.
///
/// ### ⚠️ Bug-compatibility Warning:
/// This function **does not** produce a mathematically correct HSL value. It contains
/// specific calculation errors inherited from the original `colorgram.py` library.
/// These deviations are preserved intentionally to ensure that the color extraction
/// results remain identical to the Python implementation.
///
/// ### Implementation Details:
/// * **Luminance (L)**: Calculated as the average of the most and least dominant components.
/// * **Saturation (S)**: Scaled to 0-255 range using integer division that may lose precision.
/// * **Hue (H)**: Calculated using a 0-255 scale (instead of 0-360°) with custom offsets
///   (1530, 510, 1020) and integer math.
///
/// ### Arguments:
/// * `rgb` - A reference to the input `Rgb` struct.
///
/// ### Returns:
/// * `Hsl` - The "colorgram-style" HSL representation.
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

/// Extracts a palette of dominant colors from an image buffer.
///
/// ### Process:
/// 1. **Decoding**: Loads the image from the provided byte buffer and converts it to RGB8.
/// 2. **Quantization**: Pixels are grouped into "buckets" based on a simplified 6-bit key
///    derived from Luminance (Y), Hue (H), and Lightness (L).
/// 3. **Aggregation**: Sums the R, G, and B components and counts pixels in each bucket.
/// 4. **Averaging**: Calculates the mean color for each non-empty bucket.
/// 5. **Selection**: Returns the top `number_of_color` most frequent colors.
///
/// ### Arguments:
/// * `buffer` - A byte slice containing encoded image data (e.g., JPEG, PNG).
/// * `number_of_color` - The maximum number of dominant colors to return.
///
/// ### Example:
/// ```rust
/// use colorgram::{extract, Color};
/// use std::fs;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let buf = fs::read("image.jpg")?;
///     let colors = extract(&buf, 5)?;
///
///     for color in colors {
///         println!("Color: {}, Weight: {:.2}%", color.rgb, color.proportion * 100.0);
///     }
///     Ok(())
/// }
/// ```
pub fn extract(buffer: &[u8], number_of_color: usize) -> Result<Vec<Color>, Box<dyn Error>> {
    let img = image::load_from_memory(buffer)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_rgb_to_hsl() {
        let rgb = Rgb { r: 0, g: 0, b: 0 };
        let hsl = rgb_to_hsl(&rgb);
        assert_eq!(hsl, Hsl { h: 0, s: 0, l: 0 });

        let rgb = Rgb {
            r: 255,
            g: 255,
            b: 255,
        };
        let hsl = rgb_to_hsl(&rgb);
        assert_eq!(hsl, Hsl { h: 0, s: 0, l: 255 });

        let rgb = Rgb {
            r: 100,
            g: 20,
            b: 30,
        };
        let hsl = rgb_to_hsl(&rgb);
        assert_eq!(
            hsl,
            Hsl {
                h: 249,
                s: 170,
                l: 60
            }
        );
    }

    #[test]
    fn test_extract() {
        let buf = fs::read("test.png").unwrap();
        let colors = extract(&buf, 1).unwrap();

        assert_eq!(
            colors,
            vec![Color {
                rgb: Rgb {
                    r: 214,
                    g: 163,
                    b: 101
                },
                hsl: Hsl {
                    h: 23,
                    s: 147,
                    l: 157
                },
                proportion: 1.0
            }]
        );

        let colors = extract(&buf, 1000).unwrap();
        let amount_of_colors = colors.len();

        assert_eq!(amount_of_colors, 35);
    }
}
