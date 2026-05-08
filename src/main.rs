use std::fs::File;
use std::io::Write;

use svg::node::element::tag::Type;
use svg::parser::Event;

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

    output.write_all(b"<!-- kv3 encoding:text:version{e21c7f3c-8a33-41c5-9977-a76d3a32aa0d} format:generic:version{7412167c-06e9-4698-aff2-e63eb59037e7} -->\r\n")?;
    output.write_all(b"{\r\n")?;
    output.write_all(b"\tRectangleSets =\r\n")?;
    output.write_all(b"\t[\r\n")?;
    output.write_all(b"\t\t{\r\n")?;
    output.write_all(b"\t\t\tname = \"\"\r\n")?;
    output.write_all(b"\t\t\tproperties = null\r\n")?;
    output.write_all(b"\t\t\trectangles =\r\n")?;
    output.write_all(b"\t\t\t[\r\n")?;
    for rect in &rects {
        output.write_all(b"\t\t\t\t{\r\n")?;
        output.write_all(format!("\t\t\t\t\tmin = [ {}, {} ]\r\n", rect.x, rect.y).as_bytes())?;
        output.write_all(
            format!(
                "\t\t\t\t\tmax = [ {}, {} ]\r\n",
                rect.x + rect.width,
                rect.y + rect.height
            )
            .as_bytes(),
        )?;
        output.write_all(b"\t\t\t\t\tinset = [ 0, 0 ]\r\n")?;
        if let Some(props) = &rect.properties {
            output.write_all(b"\t\t\t\t\tproperties =\r\n")?;
            output.write_all(b"\t\t\t\t\t{\r\n")?;
            output.write_all(
                format!("\t\t\t\t\t\tallowTiling = {}\r\n", props.allow_tiling).as_bytes(),
            )?;
            output.write_all(
                format!("\t\t\t\t\t\tallowRotation = {}\r\n", props.allow_rotation).as_bytes(),
            )?;
            output.write_all(b"\t\t\t\t\t}\r\n")?;
        } else {
            output.write_all(b"\t\t\t\t\tproperties = null\r\n")?;
        }
        output.write_all(b"\t\t\t\t},\r\n")?;
    }
    output.write_all(b"\t\t\t]\r\n")?;
    output.write_all(b"\t\t}\r\n")?;
    output.write_all(b"\t]\r\n")?;
    output.write_all(b"}\r\n")?;

    Ok(())
}
