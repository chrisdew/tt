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
extern crate itertools;

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
use std::iter::Iterator;
use itertools::Itertools;

// FIXME: This is my first Rust program, so please forgive the current lack of error handling.
// I just wanted to get something which compiled and ran *on my machine*.
// It probably won't compile or run on your box and you won't be told the reason when it panics.
// It also still has a half-dozen warning, which I don't yet understand.
// But it does *work*.  (On my machine.)
fn main() {
    let filename = "tt.txt";

    // open a tt.txt file in the local directory
    let mut file = OpenOptions::new()
               .read(true)
               .write(true)
               .create(true)
               .open(filename)
               .unwrap();
  
    // now read the whole file to get the latest state
    let date_re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})").unwrap();
    let time_activity_re = Regex::new(r"^(\d{2}):(\d{2})\s*(.*)").unwrap();
    let mut latest_date : Option<Date<Local>> = None;
    let mut latest_datetime : Option<DateTime<Local>> = None;
    let mut latest_activity : Option<String> = None;

    // open a new scope so we can mutably lend the file to the reader
    // but regain exclusive ownership after the scope is exited
    // see: https://stackoverflow.com/questions/33831265/how-to-use-a-file-for-reading-and-writing
    { 
        let reader = BufReader::new(&mut file);
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
    }

    file.seek(End(0));

    let now = Local::now();
    if latest_date == None 
        || latest_date.unwrap().year() != now.year()
        || latest_date.unwrap().month() != now.month()
        || latest_date.unwrap().day() != now.day() {
       if (latest_date != None) { // not an empy file, as far as tt is concerned
           file.write_all(b"\n\n");
       }
       file.write_all(format!("{}\n", now.format("%Y-%m-%d")).as_bytes());
       file.write_all(b"\n");
    }

    let activity = env::args().skip(1).join(" ");
    if (activity.len() > 0) {
        file.write_all(format!("{} {}\n", now.format("%H:%M"), activity).as_bytes());
    } else {
        // if there was no latest activity *and* there is no activity, then there's no point in writing a second blank line with just a time
        if latest_activity == None { 
            return;
        }
        file.write_all(format!("{}\n", now.format("%H:%M")).as_bytes());
    }

    // "file" is automatically closed as it goes out of scope at program end - very neat
}
