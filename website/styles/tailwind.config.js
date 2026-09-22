/** @type {import('tailwindcss').Config} */
module.exports = {
  theme: {
    extend: {
      // Configure typography styles provided by the @tailwindcss/typography plugin
      typography: ({ theme }) => ({
        DEFAULT: {
          css: {
            '--tw-prose-body': theme('colors.on-background'),
            '--tw-prose-headings': theme('colors.on-background'),
            '--tw-prose-links': theme('colors.primary'),
            '--tw-prose-bold': theme('colors.on-background'),
            '--tw-prose-counters': theme('colors.on-surface-variant'),
            '--tw-prose-bullets': theme('colors.on-surface-variant'),
            '--tw-prose-hr': theme('colors.outline'),
            '--tw-prose-quotes': theme('colors.on-background'),
            '--tw-prose-quote-borders': theme('colors.primary'),
            '--tw-prose-captions': theme('colors.on-surface-variant'),
            '--tw-prose-code': theme('colors.on-surface-variant'),
            '--tw-prose-pre-code': theme('colors.on-surface'),
            '--tw-prose-pre-bg': theme('colors.surface-variant'),
            '--tw-prose-invert-body': theme('colors.on-background'),
            '--tw-prose-invert-headings': theme('colors.on-background'),
            '--tw-prose-invert-links': theme('colors.primary'),
            '--tw-prose-invert-bold': theme('colors.on-background'),
            '--tw-prose-invert-counters': theme('colors.on-surface-variant'),
            '--tw-prose-invert-bullets': theme('colors.on-surface-variant'),
            '--tw-prose-invert-hr': theme('colors.outline'),
            '--tw-prose-invert-quotes': theme('colors.on-background'),
            '--tw-prose-invert-quote-borders': theme('colors.primary'),
            '--tw-prose-invert-captions': theme('colors.on-surface-variant'),
            '--tw-prose-invert-code': theme('colors.on-surface-variant'),
            '--tw-prose-invert-pre-code': theme('colors.on-surface'),
            '--tw-prose-invert-pre-bg': theme('colors.surface-variant'),
          }
        }
      })
    }
  }
}