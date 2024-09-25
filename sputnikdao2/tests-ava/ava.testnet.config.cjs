module.exports = {
  ...require('./ava.config.cjs'),
};

module.exports.environmentVariables = {
  NEAR_WORKSPACES_NETWORK: 'testnet',
};

module.exports.files.push(
  '!__tests__/02*',
  '!__tests__/05*',
);
