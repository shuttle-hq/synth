module.exports = {
    title: 'Synth - Documentation',
    tagline: 'Easy data generation',
    url: 'https://openquery-io.github.io/synth',
    baseUrl: '/synth/',
    onBrokenLinks: 'warn',
    onBrokenMarkdownLinks: 'warn',
    favicon: 'img/getsynth_favicon.png',
    organizationName: 'openquery-io', // Usually your GitHub org/user name.
    projectName: 'synth', // Usually your repo name.
        plugins: [require.resolve('docusaurus-plugin-fathom')],
    themeConfig: {
        fathomAnalytics: {
            siteId: 'ASXTKXUJ',
        },
        algolia: {
            apiKey: 'b0583a1f7732cee4e8c80f4a86adf57c',
            indexName: 'synth',
        },
        navbar: {
            title: 'Synth',
            logo: {
                alt: 'Synth',
                src: 'img/getsynth_identicon.png',
            },
            items: [
                {
                    to: '/getting_started/synth',
                    activeBasePath: 'getting_started',
                    label: 'Getting Started',
                    position: 'left',
                },
                {
                    to: '/examples/bank',
                    activeBasePath: 'examples',
                    label: 'Examples',
                    position: 'left',
                },
                {
                    to: '/content/null',
                    activeBasePath: 'content',
                    label: 'Generators',
                    position: 'left',
                },
		{
		    to: 'blog',
		    label: 'Blog',
		    position: 'left'
		},
                {
                    href: 'https://github.com/openquery-io/synth',
                    label: 'GitHub',
                    position: 'right',
                },
            ],
        },
        footer: {
            style: 'dark',
            links: [
                {
                    title: 'Docs',
                    items: [
                        {
                            to: '/getting_started/synth',
                            label: 'Getting Started',
                        },
                        {
                            to: '/examples/bank',
                            label: 'Examples',
                        },
                        {
                            to: '/content/null',
                            label: 'Generators',
                        },
                    ],
                },
            ],
            copyright: `Copyright © ${new Date().getFullYear()} OpenQuery.`,
        },
    },
    presets: [
        [
            '@docusaurus/preset-classic',
            {
                docs: {
                    routeBasePath: '/',
                    sidebarPath: require.resolve('./sidebars.js'),
                    // Please change this to your repo.
                    editUrl:
                        'https://github.com/openquery-io/synth/edit/master/docs/',
                },
                theme: {
                    customCss: require.resolve('./src/css/custom.css')
                },
            },
        ],
    ],
};
