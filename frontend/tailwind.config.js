/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        ink: {
          DEFAULT: "#0f0f0f",
          muted: "#606060",
          soft: "#909090",
        },
        paper: {
          DEFAULT: "#f8f8f8",
          deep: "#f1f1f1",
        },
        surface: "#ffffff",
        line: {
          DEFAULT: "#e5e5e5",
          strong: "#d3d3d3",
        },
        accent: {
          DEFAULT: "#ff0033",
          hover: "#e6002e",
          soft: "#fff0f3",
        },
        ok: {
          DEFAULT: "#065fd4",
          soft: "#e8f0fe",
        },
        warn: {
          DEFAULT: "#e37400",
          soft: "#fff4e5",
        },
        danger: {
          DEFAULT: "#d93025",
          soft: "#fce8e6",
        },
      },
      fontFamily: {
        sans: ['"Figtree"', "ui-sans-serif", "sans-serif"],
        display: ['"Figtree"', "ui-sans-serif", "sans-serif"],
      },
      borderRadius: {
        yt: "12px",
        "yt-sm": "8px",
      },
      boxShadow: {
        yt: "0 1px 2px rgba(0,0,0,0.08), 0 2px 12px rgba(0,0,0,0.06)",
        "yt-lg": "0 4px 24px rgba(0,0,0,0.08)",
      },
      keyframes: {
        rise: {
          from: { opacity: "0", transform: "translateY(12px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
        shimmer: {
          "0%": { backgroundPosition: "200% 0" },
          "100%": { backgroundPosition: "-200% 0" },
        },
        "soft-pulse": {
          "0%, 100%": { transform: "scale(1)", opacity: "0.9" },
          "50%": { transform: "scale(1.04)", opacity: "1" },
        },
      },
      animation: {
        rise: "rise 0.55s ease-out both",
        "rise-delay": "rise 0.65s ease-out 0.06s both",
        shimmer: "shimmer 1.6s linear infinite",
        "soft-pulse": "soft-pulse 2.4s ease-in-out infinite",
      },
    },
  },
  plugins: [],
};
