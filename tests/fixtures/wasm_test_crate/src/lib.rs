#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[no_mangle]
pub extern "C" fn square(a: i32) -> i32 {
    a * a
}

#[no_mangle]
pub extern "C" fn is_positive(a: i32) -> i32 {
    if a > 0 { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn float_add(a: f64, b: f64) -> f64 {
    a + b
}
