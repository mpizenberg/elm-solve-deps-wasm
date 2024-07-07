#!/usr/bin/env node
const process = require("process");
const DependencyProvider = require("./Solver/DependencyProvider.js");

const elmJsonStr = `
{
    "type": "package",
    "name": "ianmackenzie/elm-units-interval",
    "summary": "Version of elm-interval based on elm-units",
    "license": "MPL-2.0",
    "version": "2.3.0",
    "exposed-modules": [
        "Quantity.Interval",
        "Temperature.Interval",
        "Angle.Interval"
    ],
    "elm-version": "0.19.0 <= v < 0.20.0",
    "dependencies": {
        "elm/core": "1.0.0 <= v < 2.0.0",
        "elm/random": "1.0.0 <= v < 2.0.0",
        "ianmackenzie/elm-float-extra": "1.1.0 <= v < 2.0.0",
        "ianmackenzie/elm-units": "2.7.0 <= v < 3.0.0"
    },
    "test-dependencies": {
        "elm-explorations/test": "1.1.0 <= v < 2.0.0"
    }
}
`;

function solveTestDependencies(elmJson /*: string */) /*: string */ {
  const useTest = true;
  const extra = {
    "elm/core": "1.0.0 <= v < 2.0.0",
    "elm/json": "1.0.0 <= v < 2.0.0",
    "elm/time": "1.0.0 <= v < 2.0.0",
    "elm/random": "1.0.0 <= v < 2.0.0",
  };
  try {
    return DependencyProvider.solveOffline(elmJson, useTest, extra);
  } catch (_) {
    console.warn("Offline solver failed, switching to online solver.");
    return DependencyProvider.solveOnline(elmJson, useTest, extra);
  }
}

console.log("Solution for test dependencies:");
console.log(JSON.parse(solveTestDependencies(elmJsonStr)));
// Explicit exit required because the solver starts
// a worker to handle http requests synchronously.
process.exit();
