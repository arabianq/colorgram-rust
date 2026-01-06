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
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let buf = fs::read("image.jpg")?;
    let colors = extract(&buf, 5)?;

    for color in colors {
        println!("Color: {}, Weight: {:.2}%", color.rgb, color.proportion * 100.0);
    }
    Ok(())
}
```
