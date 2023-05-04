use std::collections::HashMap;
impl Solution {
    pub fn replace_digits(s: String) -> String {
        let alpha = "abcdefghijklmnopqrstuvwxyz";
        let vv: Vec<char> =Vec::new();
        let mut i = 0;
        let mut scores = HashMap::new();
        let alpha_chars: Vec<char> = alpha.chars().collect();
        let mut res: String = String::new();
        while (i < alpha.len()) {
            //vv.push(alpha_chars[i]);
            scores.insert(alpha_chars[i], i);
            i += 1;
        }
        //println!("{:?}", scores);
        i = 0;
        let mut n = s.len();
        let s_chars: Vec<char> = s.chars().collect();
        while (i < n) {
            println!("{}:{},", i, s_chars[i]);
            if (i % 2 == 1) {
                let c = s_chars[i];
                let ch = s_chars[i-1];
                let num = (c.to_string()).parse::<i32>().unwrap();
                println!("num:{}",num);
                let mut index = scores.get(&ch).unwrap();
                println!("xx{}:{:?},", i, index);
                let mut x = index + num as usize;
                let mut temp:char = alpha_chars[x];
                res.push(temp);
            } else {
                res.push(s_chars[i])
            }
            
            i += 1;
        }
        res
    }
}
