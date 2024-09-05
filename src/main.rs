extern crate core;

fn main() {
    println!("Hello, world!");
    let c = C::new();
    let d = D::new();
    c.do_something();
    d.do_something();
    call_do_something(c);
    call_do_something(d);
}

fn call_do_something<T: B>(t: T) {
    t.do_something();
}

trait A {
    fn create() -> Self where Self: Sized;
}

trait B : A {
    fn new() -> Self where Self: Sized {
        let res = Self::create();
        res.do_something();
        res
    }
    fn do_something(&self){
        println!("B");
    }
}

struct C {}

impl C {
}

impl A for C {
    fn create() -> Self {
        C {}
    }
}

impl B for C {
    fn do_something(&self) {
        println!("C");
    }}

struct D {}

impl A for D {
    fn create() -> Self {
        D {}
    }
}

impl B for D {
    fn do_something(&self) {
        println!("D");
    }}

impl D {
}