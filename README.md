Creating framebuffers from audio using julia sets. 

## Dependencies 
- minifb: https://github.com/emoon/rust_minifb
        - a cross platform cpu based renderer of a pixel buffer
    - cpal: 
        - cross platform audio io library 
    - See cargo.toml for complete list of dependencies
## How to run
    - Quick run: `cargo run`
    - Optimized run (faster):
        - `cargo clean`
        - `cargo build --release`
        - `./target/release/main`
    It is recommended to use the optimized version if not modifying the code as it is much faster
## How to use
    - after running, the fractals should change based on low/high frequency sounds captured by default audio devide
    - press Q to close window
    - Press J/K to increase/decrease the exponent of the complex valued polynomial set, see: https://en.wikipedia.org/wiki/Julia_set
    - *Note:* negative exponents are more time consuming to render
## Parameters:
    - WIDTH and HEIGHT: determine window size. If using bigger window sizes you will need a better computer and negative exponants take longer to calculate
    - FRACTAL_DEPTH and GENERATION_INFINITY determine the detail of the julia sets
    
