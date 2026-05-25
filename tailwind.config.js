/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'crayon-green': '#6BCB77',
        'crayon-pink': '#FF9AA2',
        'crayon-blue': '#B2E2F2',
        'crayon-purple': '#C5A3FF',
        'crayon-yellow': '#FFF599',
        'crayon-bg': '#FEF9F3',
      },
      fontFamily: {
        'hand': ['"Patrick Hand"', '"Ma Shan Zheng"', 'cursive'],
      },
      borderRadius: {
        'doodle': '255px 15px 225px 15px/15px 225px 15px 255px',
      },
    },
  },
  plugins: [],
  darkMode: 'class',
}
