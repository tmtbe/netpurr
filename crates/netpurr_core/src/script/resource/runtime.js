((globalThis) => {
    const core = Deno.core;

    function argsToMessage(...args) {
        return args.map((arg) => JSON.stringify(arg)).join(" ");
    }

    globalThis.sleep = async function (time) {
        await core.ops.op_sleep(time);
    }

    globalThis.fetch = async function (request) {
        let response = await core.ops.op_http_fetch(request);
        try {
            response.json = JSON.parse(response.text);
        } catch (e) {
        }
        return response
    }

    globalThis.assert = function (expect, actual) {
        if (core.ops.op_equal(expect, actual)) {
            core.ops.op_append_assert(true, `Expect equal actual is "${expect}"`);
        } else {
            core.ops.op_append_assert(false, `Expect is "${expect}" but actual is "${actual}"`);
        }
    }
    globalThis.assert_nlp = function (expect, actual, expect_similarity) {
        let actual_similarity = core.ops.op_nlp_similarity(expect, actual);
        if (actual_similarity > expect_similarity) {
            core.ops.op_append_assert(true, `NLP similarity is "${actual_similarity}"`);
        } else {
            core.ops.op_append_assert(false, `NLP similarity is "${actual_similarity}" less than "${expect_similarity}"`);
        }
    }

    globalThis.testcase = function () {
        return JSON.parse(core.ops.op_get_testcase());
    }

    globalThis.console = {
        log: (...args) => {
            core.ops.op_log(argsToMessage(...args));
        },
        warn: (...args) => {
            core.ops.op_warn(argsToMessage(...args));
        },
        error: (...args) => {
            core.ops.op_error(argsToMessage(...args));
        },
    };
    globalThis.nlp = {
        keywords: (str, k) => {
            return core.ops.op_nlp_keywords(str, k)
        },
        tags: (str) => {
            return core.ops.op_nlp_tags(str)
        },
        tag_filter: (str, tags) => {
            return core.ops.op_nlp_tag_filter(str, tags)
        },
        similarity: (a, b) => {
            return core.ops.op_nlp_similarity(a, b)
        }
    }
    globalThis.netpurr = {
        get_testcase: () => {
            return JSON.parse(core.ops.op_get_testcase());
        },
        set_env: (key, value) => {
            return core.ops.op_set_env(key, String(value))
        },
        get_env: (key) => {
            return core.ops.op_get_env(key)
        },
        add_header: (key, value) => {
            return core.ops.op_add_header(key, String(value))
        },
        add_params: (key, value) => {
            return core.ops.op_add_params(key, String(value))
        },
        set_shared: (key, value) => {
            let json_value = JSON.stringify(value)
            return core.ops.op_set_shared(key, json_value)
        },
        get_shared: (key) => {
            let json_value = core.ops.op_get_shared(key);
            return JSON.parse(json_value)
        },
        wait_shared: async (key) => {
            let json_value = await core.ops.op_wait_shared(key)
            return JSON.parse(json_value)
        },
        resp: () => {
            let response = core.ops.op_response();
            try {
                response.json = JSON.parse(response.text);
            } catch (e) {
            }
            return response
        },
        test: (name, func) => {
            core.ops.op_open_test(name);
            func();
            core.ops.op_close_test(name);
        },
        test_skip: () => {
            core.ops.op_test_skip();
        }
    }

})(globalThis);