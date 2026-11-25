import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://edithatogo.github.io',
  base: '/paignia',
  integrations: [
    starlight({
      title: 'Paignia',
      favicon: '/favicon.png',
      description: 'Rust game-theory methods and verified model exchange.',
      customCss: ['./src/styles/site.css'],
      social: [
        { icon: 'github', label: 'Paignia on GitHub', href: 'https://github.com/edithatogo/paignia' },
      ],
      sidebar: [
        { label: 'Start here', link: '/start/' },
        { label: 'Model and solve', items: [
          { label: 'Static games and chance', link: '/static-games/' },
          { label: 'RDF and JSON interchange', link: '/interchange/' },
          { label: 'Nash and information sets', link: '/equilibria/' },
          { label: 'Normal-form games', link: '/normal-form/' },
          { label: 'Information-set policies', link: '/information-sets/' },
        ] },
        { label: 'Application examples', link: '/applications/' },
        { label: 'Release and reference', link: '/reference/' },
      ],
    }),
  ],
});
