# FiveM WASM runtime
WASM рантайм для мультиплеера [FiveM](https://fivem.net/)

форк fivem лежит [здесь](https://github.com/zottce/fivem). в нем сделан [компонент](https://github.com/ZOTTCE/fivem/tree/wasm/code/components/citizen-scripting-wasm) на плюсах, который  использует внутри себя этот код

код пиздец, не смотреть ...

## структура
* [`examples\entry`](examples/entry/) - ну пример (сырой)
* [`bindings`](bindings/) - rust впоперы + биндинги + всякая залупа для написания скриптов
* [`glue`](glue/) - статическая библиотека чтобы засунуть ее в fivem
* [`runtime`](runtime/) - васм рантайм (wasmtime)
* [`standalone`](standalone/) - тупа поиграться с рантаймом выше

## сборка
* компилятор RUST
* [файвм по приколу собрать](https://github.com/citizenfx/fivem/blob/master/docs/building.md)
* стыбзить или воспользоваться [скриптом](utils/fivem-build.ps1)

## задачи на будушее
* ПРИБРАТЬСЯ
* пиздатый вппопер (wrapper) для раста для васм скриптов
* васм модуль с нативными функциями
* ?
* ждать возможность использовать `std::net::TcpSocket`
