<div align="center">
    
# SelectNES

[![Rust Tests](https://github.com/Polar-404/SelectNes/actions/workflows/test.yml/badge.svg)](https://github.com/Polar-404/SelectNes/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-Stable-orange.svg)](https://www.rust-lang.org/)

**A Nintendo Entertainment System (NES) emulator built in Rust.**
</div>


<div align="center">

| Super Mario Bros. | Kirby's Adventure |
| :---: | :---: |
| <img width="400" src="https://github.com/user-attachments/assets/20603cd5-cff6-4800-922b-20561797412f" /> | <img width="400" src="https://github.com/user-attachments/assets/dad57e11-a0f6-43f4-8ba6-d4dacc253fc9" /> |

</div>

> [!NOTE] 
> This project was created purely as a hobby, a personal challenge, and an opportunity to practice low-level programming and system architecture in Rust. So for the foreseeable future it is not intended to compete with established emulators but rather serves as a proof of concept and a deep learning experience regarding low-level programming and the NES hardware.


---
## How to Run

Make sure you have [Rust and Cargo](https://www.rust-lang.org/tools/install) installed on your system.

```bash

# Clone the repository
git clone https://github.com/Polar-404/SelectNES.git

# Navigate to the directory
cd SelectNES

# Compile and run
cargo run --release
```

## Controls

| NES Button      |    Primary    |
| :-------------- | :-----------: |
| **D-Pad Up**    |  `Up Arrow`   |
| **D-Pad Down**  | `Down Arrow`  |
| **D-Pad Left**  | `Left Arrow`  |
| **D-Pad Right** | `Right Arrow` |
| **A**           |      `Z`      |
| **B**           |      `X`      |
| **Select**      |      `C`      |
| **Start**       |      `V`      |

---
## Current Features

- Ricoh 2A03 (6502) CPU (including indirect JMP bug support).
    
- PPU (Picture Processing Unit).
    
- Partial APU (Audio Processing Unit).
    
- Controller input implemented.
    
- Supported mappers:
    
    - NROM (Mapper 0)
        
    - MMC1 (Mapper 1)
      
    - MMC3 (Mapper 4)
      
- Debug Tools: (Pattern Table viewer, Palette viewer e Hex Memory viewer)


---
## Tech Stack

- **[Rust](https://www.rust-lang.org/):** Main language used for the project, ensuring memory safety and high performance.

- **Graphics & UI:**
  - **[Glow](https://github.com/grovesNL/glow):** "GL on Whatever" — used for cross-platform OpenGL bindings.
  - **[Egui](https://github.com/emilk/egui):** Immediate mode GUI library used for the debugging tools (PPU, Memory, and CPU viewers).
  - **OpenGL:** Low-level rendering for the NES screen and implementation of custom shaders.

- **[Cpal](https://github.com/RustAudio/cpal):** Low-level library for audio processing and output.

- **[Ringbuf](https://crates.io/crates/ringbuf):** Lock-free circular buffer for efficient audio synchronization.

- **[Arboard](https://crates.io/crates/arboard):** Native system clipboard access for easy ROM path pasting.

- **Others:** `image`, `rfd`, `serde`.

  
---
## Roadmap / To-Do

As this is an ongoing learning project, several areas still need improvement:

- [x] **User Interface:** Improve the start menu and add more graphics and audio configuration options.
    
- [ ] **Audio (APU):** Implement the DMC (Delta Modulation Channel).
    
- [ ] **Synchronization:** Sync audio with FPS to maintain a more stable frame rate, faithful to the original console.
    
- [ ] **Mappers:** Add support for more mappers to increase game compatibility.
    
- [x] **Saves:** Implement Save/Load states functionality. (only in-game saves so far, saving the emulator state isn't implemented yet)
    
- [ ] **Code Quality:** Refactor and clean up the codebase, and potentially add documentation and internationalization (EN/PT-BR).
    
- [ ] **Accuracy & Testing:** Fix known bugs (e.g., failing test 3 in `official_only.nes`) and run/create more accuracy tests.
    
- [ ] **Unofficial Opcodes:** Implement undocumented 6502 instructions.
    
- [ ] **Scripting:** Implement user script support with Lua.
    
- [x] **More Palettes:** Implement the ability for the user to insert their own palettes via interface and/or a designated folder with  `.pal` files (maybe even `.hex` files).

- [x] **Custom Graphics Pipeline:** Transition from Macroquad to OpenGL/Glow for
    - lower input latency
    - better frame synchronization.
    - Custom CRT/NTSC shaders.
    
- [ ] **WebAssembly (WASM):** Browser-based emulation — play directly without installing anything.


---
## Acknowledgments & References

This project would not have been possible without the incredible emulation community and the educational resources available online. Special thanks to:

- **[Nesdev](https://www.nesdev.org/):** The Nesdev wiki and community are the absolute standard for NES development. Practically all technical knowledge here came from them.
    
- **[javidx9 (OneLoneCoder)](https://www.youtube.com/@javidx9):** His excellent [video series on creating an NES emulator](https://youtube.com/playlist?list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf&si=LfeRL_qWT_3-2GNt) was the starting point and main architectural inspiration.
    
- **[bugzmanov/nes_ebook](https://github.com/bugzmanov/nes_ebook):** The e-book "Writing NES Emulator in Rust" was a fundamental reference. Parts of the CPU implementation and Design Patterns were heavily based on his code to understand Rust's nuances applied to emulation.


---
## Licenses

Distributed under the [**MIT License**](LICENSE). 
Feel free to study, modify, and use this code in your own experiments.
