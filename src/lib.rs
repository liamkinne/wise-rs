#![doc = include_str!("../README.md")]

use progenitor::generate_api;

generate_api!(spec = { path = "index.patched.json", relative_to = OutDir });
