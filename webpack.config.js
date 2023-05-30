module.exports = {
    entry: "./src/index.js",
    output: {
        path: __dirname,
        filename: "renderer.js",
        libraryTarget: "umd",
        library: "Renderer",
        umdNamedDefine: true
    }
}