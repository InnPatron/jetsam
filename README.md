# jetsam
* Typescript binding generator
* Reads '.d.ts' starting from a root ECMA file and outputs a corresponding '.arr.js' and '.arr.json' file

See [this GitHub repo](https://github.com/InnPatron/jetsam-paper) for the soundness proofs of the generated bindings (NOT PEER REVIEWED; PROBABLY HAS MANY MISTAKES).

## Building
Requires [Rust](https://www.rust-lang.org/) (tested on 1.41 stable) and Cargo

## Usage

Basic run command: `jetsam -i ROOT_MODULE_D_TS -o OUTUPT_DIR`

Notable options:
* `-t`: change the accepted TypeScript subset
* `--require-path`: change the path to the binding's TS/JS implementation at runtime

See `jetsam --help` for more info

## Supported TypeScript Subsets

### TS-NUM
#### Types T:
  * `number`
    * TS numbers (double-precision 64-bit binary format IEEE 754)
  * `T_0 ... T_n -> T_result`
    * Function types with arbitrary number of arguments

#### Code Generation
* Automatically emits lossy conversion wrappers between TS numbers and Pyret numbers
* Emits self-contained bindings
  * Numeric conversion routines are emitted in place
* Option: `--wrap-top-level-vars`
  * Wrap exported variables in a getter function. Useful if expecting the variable to change during the course of execution
  * Otherwise, export the result of a snapshot of the value after conversion
    * NO GUARANTEES OF WHEN SNAPSHOT OCCURS

## Running the Tests

* Need to set `PYRET_COMPILER_DIR` to a built Pyret compiler (`pyret.jarr`)
  * Need a specific Pyret anchor compiler (until commits are merged into trunk)
    * https://github.com/InnPatron/pyret-lang
    * `anchor` branch
    * Commit: `e557b96eda677b5425fb88b72c8c02620156e7cf`
  * Runtime files are located in `$PYRET_COMPILER_DIR/../runtime/`
    * Alternatively, set `PYRET_RUNTIME_DIR`
* Need `node` in your `PATH`
  * Alternatively, set `NODE_PATH`
* Run `cargo test`
* All tests should pass
