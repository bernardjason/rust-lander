

use std::time::SystemTime;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct HighScore {
    pub(crate) id:i32,
    pub score:i32,
    pub level:i32,
    pub by:String,
    pub when:u64,
    pub formatted :String,
}

pub struct HighScoreTable {
    last_id:i32,
    pub table:Vec<HighScore>,
    last_entry:i32,
}

impl HighScoreTable {
    pub fn new() -> HighScoreTable {

        HighScoreTable{
            last_id:0,
            table:vec![],
            last_entry:-1,
        }
    }
    pub fn is_current_score(&self,id:i32) -> bool {

        if self.last_id == id {
            true
        } else {
            false
        }
    }
    pub fn add_score(&mut self, score:i32, level:i32) -> bool {
        self.last_id = self.last_id +1;
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        let high_score = HighScore{
            id:self.last_id,
            score,
            level,
            by:String::new(),
            when : now,
            formatted: format_time(now),
        };
        self.table.push(high_score);

        self.table.sort_by(|a,b|  b.score.partial_cmp(&a.score).unwrap());
        self.table.truncate(10);
        for t in self.table.iter() {
            if t.score == score && t.id == self.last_id {
                self.last_entry = self.last_id;
                return true;
            }
        }
        self.last_entry = -1;
        return false;
    }
    pub fn set_name(&mut self,name:&String) {
        for t in self.table.iter_mut() {
            if t.id == self.last_id {
               t.by = name.clone();
            }
        }
    }
}

fn format_time(now:u64) -> String {
    // based on https://www.geeksforgeeks.org/convert-unix-timestamp-to-dd-mm-yyyy-hhmmss-format/
    const FEB:usize = 1;
    const FEB_LEAP:i32 = 29;
    let days_of_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    //let mut date = 0;


    let mut days_till_now:i32 = (now / (24 * 60 * 60)) as i32;
    let extra_time:i32 = (now % (24 * 60 * 60)) as i32;
    let mut curr_year = 1970;

    while days_till_now >= 365 {
        if curr_year % 400 == 0 || (curr_year % 4 == 0 && curr_year % 100 != 0) {
            days_till_now -= 366
        } else {
            days_till_now -= 365
        }
        curr_year += 1;
    }

    let mut extra_days = days_till_now + 1;

    let leap_year = if curr_year % 400 == 0 || (curr_year % 4 == 0 && curr_year % 100 != 0){
        true
    } else {
        false
    };

    let mut month = 0;

    if leap_year {
        let mut index = 0;
        while index < days_of_month.len() {
            if index == FEB {
                if extra_days < FEB_LEAP  {
                    break
                }

                month += 1;
                extra_days -= FEB_LEAP;
            } else {
                if extra_days < days_of_month[index]  {
                    break
                }
                month += 1;
                extra_days -= days_of_month[index];
            }

            index += 1;
        }
    } else {
        let mut index = 0;
        while index < days_of_month.len() {
            if extra_days < days_of_month[index]  {
                break
            }

            month += 1;
            extra_days -= days_of_month[index];
            index += 1;
        }
    }

    let date = if extra_days > 0 {
        month += 1;
        extra_days
    } else {
        if month == 2 && leap_year {
            FEB_LEAP
        } else {
            if month == 0 {
                month = 12;
                curr_year = curr_year -1;
            }
            days_of_month[month -1]
        }
    };
    let hours = extra_time / 3600;
    let minutes = (extra_time % 3600) / 60;
    //seconds = (extra_time % 3600) % 60;

    return format!("{:02}/{:02}/{:04} {:02}:{:02}", date, month, curr_year, hours, minutes);
}