const isTargetVercel = () => {
    return process.env["VERCEL"] === '1'
}

module.exports = {
    title: 'Synth - Documentation',
    tagline: 'Easy data generation',
    url: isTargetVercel() ? 'https://www.getsynth.com/docs' : 'https://getsynth.github.io/synth',
    baseUrl: isTargetVercel() ? '/docs/' : '/synth/',
    onBrokenLinks: 'warn',
    onBrokenMarkdownLinks: 'warn',
    favicon: '/img/getsynth_favicon.png',
    organizationName: 'getsynth', // Usually your GitHub org/user name.
    projectName: 'synth', // Usually your repo name.
    customFields: {
        blogTitle: "Synth - Blog"
    },
    plugins: [
        require.resolve('docusaurus-plugin-fathom'),
        [
            "@papercups-io/docusaurus-plugin",
            {
                accountId: '41ff5b3d-e2c2-42ed-bed3-ef7a6c0dde62',
                title: 'Welcome to Synth',
                subtitle: 'Ask us anything in the chat window below 😊',
                newMessagePlaceholder: 'Start typing...',
                primaryColor: '#00dab8',
                greeting: '',
                requireEmailUpfront: false,
                showAgentAvailability: false,
            },
        ]
    ],
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
                src: '/img/getsynth_identicon.png',
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
                    href: 'https://github.com/getsynth/synth',
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
                        'https://github.com/getsynth/synth/edit/master/docs/',
                },
                theme: {
                    customCss: require.resolve('./src/css/custom.css')
                },
            },
        ],
    ],
};
