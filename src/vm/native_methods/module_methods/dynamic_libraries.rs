// Opening a shared library and finding a symbol in it, which Fiddle::Handle
// is built on.

use super::*;

/// The text `dlerror` holds for the call that just failed.
fn last_loader_error() -> String {
    // SAFETY: `dlerror` answers a C string the loader owns, or null when no
    // call has failed since it was last asked.
    unsafe {
        let held = libc::dlerror();
        if held.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(held).to_string_lossy().to_string()
        }
    }
}

fn loader_error(message: &str, position: Position) -> MetorexError {
    simple_exception("Fiddle::DLError", message, position)
}

/// The C string at `address`, as a binary String, or nil for a null pointer.
fn c_string_at(address: i64) -> Object {
    let address = address as *const libc::c_char;
    if address.is_null() {
        return Object::Nil;
    }
    // SAFETY: Fiddle reads the C string at an address the program handed
    // it, which is the use Fiddle is for.
    let bytes = unsafe { std::ffi::CStr::from_ptr(address) }.to_bytes();
    Object::String(Rc::new(crate::object::StringValue::from_bytes(
        crate::vm::native_methods::string_methods::bytes_as_text(bytes),
    )))
}

fn null_byte_error(position: Position) -> MetorexError {
    simple_exception("ArgumentError", "string contains null byte", position)
}

impl VirtualMachine {
    /// `Fiddle.__dynamic_library__(command, ...)`: `:open` with a path (or
    /// nil for the program itself) and flags answers the handle's address,
    /// `:symbol` with a handle and a name answers the symbol's address,
    /// `:close` with a handle closes it, and `:default` answers the handle
    /// that searches every library loaded. `:flags` answers RTLD_GLOBAL,
    /// RTLD_LAZY, and RTLD_NOW as the platform numbers them.
    pub(crate) fn dynamic_library_command(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let command = match arguments.first() {
            Some(Object::Symbol(name)) => name.as_str().to_string(),
            _ => String::new(),
        };
        let number = |at: usize| match arguments.get(at) {
            Some(Object::Int(held)) => *held,
            _ => 0,
        };
        let text = |at: usize| match arguments.get(at) {
            Some(Object::String(held)) => Some(held.as_str().to_string()),
            _ => None,
        };
        match command.as_str() {
            "open" => {
                let path = match text(1) {
                    Some(path) => {
                        Some(std::ffi::CString::new(path).map_err(|_| null_byte_error(position))?)
                    }
                    None => None,
                };
                // SAFETY: the path is a C string that lives for the call, or
                // null for the program itself.
                let opened = unsafe {
                    libc::dlopen(
                        path.as_ref().map_or(std::ptr::null(), |held| held.as_ptr()),
                        number(2) as libc::c_int,
                    )
                };
                if opened.is_null() {
                    return Err(loader_error(&last_loader_error(), position));
                }
                Ok(Object::Int(opened as i64))
            }
            "symbol" => {
                let name = text(2).unwrap_or_default();
                let wanted =
                    std::ffi::CString::new(name.clone()).map_err(|_| null_byte_error(position))?;
                // SAFETY: the handle came from `dlopen` or is RTLD_DEFAULT,
                // and the name is a C string that lives for the call.
                let found = unsafe { libc::dlsym(number(1) as *mut libc::c_void, wanted.as_ptr()) };
                if found.is_null() {
                    return Err(loader_error(
                        &format!("unknown symbol \"{name}\""),
                        position,
                    ));
                }
                Ok(Object::Int(found as i64))
            }
            "close" => {
                // SAFETY: the handle came from `dlopen`, and the Ruby side
                // closes each one once.
                let closed = unsafe { libc::dlclose(number(1) as *mut libc::c_void) };
                Ok(Object::Int(closed as i64))
            }
            "read" => {
                let length = number(2).max(0) as usize;
                // SAFETY: Fiddle::Pointer reads the memory at an address the
                // program handed it, which is the use Fiddle is for.
                let bytes = unsafe { std::slice::from_raw_parts(number(1) as *const u8, length) };
                let characters = bytes.iter().map(|byte| *byte as char).collect::<String>();
                Ok(Object::String(std::rc::Rc::new(
                    crate::object::StringValue::from_bytes(characters),
                )))
            }
            "string" => Ok(c_string_at(number(1))),
            "call" => self.call_foreign_function(arguments, position),
            "flags" => Ok(Object::array(vec![
                Object::Int(libc::RTLD_GLOBAL as i64),
                Object::Int(libc::RTLD_LAZY as i64),
                Object::Int(libc::RTLD_NOW as i64),
            ])),
            _ => Ok(Object::Int(libc::RTLD_DEFAULT as i64)),
        }
    }
}

/// The type numbers Fiddle names its argument and return types by. A
/// negative number is the unsigned type of the same width.
const TYPE_VOID: i64 = 0;
const TYPE_VOIDP: i64 = 1;
const TYPE_CHAR: i64 = 2;
const TYPE_SHORT: i64 = 3;
const TYPE_INT: i64 = 4;
const TYPE_LONG: i64 = 5;
const TYPE_LONG_LONG: i64 = 6;
const TYPE_FLOAT: i64 = 7;
const TYPE_DOUBLE: i64 = 8;
const TYPE_VARIADIC: i64 = 9;
const TYPE_CONST_STRING: i64 = 10;
const TYPE_BOOL: i64 = 11;
const TYPE_UCHAR: i64 = -TYPE_CHAR;
const TYPE_USHORT: i64 = -TYPE_SHORT;
const TYPE_UINT: i64 = -TYPE_INT;
const TYPE_ULONG: i64 = -TYPE_LONG;
const TYPE_ULONG_LONG: i64 = -TYPE_LONG_LONG;

/// How many integer and how many floating point arguments a call passes.
/// Every argument of both kinds travels in a register of its own kind on
/// x86-64 and arm64, so a call through a function type with this many of
/// each reaches any function taking no more than that.
const ARGUMENT_REGISTERS: usize = 8;

type IntegerRegisters = [i64; ARGUMENT_REGISTERS];
type FloatRegisters = [f64; ARGUMENT_REGISTERS];

type AnswersInteger = unsafe extern "C" fn(
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
) -> i64;
type AnswersFloat = unsafe extern "C" fn(
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
) -> f64;

/// A String handed to a pointer argument: the bytes the function reads and
/// may write, and the string they are written back to afterwards.
struct LentBuffer {
    string: Rc<crate::object::StringValue>,
    bytes: Vec<u8>,
}

/// A single precision float placed in the low half of a double register,
/// which is where a function taking a `float` reads it.
fn single_in_register(value: f64) -> f64 {
    f64::from_bits((value as f32).to_bits() as u64)
}

fn unsigned_integer(value: u64) -> Object {
    match i64::try_from(value) {
        Ok(fits) => Object::Int(fits),
        Err(_) => Object::BigInt(Rc::new(num_bigint::BigInt::from(value))),
    }
}

/// The Ruby value for what a function answered in its integer register.
fn integer_answer(kind: i64, answered: i64) -> Object {
    match kind {
        TYPE_VOID => Object::Nil,
        TYPE_BOOL => Object::Bool(answered as u8 != 0),
        TYPE_CHAR => Object::Int(answered as i8 as i64),
        TYPE_UCHAR => Object::Int(answered as u8 as i64),
        TYPE_SHORT => Object::Int(answered as i16 as i64),
        TYPE_USHORT => Object::Int(answered as u16 as i64),
        TYPE_INT => Object::Int(answered as i32 as i64),
        TYPE_UINT => Object::Int(answered as u32 as i64),
        TYPE_ULONG | TYPE_ULONG_LONG => unsigned_integer(answered as u64),
        _ => Object::Int(answered),
    }
}

impl VirtualMachine {
    /// `Fiddle.__dynamic_library__(:call, address, types, answer_type,
    /// values)` calls the C function at `address`. A String handed to a
    /// pointer argument is lent as a buffer, and what the function wrote
    /// into it is copied back into the String.
    fn call_foreign_function(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let address = match arguments.get(1) {
            Some(Object::Int(held)) if *held != 0 => *held,
            _ => {
                return Err(simple_exception(
                    "Fiddle::DLError",
                    "NULL pointer",
                    position,
                ));
            }
        };
        let kinds = match arguments.get(2) {
            Some(Object::Array(held)) => held.borrow().clone(),
            _ => Vec::new(),
        };
        let answer_kind = match arguments.get(3) {
            Some(Object::Int(held)) => *held,
            _ => TYPE_VOID,
        };
        let values = match arguments.get(4) {
            Some(Object::Array(held)) => held.borrow().clone(),
            _ => Vec::new(),
        };

        let mut integers: IntegerRegisters = [0; ARGUMENT_REGISTERS];
        let mut floats: FloatRegisters = [0.0; ARGUMENT_REGISTERS];
        let mut integer_count = 0;
        let mut float_count = 0;
        let mut lent: Vec<LentBuffer> = Vec::new();
        let too_many = || {
            simple_exception(
                "ArgumentError",
                &format!(
                    "Fiddle::Function passes at most {ARGUMENT_REGISTERS} integer and {ARGUMENT_REGISTERS} floating point arguments"
                ),
                position,
            )
        };
        for (kind, value) in kinds.iter().zip(values.iter()) {
            let kind = match kind {
                Object::Int(held) => *held,
                _ => TYPE_VOIDP,
            };
            match kind {
                TYPE_FLOAT | TYPE_DOUBLE => {
                    let number = match value {
                        Object::Float(held) => *held,
                        Object::Int(held) => *held as f64,
                        other => {
                            return Err(simple_exception(
                                "TypeError",
                                &format!(
                                    "can't convert {} into Float",
                                    self.builtins().class_of(other).name()
                                ),
                                position,
                            ));
                        }
                    };
                    if float_count == ARGUMENT_REGISTERS {
                        return Err(too_many());
                    }
                    floats[float_count] = if kind == TYPE_FLOAT {
                        single_in_register(number)
                    } else {
                        number
                    };
                    float_count += 1;
                }
                TYPE_VARIADIC => {
                    return Err(simple_exception(
                        "NotImplementedError",
                        "Fiddle::Function does not call variadic functions",
                        position,
                    ));
                }
                _ => {
                    let passed = match value {
                        Object::Int(held) => *held,
                        Object::BigInt(held) => {
                            let (_, digits) = held.to_u64_digits();
                            digits.first().copied().unwrap_or(0) as i64
                        }
                        Object::Bool(held) => *held as i64,
                        Object::Nil => 0,
                        Object::String(held) => {
                            let mut bytes =
                                crate::vm::native_methods::string_methods::binary_bytes(held);
                            bytes.push(0);
                            lent.push(LentBuffer {
                                string: held.clone(),
                                bytes,
                            });
                            0
                        }
                        other => {
                            return Err(simple_exception(
                                "TypeError",
                                &format!(
                                    "no implicit conversion of {} into Integer",
                                    self.builtins().class_of(other).name()
                                ),
                                position,
                            ));
                        }
                    };
                    if integer_count == ARGUMENT_REGISTERS {
                        return Err(too_many());
                    }
                    integers[integer_count] = passed;
                    integer_count += 1;
                }
            }
        }
        // The buffers are placed once every argument is read, so none of
        // them moves after its address is taken.
        let mut next_buffer = lent.iter_mut();
        for (slot, value) in integers.iter_mut().zip(
            kinds
                .iter()
                .zip(values.iter())
                .filter(|(kind, _)| !matches!(kind, Object::Int(TYPE_FLOAT | TYPE_DOUBLE))),
        ) {
            if let (_, Object::String(_)) = value
                && let Some(buffer) = next_buffer.next()
            {
                *slot = buffer.bytes.as_mut_ptr() as i64;
            }
        }

        let [a, b, c, d, e, f, g, h] = integers;
        let [s, t, u, v, w, x, y, z] = floats;
        let answer = match answer_kind {
            TYPE_FLOAT | TYPE_DOUBLE => {
                // SAFETY: Fiddle calls the function at the address the
                // program named with the types the program declared, which
                // is the use Fiddle is for. Arguments the function does not
                // take sit in registers it does not read.
                let callee: AnswersFloat = unsafe { std::mem::transmute(address as usize) };
                let answered = unsafe { callee(a, b, c, d, e, f, g, h, s, t, u, v, w, x, y, z) };
                if answer_kind == TYPE_FLOAT {
                    Object::Float(f32::from_bits(answered.to_bits() as u32) as f64)
                } else {
                    Object::Float(answered)
                }
            }
            _ => {
                // SAFETY: as above.
                let callee: AnswersInteger = unsafe { std::mem::transmute(address as usize) };
                let answered = unsafe { callee(a, b, c, d, e, f, g, h, s, t, u, v, w, x, y, z) };
                if answer_kind == TYPE_CONST_STRING {
                    c_string_at(answered)
                } else {
                    integer_answer(answer_kind, answered)
                }
            }
        };

        for buffer in lent {
            let written = &buffer.bytes[..buffer.bytes.len() - 1];
            if buffer.string.is_frozen()
                || written
                    == crate::vm::native_methods::string_methods::binary_bytes(&buffer.string)
            {
                continue;
            }
            if crate::vm::native_methods::string_methods::bytes_are_characters(&buffer.string) {
                buffer.string.replace_text(
                    crate::vm::native_methods::string_methods::bytes_as_text(written),
                );
            } else {
                match String::from_utf8(written.to_vec()) {
                    Ok(text) => buffer.string.replace_text(text),
                    Err(_) => {
                        buffer.string.replace_text(
                            crate::vm::native_methods::string_methods::bytes_as_text(written),
                        );
                        buffer.string.mark_bytes();
                    }
                }
            }
        }
        Ok(answer)
    }
}
