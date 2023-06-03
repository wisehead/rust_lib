use serde::{Deserialize, Serialize};
use serde_json;
use sled::{self, Db, Result};
use std::str;

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub is_done: bool,
}

pub struct TodoList {
    db: Db,
    latest_id: String,
}

impl TodoList {
    pub fn new(path: String) -> Self {
        let db = sled::open(path.as_str()).unwrap();
        let latest_id = match db.get("latest_id".as_bytes()) {
            Ok(id) => match id {
                Some(id) => str::from_utf8(&id).unwrap().to_string(),
                None => {
                    format!("0")
                }
            },
            _ => panic!("erro"),
        };
        TodoList { db, latest_id }
    }

    pub fn list(&self) -> Vec<Todo> {
        let mut list = Vec::new();
        for v in self.db.iter() {
            if "latest_id" == str::from_utf8(&v.as_ref().unwrap().0).unwrap() {
                continue;
            }

            let stodo = str::from_utf8(&v.as_ref().unwrap().1).unwrap();
            let todo: Todo = serde_json::from_str(stodo).unwrap();
            list.push(todo);
        }
        list
    }

    pub fn add(&mut self, title: String) -> Result<()> {
        let id = format!("{}", self.latest_id.parse::<usize>().unwrap() + 1);
        self.latest_id = id.clone();
        let todo = Todo {
            id: id.clone(),
            title,
            is_done: false,
        };
        let r = serde_json::to_string(&todo).unwrap();
        self.db.insert(id.as_bytes(), r.as_bytes())?;
        self.db.insert("latest_id".as_bytes(), id.as_bytes())?;
        Ok(())
    }

    pub fn remove(&mut self, id: String) {
        self.db.remove(id.as_bytes()).unwrap();
    }

    pub fn toggle_done(&mut self, id: String) {
        let todo = self.db.get(id.as_bytes()).unwrap();
        match todo {
            Some(todo) => {
                let s = str::from_utf8(&todo).unwrap();
                let mut todo: Todo = serde_json::from_str(s).unwrap();
                todo.is_done = !todo.is_done;
                self.db
                    .insert(
                        id.as_bytes(),
                        serde_json::to_string(&todo).unwrap().as_bytes(),
                    )
                    .unwrap();
            }
            None => {}
        }
    }

    pub fn get(&self, id: String) -> Option<Todo> {
        let todo = self.db.get(id.as_bytes()).unwrap();
        match todo {
            Some(todo) => {
                let s = str::from_utf8(&todo).unwrap();
                let todo: Todo = serde_json::from_str(s).unwrap();
                Some(todo)
            }
            None => None,
        }
    }

    pub fn edit(&mut self, id: String, title: String) {
        let todo = self.db.get(id.as_bytes()).unwrap().unwrap();
        let mut todo: Todo = serde_json::from_str(str::from_utf8(&todo).unwrap()).unwrap();
        todo.title = title;
        let todo = serde_json::to_string(&todo).unwrap();
        self.db.insert(&id.as_bytes(), todo.as_bytes()).unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    //打开数据库
    let tree = sled::open("/tmp/welcome-to-sled").expect("open");

    // 插入KV，读取Key对应的值
    tree.insert("KEY1", "VAL1");
    assert_eq!(tree.get(&"KEY1"), Ok(Some(sled::IVec::from("VAL1"))));

    // 范围查询
    for kv in tree.range("KEY1".."KEY9") {
        //println!("key:{}, value:{}", key.unw);
        //println!("key :{:?}, value:{:?}", kv.unwrap().0, kv.unwrap().1);
        println!("key value :{:?}", kv.unwrap());
    }

    // 删除
    tree.remove(&"KEY1");

    // atomic compare and swap，可以用在并发编程中
    tree.compare_and_swap("KEY1", Some("VAL1"), Some("VAL2"));

    // 阻塞直到所有修改都写入硬盘
    tree.flush();
    /* 
    let mut todo_list = TodoList::new("my_db".to_string());
    todo_list.add(format!("ok"))?;
    todo_list.add(format!("thisis good"))?;
    let todo = todo_list.get(format!("1"));
    println!("{:?}", todo);
    */
    // let todo = todo_list.get(format!("2"));
    // println!("{:?}", todo);
    // todo_list.toggle_done(format!("2"));
    // let todo = todo_list.get(format!("2"));
    // println!("{:?}", todo);
    
    /* 
    let list = todo_list.list();
    println!("{:?}", list);
    */
    Ok(())
}
