# NES Emulator 🎮

**A Nintendo Entertainment System (NES) emulator built in Rust.**

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white) ![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)

---
## 📸 Showcase

<p align="center"> <img src="https://github.com/user-attachments/assets/5d8fd168-6763-4f93-9af5-1f9c4a570b58" alt="SMB Gameplay with Debug Info" width="600" /> </p>

---

> [!NOTE] 
> This project was created purely as a hobby, a personal challenge, and an opportunity to practice low-level programming and system architecture in Rust. So for the foreseeable future it is not intended to compete with established emulators but rather serves as a proof of concept and a deep learning experience regarding NES hardware.


---
## 🚀 How to Run

Make sure you have [Rust and Cargo](https://www.rust-lang.org/tools/install) installed on your system.

```bash

# Clone the repository
git clone https://github.com/Polar-404/NES_Emulator.git

# Navigate to the directory
cd NES_Emulator

# Compile and run
cargo run --release
```

## 🎮 Controls

| NES Button      | Primary |   Secondary   |
| :-------------- | :-----: | :-----------: |
| **D-Pad Up**    |   `W`   |  `Up Arrow`   |
| **D-Pad Down**  |   `S`   | `Down Arrow`  |
| **D-Pad Left**  |   `A`   | `Left Arrow`  |
| **D-Pad Right** |   `D`   | `Right Arrow` |
| **A**           |   `J`   |      `Z`      |
| **B**           |   `K`   |      `X`      |
| **Select**      |   `N`   |      `C`      |
| **Start**       |   `M`   |      `V`      |

**System Commands:**
* **Volume Up:** `+`
* **Volume Down:** `-` 
* **Pause/Menu:** `Esc`
* **Change Color Palette:** `.` (Period)
* **Paste ROM Path:** `Ctrl + V`

---
## ✨ Current Features

- Ricoh 2A03 (6502) CPU (including indirect JMP bug support).
    
- PPU (Picture Processing Unit).
    
- Partial APU (Audio Processing Unit).
    
- Controller input implemented.
    
- Supported mappers:
    
    - NROM (Mapper 0)
        
    - MMC1 (Mapper 1)
        

---
## 🛠️ Tech Stack

- **[Rust](https://www.rust-lang.org/):** Main language used for the project.
    
- **[Macroquad](https://macroquad.rs/):** Core library for graphics rendering and input handling.
    
- **[Cpal](https://github.com/RustAudio/cpal):** Library for audio processing and output.
    
- **[Ringbuf](https://crates.io/crates/ringbuf):** Lock-free circular buffer for audio.
    
- **[Arboard](https://crates.io/crates/arboard):** System clipboard access.
    
- **Others:** `image`, `lazy_static`, `sysinfo`.
    

---
## 🚧 Roadmap / To-Do

As this is an ongoing learning project, several areas still need improvement:

- [ ] **User Interface:** Improve the start menu and add more graphics and audio configuration options.
    
- [ ] **Audio (APU):** Implement the DMC (Delta Modulation Channel).
    
- [ ] **Synchronization:** Sync audio with FPS to maintain a more stable frame rate, faithful to the original console.
    
- [ ] **Mappers:** Add support for more mappers (e.g., MMC3) to increase game compatibility.
    
- [ ] **Saves:** Implement Save/Load states functionality.
    
- [ ] **Code Quality:** Refactor and clean up the codebase, and potentially add documentation and internationalization (EN/PT-BR).
    
- [ ] **Accuracy & Testing:** Fix known bugs (e.g., failing test 3 in `official_only.nes`) and run/create more accuracy tests.
    
- [ ] **Unofficial Opcodes:** Implement undocumented 6502 instructions.
    
- [ ] **Scripting:** Implement user script support with Lua.
    
- [ ] **More Palettes:** Implement the ability for the user to insert their own palettes via interface and/or a designated folder with  `.pal` files (maybe even `.hex` files).

---
## 🙏 Acknowledgments & References

This project would not have been possible without the incredible emulation community and the educational resources available online. Special thanks to:

- **[Nesdev](https://www.nesdev.org/):** The Nesdev wiki and community are the absolute standard for NES development. Practically all technical knowledge here came from them.
    
- **[javidx9 (OneLoneCoder)](https://www.youtube.com/@javidx9):** His excellent [video series on creating an NES emulator](https://youtube.com/playlist?list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf&si=LfeRL_qWT_3-2GNt) was the starting point and main architectural inspiration.
    
- **[bugzmanov/nes_ebook](https://github.com/bugzmanov/nes_ebook):** The e-book "Writing NES Emulator in Rust" was a fundamental reference. Parts of the CPU implementation and Design Patterns were heavily based on his code to understand Rust's nuances applied to emulation.

---
## Licenses

Distributed under the [**MIT License**](LICENSE). 
Feel free to study, modify, and use this code in your own experiments.
