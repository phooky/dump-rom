use std::error::Error;
use std::vec::Vec;

#[derive(Debug,Clone)]
pub enum DumperError {
    UnrecognizedChip(String),
}

#[derive(Clone)]
pub struct ChipDesc {
    pub name : String,
    pub key : String,
}

/// A Dumper represents a hardware (or software) ROM dumper.
pub trait Dumper {
    fn is_present(&self) -> Result<bool,DumperError>;
    fn list_supported(&self) -> Result<Vec<ChipDesc>,DumperError>;
    fn selected_chip(&self) -> Result<Option<ChipDesc>,DumperError>;
    fn set_selected_chip(&mut self,chip:&ChipDesc) -> Result<(),DumperError>;
    fn select_chip_by_name(&mut self, name:&str) -> Result<(),DumperError> {
        let list = try!(self.list_supported());
        let mut candidate : Option<ChipDesc> = None;
        for cd in list {
            if cd.name.starts_with(name) {
                match candidate {
                    None => candidate = Some(cd),
                    Some(_) => return
                        Err(DumperError::UnrecognizedChip("Ambiguous name".to_string())),
                }
            }
        }
        match candidate {
            None => Err(DumperError::UnrecognizedChip("No matching chip found".to_string())),
            Some(cd) => self.set_selected_chip(&cd)
        }
    }
}
