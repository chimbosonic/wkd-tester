# K6 Latency Tests for GraphQL

This is a set of [Grafana k6](https://grafana.com/docs/k6/latest/) tests for measuring the latency.

## Setup

Make sure you have a `.configs.js` file with the following inside:

```javascript
export const config = {
  env: {
    base_url: `http(s)://url/`,
  },
};
```

And that you have k6 is installed by following these [docs](https://grafana.com/docs/k6/latest/set-up/install-k6/).

## Running the script

To run the tests provide the env you want to run agains't and provide an output location

- fish: `set -x ENV "env"; k6 run test.script.js -o csv=./results/${ENV}-$(date +%s).csv`
- bash: `export ENV="env"; k6 run test.script.js -o csv=./results/${ENV}-$(date +%s).csv`
