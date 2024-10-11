require('util').inspect.defaultOptions.depth = 5; // Increase AVA's printing depth

module.exports = {
  timeout: '300000',
  files: ['**/*.ava.ts', '**/*.ava.js', '!examples/**/*.ava.js'],
  failWithoutAssertions: false,
  extensions: [
    'ts',
    'js',
  ],
  require: [
    'ts-node/register',
  ],
};
