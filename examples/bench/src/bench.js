exports("exportBench", () => { });
on("jsEventHandler", (obj) => { });

function bench_1() {
    GetNumResources();
}

function bench_2() {
    CancelEvent();
}

bench_1();
bench_2();

// I am not sure that I got it right ...
const {
    performance,
    PerformanceObserver
} = require('perf_hooks');

{
    const wrapped = performance.timerify(bench_1);

    const obs = new PerformanceObserver((list) => {
        console.log(list.getEntries()[0].duration);
        obs.disconnect();
    });

    obs.observe({ entryTypes: ['function'] });

    wrapped();
}

{
    const wrapped = performance.timerify(bench_2);

    const obs = new PerformanceObserver((list) => {
        console.log(list.getEntries()[0].duration);
        obs.disconnect();
    });

    obs.observe({ entryTypes: ['function'] });

    wrapped();
}
