use core::panic;
use std::io::{BufRead, Write};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Sector {
    pub tiles: Vec<Tile>,
}

impl Sector {
    fn new() -> Self {
        Self { tiles: vec![] }
    }
}

#[derive(Serialize, Deserialize)]
struct Tile {
    pub offset_x: i32,
    pub offset_y: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protection_zone: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_logout: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<Item>>,
}

#[derive(Serialize, Deserialize)]
struct Item {
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chest_quest_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyhole_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub door_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub door_quest_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub door_quest_value: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charges: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_liquid_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool_liquid_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abs_teleport_destination: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responsible: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_expire_time: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saved_expire_time: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reamining_uses: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<Item>>,
}

impl Item {
    fn new(id: i32) -> Self {
        Self {
            id,
            amount: None,
            chest_quest_number: None,
            key_number: None,
            keyhole_number: None,
            level: None,
            door_level: None,
            door_quest_number: None,
            door_quest_value: None,
            charges: None,
            text: None,
            editor: None,
            container_liquid_type: None,
            pool_liquid_type: None,
            abs_teleport_destination: None,
            responsible: None,
            remaining_expire_time: None,
            saved_expire_time: None,
            reamining_uses: None,
            content: None,
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("Invalid args");
    }

    let path = std::path::Path::new(&args[1]);
    let now = std::time::Instant::now();
    if path.is_file() {
        let out_path = path.with_extension("json");
        println!("Saving to {:?}", out_path);
        let sector = parse_sector_file(path)?;
        let serialized = serde_json::to_string_pretty(&sector).unwrap();
        let mut output = std::fs::File::create(out_path).unwrap();
        output.write_all(serialized.as_bytes()).unwrap();
        output.flush().unwrap();
    } else if path.is_dir() {
        for file in path.read_dir()? {
            let file = file?.path();
            if file.is_file() && file.extension().unwrap().eq("sec") {
                let out_path = file.with_extension("json");
                println!("Saving to {:?}", out_path);
                let sector = parse_sector_file(&file)?;
                let serialized = serde_json::to_string_pretty(&sector).unwrap();
                let mut output = std::fs::File::create(out_path).unwrap();
                output.write_all(serialized.as_bytes()).unwrap();
                output.flush().unwrap();
            }
        }
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    Ok(())
}

fn parse_sector_file(path: &std::path::Path) -> std::io::Result<Sector> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(
        encoding_rs_io::DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding_rs::WINDOWS_1252))
            .build(file),
    );

    let mut sector = Sector::new();

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut i = 0;
        while i < line.len() {
            if let Some(tile) = parse_tile(&line, &mut i) {
                sector.tiles.push(tile);
            }
        }
    }

    Ok(sector)
}

fn parse_tile(text: &str, index: &mut usize) -> Option<Tile> {
    loop {
        if let Some(i) = text.chars().nth(*index) {
            if i.is_numeric() {
                break;
            }
        } else {
            return None;
        }
        *index += 1;
    }

    let split = text[*index..].split_once(": ").unwrap();
    *index += split.0.len();

    let offsets: Vec<&str> = split.0.split('-').collect();
    if offsets.len() != 2 {
        println!("Bad split: {:?}", offsets);
        return None;
    }

    let offset_x: i32 = offsets[0].parse().unwrap();
    let offset_y: i32 = offsets[1].parse().unwrap();
    let mut refresh = None;
    let mut no_logout = None;
    let mut protection_zone = None;
    let mut content = None;

    let mut i = 0;
    while i < split.1.len() {
        match split.1.chars().nth(i) {
            Some('R') => {
                refresh = Some(true);
                i += 9;
            }
            Some('N') => {
                no_logout = Some(true);
                i += 10;
            }
            Some('P') => {
                protection_zone = Some(true);
                i += 16;
            }
            Some('C') => {
                content = Some(vec![]);
                i += 8;
                parse_content(&mut content, &split.1, &mut i);
                break;
            }
            _ => {
                break;
            }
        }
    }

    *index += i;

    Some(Tile {
        offset_x,
        offset_y,
        refresh,
        no_logout,
        protection_zone,
        content,
    })
}

fn parse_content(content: &mut Option<Vec<Item>>, text: &str, i: &mut usize) {
    let content = content.as_mut().unwrap();
    while *i < text.len() {
        let x = &text[*i..*i + 1];
        if x.parse::<i32>().is_err() {
            *i += 1;
            continue;
        }

        let id = parse_number(i, text);
        let mut item = Item::new(id);

        while *i < text.len() {
            let x = &text[*i..*i + 1];
            if x == "}" {
                *i += 1;
                break;
            }

            if x == " " {
                *i += 1;
                continue;
            }

            if x == "," {
                *i += 1;
                break;
            }

            if x == "A" {
                let y = &text[*i + 1..*i + 2];
                if y == "m" {
                    *i += 7;
                    let amount = parse_number(i, text);
                    item.amount = Some(amount);
                } else if y == "b" {
                    *i += 23;
                    let dest = parse_number(i, text);
                    item.abs_teleport_destination = Some(dest);
                }
                continue;
            }

            if x == "K" {
                let y = &text[*i + 3..*i + 4];
                if y == "N" {
                    *i += 10;
                    let num = parse_number(i, text);
                    item.key_number = Some(num);
                } else if y == "h" {
                    *i += 14;
                    let num = parse_number(i, text);
                    item.keyhole_number = Some(num);
                }
                continue;
            }

            if x == "D" {
                let y = &text[*i + 4..*i + 5];
                if y == "L" {
                    *i += 10;
                    let level = parse_number(i, text);
                    item.door_level = Some(level);
                } else {
                    let y = &text[*i + 9..*i + 10];
                    if y == "N" {
                        *i += 16;
                        let num = parse_number(i, text);
                        item.door_quest_number = Some(num);
                    } else if y == "V" {
                        *i += 15;
                        let val = parse_number(i, text);
                        item.door_quest_value = Some(val);
                    }
                }
                continue;
            }

            if x == "R" {
                let y = &text[*i + 2..*i + 3];
                if y == "s" {
                    *i += 12;
                    let num = parse_number(i, text);
                    item.responsible = Some(num);
                } else {
                    let y = &text[*i + 9..*i + 10];
                    if y == "E" {
                        *i += 20;
                        let time = parse_number(i, text);
                        item.remaining_expire_time = Some(time);
                    } else if y == "U" {
                        *i += 14;
                        let uses = parse_number(i, text);
                        item.reamining_uses = Some(uses);
                    }
                }
                continue;
            }

            if x == "L" {
                *i += 6;
                let level = parse_number(i, text);
                item.level = Some(level);
                continue;
            }

            if x == "E" {
                *i += 7;
                let s = parse_text(&text[*i..]);
                *i += s.len() + 2;
                item.editor = Some(s);
                continue;
            }

            if x == "S" {
                let y = &text[*i + 1..*i + 2];
                if y == "t" {
                    *i += 7;
                    let s = parse_text(&text[*i..]);
                    *i += s.len() + 2;
                    item.text = Some(s);
                } else if y == "a" {
                    *i += 16;
                    let time = parse_number(i, text);
                    item.saved_expire_time = Some(time);
                }
                continue;
            }

            if x == "P" {
                *i += 15;
                let liquid = parse_number(i, text);
                item.pool_liquid_type = Some(liquid);
                continue;
            }

            if x == "C" {
                let y = &text[*i + 2..*i + 3];
                if y == "a" {
                    *i += 8;
                    let charges = parse_number(i, text);
                    item.charges = Some(charges);
                } else if y == "e" {
                    *i += 17;
                    let num = parse_number(i, text);
                    item.chest_quest_number = Some(num);
                } else if y == "n" {
                    let y = &text[*i + 4..*i + 5];
                    if y == "a" {
                        *i += 20;
                        let liquid = parse_number(i, text);
                        item.container_liquid_type = Some(liquid);
                    } else if y == "e" {
                        *i += 8;
                        item.content = Some(vec![]);
                        parse_content(&mut item.content, text, i);
                    }
                }
                continue;
            }
        }

        content.push(item);

        *i += 1;
    }

    *i += 1;
}

fn parse_text(text: &str) -> String {
    let mut j = 1;
    let mut non_ascii_counter = 0;
    while j < text.char_indices().count() {
        if let Some(y) = text.chars().nth(j) {
            if !y.is_ascii() {
                non_ascii_counter += y.len_utf8() - 1;
            } else if y == '\\' {
                j += 1;
            } else if y == '"' {
                break;
            }
        } else {
            break;
        }
        j += 1;
    }
    text[1..j + non_ascii_counter].to_string()
}

fn parse_number(start_index: &mut usize, text: &str) -> i32 {
    let mut j = *start_index;
    while j < text.len() {
        if let Some(y) = text[j..j + 1].chars().nth(0) {
            if y != '-' && !y.is_numeric() {
                break;
            }
        } else {
            break;
        }

        j += 1;
    }
    let num = text[*start_index..j].parse().unwrap();
    *start_index = j;
    num
}
