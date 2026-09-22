//! The Duet time kernel.
//!
//! It declares the beat domain and the audio domain, the position and span
//! types, the tempo map, and the one module of the workspace that holds an
//! `as` cast. Section 2 of `roadmap/duet-v1/architecture.md` states the
//! design.

#![forbid(unsafe_code)]
