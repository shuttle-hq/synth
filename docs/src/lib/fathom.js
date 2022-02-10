module.exports = function (context) {
  const { siteConfig } = context;
  const { themeConfig } = siteConfig;
  const { fathomAnalytics } = themeConfig || {};

  if (!fathomAnalytics) {
    throw new Error(
      `You need to specify 'fathomAnalytics' object in 'themeConfig' with 'siteId' field in it to use docusaurus-plugin-fathom`
    );
  }

  let { siteId, customDomain = 'https://cdn.usefathom.com' } = fathomAnalytics;

  if (!siteId) {
    throw new Error(
      `You specified the 'fathomAnalytics' object in 'themeConfig' but the 'siteId' field was missing. Please ensure this is not a mistake.`
    );
  }

  const isProd = process.env.NODE_ENV === 'production';

  return {
    name: 'docusaurus-plugin-fathom',

    injectHtmlTags() {
      if (!isProd) {
        return {};
      }

      return {
        headTags: [
          {
            tagName: 'script',
            attributes: {
              defer: true,
              src: `${customDomain}/script.js`,
              spa: 'auto',
              site: siteId,
            },
          },
        ],
      };
    },
  };
};
