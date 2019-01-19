#[macro_use]
extern crate jni_macros;

#[jni(net.timluq.rust.jnimacro.examples.Example)]
pub fn initSuccess() -> bool {
    true
}
#[jni(net.timluq.rust.jnimacro.examples.Example)]
pub fn initStr() -> str {
    "Yep"
}
#[jni(net.timluq.rust.jnimacro.examples.Example)]
pub fn initString() -> ::std::string::String {
    "Yep".to_string()
}
