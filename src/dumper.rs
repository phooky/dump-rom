use std::error::Error;
use std::vec::Vec;

pub type DumperError = Box<Error>;

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
}
