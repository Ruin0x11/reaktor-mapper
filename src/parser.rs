use nom::*;
use std::fmt;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::prelude::*;
use std::io;
use std::fs;
use std::path::PathBuf;
use std::result;

const MAPFILE_HEADER: &'static str = "NIMapFile";

named!(parse_bool<&[u8], bool>, map!(le_u32, |i: u32| i != 0));

pub trait Serialize {
    fn write<W: Write>(&self, wtr: &mut W) -> Result<(), io::Error>;
}

#[derive(Debug)]
pub struct MapFile {
    pub header: Header,
    pub entries: Vec<Entry>,
}

impl MapFile {
    pub fn new(path: &PathBuf) -> Self {
        let entries = fs::read_dir(path).unwrap();
        let mut paths = Vec::new();

        for entry in entries {
            if let Some(p) = path_valid(entry) {
                paths.push(p);
            }
        }

        let header = Header::new(paths.len() as u32);
        let entries: Vec<Entry> = paths.into_iter().take(128).enumerate().map(|(i, path)| {
            Entry::new(path.to_str().unwrap(), i as u32, i as u32, 0, 127, i as u32)
        }).collect();

        MapFile {
            header: header,
            entries: entries,
        }
    }
}

fn path_valid(entry: result::Result<fs::DirEntry, io::Error>) -> Option<PathBuf> {
    if let Ok(dir) = entry {
        if let Ok(metadata) = dir.metadata() {
            if metadata.is_file() {
                match dir.path().extension() {
                    Some(ext) => {
                        return if ext == "wav" || ext == "aif" || ext == "aiff" {
                            Some(dir.path())
                        } else {
                            None
                        }
                    } 
                    _ => return None
                }
            }
        }
    }

    None
}

impl fmt::Display for MapFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Samples: {}\n", self.header.sample_count)?;
        for e in self.entries.iter() {
            write!(f, "{}\n", e)?;
        }
        Ok(())
    }
}


#[derive(Debug)]
pub struct Header {
    version: u32,
    thing: u32,
    zero_c: u32,
    one_1: u32,
    one_2: u32,
    zero_1: u32,
    zero_2: u32,
    sample_count: u32,
}

impl Header {
    pub fn new(count: u32) -> Self {
        Header {
            version: 0x02E4,
            thing: 0x2,
            zero_c: 0xC,
            one_1: 0x1,
            one_2: 0x1,
            zero_1: 0x0,
            zero_2: 0x0,
            sample_count: count,
        }
    }
}

#[derive(Debug)]
pub struct EmbeddedSample {
    b: u32,
    c: u32,
    samplerate: u32,
    e: u32,
    bits: u32,
    f: u32,
    data: Vec<u8>
}

impl fmt::Display for EmbeddedSample {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n")?;
        write!(f, "    b: {}\n", self.b)?;
        write!(f, "    c: {:?}\n", self.c)?;
        write!(f, "    samplerate: {}\n", self.samplerate)?;
        write!(f, "    e: {}\n", self.e)?;
        write!(f, "    bits: {}\n", self.bits)?;
        write!(f, "    f: {}\n", self.f)?;
        write!(f, "    length: {}\n", self.data.len())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Entry {
    one_1: u32,
    is_embedded: bool,
    crc: Option<String>,
    path: String,
    sample: Option<EmbeddedSample>,
    thing_a: u32,
    thing_b: u32,
    low: u32,
    high: u32,
    lvel: u32,
    hvel: u32,
    root: u32,
}

impl Entry {
    pub fn new(path: &str, low: u32, high: u32, lvel: u32, hvel: u32, root: u32) -> Self {
        assert!(low < 128);
        assert!(high < 128);
        assert!(low <= high);
        assert!(lvel < 128);
        assert!(hvel < 128);
        assert!(lvel <= hvel);
        assert!(low <= root && root <= high);
        Entry {
            one_1: 0x1,
            is_embedded: false,
            crc: None,
            path: path.to_string().replace("\\", "/"),
            sample: None,
            thing_a: 0x54,
            thing_b: 0x2,
            low: low,
            high: high,
            lvel: lvel,
            hvel: hvel,
            root: root,
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "=== Entry ===\n")?;
        write!(f, "  Embedded: {}\n", self.is_embedded)?;
        write!(f, "  CRC: {:?}\n", self.crc)?;
        write!(f, "  Path: {}\n", self.path)?;
        if let Some(ref s) = self.sample {
            write!(f, "  Sample: {}\n", s)?;
        }
        write!(f, "L    H    Lvel HVel Root\n")?;
        write!(f, "{}    {}    {}    {}    {}\n ", self.low, self.high, self.lvel, self.hvel, self.root)?;
        Ok(())
    }
}

named!(parse_crc <&[u8], String>, do_parse!(
        thingf: le_u32 >>
        crcsiz: le_u32 >>
        crc: take_str!(crcsiz as usize) >>
        (crc.to_string())
));

named!(parse_header <&[u8], Header>, do_parse!(
    take!(4) >>
        version: le_u32 >>
        tag!(MAPFILE_HEADER) >>
        thing: le_u32 >>
        take!(1) >>
        tag!("mapp") >>
        zero_c: le_u32 >>
        one_1: le_u32 >>
        one_2: le_u32 >>
        zero_1: le_u32 >>
        zero_2: le_u32 >>
        sample_count: le_u32 >>
        (Header {
            version: version,
            thing: thing,
            zero_c: zero_c,
            one_1: one_1,
            one_2: one_2,
            zero_1: zero_1,
            zero_2: zero_2,
            sample_count: sample_count,
        })
));

impl Serialize for Header {
    fn write<W: Write>(&self, wtr: &mut W) -> Result<(), io::Error> {
        wtr.write_u32::<LittleEndian>(0)?;
        wtr.write_u32::<LittleEndian>(self.version)?;
        write!(wtr, "{}", MAPFILE_HEADER)?;
        wtr.write_u32::<LittleEndian>(self.thing)?;
        wtr.write(&[1]);
        write!(wtr, "{}", "mapp")?;
        wtr.write_u32::<LittleEndian>(self.zero_c)?;
        wtr.write_u32::<LittleEndian>(self.one_1)?;
        wtr.write_u32::<LittleEndian>(self.one_2)?;
        wtr.write_u32::<LittleEndian>(self.zero_1)?;
        wtr.write_u32::<LittleEndian>(self.zero_2)?;
        wtr.write_u32::<LittleEndian>(self.sample_count)
    }
}

fn parse_embedded_sample(i: &[u8], size: u32) -> IResult<&[u8], EmbeddedSample> {
    do_parse!(i,
              b: le_u32 >>
              c: le_u32 >>
              samplerate: le_u32 >>
              e: le_u32 >>
              bits: le_u32 >>
              f: le_u32 >>
              data: take!(size as usize - 24) >> //exclude size of above bits
              (EmbeddedSample {
                  b: b,
                  c: c,
                  samplerate: samplerate,
                  e: e,
                  bits: bits,
                  f: f,
                  data: data.to_vec(),
              }))
}

fn parse_content(i: &[u8], is_embedded: bool) -> IResult<&[u8], Option<EmbeddedSample>> {
    do_parse!(i,
              size: le_u32 >>
              sample: cond!(is_embedded, apply!(parse_embedded_sample, size)) >>
              (sample)
    )
}

named!(parse_entry <&[u8], Entry>, do_parse!(
    one_1: le_u32 >>
        is_embedded: parse_bool >>
        crc: cond!(is_embedded, parse_crc) >>
        pathsiz: le_u32 >>
        path: take_str!(pathsiz as usize) >>
        sample: apply!(parse_content, is_embedded) >>
        tag!("entr") >>
        thing_a: le_u32 >> // 84
        thing_b: le_u32 >> // 2
        low:  le_u32 >>
        high: le_u32 >>
        lvel: le_u32 >>
        hvel: le_u32 >>
        root: le_u32 >>
        take!(60) >> // 0
        (Entry {
            one_1: one_1,
            is_embedded: is_embedded,
            crc: crc,
            path: path.to_string(),
            sample: sample,
            thing_a: thing_a,
            thing_b: thing_b,
            low: low,
            high: high,
            lvel: lvel,
            hvel: hvel,
            root: root,
        })
));

fn bool_to_u32(b: bool) -> u32 {
    if b {
        1
    } else {
        0
    }
}

impl Serialize for Entry {
    fn write<W: Write>(&self, wtr: &mut W) -> Result<(), io::Error> {
        if self.is_embedded {
            return Err(io::Error::new(io::ErrorKind::Other, "Embedded files are not supported."))
        }
        wtr.write_u32::<LittleEndian>(1)?;
        wtr.write_u32::<LittleEndian>(bool_to_u32(self.is_embedded))?;
        wtr.write_u32::<LittleEndian>(self.path.len() as u32)?;
        write!(wtr, "{}", self.path)?;
        wtr.write_u32::<LittleEndian>(0)?;
        write!(wtr, "{}", "entr")?;
        wtr.write_u32::<LittleEndian>(self.thing_a)?;
        wtr.write_u32::<LittleEndian>(self.thing_b)?;
        wtr.write_u32::<LittleEndian>(self.low)?;
        wtr.write_u32::<LittleEndian>(self.high)?;
        wtr.write_u32::<LittleEndian>(self.lvel)?;
        wtr.write_u32::<LittleEndian>(self.hvel)?;
        wtr.write_u32::<LittleEndian>(self.root)?;
        let things: Vec<u32> = vec![0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0x55, 0x18F5AC];
        for i in things.into_iter() {
            wtr.write_u32::<LittleEndian>(i)?;
        }
        Ok(())
    }
}

named!(parse_map_file <&[u8], MapFile>, do_parse!(
    header: parse_header >>
        entries: count!(parse_entry, header.sample_count as usize) >>
        (MapFile {
            header: header,
            entries: entries,
        })
));

impl Serialize for MapFile {
    fn write<W: Write>(&self, wtr: &mut W) -> Result<(), io::Error> {
        self.header.write(wtr)?;
        for e in self.entries.iter() {
            e.write(wtr)?;
        }
        Ok(())
    }
}

pub fn go(byt: &[u8]) {
    match parse_map_file(byt) {
        IResult::Done(_, fil) => println!("{}", fil),
        e  => panic!("{:?}", e),
    }
}
