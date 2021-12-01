use std::io::{self, Result, Write};
use std::iter;

use ansi_term::Colour::Yellow;
use chrono::{Local, NaiveTime};
use sysinfo::{System, SystemExt};

fn parse_duration(duration: u64) -> String {
    let sec = (duration % 60) as u32;
    let min = ((duration / 60) % 60) as u32;
    let hours = ((duration / 60) / 60) as u32;
    let time = NaiveTime::from_hms(hours, min, sec);
    time.format("%H:%M:%S").to_string()
}

pub fn print_formatted_duration(duration: u64) {
    let time = parse_duration(duration);
    log::info!("Execution time (HH:MM:SS): {}", time);
}

pub fn print_divider(text: &str, len: usize) {
    let sym = '=';
    let mut header = PrettyDivider::new(text, sym, len);
    header.print_header().unwrap();
}

pub fn get_system_info() {
    let sysinfo = System::new_all();
    let total_ram = sysinfo.get_total_memory();
    let gb = 1048576;

    log::info!("{}", Yellow.paint("System Information"));

    log::info!(
        "{:18}: {} {}",
        "Operating system",
        get_os_name(&sysinfo),
        get_os_version(&sysinfo)
    );

    log::info!("{:18}: {}", "Kernel version", get_kernel_version(&sysinfo));
    log::info!("{:18}: {:?}", "Available cores", num_cpus::get_physical());
    log::info!("{:18}: {:?}", "Available threads", num_cpus::get());
    log::info!("{:18}: {} Gb", "Total RAM", total_ram / gb);
    log::info!(
        "{:18}: {}\n",
        "Date and time",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );
}

fn get_os_name(sysinfo: &System) -> String {
    match sysinfo.get_name() {
        Some(i) => i,
        None => String::from("UNKNOWN"),
    }
}

fn get_os_version(sysinfo: &System) -> String {
    match sysinfo.get_os_version() {
        Some(i) => i,
        None => String::from(""),
    }
}

fn get_kernel_version(sysinfo: &System) -> String {
    match sysinfo.get_kernel_version() {
        Some(i) => i,
        None => String::from("UNKNOWN"),
    }
}

struct PrettyDivider {
    text: String,
    sym: char,
    len: usize,
    text_len: usize,
    sym_len: usize,
    color: String,
}

impl PrettyDivider {
    fn new(text: &str, sym: char, len: usize) -> Self {
        Self {
            text: String::from(text),
            sym,
            len,
            text_len: 0,
            sym_len: 0,
            color: String::from("\x1b[0;33m"),
        }
    }

    fn print_header(&mut self) -> Result<()> {
        self.get_len();
        let io = io::stdout();
        let mut handle = io::BufWriter::new(io);
        write!(handle, "{}", self.color)?;
        if self.text_len > self.len {
            writeln!(handle, "{}", self.text)?;
        } else {
            self.print_with_symbol(&mut handle)?;
        }
        write!(handle, "\x1b[0m")?;
        Ok(())
    }

    fn print_with_symbol<W: Write>(&mut self, handle: &mut W) -> Result<()> {
        self.print_symbols(handle);
        write!(handle, " {} ", self.text)?;
        self.print_symbols(handle);

        if self.text_len % 2 != 0 {
            write!(handle, "{}", self.sym)?;
        }

        writeln!(handle)?;
        Ok(())
    }

    fn get_len(&mut self) {
        self.text_len = self.text.len();

        if self.len > self.text_len {
            self.sym_len = (self.len - self.text_len) / 2;
        } else {
            self.sym_len = self.len;
        }
    }

    fn print_symbols<W: Write>(&self, io: &mut W) {
        let sym: String = iter::repeat(self.sym).take(self.sym_len).collect();
        write!(io, "{}", sym).unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn time_parsing_test() {
        let duration = 65;
        let duration_2 = 3600;
        let time = parse_duration(duration);
        let hours = parse_duration(duration_2);

        assert_eq!("00:01:05", time);
        assert_eq!("01:00:00", hours);
    }
}
