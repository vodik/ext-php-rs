# `FromZval`

The `#[derive(FromZval)]` macro derives the `FromZval` trait on a struct or
enum.

## Structs

When used on a struct, the `FromZval` implementation will attempt to build the
struct from the properties of a PHP struct. All fields on the struct must
implement `FromZval` as well. Generics are allowed on structs that use the
derive macro, however, the implementation will add a `FromZval` bound to all
generics types.

### Examples

```rust
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[derive(FromZval)]
pub struct ExampleClass<'a> {
    a: i32,
    b: String,
    c: &'a str
}

#[php_function]
pub fn take_object(obj: ExampleClass) {
    dbg!(obj.a, obj.b, obj.c);
}
```

Calling from PHP:

```php
<?php

$obj = new stdClass;
$obj->a = 5;
$obj->b = 'Hello, world!';
$obj->c = 'another string';

take_object($obj);
```

Another example involving generics:

```rust
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

// T must implement both `PartialEq<i32>` and `FromZval`.
#[derive(Debug, FromZval)]
pub struct CompareVals<T: PartialEq<i32>> {
    a: T,
    b: T
}

#[php_function]
pub fn take_object(obj: CompareVals<i32>) {
    dbg!(obj);
}
```

## Enums

When used on an enum, the `FromZval` implementation will treat the enum as a
tagged union with a mixed datatype. This allows you to accept multiple types in
a parameter, for example, a string and an integer.

The enum variants must not have named fields, and each variant must have exactly
one field (the type to extract from the zval). Optionally, the enum may have one
default variant with no data contained, which will be used when the rest of the
variants could not be extracted from the zval.

The ordering of the variants in the enum is important, as the `FromZval`
implementation will attempt to parse the zval data in order. For example, if you
put a `String` variant before an integer variant, the integer would be converted
to a string and passed as the string variant.

### Examples

Basic example showing the importance of variant ordering and default field:

```rust
# extern crate ext_php_rs;
use ext_php_rs::prelude::*;

#[derive(Debug, FromZval)]
pub enum UnionExample<'a> {
    Long(u64), // Long
    ProperStr(&'a str), // Actual string - not a converted value
    ParsedStr(String), // Potentially parsed string, i.e. a double
    None // Zval did not contain anything that could be parsed above
}

#[php_function]
pub fn test_union(val: UnionExample) {
    dbg!(val);
}
```

Use in PHP:

```php
test_union(5); // UnionExample::Long(5)
test_union("Hello, world!"); // UnionExample::ProperStr("Hello, world!")
test_union(5.66666); // UnionExample::ParsedStr("5.6666")
test_union(null); // UnionExample::None
```
