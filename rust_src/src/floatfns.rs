extern crate libc;

use std::os::raw::c_char;
use std::ptr;

use lisp::{LispObject, LispSubr, Qnumberp, Qfloatp, EmacsDouble, CHECK_TYPE};

pub fn init_float_syms() {
    unsafe {
        ::defsubr(&*Sisnan);
        ::defsubr(&*Sacos);
        ::defsubr(&*Sasin);
        ::defsubr(&*Satan);
        ::defsubr(&*Scos);
        ::defsubr(&*Ssin);
        ::defsubr(&*Stan);
        ::defsubr(&*Slog);

        ::defsubr(&*Ssqrt);
        ::defsubr(&*Sexp);
        ::defsubr(&*Sffloor);
        ::defsubr(&*Sfceiling);
        ::defsubr(&*Sftruncate);
        ::defsubr(&*Sfloat);
    }
}

/// Either extracts a floating point number from a lisp number (of any kind) or throws an error
/// TODO eventually, this can hopefully go away when we have a better approach for error handling
#[no_mangle]
pub extern "C" fn extract_float(f: LispObject) -> EmacsDouble {
    let d = f.extract_float();
    CHECK_TYPE(d.is_some(), unsafe { Qnumberp }, f);
    match d {
        Some(d) => d,
        None => unreachable!(), // CHECK_TYPE never returns on failure
    }
}

/// checks if the argument is a float, if not, throws an error
/// TODO eventually, this can hopefully go away when we have a better approach for error handling
fn check_float(x: LispObject) {
    CHECK_TYPE(x.to_float().is_some(), unsafe { Qfloatp }, x);
}

/// Calculate the modulus of two elisp floats.
#[no_mangle]
pub extern "C" fn fmod_float(x: LispObject, y: LispObject) -> LispObject {
    let mut f1 = extract_float(x);
    let f2 = extract_float(y);

    f1 %= f2;

    // Ensure that the remainder has the correct sign.
    if f2 < 0.0 && f1 > 0.0 || f2 > 0.0 && f1 < 0.0 {
        f1 += f2
    }

    LispObject::from_float(f1)
}

macro_rules! simple_float_op {
    ($lisp_name:expr, $rust_name:ident, $sname:ident, $float_func:ident, $lisp_docs:expr) => {
        fn $rust_name(x: LispObject) -> LispObject {
            let d = extract_float(x);
            let val = d.$float_func();
            LispObject::from_float(val)
        }

        defun! (
            $lisp_name,
            $rust_name,
            $sname,
            1, 1,
            ptr::null(),
            // explicity set signature, otherwise emacs seems to name the argument ARG1
            concat!($lisp_docs, "

(fn ARG)")
        );
    }
}

simple_float_op!("acos",
                 Facos,
                 Sacos,
                 acos,
                 "Return the inverse cosine of ARG.");
simple_float_op!("asin",
                 Fasin,
                 Sasin,
                 asin,
                 "Return the inverse sine of ARG.");
// atan is special, defined later
simple_float_op!("cos", Fcos, Scos, cos, "Return the cosine of ARG.");
simple_float_op!("sin", Fsin, Ssin, sin, "Return the sine of ARG.");
simple_float_op!("tan", Ftan, Stan, tan, "Return the tangent of ARG.");

simple_float_op!("exp",
                 Fexp,
                 Sexp,
                 exp,
                 "Return the exponential base e of ARG.");
simple_float_op!("sqrt", Fsqrt, Ssqrt, sqrt, "Return the square root of ARG.");

simple_float_op!("fceiling",
                 Ffceiling,
                 Sfceiling,
                 ceil,
                 "Return the smallest integer no less than ARG, as a float.
(Round toward +inf.)");

simple_float_op!("ffloor",
                 Ffloor,
                 Sffloor,
                 floor,
                 "Return the largest integer no greater than ARG, as a float.
(Round towards -inf.)");

fn Fisnan(x: LispObject) -> LispObject {
    check_float(x);
    let d = x.to_float().unwrap();
    LispObject::from_bool(d.is_nan())
}

defun!("isnan",
       Fisnan,
       Sisnan,
       1,
       1,
       ptr::null(),
       "Return non nil if argument X is a NaN.

(fn X)");

fn Fatan(y: LispObject, x: LispObject) -> LispObject {
    let y = extract_float(y);

    if x == LispObject::constant_nil() {
        let val = y.atan();
        return LispObject::from_float(val);
    } else {
        let x = extract_float(x);
        let val = y.atan2(x);
        return LispObject::from_float(val);
    }
}

defun!("atan",
       Fatan,
       Satan,
       1,
       2,
       ptr::null(),
       "Return the inverse tangent of the arguments.
If only one argument Y is given, return the inverse tangent of Y.
If two arguments Y and X are given, return the inverse tangent of Y
divided by X, i.e. the angle in radians between the vector (X, Y)
and the x-axis

(fn Y &optional X)");

fn Flog(arg: LispObject, base: LispObject) -> LispObject {
    let mut d = extract_float(arg);

    if base == LispObject::constant_nil() {
        d = d.ln()
    } else {
        let base = extract_float(base);
        if base == 10.0 {
            d = d.log10();
        } else if base == 2.0 {
            d = d.log2();
        } else {
            d = d.log(base);
        }
    }

    LispObject::from_float(d)
}

defun!("log",
       Flog,
       Slog,
       1,
       2,
       ptr::null(),
       "Return the natural logarithm of ARG.
If the optional argument BASE is given, return log ARG using that base.

(fn ARG &optional BASE)");

fn Fftruncate(x: LispObject) -> LispObject {
    let d = extract_float(x);
    if d > 0.0 {
        return LispObject::from_float(d.floor());
    } else {
        return LispObject::from_float(d.ceil());
    }
}

defun!("ftruncate",
       Fftruncate,
       Sftruncate,
       1,
       1,
       ptr::null(),
       "Truncate a floating point number to an integral float value.
Rounds the value toward zero.

(fn ARG)");

fn Ffloat(obj: LispObject) -> LispObject {
    CHECK_TYPE(obj.is_number(), unsafe { Qnumberp }, obj); // does not return on failure

    if obj.is_float() {
        return obj;
    }

    match obj.to_fixnum() {
        Some(int) => LispObject::from_float(int as EmacsDouble),
        None => unreachable!(),
    }
}

defun!("float",
       Ffloat,
       Sfloat,
       1,
       1,
       ptr::null(),
       "Return the floating point number equal to ARG.

(fn ARG)");
