/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        risk: {
          green: "#2f9e44",
          yellow: "#e8b339",
          orange: "#e8590c",
          red: "#c92a2a",
          black: "#1a1a1a",
        },
      },
    },
  },
  plugins: [],
};
