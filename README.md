# colorgram-rust

---

colorgram-rust is a rust program that lets you extract colors from image.
It is a port of [colorgram.py](https://github.com/obskyr/colorgram.py) (which is itself a port
of [colorgram.js](https://github.com/darosh/colorgram-js), so it's basically a port of a port =D)

**Why?** Well, it's ~25 times faster than colorgram.py (9 ms vs 225 ms for test.png on my laptop)

### Installation as CLI utility
```bash
cargo install colorgram
```


### Adding to your rust project
```toml
[dependencies]
colorgram = "0.1.0"
```


### Usage Example (code from main.rs)

```rust
use colorgram::extract;

fn main() {
    let img_path = "test.png";
    let colors_amount = 10;

    let colors = extract(img_path, colors_amount).unwrap();
    for color in colors {
        println!("RGB: {} {} {} HSL: {} {} {} Proportion: {:.2}", color.rgb.r, color.rgb.g, color.rgb.b, color.hsl.h, color.hsl.s, color.hsl.l, color.proportion);
    }
}
```
