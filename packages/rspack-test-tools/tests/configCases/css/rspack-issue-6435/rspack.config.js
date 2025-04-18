const path = require("path");

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	mode: "development",
	entry: "./index.js",
	output: {
		hashFunction: "md4",
		hashDigestLength: 20
	},
	module: {
		parser: {
			"css/auto": {
				namedExports: true
			}
		},
		generator: {
			"css/auto": {
				exportsConvention: "as-is",
				localIdentName: "[hash]-[local]",
				exportsOnly: true
			}
		},
		rules: [
			{
				include: path.resolve(__dirname, "legacy"),
				test: /\.css$/,
				type: "css/module",
				parser: {
					namedExports: false
				},
				generator: {
					exportsConvention: "camel-case",
					localIdentName: "[hash]-[local]"
				}
			}
		]
	},
	experiments: {
		css: true
	}
};
