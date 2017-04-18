use std::borrow::Borrow;
use std::collections::HashMap;
use std::iter::Peekable;
use std::str;
use std::hash::Hash;
use std::hash::Hasher;
use std::fmt::Formatter;
use std::fmt::Error;
use std::fmt::Debug;

#[derive(PartialEq, Eq)]
pub struct ByteString(pub Vec<u8>);

impl Debug for ByteString {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let ByteString(ref s) = *self;
        let x : &str = unsafe { str::from_utf8_unchecked(s.as_slice()) };
        Debug::fmt(x, f)
    }
}

impl Hash for ByteString {
    fn hash<H>(&self, state: &mut H)
        where H: Hasher
    {
        let ByteString(ref s) = *self;
        unsafe { str::from_utf8_unchecked(s.as_slice()).hash(state) }
    }
}

impl Borrow<str> for ByteString {
    fn borrow(&self) -> &str {
        let ByteString(ref s) = *self;
        unsafe { str::from_utf8_unchecked(s.as_slice()) }
    }
}

#[derive(PartialEq, Debug)]
pub enum BenObject {
    I(i64),
    S(ByteString),
    L(Vec<BenObject>),
    D(HashMap<ByteString, BenObject>)
}

impl BenObject {

    pub fn as_int(&self) -> Option<i64> {
        match *self {
            BenObject::I(x) => Some(x),
            _ => None
        }
    }

    pub fn as_str(&self) -> Option<&ByteString> {
        match *self {
            BenObject::S(ref x) => Some(x),
            _ => None
        }
    }

    pub fn as_list(&self) -> Option<&Vec<BenObject>> {
        match *self {
            BenObject::L(ref x) => Some(x),
            _ => None
        }
    }

    pub fn as_dict(&self) -> Option<&HashMap<ByteString, BenObject>> {
        match *self {
            BenObject::D(ref x) => Some(x),
            _ => None
        }
    }

    pub fn decode<I>(bytes: &mut I) -> Result<Self, String>
        where I: Iterator<Item=u8>
    {
        Self::decode_benobject(&mut bytes.peekable())
    }

    fn decode_benobject<I>(bytes: &mut Peekable<I>) -> Result<Self, String>
        where I: Iterator<Item=u8>
    {
        match bytes.peek() {
            Some(&c) => match c as char {
                'd' => Self::decode_bendict(bytes),
                'i' => Self::decode_benint(bytes),
                'l' => Self::decode_benlist(bytes),
                _ => Self::decode_benstring(bytes)
            },
            None => Err("BenObject not found".to_string())
        }
    }

    fn decode_bendict<I>(bytes: &mut Peekable<I>) -> Result<Self, String>
        where I: Iterator<Item=u8>
    {
        assert_eq!(bytes.next().unwrap(), 'd' as u8);
        let mut hash = HashMap::new();
        while bytes.peek() != Some(&('e' as u8)) {
            let benstr = Self::decode_benstring(bytes);
            if benstr.is_err() {
                return benstr
            }
            let key = match benstr.unwrap() {
                BenObject::S(k) => k,
                _ => panic!("unexpected  type")
            };
            let benobj = Self::decode_benobject(bytes);
            if benobj.is_err() {
                return benobj
            }
            hash.insert(key, benobj.unwrap());
        }
        if Self::skip_if_match(bytes, 'e') {
            Ok(BenObject::D(hash))
        }
        else {
            Err("parsing dict failed: expected 'e'".to_string())
        }
    }

    fn decode_benint<I>(bytes: &mut Peekable<I>) -> Result<Self, String>
        where I: Iterator<Item=u8>
    {
        assert_eq!(bytes.next().unwrap(), 'i' as u8);
        let sign = if Self::skip_if_match(bytes, '-') { -1 } else { 1 };
        let val = sign * Self::decode_uint(bytes) as i64;
        if Self::skip_if_match(bytes, 'e') {
            Ok(BenObject::I(val))
        } else {
            Err("parsing integer failed: expected 'e'".to_string())
        }
    }

    fn decode_benlist<I>(bytes: &mut Peekable<I>) -> Result<Self, String>
        where I: Iterator<Item=u8>
    {
        assert_eq!(bytes.next().unwrap(), 'l' as u8);
        let mut vec = Vec::new();
        while bytes.peek() != Some(&('e' as u8)) {
            let benobj = Self::decode_benobject(bytes);
            if benobj.is_err() {
                return benobj
            }
            vec.push(benobj.unwrap())
        }
        if Self::skip_if_match(bytes, 'e') {
            Ok(BenObject::L(vec))
        }
        else {
            Err("parsing list failed: expected 'e'".to_string())
        }
    }

    fn decode_benstring<I>(bytes: &mut Peekable<I>) -> Result<Self, String>
        where I: Iterator<Item=u8>
    {
        let len = Self::decode_uint(bytes) as usize;
        if !Self::skip_if_match(bytes, ':') {
            return Err("parsing string failed: expected ':'".to_string())
        }
        let buf = bytes.by_ref().take(len).collect::<Vec<_>>();
        if buf.len() == len {
            Ok(BenObject::S(ByteString(buf)))
        }
        else {
            Err("parsing string failed: length mismatches".to_string())
        }
    }

    fn skip_if_match<I>(bytes: &mut Peekable<I>, ch: char) -> bool
        where I: Iterator<Item=u8>
    {
        if bytes.peek() == Some(&(ch as u8)) {
            bytes.next();
            true
        } else {
            false
        }
    }

    fn decode_uint<I>(bytes: &mut Peekable<I>) -> u64
        where I: Iterator<Item=u8>
    {
        let mut num = 0;
        while bytes.peek().map_or(false, |c| (*c as char).is_digit(10)) {
            num *= 10;
            num += (bytes.next().unwrap() - '0' as u8) as u64
        }
        num
    }

}
