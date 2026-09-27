# Public Repository Cleanup & Sanitization — ЭТАП 2

Дата: 2026-09-27. Основа: `PUBLIC_BETA_AUDIT.md` и рабочее дерево на `8754c008c6513f9b3a256344182cbd2b86c25235`. Архитектура, scope `v0.1.0-beta` и Support UI не менялись. GitHub repository, push и GitHub Release не создавались.

## Проверенный состав

На входе было 73 untracked-файла и изменённый `README.md`. Каждый кандидат проверен до добавления в Git:

| Категория | Исходные untracked | Решение |
|---|---:|---|
| A — нужные для публичного source repository | 51 | React/Rust source, tests, manifests и lockfiles, Tauri config/capabilities, необходимые `icon.ico`/`icon.png`/`source.svg` и 2 WAV, генератор собственных assets, README, `DECISIONS.md`, `CODEX_DATA_SOURCES.md`, исторический `PUBLIC_BETA_AUDIT.md`, `.gitignore`. Сохранены. |
| B — generated/build output | 18 | 4 Tauri `gen/schemas` JSON и 14 неиспользуемых icon variants удалены; будущая генерация игнорируется. `dist/`, `node_modules/`, `src-tauri/target/`, `*.tsbuildinfo` остаются локально и игнорируются. |
| C — локальные debug/probe утилиты | 3 | `scripts/probe.ps1`, `scripts/inspect-local.cjs`, `src-tauri/src/bin/diagnose.rs` удалены; из публичных docs удалены команды их запуска. Игнорируемые `.probe/` и `debug.log` удалены с диска. |
| D — чувствительные данные | 0 | В source-кандидатах не обнаружены. Публичный receiving address в Support намеренно сохранён. |
| E — устаревший внутренний документ | 1 | `PROJECT_STATE.md` удалён: содержал историю промежуточных сборок, размеров и проверок, не являющуюся текущей публичной спецификацией. |

Дополнительно созданы `LICENSE` и этот отчёт. Архивный `PUBLIC_BETA_AUDIT.md` помечен как снимок до cleanup, чтобы его исторические находки не воспринимались как текущее состояние.

## Privacy и Git history

- Проверены все tracked и untracked кандидаты, включая lockfiles, source, Markdown, scripts и бинарные icon/WAV assets. Поиск охватил ключи и token prefixes, secret assignments, private-key blocks, `.env`, emails, абсолютные пользовательские Windows paths, UUID-подобные IDs и имена локальных dump/database/log файлов. Совпадений с секретами, prompts/responses, rollout/session dumps и локальными Codex database contents в кандидатах нет.
- В Git history только один исходный commit (`8754c008c6513f9b3a256344182cbd2b86c25235`) с `README.md`. Его blob проверен; секретов, локальных данных, `.env` и probe dumps нет. Метаданные автора содержат личный email. Пользователь отдельно разрешил оставить этот email видимым в будущей публичной истории; история не переписывалась.
- В `Support.tsx` содержится только указанный пользователем **публичный** receiving address `TGSbmov4GibLQTxLNf9Soju7178Zn1AjXC`. Нет private key, seed phrase, wallet authentication, blockchain/payment integration или скрытых external requests. QR создаётся локально; копирование обращается только к clipboard. CSP приложения ограничивает загрузку локальными ресурсами и Tauri IPC.
- `diagnostics.rs` пишет только timestamp и статическую категорию. Персональные task names и тексты разговоров в лог не записываются.

## `.gitignore` и структура

`.gitignore` расширен для Node/Vite/Rust/Tauri/Windows: dependencies, frontend/Rust output, installer/exe/PDB, generated Tauri schemas и icon variants, coverage, IDE-local файлы, env/local config, временные/backup файлы, logs/crash dumps, локальные SQLite/database файлы, `.codex`/sessions/rollouts и probe directories. `git check-ignore -v` подтвердил исключение типовых `.env`, IDE, Codex, probe и Tauri build путей. Необходимые manifests, lockfiles, config, source и assets не игнорируются.

В исходниках удалены только остатки окончательно исключённого automatic compaction count: поля Rust/TypeScript Task, счётчик и обработка в lifecycle, их тестовые значения/тесты. Тест нормализации названий сохранён. UI-счётчики по трём статусам и существующая логика задач не менялись. `README.md`, `DECISIONS.md` и `CODEX_DATA_SOURCES.md` больше не описывают старый compaction indicator или удалённые probe-команды; README кратко отражает уже существующий Support modal.

## Dependencies и licensing

- `package.json`/lockfile и `Cargo.toml`/lockfile проверены. `npm ls --depth=0` не выявил отсутствующих или лишних прямых пакетов. Все npm `resolved` в lockfile указывают на стандартный npm registry; Cargo lockfile — на crates.io registry. Git, private registry, local file и path dependencies не обнаружены.
- Прямые зависимости используются текущим приложением либо тестами. Версии и lockfiles не обновлялись, зависимости не добавлялись и не удалялись.
- License metadata прямых npm/Rust packages: MIT, Apache-2.0, ISC или CC0-1.0 (комбинации разрешительных лицензий). В transitive Rust metadata есть MPL-2.0 и альтернативный LGPL-вариант при наличии MIT/Apache-варианта; в transitive npm dev packages — BlueOak-1.0.0, Python-2.0, CC-BY-4.0. Очевидной несовместимой прямой зависимости для MIT source release не найдено. Это проверка package metadata, не юридическое заключение по всем условиям бинарного распространения.
- Корневой `LICENSE` содержит стандартный MIT License с copyright `2026 Danil Shulgin`; имя правообладателя подтверждено пользователем. Сторонние лицензии не изменялись.
- Публичные project metadata сохраняют имя `Codex Monitor` и текущую техническую версию `0.1.0`; beta version/tag будет назначена на этапе релиза, без хаотичной смены номеров сейчас.

## Проверки после cleanup

| Команда | Результат |
|---|---|
| `npm test` | PASS, 16 frontend tests. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS без warnings. |
| `npm run build` | PASS, Vite production bundle. |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS: 23 Rust tests, 1 штатно ignored native audio test. |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | PASS. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | PASS. |
| `npm run tauri:build` | PASS: локальные release exe и NSIS installer собраны после cleanup, без публикации. SHA-256 exe `C8A7B119A83427EBC52B27CA999835559D49870CCF62DA74017579839A11AE76`. |

## Итог

Содержимое source repository отделено от local/generated данных. Подтверждённых secrets, private content и оставшихся устаревших product features в публикуемых файлах нет. Архитектура приложения не менялась. Локальная Windows production-сборка прошла. **READY FOR STAGE 3** с точки зрения очистки репозитория; это не публикация и не запуск Stage 3.
