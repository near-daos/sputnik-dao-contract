NEAR Units
==========

TypeScript/JavaScript tools to help parse and format NEAR units. For now, this means NEAR tokens and [gas units](https://docs.near.org/docs/concepts/gas).

# Install

    npm i --save near-units

Or using [Yarn](https://yarnpkg.com/):

    yarn add near-units

# Parsing strings

```js
import { NEAR, Gas, parse } from 'near-units';

const near = NEAR.parse('1.25 mN');
console.log(near.toHuman()); // 1.25 mN

const gas = Gas.parse('1 Tgas');
console.log(gas.toHuman()); // 1 Tgas

// equivalent to the above, but TS typings might not be as helpful
const near = parse('1.25 mN');
const gas = parse('1 Tgas');
```

See [\_\_tests__](./__tests__) for a full list of examples of inputs that can be parsed and the human-readable version that can be returned by `toHuman`.

# Doing math

`NEAR` and `Gas` both wrap `BN` from [bn.js], so you can perform any math with them that you need:

```js
import { NEAR } from 'near-units';

const amount1 = NEAR.parse('100');
const amount2 = NEAR.parse('0.5');
const amount3 = amount1.mul(amount2);
```

See [the bn.js docs for all possible operations][bn.js].

  [bn.js]: https://github.com/indutny/bn.js/

# Interop

Since they wrap `BN`, they can be passed directly to function calls with [near-api-js](https://github.com/near/near-api-js) or [near-runner](https://github.com/near/runner-js):

```js
// with an Account object from near-api-js
someAccount.functionCall({
  contractId: 'example.near',
  methodName: 'do_something',
  args: { param1: 'whatever' },
  gas: Gas.parse('50,000,000,000,000'),
  attachedDeposit: NEAR.parse('1'),
});

// with an Account object from near-runner
someAccount.call(
  'example.near',
  'do_something',
  { param1: 'whatever' },
  {
    gas: Gas.parse('50,000,000,000,000'),
    attachedDeposit: NEAR.parse('1'),
  }
});
```

`NEAR` and `Gas` also both override `toJSON` to get to a string version that can be passed as an argument to near-cli and in other contexts.

# CLI

This package ships with a minimal CLI:

    npm i -g near-units

Now you can `near-units --help`:

    Parse and format NEAR tokens and gas units. Examples:

        near-units 10 N # => 10000000000000000000000000
        near-units -h 10000000000000000000000000 yN # => 10 N
        near-units 50 Tgas # => 50000000000000
        near-units -h 50000000000000 gas # => 50 Tgas

You can use it anywhere near units are accepted. For example, on macOS & Linux, you can:

```bash
near call $LOCKUP transfer '{
  "receiver_id": "example.near",
  "amount": "'$(near-units 1N)'"
}' --accountId=$ACCOUNT --gas=$(near-units 50Tgas)
```
