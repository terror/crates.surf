import defaultTheme from 'tailwindcss/defaultTheme';

module.exports = {
  content: ['./client/**/*.{html,js,svelte,ts}'],
  theme: {
    fontFamily: {
      sans: ['Inter', ...defaultTheme.fontFamily.sans],
    },
  },
  plugins: [],
};
