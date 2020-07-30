# jetsam
* Typescript binding generator
* Reads '.d.ts' starting from a root ECMA file and outputs a corresponding '.arr.js' and '.arr.json' file
* Currently emits non-primitive types as opaque data structures (no variants)
* Currently does NOT wrap JS code to guard against Pyret numbers

## Building
Requires [Rust](https://www.rust-lang.org/) (tested on 1.41 stable) and Cargo

## Usage

`jetsam -i ROOT_MODULE_D_TS -o OUTUPT_DIR`

See `jetsam --help` for more info

## Running the Tests

* Need to set `PYRET_COMPILER_DIR` to a built Pyret compiler (`pyret.jarr`)
  * Need a specific Pyret anchor compiler (until commits are merged into trunk)
    * https://github.com/InnPatron/pyret-lang
    * `anchor` branch
    * Commit: e557b96eda677b5425fb88b72c8c02620156e7cf
  * Runtime files are located in `$PYRET_COMPILER_DIR/../runtime/`
    * Alternatively, set `PYRET_RUNTIME_DIR`
* Need `node` in your `PATH`
  * Alternatively, set `NODE_PATH`
* Run `cargo test`
* All tests should pass
