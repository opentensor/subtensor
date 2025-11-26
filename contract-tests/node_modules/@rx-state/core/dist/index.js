"use strict"

if (process.env.NODE_ENV === "production") {
  module.exports = require("./rxstate.core.cjs.production.min.js")
} else {
  module.exports = require("./rxstate.core.cjs.development.js")
}
