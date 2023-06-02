//结构体使用泛型

//指定一个泛型参数
struct Point<T> {
    x: T,
    y: T,
}

fn generic_struct_one() {
    let integer = Point { x: 5, y: 10 };
    let float = Point { x: 1.0, y: 4.0 };
}

//指定两个泛型参数
struct Point<T, U> {
    x: T,
    y: U,
}

fn generic_struct_two() {
    let both_integer = Point { x: 5, y: 10 };
    let both_float = Point { x: 1.0, y: 4.0 };
    let integer_and_float = Point { x: 5, y: 4.0 };
}

//枚举中使用泛型

//标准库中的Option<T>
enum Option {
    Some(T),
    None,
}

// 标准库中的Result<T, E>
enum Result {
    Ok(T),
    Err(E),
}

// 方法定义中使用泛型
struct Point1<T> {
    x: T,
    y: T,
}

impl<T> Point1<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

impl Point1<f32> {
    fn distance_from_origin(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
