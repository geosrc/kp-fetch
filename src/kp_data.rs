//! Collection of items to work with geomagnetic activity data published by [GFZ] 
//! 
//! [GFZ]: https://www.gfz-potsdam.de/en/kp-index

#![allow(dead_code)]

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io::{self},
    num::{ParseIntError, ParseFloatError},
    str::FromStr,
};

use chrono::NaiveDateTime;

pub static DEFAULT_URL_GFZ_KP_AP_NOWCAST: &str = r#"http://www-app3.gfz-potsdam.de/kp_index/Kp_ap_nowcast.txt"#;

/// Contains the parsed content of a 'Kp and ap' file
#[derive(Debug, Clone)]
pub struct KpFile {
    pub entries: Vec<Entry>,
    last_final_idx: isize
}

impl KpFile {
    pub fn new() -> KpFile {
        KpFile {
            entries: Vec::new(),
            last_final_idx: -1
        }
    }

    pub fn get_new_entries(&self, previous: &Self) -> &[Entry] {
        // All entries are new
        if previous.entries.len() == 0 {
            return &self.entries[..]
        }

        let date = &previous.entries.last().unwrap().date;
        for idx in 0..&self.entries.len()-1 {
            if &self.entries[idx].date > date {
                return &self.entries[idx..]
            }
        }

        // No entries are new
        &[]
    }
}

impl Display for KpFile {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{} entries, last entry: {:?}", self.entries.len(), &self.entries.last() )
    }
}

impl FromStr for KpFile {
    type Err = ParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {

        let mut file = KpFile::new();
        for mut line in text.lines() {
            line = line.trim();
            if line.starts_with("#") || line.is_empty() {
                continue;
            }
            let entry = line.parse::<Entry>()?;

            if entry.kp < 0.0 || entry.ap < 0 {
                continue;
            }

            if entry.d == 1 {
                file.last_final_idx = file.entries.len() as isize;
            }

            file.entries.push( entry );
        }

        Ok(file)
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub date: NaiveDateTime,
    pub kp: f32,
    pub ap: i8,
    pub d: i8
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Time = {}, Kp = {}, ap = {}, d = {}", self.date, self.kp, self.ap, self.d)
    }
}

impl FromStr for Entry {
    type Err = ParseError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        if parts.len() != 10 {
            return Err(ParseError::Error("Line with invalid number of elements"))
        }

        let hp: f32 = parts[4].parse()?;
        let h = hp.floor() as u32;
        let m = ((hp - h as f32) * 60.0) as u32;
        let date =
            chrono::NaiveDate::from_ymd(parts[0].parse()?, parts[1].parse()?, parts[2].parse()?)
                .and_hms(h, m, 0);

        Ok(Entry {
            date: date,
            kp: parts[7].parse()?,
            ap: parts[8].parse()?,
            d: parts[9].parse()?
        })
    }
}

#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    Error(&'static str),
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError)
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::Error(err) => write!(f, "{}", err),
            ParseError::ParseInt(int_err) => write!(f, "{}", int_err),
            ParseError::ParseFloat(float_err) => write!(f, "{}", float_err),
            ParseError::Io(io_err) => write!(f, "{}", io_err),
        }
    }
}

impl Error for ParseError {
}

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> Self {
        ParseError::Io(error)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(error: ParseIntError) -> Self {
        ParseError::ParseInt(error)
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(error: ParseFloatError) -> Self {
        ParseError::ParseFloat(error)
    }
}

impl From<&'static str> for ParseError {
    fn from(error: &'static str) -> Self {
        ParseError::Error(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike};

    const LINE: &str = r#"2022 07 31 18.0 19.50 33084.75000 33084.81250  2.000    7 1"#;
    
    static FILE_ONE: &str = r#"
    # Foo bar
    2022 07 31 18.0 19.50 33084.75000 33084.81250  2.000    7 1
    2022 07 31 21.0 22.50 33084.87500 33084.93750  3.000   15 1
    2022 08 01 00.0 01.50 33085.00000 33085.06250  2.667   12 0
    2022 08 01 03.0 04.50 33085.12500 33085.18750  2.333    9 0
    2022 08 18 03.0 04.50 33102.12500 33102.18750  2.333    9 0
    2022 08 18 06.0 07.50 33102.25000 33102.31250  3.000   15 0
    2022 08 18 09.0 10.50 33102.37500 33102.43750 -1.000   -1 0
    2022 08 18 12.0 13.50 33102.50000 33102.56250 -1.000   -1 0
    "#;

    static FILE_TWO: &str = r#"
    2022 07 31 18.0 19.50 33084.75000 33084.81250  2.000    7 1
    2022 07 31 21.0 22.50 33084.87500 33084.93750  3.000   15 1
    2022 08 01 00.0 01.50 33085.00000 33085.06250  2.667   12 0
    2022 08 01 03.0 04.50 33085.12500 33085.18750  2.333    9 0
    2022 08 18 00.0 01.50 33102.00000 33102.06250  2.667   12 0
    2022 08 18 03.0 04.50 33102.12500 33102.18750  2.333    9 0
    2022 08 18 06.0 07.50 33102.25000 33102.31250  3.333   18 0
    2022 08 18 09.0 10.50 33102.37500 33102.43750  2.667   12 0
    2022 08 18 12.0 13.50 33102.50000 33102.56250  5.000   48 0
    2022 08 27 12.0 13.50 33111.50000 33111.56250 -1.000   -1 0
    "#;

    #[test]
    fn test_parse_entry() {
        let entry = LINE.parse::<Entry>();
        //assert!(entry.is_ok());
        let entry = entry.unwrap();

        let date = chrono::NaiveDate::from_ymd(2022, 7, 31).and_hms(19, 30, 0);

        assert_eq!(entry.kp, 2.0);
        assert_eq!(entry.ap, 7);
        assert_eq!(entry.d, 1);
        assert_eq!(entry.date, date);
    }

    #[test]
    fn test_parse_entry_failure() {
        let entry = "Foobar".parse::<Entry>();
        assert!(entry.is_err());
    }

    #[test]
    fn test_parse_file() {
        let kp_file = FILE_ONE.parse::<KpFile>().unwrap();

        assert_eq!(kp_file.entries.len(), 6);
        assert_eq!(kp_file.last_final_idx, 1);

        let last = &kp_file.entries[kp_file.last_final_idx as usize]; 
        assert_eq!(last.date.date().month(), 7);
        assert_eq!(last.date.date().day(), 31);
    }

    #[test]
    fn test_get_new_entries() {
        let kp_empty = KpFile::new();
        let kp_ref = FILE_ONE.parse::<KpFile>().unwrap();
        let kp_up = FILE_TWO.parse::<KpFile>().unwrap();

        let new = kp_ref.get_new_entries(&kp_empty);
        assert_eq!(new.len(), 6);

        let new = kp_ref.get_new_entries(&kp_up);
        assert_eq!(new.len(), 0);

        let new = kp_up.get_new_entries(&kp_ref);
        assert_eq!(new.len(), 2);
        //     2022 08 18 09.0 10.50 33102.37500 33102.43750  2.667   12 0
        assert_eq!(new.first().unwrap().date, NaiveDateTime::parse_from_str("2022-08-18 10:30:00", "%Y-%m-%d %H:%M:%S").unwrap() );
    }
}