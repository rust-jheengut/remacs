extern crate libc;

use std::os::raw::c_char;
use std::ptr;
use std::mem;

use lisp::{CHECK_TYPE, LispObject, LispType, XTYPE, XUNTAG, Qnil, LispSubr, PvecType, EmacsInt, VectorLikeHeader,
           PSEUDOVECTOR_AREA_BITS, wrong_type_argument};
use globals::globals;

extern "C" {
    static Qconsp: LispObject;
    fn CHECK_IMPURE(obj: LispObject, ptr: *const libc::c_void);
    static Qlistp: LispObject;
}


pub fn CONSP(x: LispObject) -> bool {
    XTYPE(x) == LispType::Lisp_Cons
}

fn Fatom(object: LispObject) -> LispObject {
    if CONSP(object) {
        Qnil
    } else {
        LispObject::constant_t()
    }
}

defun!("atom",
       Fatom,
       Satom,
       1,
       1,
       ptr::null(),
       "Return t if OBJECT is not a cons cell.  This includes nil.");

fn Fconsp(object: LispObject) -> LispObject {
    if CONSP(object) {
        LispObject::constant_t()
    } else {
        Qnil
    }
}

defun!("consp",
       Fconsp,
       Sconsp,
       1,
       1,
       ptr::null(),
       "Return t if OBJECT is a cons cell.

(fn OBJECT)");

/// Represents a cons cell, or GC bookkeeping for cons cells.
///
/// A cons cell is pair of two pointers, used to build linked lists in
/// lisp.
///
/// # C Porting Notes
///
/// The equivalent C struct is `Lisp_Cons`. Note that the second field
/// may be used as the cdr or GC bookkeeping.
// TODO: this should be aligned to 8 bytes.
#[repr(C)]
#[allow(unused_variables)]
struct LispCons {
    /// Car of this cons cell.
    car: LispObject,
    /// Cdr of this cons cell, or the chain used for the free list.
    cdr: LispObject,
}

// alloc.c uses a union for `Lisp_Cons`, which we emulate with an
// opaque struct.
#[repr(C)]
#[allow(dead_code)]
pub struct LispConsChain {
    chain: *mut LispConsChain,
}

/// Extract the LispCons data from an elisp value.
fn XCONS(a: LispObject) -> *mut LispCons {
    debug_assert!(CONSP(a));
    unsafe { mem::transmute(XUNTAG(a, LispType::Lisp_Cons)) }
}

/// Set the car of a cons cell.
fn XSETCAR(c: LispObject, n: LispObject) {
    let cons_cell = XCONS(c);
    unsafe {
        (*cons_cell).car = n;
    }
}

/// Set the cdr of a cons cell.
fn XSETCDR(c: LispObject, n: LispObject) {
    let cons_cell = XCONS(c);
    unsafe {
        (*cons_cell).cdr = n;
    }
}

/// Create a LispObject that's a tagged pointer to this cons cell
/// pointer.
unsafe fn XSETCONS(ptr: *mut libc::c_void) -> LispObject {
    make_lisp_ptr(ptr, LispType::Lisp_Cons)
}

#[no_mangle]
pub extern "C" fn Fsetcar(cell: LispObject, newcar: LispObject) -> LispObject {
    unsafe {
        CHECK_TYPE(CONSP(cell), Qconsp, cell);
        CHECK_IMPURE(cell, XCONS(cell) as *const libc::c_void);
    }

    XSETCAR(cell, newcar);
    newcar
}

defun!("setcar",
       Fsetcar,
       Ssetcar,
       2,
       2,
       ptr::null(),
       "Set the car of CELL to be NEWCAR. Returns NEWCAR.

(fn CELL NEWCAR)");

#[no_mangle]
pub extern "C" fn Fsetcdr(cell: LispObject, newcar: LispObject) -> LispObject {
    unsafe {
        CHECK_TYPE(CONSP(cell), Qconsp, cell);
        CHECK_IMPURE(cell, XCONS(cell) as *const libc::c_void);
    }

    XSETCDR(cell, newcar);
    newcar
}

defun!("setcdr",
       Fsetcdr,
       Ssetcdr,
       2,
       2,
       ptr::null(),
       "Set the cdr of CELL to be NEWCDR.  Returns NEWCDR.

(fn CELL NEWCDR)");

/// Is `object` nil?
pub fn NILP(object: LispObject) -> bool {
    object == Qnil
}

unsafe fn XCAR(object: LispObject) -> LispObject {
    (*XCONS(object)).car
}

unsafe fn XCDR(object: LispObject) -> LispObject {
    (*XCONS(object)).cdr
}

/// Take the car/cdr of a cons cell, or signal an error if it's a
/// different type.
///
/// # Porting Notes
///
/// This is equivalent to `CAR`/`CDR` in C code.
fn car(object: LispObject) -> LispObject {
    if CONSP(object) {
        unsafe { XCAR(object) }
    } else if NILP(object) {
        Qnil
    } else {
        unsafe { wrong_type_argument(Qlistp, object) }
    }
}
fn cdr(object: LispObject) -> LispObject {
    if CONSP(object) {
        unsafe { XCDR(object) }
    } else if NILP(object) {
        Qnil
    } else {
        unsafe { wrong_type_argument(Qlistp, object) }
    }
}

#[no_mangle]
pub extern "C" fn Fcar(list: LispObject) -> LispObject {
    car(list)
}

defun!("car",
       Fcar,
       Scar,
       1,
       1,
       ptr::null(),
       "Return the car of LIST.  If arg is nil, return nil.
Error if arg is not nil and not a \
        cons cell.  See also `car-safe'.

See Info node `(elisp)Cons Cells' for a discussion of \
        related basic
Lisp concepts such as car, cdr, cons cell and list.

(fn LIST)");

#[no_mangle]
pub extern "C" fn Fcdr(list: LispObject) -> LispObject {
    cdr(list)
}

defun!("cdr",
       Fcdr,
       Scdr,
       1,
       1,
       ptr::null(),
       "Return the cdr of LIST.  If arg is nil, return nil.
Error if arg is not nil and not a \
        cons cell.  See also `cdr-safe'.

See Info node `(elisp)Cons Cells' for a discussion of \
        related basic
Lisp concepts such as cdr, car, cons cell and list.

(fn LIST)");

#[no_mangle]
pub extern "C" fn Flistp(object: LispObject) -> LispObject {
    if CONSP(object) || NILP(object) {
        LispObject::constant_t()
    } else {
        Qnil
    }
}

defun!("listp",
       Flistp,
       Slistp,
       1,
       1,
       ptr::null(),
       "return t if OBJECT is a list, that is a cons cell or nil, Otherwise, return nil.

(fn \
        OBJECT)");

fn Fnlistp(object: LispObject) -> LispObject {
    if CONSP(object) || NILP(object) {
        Qnil
    } else {
        LispObject::constant_t()
    }
}

defun!("nlistp",
       Fnlistp,
       Snlistp,
       1,
       2,
       ptr::null(),
       "Return t if OBJECT is not a list.  Lists include nil.

(fn OBJECT)");

// When scanning the C stack for live Lisp objects, Emacs keeps track of
// what memory allocated via lisp_malloc and lisp_align_malloc is intended
// for what purpose.  This enumeration specifies the type of memory.
//
// # Porting Notes
//
// `mem_type` in C.
#[repr(u8)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
enum MemType {
    MEM_TYPE_NON_LISP,
    MEM_TYPE_BUFFER,
    MEM_TYPE_CONS,
    MEM_TYPE_STRING,
    MEM_TYPE_MISC,
    MEM_TYPE_SYMBOL,
    MEM_TYPE_FLOAT,
    // Since all non-bool pseudovectors are small enough to be
    // allocated from vector blocks, this memory type denotes
    // large regular vectors and large bool pseudovectors.
    MEM_TYPE_VECTORLIKE,
    // Special type to denote vector blocks.
    MEM_TYPE_VECTOR_BLOCK,
    // Special type to denote reserved memory.
    MEM_TYPE_SPARE,
}

extern "C" {
    /// Construct a LispObject from a value or address.
    ///
    /// # Porting Notes
    ///
    /// This function also replaces the C macros `XSETCONS`,
    /// `XSETVECTOR`, `XSETSTRING`, `XSETFLOAT` and `XSETMISC`.
    fn make_lisp_ptr(ptr: *mut libc::c_void, ty: LispType) -> LispObject;
    fn lisp_align_malloc(nbytes: libc::size_t, ty: MemType) -> *mut libc::c_void;
    /// Free-list of Lisp_Cons structures.
    static mut cons_free_list: *mut LispConsChain;
    static mut consing_since_gc: EmacsInt;
    static mut total_free_conses: EmacsInt;
    // Current cons_block.
    static mut cons_block: *mut ConsBlock;
    // Index of first unused Lisp_Cons in the current block.
    static mut cons_block_index: libc::c_int;
}

// TODO: find actual value for this.
const CONS_BLOCK_SIZE: libc::c_int = 100;

/// An unsigned integer type representing a fixed-length bit sequence,
/// suitable for bool vector words, GC mark bits, etc.
#[allow(non_camel_case_types)]
type bits_word = libc::size_t;

const BITS_PER_BITS_WORD: libc::c_int = 8 * 8;

/// The ConsBlock is used to store cons cells.
///
/// We allocate new ConsBlock values when needed. Cons cells reclaimed
/// by GC are put on a free list to be reallocated before allocating
/// any new cons cells from the latest ConsBlock.
///
/// # Porting Notes
///
/// This is `cons_block` in C.
#[repr(C)]
struct ConsBlock {
    conses: [LispCons; CONS_BLOCK_SIZE as usize],
    gcmarkbits: [bits_word; (1 + CONS_BLOCK_SIZE / BITS_PER_BITS_WORD) as usize],
    next: *mut ConsBlock,
}

fn Fcons(car: LispObject, cdr: LispObject) -> LispObject {
    // MALLOC_BLOCK_INPUT; is a no-op.

    let val: LispObject;
    unsafe {
        if !cons_free_list.is_null() {
            // Use the current head of the free list for this cons
            // cell, and remove it from the free list.
            val = XSETCONS(cons_free_list as *mut libc::c_void);
            cons_free_list = (*cons_free_list).chain;
        } else {
            // Otherwise, we need to malloc some memory.
            if cons_block_index == CONS_BLOCK_SIZE {
                let new: *mut ConsBlock = lisp_align_malloc(mem::size_of::<*mut ConsBlock>(), MemType::MEM_TYPE_CONS) as *mut ConsBlock;
                libc::memset(new as *mut libc::c_void,
                             0, mem::size_of_val(&(*new).gcmarkbits));
                (*new).next = cons_block;
                cons_block = new;
                cons_block_index = 0;
                total_free_conses += CONS_BLOCK_SIZE as EmacsInt;
            }

            let new_cons_cell_ptr = &mut (*cons_block).conses[cons_block_index as usize] as *mut LispCons;
            val = XSETCONS(new_cons_cell_ptr as *mut libc::c_void);
            cons_block_index += 1;
        }
    }

    XSETCAR(val, car);
    XSETCDR(val, cdr);

    // debug_assert!(CONS_MARKED_P(XCONS(val)));

    unsafe {
        consing_since_gc += mem::size_of::<LispCons>() as i64;
        total_free_conses -= 1;
        globals.f_cons_cells_consed += 1;
    }

    val
}

lazy_static! {
    pub static ref Scons: LispSubr = LispSubr {
        header: VectorLikeHeader {
            size: ((PvecType::PVEC_SUBR as libc::c_int) <<
                   PSEUDOVECTOR_AREA_BITS) as libc::ptrdiff_t,
        },
        function: (Fcons as *const libc::c_void),
        min_args: 2,
        max_args: 2,
        symbol_name: ("rust-cons\0".as_ptr()) as *const c_char,
        intspec: ptr::null(),
        doc: ("Create a new cons, give it CAR and CDR as components, and return it.

(fn CAR CDR)\0".as_ptr()) as *const c_char,
    };
}
