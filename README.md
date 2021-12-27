# Dependency solver for Elm, made in WebAssembly

This repo holds a dependency solver for the elm ecosystem compiled to a WebAssembly module.
The wasm module is published on npm, so you can easily use it in your JS projects with:

```js
let wasm = require("elm-solve-deps-wasm");
wasm.init();
let use_test = false; // solve for normal dependencies, not test dependencies
let additional_constraints = {}; // no additional package needed
let solution = wasm.solve_deps(
  elm_json_config, // the elm.json that we have to solve
  use_test,
  additional_constraints,
  fetchElmJson, // user defined (cf example/dependency-provider-offline.js)
  listAvailableVersions // user defined (cf example/dependency-provider-offline.js)
);
```
