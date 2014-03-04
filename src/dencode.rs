use Encodable;
use collections::TreeMap;

macro_rules! try( ($e:expr) => (
    match $e { Ok(e) => e, Err(e) => { self.error = Err(e); return } }
) )


pub struct Encoder {
    priv wr: &'a mut io::Writer,
    priv error: io::IoResult<()>,
}

impl<'a> Encoder<'a> {
    pub fn new<'a>(wr: &<'a mut io::writer) -> Encode<'a> {
        Encoder { wr: wr, error: Ok(()) }
    }

    pub fn buffer_encode<T:Encodable<Encoder<'a>>>(to_encode_object: &T) -> ~[u8]  {
       //Serialize the object in a string using a writer
        let mut m = MemWriter::new();
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


impl<'a> ::Encoder for Encoder<'a> {
    // TODO: cannot marshal None unless allow_none is enabled
    fn emit_nil(&mut self) { try!(write!(self.wr, "<nil/>")) }

    fn emit_uint(&mut self, v: uint) { self.emit_u64(v as i32); }
    fn emit_u64(&mut self, v: u64) { self.emit_f64(v as i32); }
    fn emit_u32(&mut self, v: u32) { self.emit_f64(v as i32); }
    fn emit_u16(&mut self, v: u16) { self.emit_f64(v as i32); }
    fn emit_u8(&mut self, v: u8)   { self.emit_f64(v as i32); }

    fn emit_i64(&mut self, v: i64) { self.emit_f64(v as i32); }
    fn emit_int(&mut self, v: int) { self.emit_f64(v as i32); }
    fn emit_i32(&mut self, v: i32) {
        if (v > MAXINT || v < MININT) {
        }
        try!(write!(self.wr, "<value><int>{}</int></value>"));
    }
    fn emit_i16(&mut self, v: i16) { self.emit_f64(v as i32); }
    fn emit_i8(&mut self, v: i8)   { self.emit_f64(v as i32); }

    fn emit_bool(&mut self, v: bool) {
        let b = if v { 1} else { 0 };
        try!(write!(self.wr, "<value><boolean>{}</boolean></value>", b));
    }

    fn emit_f64(&mut self, v: f64) {
        try!(write!(self.wr,
                    "<value><double>{}</double></value>",
                    f64::to_str_digits(v, 6u)))
    }
    fn emit_f32(&mut self, v: f32) { self.emit_f64(v as f64); }

    fn emit_char(&mut self, v: char) { self.emit_str(str::from_char(v)) }
    fn emit_str(&mut self, v: &str) {
        try!(write!(self.wr, "<value><string>{}</value></string>", escape_str(v)))
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
            try!(write!(self.wr, "{}", escape_str(name)));
        } else {
            try!(write!(self.wr, "\\{\"variant\":"));
            try!(write!(self.wr, "{}", escape_str(name)));
            try!(write!(self.wr, ",\"fields\":["));
            f(self);
            try!(write!(self.wr, "]\\}"));
        }
    }

    fn emit_enum_variant_arg(&mut self, idx: uint, f: |&mut Encoder<'a>|) {
        if idx != 0 {
            try!(write!(self.wr, ","));
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
        try!(write!(self.wr, r"<value><struct>"));
        f(self);
        try!(write!(self.wr, r"</struct></value>"));
    }

    fn emit_struct_field(&mut self,
                         name: &str,
                         idx: uint,
                         f: |&mut Encoder<'a>|) {
        if idx != 0 { try!(write!(self.wr, ",")) }
        try!(write!(self.wr, "<member><name>{}</name>", escape_str(name)));
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
        try!(write!(self.wr, "["));
        f(self);
        try!(write!(self.wr, "]"));
    }

    fn emit_seq_elt(&mut self, idx: uint, f: |&mut Encoder<'a>|) {
        if idx != 0 {
            try!(write!(self.wr, ","));
        }
        f(self)
    }

    fn emit_map(&mut self, _len: uint, f: |&mut Encoder<'a>|) {
        try!(write!(self.wr, r"\{"));
        f(self);
        try!(write!(self.wr, r"\}"));
    }

    fn emit_map_elt_key(&mut self, idx: uint, f: |&mut Encoder<'a>|) {
        if idx != 0 { try!(write!(self.wr, ",")) }
        f(self)
    }

    fn emit_map_elt_val(&mut self, _idx: uint, f: |&mut Encoder<'a>|) {
        try!(write!(self.wr, ":"));
        f(self)
    }
}
