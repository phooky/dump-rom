extern crate clap;
extern crate serial;
extern crate xmodem;
extern crate env_logger;

use clap::{Arg,App};

fn default_port_name() -> &'static str {
    "/dev/ttyACM0"
}

mod dumper;
mod promdate;

use dumper::Dumper;
use promdate::Promdate;

use serial::SerialPort;

fn main() {
    env_logger::init();
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
    
    let mut serial = serial::open(portname).expect("Couldn't open serial port!");

    const SETTINGS: serial::PortSettings = serial::PortSettings {
        baud_rate:    serial::Baud9600,
        char_size:    serial::Bits8,
        parity:       serial::ParityNone,
        stop_bits:    serial::Stop1,
        flow_control: serial::FlowNone,
    };
    serial.configure(&SETTINGS);

    println!("Portname is {}, port is open!",portname);
    let mut dumper = Promdate::new(serial);
    
    println!("Present: {}", dumper.is_present().unwrap());

    match opts.value_of("chip") {
        None => (),
        Some(name) => dumper.select_chip_by_name(name).unwrap(),
    }

    match dumper.selected_chip() {
        Ok(None) => println!("No chip selected."),
        Ok(Some(c)) => println!("{} selected.",c.name),
        Err(_) => println!("sad problems"),
    }
    match opts.value_of("out_path") {
        None => {
            println!("dumping to stdout");
            let mut o = std::io::stdout();
            dumper.dump_chip(&mut o);
        },
        Some(path) => {
            println!("Dumping to {}",path);
            let mut file = std::fs::File::create(path).unwrap();
            dumper.dump_chip(&mut file);
        }
    }
}
