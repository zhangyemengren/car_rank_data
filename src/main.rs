use chrono::{Datelike, Duration, Local, NaiveDate, TimeZone};
use serde::Serialize;
use serde_json::ser::PrettyFormatter;
use serde_json::{Serializer, Value};
use std::cell::OnceCell;
use std::io::BufWriter;
use std::path;
use std::{
    env,
    fs::{self, File},
    thread,
};

const URL: &str = "https://www.dongchedi.com/motor/pc/car/rank_data";
const CLIENT: OnceCell<reqwest::blocking::Client> = OnceCell::new();

fn get_data(new_energy_type: Option<&str>) {
    let binding = CLIENT;
    let client = binding.get_or_init(|| reqwest::blocking::Client::new());

    let env_date = env::var("DATE");
    let past_date = if let Ok(env_date) = env_date {
        NaiveDate::parse_from_str(&env_date, "%Y%m%d")
            .map(|naive_date| {
                let naive_datetime = naive_date.and_hms_opt(0, 0, 0);
                Local.from_local_datetime(&naive_datetime.unwrap()).unwrap()
            })
            .unwrap_or_else(|_| {
                println!("Invalid DATE env, use current date instead.");
                Local::now()
            })
    } else {
        // 通常10日后出上一月统计 定时任务20号执行
        Local::now() - Duration::days(30)
    };
    println!("Past date: {}", past_date);
    let year = past_date.year();
    let month = format!("{:02}", past_date.month());
    let date_str = format!("{}{}", year, month);
    let t = client
        .get(URL)
        .query(&[
            ("count", "100"),
            ("offset", "0"),
            ("month", &date_str),
            ("rank_data_type", "11"),
            ("new_energy_type", new_energy_type.unwrap_or("")),
            ("outter_detail_type", ""),
            ("nation", "0"),
        ])
        .send()
        .unwrap()
        .json::<Value>()
        .unwrap();

    let file_name = if new_energy_type.is_none() {
        "all"
    } else {
        "new_energy_type"
    };

    let file_path = format!("data/dcd/{}/{}.json", date_str, file_name);
    if let Some(parent) = path::Path::new(&file_path).parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let file = File::create(&file_path).unwrap();
    let writer = BufWriter::new(file);

    let mut ser = Serializer::with_formatter(writer, PrettyFormatter::with_indent(b"    "));

    t.serialize(&mut ser).unwrap();
}

fn main() {
    // all
    let handle1 = thread::spawn(|| get_data(None));
    // new_energy_type
    let handle2 = thread::spawn(|| get_data(Some("1,2,3")));
    handle1.join().unwrap();
    handle2.join().unwrap();
    println!("Done!");
}
