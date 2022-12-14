/*
 * Compile with:
 cargo run -- tests/hello.tig
 * Assembly with:
 nasm -f elf64 tests/hello.s
 * Link with:
  ld -dynamic-linker /lib64/ld-linux-x86-64.so.2 -o hello /usr/lib/Scrt1.o /usr/lib/crti.o -L/usr/bin/../lib64/gcc/x86_64-pc-linux-gnu/8.3.0 \
                      -L/usr/bin/../lib64/gcc/x86_64-pc-linux-gnu/8.3.0/../../.. tests/hello.o target/debug/libruntime.a -lpthread -ldl --no-as-needed -lc -lgcc --as-needed \
                      -lgcc_s --no-as-needed /usr/lib/crtn.o
 */

use std::ffi::{CStr, CString};
//use std::process;
use std::io::{Read, Write, stdin, stdout};

/*extern {
    fn main();
}*/

#[no_mangle]
extern fn ord(string: *const i8) -> i64 {
    let cstring = unsafe { CStr::from_ptr(string) };
    cstring.to_str().expect("cstr to_str").chars().next().expect("ord string is empty") as i64
}

#[no_mangle]
extern fn chr(num: i64) -> *const i8 {
    let char = num as u8 as char;
    let cstring = CString::new(char.to_string()).expect("CString::new");
    cstring.into_raw()
}

#[no_mangle]
extern fn getchar() -> *const i8 {
    let stdin = stdin();
    let char = stdin.bytes().next().expect("next char").expect("read stdin") as char;
    let cstring = CString::new(char.to_string()).expect("CString::new");
    cstring.into_raw()
}

#[no_mangle]
extern fn concat(string1: *const i8, string2: *const i8) -> *const i8 {
    let cstring1 = unsafe { CStr::from_ptr(string1) };
    let cstring2 = unsafe { CStr::from_ptr(string2) };
    let mut string1 = cstring1.to_str().expect("to_str").to_string();
    let string2 = cstring2.to_str().expect("to_str").to_string();
    string1.push_str(&string2);
    let cstring = CString::new(string1).expect("CString::new");
    cstring.into_raw()
}

#[no_mangle]
extern fn stringEqual(string1: *const i8, string2: *const i8) -> i64 {
    let cstring1 = unsafe { CStr::from_ptr(string1) };
    let cstring2 = unsafe { CStr::from_ptr(string2) };
    (cstring1 == cstring2) as i64
}

#[no_mangle]
extern fn initArray(length: usize, init_value: i64) -> i64 {
    Box::into_raw(vec![init_value; length].into_boxed_slice()) as *mut i64 as i64
}

#[no_mangle]
extern fn print(string: *const i8) {
    let cstring = unsafe { CStr::from_ptr(string) };
    if let Ok(string) = cstring.to_str() {
        print!("{}", string);
    }
    let _ = stdout().flush();
}

#[no_mangle]
extern fn printi(num: i32) {
    println!("{}", num);
}

/*#[no_mangle]
extern fn _start() {
    unsafe {
        main();
    }
    process::exit(0);
}*/
