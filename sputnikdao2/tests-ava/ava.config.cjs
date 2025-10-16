require('util').inspect.defaultOptions.depth = 5; // Increase AVA's printing depth
const os = require('os');

function getPlatform() {
    const type = os.type();
    const arch = os.arch();
    if ((type === "Linux" || type === "Darwin") && arch === "x64") {
        return [type, "x86_64"];
    }
    else if (type === "Darwin" && arch === "arm64") {
        return [type, "arm64"];
    }
    throw new Error(`Unsupported platform: ${type} ${arch}`);
}

function AWSUrl(version) {
  const [platform, arch] = getPlatform();
  return `https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore/${platform}-${arch}/${version}/near-sandbox.tar.gz`;
}

process.env.SANDBOX_ARTIFACT_URL= AWSUrl("2.8.0");

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
