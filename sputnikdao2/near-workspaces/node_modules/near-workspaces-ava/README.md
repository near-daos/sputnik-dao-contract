near-workspaces + AVA
=====================

A thin wrapper around [near-workspaces] to make it easier to use with [AVA] and [TypeScript]. If you don't want AVA, use near-workspaces directly.

Controlled, concurrent workspaces in local [NEAR Sandbox](https://github.com/near/sandbox) blockchains or on [NEAR TestNet](https://docs.near.org/docs/concepts/networks) meets powerful, concurrent testing with [AVA].

  [near-workspaces]: https://github.com/near/workspaces-js
  [AVA]: https://github.com/avajs/ava
  [TypeScript]: https://www.typescriptlang.org/

Quick Start
===========

`near-workspaces-init` is a one-time command to quickly initialize a project with `near-workspaces-ava`. You will need [NodeJS] installed. Then:

    npx near-workspaces-init

It will:

* Add a `near-workspaces` directory to the folder where you ran the command. This directory contains all the configuration needed to get you started with near-workspaces-ava, and a `__tests__` subfolder with a well-commented example test file.
* Create `test.sh` and `test.bat` scripts in the folder where you ran the command. These can be used to quickly run the tests in `near-workspaces`. Feel free to integrate test-running into your project in a way that makes more sense for you, and then remove these scripts.
* Install NPM dependencies using `npm install`. Most of the output you see when running the command comes from this step. You can skip this: `npx near-workspaces-init --no-install`.

  [NodeJS]: https://nodejs.dev/

Manual Install
==============

1. Install.

   ```bash
   npm install --save-dev near-workspaces-ava # npm
   yarn add --dev near-workspaces-ava         # yarn
   ```

2. Configure.

   AVA [currently requires](https://github.com/avajs/ava/issues/2285) that your project have its own [AVA config file](https://github.com/avajs/ava/blob/main/docs/06-configuration.md). Add a file called `ava.config.cjs` next to your `package.json` with the following contents:

   ```js
   module.exports = require('near-workspaces-ava/ava.config.cjs');
   ```

   We also recommend using the `near-workspaces-ava` script to run your tests. This is mostly an alias for `ava`, and passes CLI arguments right through.

       "test": "near-workspaces-ava"

   Now you can run tests with `npm run test` or `yarn test`.

   If you want to write tests with TypeScript (recommended), you can add a `tsconfig.json` to your project root with the following contents:

       {"extends": "near-workspaces-ava/tsconfig.ava.json"}

   If you already have TypeScript set up and you don't want to extend the config from `near-workspaces-ava`, feel free to just copy the settings you want from [tsconfig.ava.json](./tsconfig.ava.json).

   If you have test files that should only run in Sandbox mode, you can create an `ava.testnet.config.cjs` config file in the same directory as your `package.json` with the following contents:

   ```js
   module.exports = {
     ...require('near-workspaces-ava/ava.testnet.config.cjs'),
     ...require('./ava.config.cjs'),
   };

   module.exports.files.push(
     '!__tests__/pattern-to-ignore*',
     '!__tests__/other-pattern-to-ignore*',
   );
   ```

   See [this project's testnet config](../../ava.testnet.config.cjs) for an example. The [near-workspaces-ava/ava.testnet.config.cjs](./ava.testnet.config.cjs) import sets the `NEAR_WORKSPACES_NETWORK` environment variable for you, so now you can add a `test:testnet` script to your `package.json`'s `scripts` section:

   ```diff
    "scripts": {
      "test": "near-workspaces-ava",
   +  "test:testnet": "near-workspaces-ava --config ./ava.testnet.config.cjs"
    }
    ```

2. Initialize.

   Make a `__tests__` folder, make your first test file. Call it `main.ava.ts` if you're not sure what else to call it. The AVA config you extended above will find files that match the `*.ava.(ts|js)` suffix.

   In `main.ava.ts`, initialize a `workspace` with NEAR accounts, contracts, and state that will be used in all of your tests.

   ```ts
   import {Workspace} from 'near-workspaces-ava';

   const workspaces = Workspace.init(async ({root}) => {
      const alice = await root.createAccount('alice');
      const contract = await root.createAndDeploy(
        'contract-account-name',
        'path/to/compiled.wasm'
      );

      // make other contract calls that you want as a starting point for all tests

      return {alice, contract};
   });
   ```

4. Write tests.

   ```ts
    workspace.fork("does something", async (test, { alice, contract }) => {
      await alice.call(contract, "some_update_function", {
        some_string_argument: "cool",
        some_number_argument: 42,
      });
      const result = await contract.view("some_view_function", {
        account_id: alice,
      });
      // When --verbose option is used this will print neatly underneath the test in the output.
      test.log(result)
      test.is(result, "whatever");
    });

    workspaces.fork("does something else", async (test, { alice, contract }) => {
      const result = await contract.view("some_view_function", {
        account_id: alice,
      });
      test.is(result, "some default");
    });
    ```

   `workspace.test` is added to `near-workspaces` by `near-workspaces-ava`, and is
   shorthand for:

    ```ts
    import avaTest from 'ava';
    import {Workspace} from 'near-workspaces';
    // Alternatively, you can import Workspace and ava both from near-workspaces-ava:
    // import {ava as avaTest, Workspace} from 'near-workspaces-ava';

    const workspace = Workspace.init(…);

    avaTest('does something', async test => {
      await workspaces.fork(async ({…}) => {
        // tests go here
      });
    });
   ```

   Where [`avaTest`](https://github.com/avajs/ava/blob/main/docs/01-writing-tests.md) and [`t`](https://github.com/avajs/ava/blob/main/docs/03-assertions.md) come from AVA and [`workspace.fork`](https://github.com/near/workspaces-js#how-it-works) comes from near-workspaces.

See the [`__tests__`](https://github.com/near/workspaces-js/tree/main/__tests__) directory for more examples.
