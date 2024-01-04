((globalThis) => {
    const core = Deno.core;

    function argsToMessage(...args) {
        return args.map((arg) => JSON.stringify(arg)).join(" ");
    }

    globalThis.console = {
        log: (...args) => {
            core.ops.op_log(argsToMessage(...args));
        },
        error: (...args) => {
            core.ops.op_error(argsToMessage(...args));
        },
    };
    globalThis.poster = {
        set_env: (key, value) => {
            return core.ops.op_set_env(key, value)
        },
        get_env: (key) => {
            return core.ops.op_get_env(key)
        },
        add_header: (key, value) => {
            return core.ops.op_add_header(key, value)
        },
        add_params: (key, value) => {
            return core.ops.op_add_params(key, value)
        },
        fetch: async (request) => {
            let response = await core.ops.op_http_fetch(request);
            try {
                response.json = JSON.parse(response.text);
            } catch (e) {
            }
            return response
        }
    }

})(globalThis);