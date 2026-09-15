import { defineConfig, presetUno } from "unocss";

export default defineConfig({
  presets: [presetUno()],
  theme: {
    colors: {
      ink: "#0c0f14",
      surface: "#151a22",
      line: "#2a313d",
      paper: "#f4f1ea",
      muted: "#9aa3b2",
      amber: "#f3a45b",
      coral: "#ff7657",
      teal: "#4dc6b8"
    },
    fontFamily: {
      display: ['"Playfair Display"', "Georgia", "serif"],
      body: ['"Inter"', '"Segoe UI"', "sans-serif"]
    }
  }
});
