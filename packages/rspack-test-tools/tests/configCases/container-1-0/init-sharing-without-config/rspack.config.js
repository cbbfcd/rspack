const { ModuleFederationPluginV1: ModuleFederationPlugin } =
	require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [new ModuleFederationPlugin({})]
};
