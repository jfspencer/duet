//! The Duet signal kernels.
//!
//! A signal block allocates nothing and locks nothing, and one peak file
//! format serves every reader. Sections 5.5, 5.10, and 7.3 of
//! `roadmap/duet-v1/architecture.md` state the design.

#![forbid(unsafe_code)]
