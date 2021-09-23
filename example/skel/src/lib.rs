use ext_php_rs::prelude::*;

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

#[php_function]
pub fn test_stdclass(obj: TestStdClass<i32, &str, &str>) {
    dbg!(obj);
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
