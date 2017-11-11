use dumper::{Dumper,DumperError,ChipDesc};
use std::cell::RefCell;
use serial::SerialPort;
use std::time::Duration;
use std::str::from_utf8;
use xmodem::{Xmodem,Checksum};
use std::io::Write;

/// Interface for the "promdate" teensy2++ based ROM dumper.
pub struct Promdate<P : SerialPort> {
    internals : RefCell<PromdateInterior<P>>,
}

struct PromdateInterior<P : SerialPort> {
    serial : P,
    supported : Option<Vec<ChipDesc>>,
    selected : Option<ChipDesc>,
}

impl<P : SerialPort> Promdate<P> {
    pub fn new(mut serial : P) -> Promdate<P> {
        serial.set_timeout(Duration::from_millis(10));
        Promdate{
            internals : RefCell::new(
                PromdateInterior {
                    serial : serial,
                    supported : None,
                    selected : None,
                }),
        }
    }

    fn flush_input(&self) {
        let ref mut serial = self.internals.borrow_mut().serial;
        let mut buf = [0; 128];
        loop {
            match serial.read(&mut buf) {
                Ok(0) => break,
                Err(_) => break,
                _ => continue,
            }
        }
    }

    fn update_supported(&self) {
        let mut v = Vec::new();
        let mut selected = None;
        self.flush_input();
        {
            let ref mut serial = self.internals.borrow_mut().serial;
            serial.write(b"l\n");
            let mut buf = [0;128];
            let mut raw = Vec::new();
            loop {
                match serial.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => raw.extend_from_slice(&buf[..n]),
                    Err(_) => break,
                }
            }
            let mut lines = raw.split(|c| *c == b'\n');
            for line in lines {
                match from_utf8(line) {
                    Err(_) => continue,
                    Ok(s) => {
                        match parse_desc(s.trim()) {
                            Some(c) => { if c.1 { selected = Some(c.0.clone()) }; v.push(c.0) },
                            None => continue,
                        }
                    }
                }
            }
        }
        self.internals.borrow_mut().supported = Some(v);
        self.internals.borrow_mut().selected = selected;
    }
}

fn parse_desc(mut s : &str) -> Option<(ChipDesc,bool)> {
    let mut flag = false;
    if s.starts_with("*** ") {
        flag = true;
        s = &s[4..];
    }
    let mut iter = s.splitn(2,|c| c==' ');
    match iter.next() {
        Some(s) => if s.parse::<u32>().is_err() { return None; },
        None => return None,
    }
    match iter.next() {
        None => None,
        Some(s) => if s != "NONE" {
            Some( (ChipDesc { name : String::from(s), key : String::from(s) }, flag) )
        } else { None }
    }
}

impl<P : SerialPort> Dumper for Promdate<P> {
    /// Currently, we do this by sending a linefeed and looking for the "\n> " prompt.
    /// In the future, maybe Promdate should have a version checking facility. :)
    fn is_present(&self) -> Result<bool,DumperError> {
        self.flush_input();
        let ref mut serial = self.internals.borrow_mut().serial;
        serial.write(b"\n");
        serial.flush();
        let mut v = Vec::new();
        serial.read_to_end(&mut v);
        match v.len() {
            n if n >= 3 => Ok( b"\n> " == &v[n-3..]),
            _ => Ok(false),
        }
    }
    
    fn list_supported(&self) -> Result<Vec<ChipDesc>,DumperError> {
        self.update_supported();
        let supported = self.internals.borrow().supported.clone();
        match supported {
            None => Ok(Vec::new()),
            Some(v) => Ok(v),
        }
    }
    
    fn selected_chip(&self) -> Result<Option<ChipDesc>,DumperError> {
        self.update_supported();
        Ok(self.internals.borrow().selected.clone())
    }
    
    fn set_selected_chip(&mut self,chip:&ChipDesc) -> Result<(),DumperError> {
        self.flush_input();
        let ref mut serial = self.internals.borrow_mut().serial;
        serial.write(format!("m{}\n",chip.key).as_bytes());
        serial.flush();
        let mut v = Vec::new();
        serial.read_to_end(&mut v);
        let rsp = String::from_utf8(v).unwrap();

        for line in rsp.split("\n") {
            if line.trim().starts_with("***") { return Ok(()) }
        }
        Err(DumperError::UnrecognizedChip(format!("{}/{}",rsp,chip.name.clone())))
    }

    fn dump_chip(&mut self, outstream : &mut Write) -> Result<usize,DumperError> {
        let mut xm = Xmodem::new();
        let ref mut serial = self.internals.borrow_mut().serial;
        serial.set_timeout(Duration::new(2,0));
        match xm.recv(serial,outstream,Checksum::Standard) {
            Ok(()) => (),
            Err(x) => { println!("ERROR {:?}",x); }
        }
        serial.set_timeout(Duration::from_millis(10));
        Ok((0))
    }

}
    
