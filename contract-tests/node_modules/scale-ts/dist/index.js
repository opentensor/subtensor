"use strict"

if (process.env.NODE_ENV === "production") {
  module.exports = require("./scale-ts.cjs.production.min.js")
} else {
  module.exports = require("./scale-ts.cjs.development.js")
}
