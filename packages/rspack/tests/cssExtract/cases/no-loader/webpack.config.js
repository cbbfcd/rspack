const { CssExtractRspackPlugin } = require("../../../../");

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: ["css-loader"]
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
