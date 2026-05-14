use std::fs::{self, File};

use indexmap::indexmap;
use svg::node::element::tag::Type;
use svg::parser::Event;
use valve_key_value::{KvFormat, KvObject};

#[derive(Debug, Clone)]
struct RectProperties {
    allow_tiling: bool,
    allow_rotation: bool,
}

#[derive(Debug, Clone)]
struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    properties: Option<RectProperties>,
}

fn f2size(size: f64, v: f64) -> u16 {
    (v / size * 32768.0/* 2^15 */) as u16
}

fn parse_svg(file_str: &str) -> Option<Vec<Rect>> {
    let mut rects: Vec<Rect> = vec![];

    let mut width: Option<f64> = None;
    let mut height: Option<f64> = None;

    for e in svg::read(file_str).expect("not a valid svg") {
        match e {
            Event::Tag(path, node_type, attribs) => match path {
                "svg" if node_type == Type::Start => {
                    width = Some(attribs.get("width")?.parse::<f64>().ok()?);
                    height = Some(attribs.get("height")?.parse::<f64>().ok()?);
                }
                "rect" if node_type == Type::Start => {
                    if let Some(width) = width
                        && let Some(height) = height
                    {
                        let x = f2size(width, attribs.get("x")?.parse::<f64>().ok()?);
                        let y = f2size(height, attribs.get("y")?.parse::<f64>().ok()?);
                        let w = f2size(width, attribs.get("width")?.parse::<f64>().ok()?);
                        let h = f2size(height, attribs.get("height")?.parse::<f64>().ok()?);
                        rects.push(Rect {
                            x,
                            y,
                            width: w,
                            height: h,
                            properties: Some(RectProperties {
                                allow_tiling: false,
                                allow_rotation: true,
                            }),
                        });
                    } else {
                        return None; // Missing header
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }

    Some(rects)
}

fn main() -> std::io::Result<()> {
    let input_path = std::env::args().nth(1).expect("No argument provided");

    let mut output = File::create("output.rect")?;

    let file_str = fs::read_to_string(input_path)?;
    let rects = parse_svg(&file_str).expect("failed to parse SVG file");

    let kv_rects = rects
        .iter()
        .map(|rect| {
            let properties: KvObject;

            if let Some(props) = &rect.properties {
                properties = KvObject::Map(indexmap! {
                    "allowTiling".to_string() => KvObject::Bool(props.allow_tiling),
                    "allowRotation".to_string() => KvObject::Bool(props.allow_rotation),
                });
            } else {
                properties = KvObject::Null;
            }

            KvObject::Map(indexmap! {
                "min".to_string() => KvObject::Array(vec![
                    KvObject::Int(rect.x.into()),
                    KvObject::Int(rect.y.into()),
                ]),
                "max".to_string() => KvObject::Array(vec![
                    KvObject::Int((rect.x + rect.width).into()),
                    KvObject::Int((rect.y + rect.height).into()),
                ]),
                "inset".to_string() => KvObject::Array(vec![
                    KvObject::Int(0),
                    KvObject::Int(0)
                ]),
               "properties".to_string() => properties,
            })
        })
        .collect::<Vec<_>>();

    let kv = KvObject::Map(indexmap! {
        "RectangleSets".to_string() => KvObject::Array(vec![
            KvObject::Map(indexmap!{
                "name".to_string() => KvObject::String("".to_string()),
                "properties".to_string() => KvObject::Null,
                "rectangles".to_string() => KvObject::Array(kv_rects),
            })
        ],
    )});

    kv.serialize(&mut output, KvFormat::Kv3Text, None)
}
