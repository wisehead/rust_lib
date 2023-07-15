use std::collections::HashMap;
    pub fn sort_sentence(s: String) -> String {
        let mut map = HashMap::new();
        //map.insert("color", "red");
        
        let v:Vec<&str> = s.split(' ').collect();
        let mut vec: Vec<String> = Vec::with_capacity(v.len());
        /*
        for sss in v.iter() {
            println!("sss {}", sss);
        }
        */
        /*
        v.iter()
        .map(|str| {

                let ss = str.to_string();
                println!("ss {}", ss);
                let len = ss.len();
                let key = ss[len-1..len-1].to_string();
                let value = ss[0..len-2].to_string();
                println!("xxx {} / {}", key, value);
                map.insert(key, value );
            });
            */
        for str in v.iter() {
            println!("str:{}", str);
            let ss = str.to_string();
            println!("ss:{}", ss);
            let len = ss.len();
            println!("len:{}", len);
            let key = ss[len-1..].to_string();
            let value = ss[0..len-1].to_string();
            println!("xxx key:{} / value:{}", key, value);
            map.insert(key, value );
            vec.push(String::new());
        }
        let mapsize = map.len();
        println!("xxx map size is :{}", mapsize);
        
        println!("Capacity: {}", vec.capacity());
        //let mut vec = vec![1, 2, 3];
        let slice = vec.as_mut_slice();
        //slice[0] = 5; // 修改第一个元素的值
        //println!("Modified Vec: {:?}", vec);
        for (key, value) in map.iter() {
            println!("{} / {}", key, value);
            let i = key.parse::<usize>().unwrap();
            slice[i-1] = value.to_string();
        }
        //let r = vec.collect();
        let mut r = String::new();
        let mut first = true;
        for str in slice.iter() {
            if first {
                first = false;
            } else {
                r += &" ".to_string();
            }
            r += str;
        }

        return r;
    }

fn main() {}
