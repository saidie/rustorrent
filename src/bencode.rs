use std::collections::HashMap;
use std::iter::Peekable;

#[derive(PartialEq, Debug)]
pub enum BenObject {
    BenInt(i64),
    BenStr(String),
    BenList(Vec<BenObject>),
    BenDict(HashMap<String, BenObject>)
}

use bencode::BenObject::*;

pub fn decode<I>(bytes: &mut I) -> Result<BenObject, String>
    where I: Iterator<Item=u8>
{
    decode_benobject(&mut bytes.peekable())
}

fn decode_benobject<I>(bytes: &mut Peekable<I>) -> Result<BenObject, String>
    where I: Iterator<Item=u8>
{
    match bytes.peek() {
        Some(&c) => match c as char {
            'd' => decode_bendict(bytes),
            'i' => decode_benint(bytes),
            'l' => decode_benlist(bytes),
            _ => decode_benstring(bytes)
        },
        None => Err("BenObject not found".to_string())
    }
}

fn decode_bendict<I>(bytes: &mut Peekable<I>) -> Result<BenObject, String>
    where I: Iterator<Item=u8>
{
    assert_eq!(bytes.next().unwrap(), 'd' as u8);
    let mut hash = HashMap::new();
    while bytes.peek() != Some(&('e' as u8)) {
        let benstr = decode_benstring(bytes);
        if benstr.is_err() {
            return benstr
        }
        let key = match benstr.unwrap() {
            BenStr(k) => k,
            _ => panic!("unexpected  type")
        };
        let benobj = decode_benobject(bytes);
        if benobj.is_err() {
            return benobj
        }
        hash.insert(key, benobj.unwrap());
    }
    if skip_if_match(bytes, 'e') {
        Ok(BenDict(hash))
    }
    else {
        Err("parsing dict failed: expected 'e'".to_string())
    }
}

fn decode_benint<I>(bytes: &mut Peekable<I>) -> Result<BenObject, String>
    where I: Iterator<Item=u8>
{
    assert_eq!(bytes.next().unwrap(), 'i' as u8);
    let sign = if skip_if_match(bytes, '-') { -1 } else { 1 };
    let val = sign * decode_uint(bytes) as i64;
    if skip_if_match(bytes, 'e') {
        Ok(BenInt(val))
    } else {
        Err("parsing integer failed: expected 'e'".to_string())
    }
}

fn decode_benlist<I>(bytes: &mut Peekable<I>) -> Result<BenObject, String>
    where I: Iterator<Item=u8>
{
    assert_eq!(bytes.next().unwrap(), 'l' as u8);
    let mut vec = Vec::new();
    while bytes.peek() != Some(&('e' as u8)) {
        let benobj = decode_benobject(bytes);
        if benobj.is_err() {
            return benobj
        }
        vec.push(benobj.unwrap())
    }
    if skip_if_match(bytes, 'e') {
        Ok(BenList(vec))
    }
    else {
        Err("parsing list failed: expected 'e'".to_string())
    }
}

fn decode_benstring<I>(bytes: &mut Peekable<I>) -> Result<BenObject, String>
    where I: Iterator<Item=u8>
{
    let len = decode_uint(bytes) as usize;
    if !skip_if_match(bytes, ':') {
        return Err("parsing string failed: expected ':'".to_string())
    }
    let buf: Vec<_> = bytes.by_ref().take(len).collect();
    if buf.len() == len {
        Ok(BenStr(String::from_utf8_lossy(buf.as_slice()).into_owned()))
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