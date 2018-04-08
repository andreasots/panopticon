var ManifestPlugin = require('webpack-manifest-plugin');

var seed = {};

module.exports = [
  {
    entry: {
      style: './static/style.scss',
    },
    output: {
      filename: './static/style.[hash].js',
    },
    plugins: [
      new ManifestPlugin({
        seed: seed,
      }),
    ],
    module: {
      rules: [
        {
          test: /\.scss$/,
          use: [
            {
              loader: 'file-loader',
              options: {
                name: './static/style.[hash].css',
              },
            },
            { loader: 'extract-loader' },
            { loader: 'css-loader' },
            { loader: 'postcss-loader' },
            {
              loader: 'sass-loader',
              options: {
                importer: function(url, prev) {
                  if (url.indexOf('@material') === 0) {
                    var filePath = url.split('@material')[1];
                    var nodeModulePath = `./node_modules/@material/${filePath}`;
                    return { file: require('path').resolve(nodeModulePath) };
                  }
                  return { file: url };
                },
              },
            },
          ],
        },
      ]
    },
  },
  {
    entry: {
      material: './static/material.ts',
      search: './static/search.ts',
    },
    output: {
      filename: './static/[name].[chunkhash].js',
    },
    plugins: [
      new ManifestPlugin({
        seed: seed,
      }),
    ],
    module: {
      rules: [
        {
          test: /\.ts$/,
          use: 'ts-loader',
          exclude: /node_modules/,
        }
      ]
    },
    resolve: {
      extensions: ['.ts', '.js'],
    },
  }
];
