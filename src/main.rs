// tt - time tracker
//
// Usage: tt (stop the current task)
//        tt <task name or description> start or resume a task
//
// Time tracking state is read from, and appended to ./tt.txt
// There is no reporting functionality yet.
//
// Released under the GPL v3 licence.

extern crate chrono;
extern crate regex;

use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Write;
use std::io::Seek;
use regex::Regex;
use regex::Captures;
use chrono::*;
use std::env;
use std::fmt;
use std::io::SeekFrom::End;

// FIXME: This is my first Rust program, so please forgive the current lack of error handling.
// I just wanted to get something which compiled and ran *on my machine*.
// It probably won't compile or run on your box and you won't be told the reason when it panics.
// It also still has a half-dozen warning, which I don't yet understand.
// But it does *work*.  (On my machine.)
fn main() {
    let filename = "tt.txt";

    // open a tt.txt file in the local directory
    let file = OpenOptions::new()
               .read(true)
               .write(true)
               .create(true)
               .open(filename)
               .unwrap();
  
    // now read the whole file to get the latest state
    let date_re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})").unwrap();
    let time_activity_re = Regex::new(r"^(\d{2}):(\d{2})\s*(.*)").unwrap();
    let reader = BufReader::new(file);
    let mut latest_date : Option<Date<Local>> = None;
    let mut latest_datetime : Option<DateTime<Local>> = None;
    let mut latest_activity : Option<String> = None;

    for wrapped_line in reader.lines() {
        let line = wrapped_line.unwrap();
        println!("line: {}", line);

        if date_re.is_match(&line) {
            let captures = date_re.captures(&line).unwrap();
            let year = captures.at(1).unwrap().parse::<i32>().unwrap();
            let month = captures.at(2).unwrap().parse::<u32>().unwrap();
            let day = captures.at(3).unwrap().parse::<u32>().unwrap();
            latest_date = Some(Local.ymd(year, month, day));
            latest_datetime = None;
            latest_activity = None;
        }

        if time_activity_re.is_match(&line) && latest_date != None {
            let captures = time_activity_re.captures(&line).unwrap();
            let hour = captures.at(1).unwrap().parse::<u32>().unwrap();
            let minute = captures.at(2).unwrap().parse::<u32>().unwrap();
            let activity = captures.at(3).unwrap();

            latest_datetime = Some(latest_date.unwrap().and_hms(hour, minute, 0));

            latest_activity = if activity.len() > 0 { 
                // TODO: if latest_activity already constains a string, clear it and reuse it
                // as per: https://stackoverflow.com/questions/33781625/how-to-allocate-a-string-before-you-know-how-big-it-needs-to-be
                Some(activity.to_string()) 
            } else { 
                None 
            };

            println!("time activity: {} |{}|", latest_datetime.unwrap(), activity);
        }
    }

    // FIXME: I have to open a seconds file descriptor to the same file, in order to be able to write to it
    let mut out = OpenOptions::new()
               .read(true)
               .write(true)
               .create(true)
               .open(filename)
               .unwrap();

    out.seek(End(0));

    let now = Local::now();
    if latest_date == None 
        || latest_date.unwrap().year() != now.year()
        || latest_date.unwrap().month() != now.month()
        || latest_date.unwrap().day() != now.day() {
       if (latest_date != None) { // not an empy file, as far as tt is concerned
           out.write_all(b"\n\n");
       }
       out.write_all(format!("{}\n", now.format("%Y-%m-%d")).as_bytes());
       out.write_all(b"\n");
    }

    if let Some(activity) = env::args().nth(1) {
        out.write_all(format!("{} {}\n", now.format("%H:%M"), activity).as_bytes());
    } else {
        // if there was no latest activity *and* there is no activity, then there's no point in writing a second blank line with just a time
        if latest_activity == None { 
            return;
        }
        out.write_all(format!("{}\n", now.format("%H:%M")).as_bytes());
    }

    // FIXME: we're just relying on the program exit to close the two file descriptors (which point at the same file).
}