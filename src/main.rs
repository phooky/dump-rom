extern crate clap;
extern crate serial;

use clap::{Arg,App};

fn default_port_name() -> &'static str {
    "/dev/ttyACM0"
}

mod dumper;
mod promdate;

use dumper::Dumper;
use promdate::Promdate;

fn main() {
    let opts = App::new("ROM dump helper")
        .version("0.1")
        .author("phooky@gmail.com")
        .about("Reliably dump ROMs using PROMDate dumper")
        .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .help("serial port that dumper is installed on")
             .takes_value(true))
        .arg(Arg::with_name("out_path")
             .short("o")
             .long("output")
             .help("path to save PROM to")
             .takes_value(true))
        .arg(Arg::with_name("chip")
             .short("c")
             .long("chip")
             .help("String specifying the name of the chip to use.")
             .takes_value(true))
        .get_matches();

    let portname = match opts.value_of("port") {
        Some(x) => x,
        None => default_port_name(),
    };

    let serial = serial::open(portname).expect("Couldn't open serial port!");

    println!("Portname is {}, port is open!",portname);
    let mut dumper = Promdate::new(serial);
    
    println!("Present: {}", dumper.is_present().unwrap());

    for cd in dumper.list_supported().unwrap() {
        println!("Supported: {}", cd.name);
    }

    match opts.value_of("chip") {
        None => (),
        Some(name) => dumper.select_chip_by_name(name).unwrap(),
    }

    match dumper.selected_chip() {
        Ok(None) => println!("No chip selected."),
        Ok(Some(c)) => println!("{} selected.",c.name),
        Err(_) => println!("sad problems"),
    }

}
