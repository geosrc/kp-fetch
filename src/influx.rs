use std::{
    fmt::{self, Display, Formatter, Write},
    collections::{HashMap, BTreeMap},
    //num::{Integer, Float, Signed, Unsigned}
    //str::FromStr
};
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct Measurement {
    name: String,
    event_time: Option<NaiveDateTime>,
    values: HashMap<String, Value>,
    tags: BTreeMap<String, String>
}

#[allow(dead_code)]
impl Measurement {
    pub fn new(name: &str) -> Self {
        //TODO: empty check of name
        Measurement {
            name: name.to_string(),
            event_time: Option::None,
            values: HashMap::new(),
            tags: BTreeMap::new()
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn set_time(&mut self, time: chrono::NaiveDateTime) -> &mut Self {
        self.event_time = Some(time);
        self
    }

    pub fn add_value(&mut self, key: &str, value: Value, replace: bool) -> &mut Self {
        if replace || !self.values.contains_key(key) {
            self.values.insert(key.trim().to_string(), value);
        }
        self
    }

    pub fn add_tag(&mut self, key: &str, value: &str, replace: bool) -> &mut Self {
        if replace || !self.tags.contains_key(key) {
            self.tags.insert(key.trim().to_string(), value.to_string());
        }
        self
    }

    pub fn to_line_protocol(&self, timestamp_format: TimestampFormat) -> Result<String, fmt::Error> {
        let mut buf = String::new();
        write!(buf, "{}", escape( &self.name, false, true, true, true, false)? )?;

        for (key, value) in &self.tags {
            write!(buf, ",{}={}",
                escape( key, true, true, true, true, false)?,
                escape( value, true, true, true, true, false)?
            )?;
        }

        let mut sep = " ";

        for (key, value) in &self.values {
            write!(buf, "{}{}={}",
                sep,
                escape( key, true, true, true, true, false)?,
                value
            )?;
            sep = ",";
        }

        if let Some(ts) = self.event_time {
            
            match timestamp_format {
                TimestampFormat::S => write!(buf, " {}", ts.timestamp())?,
                TimestampFormat::Ms => write!(buf, " {}", ts.timestamp_millis())?,
                TimestampFormat::Us => write!(buf, " {}", ts.timestamp_micros())?,
                TimestampFormat::Ns => write!(buf, " {}", ts.timestamp_nanos())?,
                TimestampFormat::None => {}
            }
        }

        Ok(buf)
    }

}

impl Display for Measurement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.to_line_protocol(TimestampFormat::Ms)? )
    }
}

#[derive(PartialEq)]
#[allow(dead_code)]
pub enum TimestampFormat {
    None = 0,
    Ms = 1,
    S = 2,
    Us = 3,
    Ns = 4
}

#[derive(Debug)]
pub enum Value {
    Float(f32),
    Double(f64),
    Signed(i128),
    Unsigned(u128),
    String(String),
    True,
    False
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Float(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Double(v)
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Value::Signed(v as i128)
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Value::Signed(v as i128)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Signed(v as i128)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Signed(v as i128)
    }
}

impl From<i128> for Value {
    fn from(v: i128) -> Self {
        Value::Signed(v)
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::Unsigned(v as u128)
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Value::Unsigned(v as u128)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::Unsigned(v as u128)
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::Unsigned(v as u128)
    }
}

impl From<u128> for Value {
    fn from(v: u128) -> Self {
        Value::Unsigned(v)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        match v {
            true => Value::True,
            false => Value::False
        }
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String( v.to_string() )
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Double(dbl) => write!(f, "{}", dbl),
            Value::Signed(s) => write!(f, "{}i", s),
            Value::Unsigned(u) => write!(f, "{}u", u),
            Value::String(st) => write!(f, "{}", escape(st, false, false, false, true, true)? ),
            Value::True => write!(f, "true"),
            Value::False => write!(f, "false")
        }
     }
}

fn escape(value: &str, equal: bool, commas: bool, spaces: bool, double_quotes: bool, str_escape: bool ) -> Result<String, fmt::Error>  {
    let mut res = String::with_capacity(value.len() + 10);
    if str_escape {
        write!(res, r#"""#)?;
    }

    for c in value.chars() {
        match c {
            '\n' => write!(res, r"\n")?,
            '\r' => write!(res, r"\r")?,
            '\t' => write!(res, r"\t")?,
            '=' => if equal { write!(res, r"\=")? },
            ' ' => if spaces { write!(res, r"\ ")? },
            ',' => if commas { write!(res, r"\,")? },
            '"' => if double_quotes { write!(res, r#"\""#)? }
            _ => write!(res, "{}", c)?
        }
    }

    if str_escape {
        write!(res, r#"""#)?;
    }

    Ok(res)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        let v: Value = (60.5).into();
        assert_eq!( "60.5", v.to_string() );

        let v: Value = (8i8).into();
        assert_eq!( "8i", v.to_string() );
        
        let v: Value = (8u8).into();
        assert_eq!( "8u", v.to_string() );

        let v: Value = (10048i64).into();
        assert_eq!( "10048i", v.to_string() );
        
        let v: Value = (10048u64).into();
        assert_eq!( "10048u", v.to_string() );

        let v: Value = true.into();
        assert_eq!( "true", v.to_string() );

        let v: Value = "FooBar".into();
        assert_eq!( r#""FooBar""#, v.to_string() );

        let v: Value = "".into();
        assert_eq!( r#""""#, v.to_string() );
    }

    #[test]
    fn test_measurement_creation() {
        let mut m = Measurement::new("Size");
        m.add_value("First Value", 42.into() , false)
        .add_value("Second Value", "Foobar".into() , false)
        .add_tag("location", "West Center", false)
        .add_tag("machine", "Cluster", false)
        .set_time(NaiveDateTime::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap() );
        
        //println!("M: \"{}\"", m);
        assert_eq!( "Size", m.name() );
    }

}