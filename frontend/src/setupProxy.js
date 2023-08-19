const { createProxyMiddleware } = require('http-proxy-middleware');

// This proxy is only active in development
module.exports = function (app) {
  app.use(
    '/api',
    createProxyMiddleware({
      target: "http://127.0.0.1:8080",
      changeOrigin: true,
      pathRewrite: {
          '^/api': '', // Remove the /api prefix
      }
    })
  );
};

