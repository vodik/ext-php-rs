use ext_php_rs::{php::types::object::ZendObject, prelude::*};

#[php_class]
struct TestClass {
    #[prop(rename = "Hello")]
    a: i32,
    #[prop]
    b: i64,
    #[prop]
    c: String,
}

impl Default for TestClass {
    fn default() -> Self {
        Self {
            a: 100,
            b: 123,
            c: "Hello, world!".into(),
        }
    }
}

#[php_impl]
impl TestClass {
    #[getter]
    fn get_test_name(&self) -> String {
        self.c.clone()
    }

    #[setter]
    fn set_test_name(&mut self, c: String) {
        self.c = c;
    }
}

#[derive(Debug, FromZval)]
pub struct TestStdClass<A, B, C>
where
    A: PartialEq<i32>,
{
    a: A,
    b: B,
    c: C,
}

#[derive(Debug, FromZval)]
pub enum UnionExample<'a, T> {
    B(T),
    C(&'a str),
    Obj(&'a ZendObject),
    None,
}

#[php_function]
pub fn test_union(union: UnionExample<i32>) {
    dbg!(union);
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
