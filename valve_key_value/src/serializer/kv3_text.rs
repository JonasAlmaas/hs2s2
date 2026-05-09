use std::io::Write;

use indexmap::IndexMap;

use crate::KvObject;

const NEW_LINE: &[u8] = b"\r\n";

fn write_indent(indent: usize, out: &mut impl Write) -> Result<(), std::io::Error> {
    for _ in 0..indent {
        out.write_all(b"\t")?;
    }
    Ok(())
}

fn serialize_map(
    map: &IndexMap<String, KvObject>,
    indent: usize,
    out: &mut impl Write,
) -> Result<(), std::io::Error> {
    out.write_all(NEW_LINE)?;
    write_indent(indent, out)?;
    out.write_all(b"{")?;
    out.write_all(NEW_LINE)?;

    {
        let indent = indent + 1;

        for (k, v) in map {
            write_indent(indent, out)?;
            out.write_fmt(format_args!("{} =", k))?;
            serialize_object(v, indent, out)?;
            out.write_all(NEW_LINE)?;
        }
    }

    write_indent(indent, out)?;
    out.write_all(b"}")?;

    Ok(())
}

fn serialize_array(
    arr: &Vec<KvObject>,
    indent: usize,
    out: &mut impl Write,
) -> Result<(), std::io::Error> {
    let multiline = arr.len() > 0 && matches!(arr[0], KvObject::Map(_));

    if multiline {
        out.write_all(NEW_LINE)?;
        write_indent(indent, out)?;
    } else {
        out.write_all(b" ")?;
    }

    out.write_all(b"[")?;

    for (ix, v) in arr.iter().enumerate() {
        serialize_object(v, indent + 1, out)?;

        if ix < arr.len() - 1 {
            out.write_all(b",")?;
        } else if !multiline {
            out.write_all(b" ")?;
        }
    }

    if multiline {
        out.write_all(NEW_LINE)?;
        write_indent(indent, out)?;
    }

    out.write_all(b"]")?;

    Ok(())
}

pub fn serialize_object(
    obj: &KvObject,
    indent: usize,
    out: &mut impl Write,
) -> Result<(), std::io::Error> {
    match obj {
        KvObject::Null => out.write_all(b" null"),
        KvObject::String(v) => out.write_fmt(format_args!(" \"{v}\"")),
        KvObject::Int(v) => out.write_fmt(format_args!(" {v}")),
        KvObject::Float(v) => out.write_fmt(format_args!(" {v}")),
        KvObject::Bool(v) => out.write_fmt(format_args!(" {v}")),
        KvObject::Map(map) => serialize_map(map, indent, out),
        KvObject::Array(arr) => serialize_array(arr, indent, out),
        KvObject::Bin(_) => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::indexmap;

    #[test]
    fn test_null() {
        let mut buf = Vec::new();
        serialize_object(&KvObject::Null, 0, &mut buf).unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(str, " null");
    }

    #[test]
    fn test_string() {
        let mut buf = Vec::new();
        serialize_object(&KvObject::String("test string".to_string()), 0, &mut buf).unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(str, " \"test string\"");
    }

    #[test]
    fn test_int() {
        let mut buf = Vec::new();
        serialize_object(&KvObject::Int(1234567890), 0, &mut buf).unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(str, " 1234567890");
    }

    #[test]
    fn test_bool_true() {
        let mut buf = Vec::new();
        serialize_object(&KvObject::Bool(true), 0, &mut buf).unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(str, " true");
    }

    #[test]
    fn test_bool_false() {
        let mut buf = Vec::new();
        serialize_object(&KvObject::Bool(false), 0, &mut buf).unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(str, " false");
    }

    #[test]
    fn test_serialize_map() {
        let mut buf = Vec::new();
        serialize_map(
            &indexmap! {
               "foo".to_string() => KvObject::Null,
               "bar".to_string() => KvObject::Int(1234),
            },
            0,
            &mut buf,
        )
        .unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(
            str,
            "\r
{\r
\tfoo = null\r
\tbar = 1234\r
}"
        );
    }

    #[test]
    fn test_serialize_array() {
        let mut buf = Vec::new();
        serialize_array(
            &vec![
                KvObject::Int(1234),
                KvObject::Int(5678),
                KvObject::Int(9012),
            ],
            0,
            &mut buf,
        )
        .unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(str, " [ 1234, 5678, 9012 ]");
    }

    #[test]
    fn test_serialize_multiline_array() {
        let mut buf = Vec::new();
        serialize_array(
            &vec![
                KvObject::Map(indexmap! {
                    "foo".to_string() => KvObject::Int(1),
                    "bar".to_string() => KvObject::Int(2),
                }),
                KvObject::Map(indexmap! {
                    "foo".to_string() => KvObject::Int(3),
                    "bar".to_string() => KvObject::Int(4),
                }),
            ],
            0,
            &mut buf,
        )
        .unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(
            str,
            "\r
[\r
\t{\r
\t\tfoo = 1\r
\t\tbar = 2\r
\t},\r
\t{\r
\t\tfoo = 3\r
\t\tbar = 4\r
\t}\r
]"
        );
    }
}
