# Baritone for Horizon

Ходьба и копание в духе [Baritone](https://github.com/cabaletta/baritone) для [Horizon](https://github.com/horizon-rust/horizon). Это не порт Java-мода: Fabric-jar в Horizon не исполняется. Поиск пути, поворот и шаг написаны заново и ходят только по уже загруженным чанкам.

## Команды

- `/b goto <x> <y> <z>` — встать ногами в этот блок
- `/b mine <блок>` — ближайший такой блок в радиусе 16, например `/b mine diamond_ore`
- `/b stop`
- `/b help`

Слева сверху виден статус. Блок ломается как удар игрока, добыча идёт в инвентарь.

## Сборка

```sh
cd native && cargo test
```

В лаунчере, из клона с исходниками Horizon:

```sh
horizon-launcher native-build /path/to/baritone-horizon/native/mixins/baritone --trust-native-code
```

Нужен Horizon с `NATIVE_HOOKS_API` не ниже 413, ревизия `c98a51a664c8ce8ec166ecedd85bb47b3687b336`.

## Чего нет относительно Baritone

Нет элитры, строительства по схематике, обхода воды, паркура через пропасть, фарм-лупов и настроек из `baritone/settings.txt`. Шаг вверх на один блок и спуск до трёх есть. Копание в этом срезе снимает блок сразу, без времени добычи киркой.
