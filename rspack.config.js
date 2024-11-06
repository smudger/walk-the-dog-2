const path = require("path");
const rspack = require("@rspack/core");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  mode: "development",
  experiments: {
    asyncWebAssembly: true,
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name].js",
  },
  entry: {
    index: "./js/index.js",
  },
  devServer: {
    static: path.resolve(__dirname, "dist"),
    hot: false,
  },
  plugins: [
    new rspack.CopyRspackPlugin({
      patterns: [path.resolve(__dirname, "static")]
    }),

    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
  ],
};
