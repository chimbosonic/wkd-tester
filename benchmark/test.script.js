import { group } from "k6";
import http from "k6/http";
import { check } from "k6";
import { config } from "./.config.js";

export let options = {
    vus: 1000,
    duration: "30s",
};

export function setup() {
    const env_name = __ENV.ENV || "localhost";
    const env_config = config[env_name];
    console.log(`Running against ${env_name} environment`);

    if (!env_config) {
        throw new Error(`Environment ${env_name} not found in config`);
    }

    return {
        env_config,
    };
}

export default function (data) {
    const tests = [
        { name: 'index', url: '/' },
        { name: 'index_lookup', url: '/?email=test@dp42.dev' },
        { name: 'api_lookup', url: '/api/lookup?email=test@dp42.dev' }
    ]

    for (let test of tests) {
        group(test.name, function () {
            get_result(`${data.env_config.base_url}${test.url}`);
        });
    }
}


export function get_result(url) {
    let res = http.get(url);
    check(
        res,
        {
            "status is 200": (r) => r.status === 200,
        },
        {},
    );
}
