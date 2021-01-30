module.exports = {
    title: 'Synth - Documentation',
    tagline: 'Easy data generation',
    url: 'https://openquery-io.github.io/synth',
    baseUrl: '/',
    onBrokenLinks: 'warn',
    onBrokenMarkdownLinks: 'warn',
    favicon: 'img/getsynth_favicon.png',
    organizationName: 'openquery-io', // Usually your GitHub org/user name.
    projectName: 'synth', // Usually your repo name.
    themeConfig: {
	navbar: {
	    title: 'Docs',
	    logo: {
		alt: 'Synth',
		src: 'img/getsynth_identicon.png',
	    },
	    items: [
		{
		    to: '/',
		    activeBasePath: '/',
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
		    to: '/schema',
		    activeBasePath: 'content',
		    label: 'Schema',
		    position: 'left',
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
			    to: '/',
			    label: 'Getting Started',
			},
			{
			    to: '/examples/bank',
			    label: 'Examples',
			},
			{
			    to: '/schema',
			    label: 'Schema',
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
				 'https://github.com/facebook/docusaurus/edit/master/website/',
		},
		theme: {
		    customCss: require.resolve('./src/css/custom.css'),
		},
	    },
	],
    ],
};
