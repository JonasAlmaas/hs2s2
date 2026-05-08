mod valve_key_value;

use std::fs::File;

use svg::node::element::tag::Type;
use svg::parser::Event;

use crate::valve_key_value::{KvObject, KvSerilizationFormat};

fn f2size(size: f64, v: f64) -> u16 {
    (v / size * 32768.0/* 2^15 */) as u16
}

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

fn main() -> std::io::Result<()> {
    let input_path = std::env::args().nth(1).expect("No argument provided");

    let mut output = File::create("output.rect")?;

    let mut rects: Vec<Rect> = vec![];
    let mut width: Option<f64> = None;
    let mut height: Option<f64> = None;

    let mut content = String::new();
    for e in svg::open(input_path, &mut content).expect("not a valid svg") {
        match e {
            Event::Tag(path, node_type, attributes) => match path {
                "svg" if node_type == Type::Start => {
                    width = Some(
                        attributes
                            .get("width")
                            .expect("width not found")
                            .parse::<f64>()
                            .expect("invalid value for width"),
                    );
                    height = Some(
                        attributes
                            .get("height")
                            .expect("height not found")
                            .parse::<f64>()
                            .expect("invalid value for height"),
                    );
                }
                "rect" if node_type == Type::Start => {
                    let rect_x = f2size(
                        width.expect(""),
                        attributes.get("x").expect("").parse::<f64>().expect(""),
                    );
                    let rect_y = f2size(
                        height.expect(""),
                        attributes.get("y").expect("").parse::<f64>().expect(""),
                    );
                    let rect_width = f2size(
                        width.expect(""),
                        attributes.get("width").expect("").parse::<f64>().expect(""),
                    );
                    let rect_height = f2size(
                        height.expect(""),
                        attributes
                            .get("height")
                            .expect("")
                            .parse::<f64>()
                            .expect(""),
                    );
                    rects.push(Rect {
                        x: rect_x,
                        y: rect_y,
                        width: rect_width,
                        height: rect_height,
                        properties: None,
                    });
                }
                _ => (),
            },
            _ => (),
        }
    }

    let mut kv_rects: Vec<KvObject> = vec![];

    for rect in &rects {
        let properties: KvObject;

        if let Some(props) = &rect.properties {
            properties = KvObject::Map(vec![
                (
                    "allowRotation".to_string(),
                    KvObject::Bool(props.allow_tiling),
                ),
                (
                    "allowRotation".to_string(),
                    KvObject::Bool(props.allow_rotation),
                ),
            ]);
        } else {
            properties = KvObject::Null;
        }

        kv_rects.push(KvObject::Map(vec![
            (
                "min".to_string(),
                KvObject::Array(vec![
                    KvObject::Int(rect.x.into()),
                    KvObject::Int(rect.y.into()),
                ]),
            ),
            (
                "max".to_string(),
                KvObject::Array(vec![
                    KvObject::Int((rect.x + rect.width).into()),
                    KvObject::Int((rect.y + rect.height).into()),
                ]),
            ),
            (
                "inset".to_string(),
                KvObject::Array(vec![KvObject::Int(0), KvObject::Int(0)]),
            ),
            ("properties".to_string(), properties),
        ]));
    }

    let kv = KvObject::Map(vec![(
        "RectangleSets".to_string(),
        KvObject::Array(vec![KvObject::Map(vec![
            ("name".to_string(), KvObject::String("".to_string())),
            ("properties".to_string(), KvObject::Null),
            ("rectangles".to_string(), KvObject::Array(kv_rects)),
        ])]),
    )]);

    kv.serialize(KvSerilizationFormat::Kv3Text, &mut output)
}
