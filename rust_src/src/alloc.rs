#![allow(dead_code)]

use libc;

/// An aligned block of memory.
///
/// # C Porting Notes
///
/// This is equivalent to `ablock` in C.
struct AlignedBlock {
    
}

// Constants used by `lisp_align_malloc`, which returns blocks of at
// most `BLOCK_BYTES` and guarantees they are aligned on a `BLOCK_ALIGN`
// boundary.

// Byte alignment of storage blocks.
const BLOCK_ALIGN: libc::c_int = 1 << 10;

/* Padding to leave at the end of a malloc'd block.  This is to give
   malloc a chance to minimize the amount of memory wasted to alignment.
   It should be tuned to the particular malloc library used.
   On glibc-2.3.2, malloc never tries to align, so a padding of 0 is best.
   aligned_alloc on the other hand would ideally prefer a value of 4
   because otherwise, there's 1020 bytes wasted between each ablocks.
   In Emacs, testing shows that those 1020 can most of the time be
   efficiently used by malloc to place other objects, so a value of 0 can
   still preferable unless you have a lot of aligned blocks and virtually
   nothing else.  */
const BLOCK_PADDING: libc::c_int = 0;
const BLOCK_BYTES: libc::c_int = BLOCK_ALIGN; // TODO

