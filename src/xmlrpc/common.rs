use collections::TreeMap;
use serialize;
use serialize::{Encodable};

use std::io;
use std::str;

macro_rules! try( ($e:expr) => (
    match $e { Ok(e) => e, Err(e) => { self.error = Err(e); return } }
) )

#[deriving(Clone, Eq)]
pub enum Value {
    Boolean(bool),
    Int(i32),
    Double(f32),
    String(~str),
    DateTime(~str),
    Base64(BinaryType),
    Array(ArrayType),
    Struct(~StructType),
    Nil,
}

pub type BinaryType = ~[u8];
pub type StructType = TreeMap<~str, Value>;
pub type ArrayType = ~[Value];


pub trait ToValue {
    fn to_value(&self) -> Value;
}


impl ToValue for int { fn to_value(&self) -> Value { Int(*self as i32) } }
impl ToValue for i8  { fn to_value(&self) -> Value { Int(*self as i32) } }
impl ToValue for i16 { fn to_value(&self) -> Value { Int(*self as i32) } }
impl ToValue for i32 { fn to_value(&self) -> Value { Int(*self as i32) } }
impl ToValue for i64 { fn to_value(&self) -> Value { Int(*self as i32) } }
impl ToValue for u8  { fn to_value(&self) -> Value { Int(*self as i32) } }
impl ToValue for u16 { fn to_value(&self) -> Value { Int(*self as i32) } }
impl ToValue for u32 { fn to_value(&self) -> Value { Int(*self as i32) } }
impl ToValue for u64 { fn to_value(&self) -> Value { Int(*self as i32) } }

impl ToValue for f32 { fn to_value(&self) -> Value { Double(*self as f32) } }
impl ToValue for f64 { fn to_value(&self) -> Value { Double(*self as f32) } }
impl ToValue for ~str { fn to_value(&self) -> Value { String((*self).clone()) } }

impl<A:ToValue> ToValue for ~[A] {
    fn to_value(&self) -> Value { Array(self.map(|e| e.to_value())) }
}

impl<A:ToValue> ToValue for TreeMap<~str, A> {
    fn to_value(&self) -> Value {
        let mut d = TreeMap::new();
        for (key, value) in self.iter() {
            d.insert((*key).clone(), value.to_value());
        }
        Struct(~d)
    }
}


pub struct Encoder<'a> {
    priv wr: &'a mut io::Writer,
    priv error: io::IoResult<()>,
}


impl<'a> Encoder<'a> {
    pub fn new<'a>(wr: &'a mut io::Writer) -> Encoder<'a> {
        Encoder { wr: wr, error: Ok(()) }
    }

    pub fn buffer_encode<T:Encodable<Encoder<'a>>>(to_encode_object: &T) -> ~[u8]  {
       //Serialize the object in a string using a writer
        let mut m = io::MemWriter::new();
        {
            let mut encoder = Encoder::new(&mut m as &mut io::Writer);
            to_encode_object.encode(&mut encoder);
        }
        m.unwrap()
    }

    pub fn str_encode<T:Encodable<Encoder<'a>>>(to_encode_object: &T) -> ~str {
        let buff:~[u8] = Encoder::buffer_encode(to_encode_object);
        str::from_utf8_owned(buff).unwrap()
    }
}


impl<E: serialize::Encoder> Encodable<E> for Value {
    fn encode(&self, e: &mut E) {
        match *self {
            Boolean(v)      => v.encode(e),
            Int(v)          => v.encode(e),
            Double(v)       => v.encode(e),
            String(ref v)   => v.encode(e),
            DateTime(ref v) => v.encode(e),
            Base64(ref v)   => v.encode(e),
            Array(ref v)    => v.encode(e),
            Struct(ref v)   => v.encode(e),
            Nil             => e.emit_nil(),
        }
    }
}

macro_rules! assert_within_range(
    ($var:ident) => (
        if $var >> 32 != 0 {
            fail!("assertion failed: `{:?} exceeds range`", $var)
        }
    );
)

impl<'a> serialize::Encoder for Encoder<'a> {
    fn emit_nil(&mut self) { try!(write!(self.wr, "<nil/>")) }

    fn emit_uint(&mut self, v: uint) {
        assert_within_range!(v);
        self.emit_i32(v as i32);
    }
    fn emit_u64(&mut self, v: u64) {
        assert_within_range!(v);
        self.emit_i32(v as i32);
    }
    fn emit_u32(&mut self, v: u32) {
        assert_within_range!(v);
        self.emit_i32(v as i32);
    }
    fn emit_u16(&mut self, v: u16) { self.emit_i32(v as i32); }
    fn emit_u8(&mut self, v: u8)   { self.emit_i32(v as i32); }

    fn emit_i64(&mut self, v: i64) {
        assert_within_range!(v);
        self.emit_i32(v as i32);
    }
    fn emit_int(&mut self, v: int) {
        assert_within_range!(v);
        self.emit_i32(v as i32);
    }
    fn emit_i32(&mut self, v: i32) {
        try!(write!(self.wr, "<value><int>{}</int></value>", v));
    }
    fn emit_i16(&mut self, v: i16) {
        self.emit_i32(v as i32);
    }
    fn emit_i8(&mut self, v: i8)   {
        self.emit_i32(v as i32);
    }

    fn emit_bool(&mut self, v: bool) {
        let b = if v { 1 } else { 0 };
        try!(write!(self.wr, "<value><boolean>{}</boolean></value>", b));
    }

    fn emit_f64(&mut self, v: f64) {
        try!(write!(self.wr, "<value><double>{}</double></value>",v))
    }
    fn emit_f32(&mut self, v: f32) { self.emit_f64(v as f64); }

    fn emit_char(&mut self, v: char) { self.emit_str(str::from_char(v)) }
    fn emit_str(&mut self, v: &str) {
        try!(write!(self.wr, "<value><string>{}</string></value>", v))
    }

    fn emit_enum(&mut self, _name: &str, f: |&mut Encoder<'a>|) { f(self) }

    fn emit_enum_variant(&mut self,
                         name: &str,
                         _id: uint,
                         cnt: uint,
                         f: |&mut Encoder<'a>|) {
        // enums are encoded as strings or objects
        // Bunny => "Bunny"
        // Kangaroo(34,"William") => {"variant": "Kangaroo", "fields": [34,"William"]}
        if cnt == 0 {
            try!(write!(self.wr, "<value><string>{}</string></value>", name));
        } else {
            try!(write!(self.wr, "<value><struct><member>"));
            try!(write!(self.wr, "<name>variant</name>"));
            try!(write!(self.wr, "<value>{}</value></member>", name));
            try!(write!(self.wr, "<member><name>fields</name>"));
            try!(write!(self.wr, "<value><array><data>"));
            f(self);
            try!(write!(self.wr, "</data></array></value>"));
            try!(write!(self.wr, "</member></struct>"));
        }
    }

    fn emit_enum_variant_arg(&mut self, idx: uint, f: |&mut Encoder<'a>|) {
        if idx != 0 {
            try!(write!(self.wr, "\n"));
        }
        f(self);
    }

    fn emit_enum_struct_variant(&mut self,
                                name: &str,
                                id: uint,
                                cnt: uint,
                                f: |&mut Encoder<'a>|) {
        self.emit_enum_variant(name, id, cnt, f)
    }

    fn emit_enum_struct_variant_field(&mut self,
                                      _: &str,
                                      idx: uint,
                                      f: |&mut Encoder<'a>|) {
        self.emit_enum_variant_arg(idx, f)
    }

    fn emit_struct(&mut self, _: &str, _: uint, f: |&mut Encoder<'a>|) {
        try!(write!(self.wr, "<value><struct>"));
        f(self);
        try!(write!(self.wr, "</struct></value>"));
    }

    fn emit_struct_field(&mut self,
                         name: &str,
                         idx: uint,
                         f: |&mut Encoder<'a>|) {
        if idx != 0 { try!(write!(self.wr, ",")) }
        try!(write!(self.wr, "<member><name>{}</name>", name));
        f(self);
        try!(write!(self.wr, "</member>"));
    }

    fn emit_tuple(&mut self, len: uint, f: |&mut Encoder<'a>|) {
        self.emit_seq(len, f)
    }
    fn emit_tuple_arg(&mut self, idx: uint, f: |&mut Encoder<'a>|) {
        self.emit_seq_elt(idx, f)
    }

    fn emit_tuple_struct(&mut self,
                         _name: &str,
                         len: uint,
                         f: |&mut Encoder<'a>|) {
        self.emit_seq(len, f)
    }
    fn emit_tuple_struct_arg(&mut self, idx: uint, f: |&mut Encoder<'a>|) {
        self.emit_seq_elt(idx, f)
    }

    fn emit_option(&mut self, f: |&mut Encoder<'a>|) { f(self); }
    fn emit_option_none(&mut self) { self.emit_nil(); }
    fn emit_option_some(&mut self, f: |&mut Encoder<'a>|) { f(self); }

    fn emit_seq(&mut self, _len: uint, f: |&mut Encoder<'a>|) {
        try!(write!(self.wr, "<value><array>\n<data>\n"));
        f(self);
        try!(write!(self.wr, "</data>\n</array></value>"));
    }

    fn emit_seq_elt(&mut self, _idx: uint, f: |&mut Encoder<'a>|) {
        f(self);
        try!(write!(self.wr, "\n"))
    }

    fn emit_map(&mut self, _len: uint, f: |&mut Encoder<'a>|) {
        try!(write!(self.wr, "<value><struct>"));
        f(self);
        try!(write!(self.wr, "</struct></value>"));
    }

    fn emit_map_elt_key(&mut self, _idx: uint, f: |&mut Encoder<'a>|) {
        try!(write!(self.wr, "<member><name>"));
        f(self);
        try!(write!(self.wr, "</name>"))
    }

    fn emit_map_elt_val(&mut self, _idx: uint, f: |&mut Encoder<'a>|) {
        f(self);
        try!(write!(self.wr, "</member>"))
    }
}

