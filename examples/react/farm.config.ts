import type { UserConfig } from "@farmfe/core";

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  plugins: [
    "@farmfe/plugin-react",
    "@farmfe/plugin-sass",
    // {
    //   name: "one-plugin",
    //   priority: 700,
    //   config(config, env) {
    //   },
    //   configResolved(config) {
    //   },
    // },
  ],
  // vitePlugins: [
  //   {
  //     name: "virtual-module",
  //     config(config) {
  //       config.a = 123123;
  //     },
  //   },
  //   {
  //     name: "virtual-module2",
  //     config(config) {
  //       config.b = 456456;
  //     },
  //   },
  // ],
});
