import { defineConfig } from "orval";

export default defineConfig({
  "reddit-api": {
    input: {
      target: "../backend/static/openapi.json",
    },
    output: {
      target: "./src/generated/api.ts",
      client: "react-query",
      mode: "split",
      schemas: "./src/generated/schemas",
      override: {
        mutator: {
          path: "./src/lib/api-client.ts",
          name: "customInstance",
        },
      },
    },
    hooks: {
      afterAllFilesWrite: "prettier --write",
    },
  },
});
