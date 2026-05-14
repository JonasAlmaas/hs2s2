use std::io::Write;

use indexmap::IndexMap;

use crate::kv3::kv3_text::NEW_LINE;
use crate::{KvMapEntry, KvObject};

fn write_indent(out: &mut impl Write, indent: usize) -> Result<(), std::io::Error> {
    for _ in 0..indent {
        out.write_all(b"\t")?;
    }
    Ok(())
}

fn is_multiline(obj: &KvObject) -> bool {
    match obj {
        KvObject::Null => false,
        KvObject::String(_) => false,
        KvObject::Int(_) => false,
        KvObject::Float(_) => false,
        KvObject::Bool(_) => false,
        KvObject::Map(_) => true,
        KvObject::Array(arr) => {
            arr.len() > 0 && matches!(arr[0], KvObject::Array(_) | KvObject::Map(_))
        }
        KvObject::Bin(_) => todo!(),
    }
}

fn serialize_map(
    out: &mut impl Write,
    map: &IndexMap<String, KvMapEntry>,
    indent: usize,
) -> Result<(), std::io::Error> {
    out.write_all(b"{")?;
    out.write_all(NEW_LINE)?;

    {
        let indent = indent + 1;

        for (k, v) in map {
            write_indent(out, indent)?;
            out.write_fmt(format_args!("{} = ", k))?;
            if let Some(flag) = &v.flag {
                out.write_fmt(format_args!("{}:", flag))?;
            }
            if is_multiline(&v.value) {
                out.write_all(NEW_LINE)?;
                write_indent(out, indent)?;
            }
            serialize_object(out, &v.value, indent)?;
            out.write_all(NEW_LINE)?;
        }
    }

    write_indent(out, indent)?;
    out.write_all(b"}")?;

    Ok(())
}

fn serialize_array(
    out: &mut impl Write,
    arr: &Vec<KvObject>,
    indent: usize,
) -> Result<(), std::io::Error> {
    let multiline = arr.len() > 0 && matches!(arr[0], KvObject::Array(_) | KvObject::Map(_));

    out.write_all(b"[")?;

    {
        let indent = indent + 1;

        if multiline {
            out.write_all(NEW_LINE)?;
            write_indent(out, indent)?;
        } else {
            out.write_all(b" ")?;
        }

        for (ix, v) in arr.iter().enumerate() {
            serialize_object(out, v, indent)?;

            if ix < arr.len() - 1 {
                out.write_all(b",")?;
                if multiline {
                    out.write_all(NEW_LINE)?;
                    write_indent(out, indent)?;
                } else {
                    out.write_all(b" ")?;
                }
            } else if !multiline {
                out.write_all(b" ")?;
            }
        }
    }

    if multiline {
        out.write_all(NEW_LINE)?;
        write_indent(out, indent)?;
    }

    out.write_all(b"]")?;

    Ok(())
}

pub fn serialize_object(
    out: &mut impl Write,
    obj: &KvObject,
    indent: usize,
) -> Result<(), std::io::Error> {
    match obj {
        KvObject::Null => out.write_all(b"null"),
        KvObject::String(v) => out.write_fmt(format_args!("\"{v}\"")),
        KvObject::Int(v) => out.write_all(v.to_string().as_bytes()),
        KvObject::Float(v) => out.write_all(v.to_string().as_bytes()),
        KvObject::Bool(v) => out.write_all(v.to_string().as_bytes()),
        KvObject::Map(map) => serialize_map(out, map, indent),
        KvObject::Array(arr) => serialize_array(out, arr, indent),
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
        serialize_object(&mut buf, &KvObject::Null, 0).unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(str, "null");
    }

    #[test]
    fn test_string() {
        let mut buf = Vec::new();
        serialize_object(&mut buf, &KvObject::String("test string".to_string()), 0).unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(str, "\"test string\"");
    }

    #[test]
    fn test_int() {
        let mut buf = Vec::new();
        serialize_object(&mut buf, &KvObject::Int(1234567890), 0).unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(str, "1234567890");
    }

    #[test]
    fn test_bool_true() {
        let mut buf = Vec::new();
        serialize_object(&mut buf, &KvObject::Bool(true), 0).unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(str, "true");
    }

    #[test]
    fn test_bool_false() {
        let mut buf = Vec::new();
        serialize_object(&mut buf, &KvObject::Bool(false), 0).unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(str, "false");
    }

    #[test]
    fn test_serialize_map() {
        let mut buf = Vec::new();
        serialize_map(
            &mut buf,
            &indexmap! {
               "foo".to_string() => KvMapEntry::new(KvObject::Null),
               "bar".to_string() => KvMapEntry::new(KvObject::Int(1234)),
               "baz".to_string() => KvMapEntry {
                   value: KvObject::String("my string".to_string()),
                   flag: Some("flag1".to_string()),
               },
            },
            0,
        )
        .unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(
            str,
            "{\r
\tfoo = null\r
\tbar = 1234\r
\tbaz = flag1:\"my string\"\r
}"
        );
    }

    #[test]
    fn test_serialize_array() {
        let mut buf = Vec::new();
        serialize_array(
            &mut buf,
            &vec![
                KvObject::Int(1234),
                KvObject::Int(5678),
                KvObject::Int(9012),
            ],
            0,
        )
        .unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(str, "[ 1234, 5678, 9012 ]");
    }

    #[test]
    fn test_serialize_multiline_array() {
        let mut buf = Vec::new();
        serialize_array(
            &mut buf,
            &vec![
                KvObject::Map(indexmap! {
                    "foo".to_string() => KvMapEntry::new(KvObject::Int(1)),
                    "bar".to_string() => KvMapEntry::new(KvObject::Int(2)),
                }),
                KvObject::Map(indexmap! {
                    "foo".to_string() => KvMapEntry::new(KvObject::Int(3)),
                    "bar".to_string() => KvMapEntry::new(KvObject::Int(4)),
                }),
            ],
            0,
        )
        .unwrap();
        let str = String::from_utf8(buf).unwrap();
        assert_eq!(
            str,
            "[\r
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
