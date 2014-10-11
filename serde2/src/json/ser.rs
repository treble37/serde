use std::f64;
use std::io::{IoError, MemWriter};
use std::num::{FPNaN, FPInfinite};

use ser;

/// A structure for implementing serialization to JSON.
pub struct Serializer<W> {
    writer: W,
}

impl<W: Writer> Serializer<W> {
    /// Creates a new JSON serializer whose output will be written to the writer
    /// specified.
    #[inline]
    pub fn new(writer: W) -> Serializer<W> {
        Serializer {
            writer: writer,
        }
    }

    /// Unwrap the Writer from the Serializer.
    #[inline]
    pub fn unwrap(self) -> W {
        self.writer
    }
}

impl<W: Writer> ser::Visitor<(), IoError> for Serializer<W> {
    #[inline]
    fn visit_null(&mut self) -> Result<(), IoError> {
        self.writer.write_str("null")
    }

    #[inline]
    fn visit_bool(&mut self, value: bool) -> Result<(), IoError> {
        if value {
            self.writer.write_str("true")
        } else {
            self.writer.write_str("false")
        }
    }

    #[inline]
    fn visit_int(&mut self, value: int) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i8(&mut self, value: i8) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i16(&mut self, value: i16) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i32(&mut self, value: i32) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_i64(&mut self, value: i64) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_uint(&mut self, value: uint) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u8(&mut self, value: u8) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u16(&mut self, value: u16) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u32(&mut self, value: u32) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_u64(&mut self, value: u64) -> Result<(), IoError> {
        write!(self.writer, "{}", value)
    }

    #[inline]
    fn visit_f64(&mut self, value: f64) -> Result<(), IoError> {
        fmt_f64_or_null(&mut self.writer, value)
    }

    #[inline]
    fn visit_char(&mut self, v: char) -> Result<(), IoError> {
        escape_char(&mut self.writer, v)
    }

    #[inline]
    fn visit_str(&mut self, value: &str) -> Result<(), IoError> {
        escape_str(&mut self.writer, value)
    }

    #[inline]
    fn visit_seq<
        V: ser::SeqVisitor<Serializer<W>, (), IoError>
    >(&mut self, mut visitor: V) -> Result<(), IoError> {
        try!(self.writer.write_str("["));

        loop {
            match try!(visitor.visit(self)) {
                Some(()) => { }
                None => { break; }
            }
        }

        self.writer.write_str("]")
    }

    #[inline]
    fn visit_seq_elt<
        T: ser::Serialize<Serializer<W>, (), IoError>
    >(&mut self, first: bool, value: T) -> Result<(), IoError> {
        if !first {
            try!(self.writer.write_str(","));
        }

        value.serialize(self)
    }

    #[inline]
    fn visit_map<
        V: ser::MapVisitor<Serializer<W>, (), IoError>
    >(&mut self, mut visitor: V) -> Result<(), IoError> {
        try!(self.writer.write_str("{{"));

        loop {
            match try!(visitor.visit(self)) {
                Some(()) => { }
                None => { break; }
            }
        }

        self.writer.write_str("}}")
    }

    #[inline]
    fn visit_map_elt<
        K: ser::Serialize<Serializer<W>, (), IoError>,
        V: ser::Serialize<Serializer<W>, (), IoError>
    >(&mut self, first: bool, key: K, value: V) -> Result<(), IoError> {
        if !first {
            try!(self.writer.write_str(","));
        }

        try!(key.serialize(self));
        try!(self.writer.write_str(":"));
        value.serialize(self)
    }
}

#[inline]
pub fn escape_bytes<W: Writer>(wr: &mut W, bytes: &[u8]) -> Result<(), IoError> {
    try!(wr.write_str("\""));

    let mut start = 0;

    for (i, byte) in bytes.iter().enumerate() {
        let escaped = match *byte {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            b'\x08' => "\\b",
            b'\x0c' => "\\f",
            b'\n' => "\\n",
            b'\r' => "\\r",
            b'\t' => "\\t",
            _ => { continue; }
        };

        if start < i {
            try!(wr.write(bytes.slice(start, i)));
        }

        try!(wr.write_str(escaped));

        start = i + 1;
    }

    if start != bytes.len() {
        try!(wr.write(bytes.slice_from(start)));
    }

    wr.write_str("\"")
}

#[inline]
pub fn escape_str<W: Writer>(wr: &mut W, value: &str) -> Result<(), IoError> {
    escape_bytes(wr, value.as_bytes())
}

#[inline]
pub fn escape_char<W: Writer>(wr: &mut W, value: char) -> Result<(), IoError> {
    let mut buf = [0, .. 4];
    value.encode_utf8(buf);
    escape_bytes(wr, buf)
}

fn fmt_f64_or_null<W: Writer>(wr: &mut W, value: f64) -> Result<(), IoError> {
    match value.classify() {
        FPNaN | FPInfinite => wr.write_str("null"),
        _ => wr.write_str(f64::to_str_digits(value, 6).as_slice()),
    }
}

#[inline]
pub fn to_vec<
    T: ser::Serialize<Serializer<MemWriter>, (), IoError>
>(value: &T) -> Result<Vec<u8>, IoError> {
    let writer = MemWriter::with_capacity(1024);
    let mut state = Serializer::new(writer);
    try!(value.serialize(&mut state));
    Ok(state.unwrap().unwrap())
}

#[inline]
pub fn to_string<
    T: ser::Serialize<Serializer<MemWriter>, (), IoError>
>(value: &T) -> Result<Result<String, Vec<u8>>, IoError> {
    let vec = try!(to_vec(value));
    Ok(String::from_utf8(vec))
}