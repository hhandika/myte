use chrono::NaiveTime;

fn parse_duration(duration: u64) -> String {
    let sec = (duration % 60) as u32;
    let min = ((duration / 60) % 60) as u32;
    let hours = ((duration / 60) / 60) as u32;
    let time = NaiveTime::from_hms(hours, min, sec);
    time.format("%H:%M:%S").to_string()
}

pub fn print_formatted_duration(duration: u64) {
    let time = parse_duration(duration);
    println!("Execution time (HH:MM:SS): {}", time);
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
