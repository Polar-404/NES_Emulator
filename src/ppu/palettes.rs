#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NESColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}


//NES PALETTE:
pub const NTSC_PALETTE: [NESColor; 64] = [
    NESColor { r: 84, g: 84, b: 84 },     NESColor { r: 0, g: 30, b: 116 },      NESColor { r: 8, g: 16, b: 144 },       NESColor { r: 48, g: 0, b: 136 },
    NESColor { r: 68, g: 0, b: 100 },     NESColor { r: 88, g: 0, b: 40 },       NESColor { r: 84, g: 4, b: 0 },         NESColor { r: 68, g: 24, b: 0 },
    NESColor { r: 32, g: 42, b: 0 },      NESColor { r: 0, g: 58, b: 0 },        NESColor { r: 0, g: 64, b: 0 },         NESColor { r: 0, g: 60, b: 0 },
    NESColor { r: 0, g: 50, b: 60 },      NESColor { r: 0, g: 0, b: 0 },         NESColor { r: 0, g: 0, b: 0 },          NESColor { r: 0, g: 0, b: 0 },
    NESColor { r: 152, g: 152, b: 152 },  NESColor { r: 0, g: 80, b: 188 },      NESColor { r: 56, g: 72, b: 240 },      NESColor { r: 104, g: 64, b: 240 },
    NESColor { r: 140, g: 48, b: 224 },   NESColor { r: 160, g: 32, b: 176 },    NESColor { r: 160, g: 32, b: 100 },     NESColor { r: 144, g: 48, b: 32 },
    NESColor { r: 104, g: 64, b: 32 },    NESColor { r: 60, g: 82, b: 0 },       NESColor { r: 0, g: 96, b: 0 },         NESColor { r: 20, g: 100, b: 0 },
    NESColor { r: 48, g: 96, b: 0 },      NESColor { r: 0, g: 84, b: 96 },       NESColor { r: 0, g: 0, b: 0 },          NESColor { r: 0, g: 0, b: 0 },
    NESColor { r: 240, g: 240, b: 240 },  NESColor { r: 124, g: 136, b: 252 },   NESColor { r: 188, g: 188, b: 252 },    NESColor { r: 216, g: 176, b: 252 },
    NESColor { r: 228, g: 160, b: 236 },  NESColor { r: 236, g: 144, b: 228 },   NESColor { r: 236, g: 144, b: 176 },    NESColor { r: 220, g: 160, b: 112 },
    NESColor { r: 196, g: 176, b: 96 },   NESColor { r: 148, g: 192, b: 80 },    NESColor { r: 120, g: 204, b: 80 },     NESColor { r: 88, g: 216, b: 120 },
    NESColor { r: 116, g: 208, b: 196 },  NESColor { r: 160, g: 160, b: 160 },   NESColor { r: 0, g: 0, b: 0 },          NESColor { r: 0, g: 0, b: 0 },
    NESColor { r: 252, g: 252, b: 252 },  NESColor { r: 188, g: 216, b: 252 },   NESColor { r: 224, g: 224, b: 252 },    NESColor { r: 236, g: 236, b: 252 },
    NESColor { r: 248, g: 216, b: 252 },  NESColor { r: 252, g: 204, b: 240 },   NESColor { r: 252, g: 196, b: 224 },    NESColor { r: 244, g: 204, b: 168 },
    NESColor { r: 228, g: 212, b: 148 },  NESColor { r: 204, g: 224, b: 132 },   NESColor { r: 184, g: 232, b: 144 },    NESColor { r: 152, g: 240, b: 180 },
    NESColor { r: 168, g: 236, b: 224 },  NESColor { r: 200, g: 200, b: 200 },   NESColor { r: 0, g: 0, b: 0 },          NESColor { r: 0, g: 0, b: 0 },
];