use chrono::prelude::*;
pub fn time_x(){
    //获取当前毫秒值
    let now: DateTime<Local> = Local::now();
    let mills: i64 = now.timestamp_millis(); 
    println!("获取毫秒的值: {}", mills);
     
    let dt: DateTime<Local> = Local.timestamp_millis(mills);
    // date time parsed millis: 2021-01-04 20:01:36.945 +08:00
    println!("获取精确到毫秒时间: {}", dt); 

    let seconds = now.timestamp();
    println!("获取秒的值: {}", seconds); 

    // 字符串转时间对象
    let fmt = "%Y-%m-%d %H:%M:%S";
    let t_str = "2023-06-09 17:17:54";
    let t = NaiveDateTime::parse_from_str(t_str, fmt).unwrap();
    
    //let t = DateTime::parse_from_str(t_str, fmt).unwrap();
    let t_sec = t.timestamp();

    let no_native: DateTime<Utc> = DateTime::from_utc(t,Utc);
    let no_native_sec = t.timestamp();

    let newdt: DateTime<Local> = Local.timestamp(1686303331,0);
    println!("t_str: {}, t:{}, t_sec:{}, no_native:{}, no_native_sec:{}, newdt:{}",t_str, t, t_sec, no_native, no_native_sec, newdt);
}
