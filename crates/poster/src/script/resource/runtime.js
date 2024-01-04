((globalThis) => {
    const core = Deno.core;

    function argsToMessage(...args) {
        return args.map((arg) => JSON.stringify(arg)).join(" ");
    }

    globalThis.console = {
        log: (...args) => {
            core.ops.log(argsToMessage(...args));
        },
        error: (...args) => {
            core.ops.error(argsToMessage(...args));
        },
    };
    globalThis.poster = {
        set_env: (key, value) => {
            core.ops.set_env(key, value)
        },
        get_env: (key) => {
            core.ops.get_env(key)
        },
        add_header: (key, value) => {
            core.ops.add_header(key, value)
        },
        add_params: (key, value) => {
            core.ops.add_params(key, value)
        },
    }

})(globalThis);