use crossterm::style::Color;
use phf::phf_map;
use syntect::highlighting::Color as SyntectColor;


pub static ICONS: phf::Map<&'static str, &'static str> = phf_map! {
    "py" => " ",
    "rs" => " ",
    "js" => " ",
    "ts" => "󰛦 ",
    "svelte" => " ",
    "c" => " ",
    "h" => " ",
    "cpp" => " ",
    "hpp" => " ",
    "zig" => " ",
    "go" => "󰟓 ",
    "pdf" => " ",
    "json" => " ",
    "toml" => " ",
    "wav" => "󰎈 ",
    "mp3" => "󰎈 ",
    "flaac" => "󰎈 ",
    "folder" => "󰉋 ",
    "gitignore" => " ",
    "git" => " ",
    "github" => " "
};

pub const LIGHT_CONTRAST: Color = Color::Rgb {
    r: 0x56,
    g: 0x5f,
    b: 0x89,
};

pub const ERR: Color = Color::Rgb {
    r: 0xf7,
    g: 0x76,
    b: 0x8e,
};

pub const DEFAULT: Color = Color::Rgb {
    r: 0xcf,
    g: 0xc9,
    b: 0xc2,
};

pub const BACKGROUND_COLOR: SyntectColor = SyntectColor {
    r: 0x1e,
    g: 0x1e,
    b: 0x2e,
    a: 0x00,
};

pub const DIVIDER_COLOR: Color = Color::Rgb {
    r: 0xa6,
    g: 0xe3,
    b: 0xa1
};

pub const FILE_ICON: &str = "󰈔 ";
pub const FOLDER_ICON: &str = " ";

