/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['*.html', './src/**/*.rs',],
  theme: {
    extend: {
      fontFamily: {
        poppins: ['Poppins', 'sans-serif'],
      },
      colors: {
        'yellow-header': '#ffcc03',
      },
      dropShadow: {
        'header': ['-1px 1px 2px #000']
      }
    },
  },
  plugins: [],
}
