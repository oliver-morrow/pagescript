import { defineConfig } from "cf/config";

export default defineConfig({
  worker: {
    name: "pagescript",
    compatibilityDate: "2026-11-20",
    domains: ["pagescript.olivermorrow.com"],
    workersDev: false,
    previewUrls: false,
  },
});
