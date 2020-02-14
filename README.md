# jetsam
* Typescript binding generator
* Reads '.d.ts' starting from a root ECMA file and outputs a corresponding '.arr.js' and '.arr.json' file
* Currently emits non-primitive types as opaque data structures (no variants)
* Currently does NOT wrap JS code to guard against Pyret numbers

## Building
Requires Rust (tested on 1.41 stable)
